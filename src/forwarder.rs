#![allow(dead_code)]

use crate::dns::{
    encode_questions, DecodedQuestion, DnsHeaderRequest, DnsHeaderResponse, HEADER_LEN,
};
use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;
use thiserror::Error;

/// Errors that can occur while forwarding DNS packets to the upstream resolver.
#[derive(Debug, Error)]
pub enum ForwardError {
    #[error("upstream lookup timed out before receiving a response")]
    Timeout,
    #[error("socket error when communicating with upstream resolver: {0}")]
    Io(#[from] std::io::Error),
    #[error("malformed upstream response: {0}")]
    Malformed(&'static str),
    #[error("upstream response question mismatch")]
    QuestionMismatch,
}

/// Result of forwarding a single-question packet to the upstream resolver.
#[derive(Debug)]
pub enum ForwardResult {
    Success {
        upstream_header: DnsHeaderResponse,
        answer_section: Vec<u8>,
    },
    Failure(ForwardError),
}

impl ForwardResult {
    pub fn success(upstream_header: DnsHeaderResponse, answer_section: Vec<u8>) -> Self {
        Self::Success {
            upstream_header,
            answer_section,
        }
    }

    pub fn failure(error: ForwardError) -> Self {
        Self::Failure(error)
    }
}

/// Represents a single-question packet derived from the tester's multi-question request.
#[derive(Debug, Clone)]
pub struct SplitQuestion {
    pub index: usize,
    pub question: DecodedQuestion,
    pub forward_packet: Vec<u8>,
}

impl SplitQuestion {
    pub fn new(index: usize, question: DecodedQuestion, forward_packet: Vec<u8>) -> Self {
        Self {
            index,
            question,
            forward_packet,
        }
    }
}

/// Tracks the lifecycle of a forwarding operation for one tester packet.
#[derive(Debug)]
pub struct ForwardingJob {
    pub header: DnsHeaderRequest,
    pub questions: Vec<DecodedQuestion>,
    pub split_jobs: Vec<SplitQuestion>,
    pub responses: Vec<ForwardResult>,
}

impl ForwardingJob {
    pub fn from_request(header: DnsHeaderRequest, questions: Vec<DecodedQuestion>) -> Self {
        let mut split_jobs = Vec::with_capacity(questions.len());
        for (index, question) in questions.iter().enumerate() {
            let forward_packet = build_single_question_packet(&header, question);
            split_jobs.push(SplitQuestion::new(index, question.clone(), forward_packet));
        }

        let response_capacity = split_jobs.len();
        Self {
            header,
            questions,
            split_jobs,
            responses: Vec::with_capacity(response_capacity),
        }
    }

    pub fn push_response(&mut self, response: ForwardResult) {
        self.responses.push(response);
    }

    pub fn aggregate_answers(&self) -> (Vec<u8>, u16, u8) {
        let mut answers = Vec::new();
        let mut ancount: u16 = 0;
        let mut rcode = 0;
        for response in &self.responses {
            if let ForwardResult::Success {
                upstream_header,
                answer_section,
            } = response
            {
                answers.extend_from_slice(answer_section);
                ancount = ancount.saturating_add(upstream_header.ancount);
                if upstream_header.rcode != 0 {
                    rcode = upstream_header.rcode;
                }
            }
        }
        (answers, ancount, rcode)
    }
}

/// Serializes DNS responses back to the tester using mirrored question sections.
pub struct ResponseAssembler<'a> {
    header: &'a DnsHeaderRequest,
    questions: &'a [DecodedQuestion],
}

impl<'a> ResponseAssembler<'a> {
    pub fn new(header: &'a DnsHeaderRequest, questions: &'a [DecodedQuestion]) -> Self {
        Self { header, questions }
    }

    /// Builds a DNS response packet using the provided answer section, answer count, and rcode.
    pub fn assemble(&self, answer_section: &[u8], ancount: u16, rcode: u8) -> Vec<u8> {
        let mut header_buf = [0u8; HEADER_LEN];
        let header = DnsHeaderResponse::with_counts_and_rcode(self.header, ancount, rcode);
        header.write_into(&mut header_buf);

        let question_bytes = encode_questions(self.questions);
        let mut response =
            Vec::with_capacity(HEADER_LEN + question_bytes.len() + answer_section.len());
        response.extend_from_slice(&header_buf);
        response.extend_from_slice(&question_bytes);
        response.extend_from_slice(answer_section);
        response
    }

