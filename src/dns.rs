use bytes::{BufMut, BytesMut};
use std::convert::TryInto;

pub const HEADER_LEN: usize = 12;

#[derive(Clone, Copy, Debug)]
pub struct AnswerConfig {
    pub ttl_seconds: u32,
    pub ipv4: [u8; 4],
}

impl AnswerConfig {
    pub const fn new(ipv4: [u8; 4], ttl_seconds: u32) -> Self {
        Self { ttl_seconds, ipv4 }
    }
}

pub const ANSWER_CONFIG: AnswerConfig = AnswerConfig::new([8, 8, 8, 8], 60);

pub const fn default_answer_config() -> AnswerConfig {
    ANSWER_CONFIG
}

#[derive(Clone, Copy, Debug)]
pub struct DnsHeaderRequest {
    pub id: u16,
    pub flags: u16,
    pub qdcount: u16,
}

impl DnsHeaderRequest {
    pub fn parse(packet: &[u8]) -> Result<Self, ()> {
        if packet.len() < HEADER_LEN {
            return Err(());
        }
        Ok(Self {
            id: u16::from_be_bytes([packet[0], packet[1]]),
            flags: u16::from_be_bytes([packet[2], packet[3]]),
            qdcount: u16::from_be_bytes([packet[4], packet[5]]),
        })
    }

    pub fn opcode(&self) -> u8 {
        ((self.flags & 0x7800) >> 11) as u8
    }

    pub fn rd(&self) -> bool {
        (self.flags & 0x0100) != 0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DnsHeaderResponse {
    pub id: u16,
    pub opcode: u8,
    pub rd: bool,
    pub qdcount: u16,
    pub ancount: u16,
    pub nscount: u16,
    pub arcount: u16,
}

impl DnsHeaderResponse {
    pub fn from_request(request: &DnsHeaderRequest) -> Self {
        Self {
            id: request.id,
            opcode: request.opcode(),
            rd: request.rd(),
            qdcount: request.qdcount,
            ancount: 1,
            nscount: 0,
            arcount: 0,
        }
    }

    pub fn write_into(&self, buffer: &mut [u8]) {
        buffer[..2].copy_from_slice(&self.id.to_be_bytes());
        let mut flags: u16 = 0;
        flags |= 1 << 15; // QR = 1 (response)
        flags |= ((self.opcode & 0x0F) as u16) << 11;
        if self.rd {
            flags |= 1 << 8;
        }
        let rcode = if self.opcode == 0 { 0 } else { 4 };
        flags |= rcode as u16;
        buffer[2..4].copy_from_slice(&flags.to_be_bytes());
        buffer[4..6].copy_from_slice(&self.qdcount.to_be_bytes());
        buffer[6..8].copy_from_slice(&self.ancount.to_be_bytes());
        buffer[8..10].copy_from_slice(&self.nscount.to_be_bytes());
        buffer[10..12].copy_from_slice(&self.arcount.to_be_bytes());
    }
}

#[derive(Clone, Debug)]
pub struct DnsQuestion {
    name: Vec<u8>,
    qtype: u16,
    qclass: u16,
}

impl DnsQuestion {
    pub fn from_packet(packet: &[u8], offset: usize) -> Option<(Self, usize)> {
        let (name, name_end) = read_qname(packet, offset)?;
        let qtype_start = name_end;
        let qtype_end = qtype_start.checked_add(2)?;
        let qclass_end = qtype_end.checked_add(2)?;
        if qclass_end > packet.len() {
            return None;
        }
        let qtype = u16::from_be_bytes(packet[qtype_start..qtype_end].try_into().ok()?);
        let qclass = u16::from_be_bytes(packet[qtype_end..qclass_end].try_into().ok()?);
        Some((
            Self {
                name,
                qtype,
                qclass,
            },
            qclass_end,
        ))
    }

    pub fn is_supported(&self) -> bool {
        self.qtype == 1 && self.qclass == 1
    }

    pub fn name(&self) -> &[u8] {
        &self.name
    }
}

pub fn build_dns_response(request: &[u8]) -> Option<Vec<u8>> {
    build_dns_response_with_config(request, default_answer_config())
}

fn build_dns_response_with_config(request: &[u8], config: AnswerConfig) -> Option<Vec<u8>> {
    let header = DnsHeaderRequest::parse(request).ok()?;
    if header.qdcount == 0 {
        return None;
    }
    let (question, questions_end_offset) = parse_question_section(request, header.qdcount)?;
    if !question.is_supported() {
        return None;
    }

    let question_bytes = &request[HEADER_LEN..questions_end_offset];
    let answer_bytes = build_answer_section(question.name(), &config);

    let mut header_response = DnsHeaderResponse::from_request(&header);
    header_response.ancount = 1;
    header_response.nscount = 0;
    header_response.arcount = 0;

    let mut response = Vec::with_capacity(HEADER_LEN + question_bytes.len() + answer_bytes.len());
    let mut header_buf = [0u8; HEADER_LEN];
    header_response.write_into(&mut header_buf);
    response.extend_from_slice(&header_buf);
    response.extend_from_slice(question_bytes);
    response.extend_from_slice(&answer_bytes);
    Some(response)
}

fn parse_question_section(packet: &[u8], qdcount: u16) -> Option<(DnsQuestion, usize)> {
    let mut offset = HEADER_LEN;
    let mut first_question: Option<DnsQuestion> = None;
    for idx in 0..qdcount {
        let (question, next_offset) = DnsQuestion::from_packet(packet, offset)?;
        if !question.is_supported() {
            return None;
        }
        if idx == 0 {
            first_question = Some(question.clone());
        }
        offset = next_offset;
    }
    first_question.map(|q| (q, offset))
}

fn read_qname(packet: &[u8], offset: usize) -> Option<(Vec<u8>, usize)> {
    let mut idx = offset;
    let mut total_len = 0usize;
    while idx < packet.len() {
        let len = *packet.get(idx)? as usize;
        idx += 1;
        if len == 0 {
            break;
        }
        if len > 63 {
            return None;
        }
        let end = idx.checked_add(len)?;
        total_len = total_len.checked_add(len + 1)?;
        if total_len > 255 {
            return None;
        }
        idx = end;
    }
    if idx > packet.len() {
        return None;
    }
    let name_end = idx;
    Some((packet[offset..name_end].to_vec(), idx))
}

fn build_answer_section(name: &[u8], config: &AnswerConfig) -> Vec<u8> {
    let mut buffer = BytesMut::with_capacity(name.len() + 16);
    buffer.put_slice(name);
    buffer.put_u16(1); // TYPE A
    buffer.put_u16(1); // CLASS IN
    buffer.put_u32(config.ttl_seconds);
    buffer.put_u16(4); // RDLENGTH
    buffer.put_slice(&config.ipv4);
    buffer.to_vec()
}
