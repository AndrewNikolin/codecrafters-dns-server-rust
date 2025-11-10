use bytes::{BufMut, BytesMut};
use std::collections::HashSet;
use std::convert::TryInto;

pub const HEADER_LEN: usize = 12;
const MAX_POINTER_HOPS: usize = 10;

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
            ancount: request.qdcount,
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
pub struct DecodedQuestion {
    pub name: Vec<u8>,
    pub qtype: u16,
    pub qclass: u16,
}

impl DecodedQuestion {
    pub fn from_packet(packet: &[u8], offset: usize) -> Option<(Self, usize)> {
        let (name, name_end) = decode_name(packet, offset)?;
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
}

pub fn build_dns_response(request: &[u8]) -> Option<Vec<u8>> {
    build_dns_response_with_config(request, default_answer_config())
}

fn build_dns_response_with_config(request: &[u8], config: AnswerConfig) -> Option<Vec<u8>> {
    let header = DnsHeaderRequest::parse(request).ok()?;
    if header.qdcount == 0 {
        return None;
    }
    let questions = parse_questions(request, header.qdcount)?;
    let question_bytes = serialize_questions(&questions);
    let answers_bytes = serialize_answers(&questions, &config);

    let header_response = DnsHeaderResponse::from_request(&header);

    let mut response = Vec::with_capacity(HEADER_LEN + question_bytes.len() + answers_bytes.len());
    let mut header_buf = [0u8; HEADER_LEN];
    header_response.write_into(&mut header_buf);
    response.extend_from_slice(&header_buf);
    response.extend_from_slice(&question_bytes);
    response.extend_from_slice(&answers_bytes);

    Some(response)
}

fn parse_questions(packet: &[u8], qdcount: u16) -> Option<Vec<DecodedQuestion>> {
    let mut offset = HEADER_LEN;
    let mut questions = Vec::with_capacity(qdcount as usize);
    for _ in 0..qdcount {
        let (question, next_offset) = DecodedQuestion::from_packet(packet, offset)?;
        if !question.is_supported() {
            return None;
        }
        questions.push(question);
        offset = next_offset;
    }
    Some(questions)
}

fn decode_name(packet: &[u8], offset: usize) -> Option<(Vec<u8>, usize)> {
    let mut cursor = offset;
    let mut consumed_offset: Option<usize> = None;
    let mut hops = 0;
    let mut visited_pointers = HashSet::new();
    let mut name = Vec::new();

    loop {
        if cursor >= packet.len() {
            return None;
        }
        let len = packet[cursor];
        if len & 0xC0 == 0xC0 {
            if cursor + 1 >= packet.len() {
                return None;
            }
            if consumed_offset.is_none() {
                consumed_offset = Some(cursor + 2);
            }
            let pointer = ((((len & 0x3F) as u16) << 8) | packet[cursor + 1] as u16) as usize;
            if pointer >= packet.len() {
                return None;
            }
            if !visited_pointers.insert(pointer) {
                return None;
            }
            hops += 1;
            if hops > MAX_POINTER_HOPS {
                return None;
            }
            cursor = pointer;
            continue;
        } else if len == 0 {
            if consumed_offset.is_none() {
                consumed_offset = Some(cursor + 1);
            }
            break;
        } else {
            let label_len = len as usize;
            let start = cursor + 1;
            let end = start.checked_add(label_len)?;
            if label_len == 0 || label_len > 63 || end > packet.len() {
                return None;
            }
            if name.len() + label_len + 1 > 255 {
                return None;
            }
            name.push(label_len as u8);
            name.extend_from_slice(&packet[start..end]);
            cursor = end;
            if consumed_offset.is_none() {
                consumed_offset = Some(cursor);
            }
        }
    }

    name.push(0);
    Some((name, consumed_offset.unwrap_or(cursor)))
}

fn serialize_questions(questions: &[DecodedQuestion]) -> Vec<u8> {
    let mut buffer = Vec::new();
    for question in questions {
        buffer.extend_from_slice(&question.name);
        buffer.extend_from_slice(&question.qtype.to_be_bytes());
        buffer.extend_from_slice(&question.qclass.to_be_bytes());
    }
    buffer
}

fn serialize_answers(questions: &[DecodedQuestion], config: &AnswerConfig) -> Vec<u8> {
    let mut buffer = BytesMut::with_capacity(questions.len() * 20);
    for question in questions {
        buffer.put_slice(&question.name);
        buffer.put_u16(1);
        buffer.put_u16(1);
        buffer.put_u32(config.ttl_seconds);
        buffer.put_u16(4);
        buffer.put_slice(&config.ipv4);
    }
    buffer.to_vec()
}