    /// Builds a SERVFAIL response with zero answer records.
    pub fn assemble_servfail(&self) -> Vec<u8> {
        self.assemble(&[], 0, 2)
    }
}

/// Sends DNS packets to an upstream resolver and parses the responses.
pub struct ForwardingResolver {
    socket: UdpSocket,
    resolver_addr: SocketAddr,
    timeout: Duration,
}

impl ForwardingResolver {
    pub fn new(resolver_addr: SocketAddr, timeout: Duration) -> io::Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(resolver_addr)?;
        socket.set_read_timeout(Some(timeout))?;
        Ok(Self {
            socket,
            resolver_addr,
            timeout,
        })
    }

    pub fn send_and_recv(
        &self,
        outbound_packet: &[u8],
        tester_id: u16,
        expected_question: &DecodedQuestion,
    ) -> ForwardResult {
        if let Err(err) = self.socket.send(outbound_packet) {
            return ForwardResult::failure(ForwardError::Io(err));
        }

        let mut buffer = [0u8; 512];
        match self.socket.recv(&mut buffer) {
            Ok(size) => {
                let response = &buffer[..size];
                match self.parse_upstream_response(response, tester_id, expected_question) {
                    Ok((header, answers)) => ForwardResult::success(header, answers),
                    Err(err) => ForwardResult::failure(err),
                }
            }
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
                ForwardResult::failure(ForwardError::Timeout)
            }
            Err(err) if err.kind() == io::ErrorKind::TimedOut => {
                ForwardResult::failure(ForwardError::Timeout)
            }
            Err(err) => ForwardResult::failure(ForwardError::Io(err)),
        }
    }

    fn parse_upstream_response(
        &self,
        packet: &[u8],
        tester_id: u16,
        expected_question: &DecodedQuestion,
    ) -> Result<(DnsHeaderResponse, Vec<u8>), ForwardError> {
        if packet.len() < HEADER_LEN {
            return Err(ForwardError::Malformed("packet smaller than DNS header"));
        }

        let mut header = parse_response_header(packet)?;
        if header.qdcount != 1 {
            return Err(ForwardError::Malformed("upstream response qdcount != 1"));
        }

        let (question, answer_offset) = DecodedQuestion::from_packet(packet, HEADER_LEN)
            .ok_or(ForwardError::Malformed("unable to parse upstream question"))?;
        if question.name != expected_question.name
            || question.qtype != expected_question.qtype
            || question.qclass != expected_question.qclass
        {
            return Err(ForwardError::QuestionMismatch);
        }

        if answer_offset > packet.len() {
            return Err(ForwardError::Malformed("invalid answer offset"));
        }

        let answer_section = packet[answer_offset..].to_vec();
        header.id = tester_id;
        Ok((header, answer_section))
    }
}

fn parse_response_header(packet: &[u8]) -> Result<DnsHeaderResponse, ForwardError> {
    if packet.len() < HEADER_LEN {
        return Err(ForwardError::Malformed("packet smaller than DNS header"));
    }
    let id = u16::from_be_bytes([packet[0], packet[1]]);
    let flags = u16::from_be_bytes([packet[2], packet[3]]);
    let opcode = ((flags & 0x7800) >> 11) as u8;
    let rd = (flags & 0x0100) != 0;
    let rcode = (flags & 0x000F) as u8;
    let qdcount = u16::from_be_bytes([packet[4], packet[5]]);
    let ancount = u16::from_be_bytes([packet[6], packet[7]]);
    let nscount = u16::from_be_bytes([packet[8], packet[9]]);
    let arcount = u16::from_be_bytes([packet[10], packet[11]]);

    Ok(DnsHeaderResponse {
        id,
        opcode,
        rd,
        qdcount,
        ancount,
        nscount,
        arcount,
        rcode,
    })
}

fn build_single_question_packet(header: &DnsHeaderRequest, question: &DecodedQuestion) -> Vec<u8> {
    let mut packet =
        Vec::with_capacity(HEADER_LEN + question.name.len() + std::mem::size_of::<u32>());
    packet.extend_from_slice(&header.id.to_be_bytes());
    packet.extend_from_slice(&header.flags.to_be_bytes());
    packet.extend_from_slice(&1u16.to_be_bytes());
    packet.extend_from_slice(&0u16.to_be_bytes()); // ancount
    packet.extend_from_slice(&0u16.to_be_bytes()); // nscount
    packet.extend_from_slice(&0u16.to_be_bytes()); // arcount
    packet.extend_from_slice(&question.name);
    packet.extend_from_slice(&question.qtype.to_be_bytes());
    packet.extend_from_slice(&question.qclass.to_be_bytes());
    packet
}
