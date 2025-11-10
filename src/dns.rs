use bytes::{BufMut, BytesMut};
use std::convert::TryInto;

const HEADER_LEN: usize = 12;

#[derive(Clone, Copy, Debug)]
pub struct AnswerConfig {
    pub domain: &'static str,
    pub ipv4: [u8; 4],
    pub ttl_seconds: u32,
}

impl AnswerConfig {
    pub const fn new(domain: &'static str, ipv4: [u8; 4], ttl_seconds: u32) -> Self {
        Self {
            domain,
            ipv4,
            ttl_seconds,
        }
    }
}

pub const ANSWER_CONFIG: AnswerConfig = AnswerConfig::new("codecrafters.io", [8, 8, 8, 8], 60);

pub const fn default_answer_config() -> AnswerConfig {
    ANSWER_CONFIG
}

#[derive(Clone, Copy, Debug)]
pub struct DnsHeaderResponse {
    bytes: [u8; HEADER_LEN],
}

impl DnsHeaderResponse {
    pub const fn new_with_question_count(qdcount: u16) -> Self {
        let qd_high = (qdcount >> 8) as u8;
        let qd_low = (qdcount & 0x00FF) as u8;
        Self {
            bytes: [
                0x04, 0xD2, // ID = 1234
                0x80, 0x00, // QR=1, Opcode=0, AA=0, TC=0, RD=0, RA=0, Z=0, RCODE=0
                qd_high, qd_low, // QDCOUNT
                0x00, 0x00, // ANCOUNT = 0
                0x00, 0x00, // NSCOUNT = 0
                0x00, 0x00, // ARCOUNT = 0
            ],
        }
    }

    pub const fn bytes(&self) -> &[u8; HEADER_LEN] {
        &self.bytes
    }
}

pub const STANDARD_DNS_HEADER: DnsHeaderResponse = DnsHeaderResponse::new_with_question_count(1);

#[derive(Clone, Debug)]
pub struct DnsQuestion {
    name: Vec<u8>,
    qtype: u16,
    qclass: u16,
}

impl DnsQuestion {
    pub fn canonical_codecrafters() -> Self {
        Self {
            name: encode_labels(&["codecrafters", "io"]),
            qtype: 1,
            qclass: 1,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.name.clone();
        bytes.extend_from_slice(&self.qtype.to_be_bytes());
        bytes.extend_from_slice(&self.qclass.to_be_bytes());
        bytes
    }

    pub fn from_packet(packet: &[u8], offset: usize) -> Option<(Self, usize)> {
        let qname_len = qname_wire_length(&packet[offset..])?;
        let name_end = offset + qname_len;
        let qtype_start = name_end;
        let qtype_end = qtype_start.checked_add(2)?;
        let qclass_end = qtype_end.checked_add(2)?;
        if qclass_end > packet.len() {
            return None;
        }
        let name = packet[offset..name_end].to_vec();
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

    pub fn matches_target(&self, config: &AnswerConfig) -> bool {
        self.qclass == 1 && qname_matches_config(&self.name, config)
    }
}

pub fn encode_labels(labels: &[&str]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for label in labels {
        bytes.push(label.len() as u8);
        bytes.extend_from_slice(label.as_bytes());
    }
    bytes.push(0x00);
    bytes
}

pub fn normalize_qname(raw: &[u8]) -> Option<String> {
    let mut labels = Vec::new();
    let mut idx = 0;
    while idx < raw.len() {
        let len = *raw.get(idx)? as usize;
        idx += 1;
        if len == 0 {
            break;
        }
        let end = idx.checked_add(len)?;
        let label_bytes = raw.get(idx..end)?;
        let label = std::str::from_utf8(label_bytes).ok()?.to_ascii_lowercase();
        labels.push(label);
        idx = end;
    }
    if labels.is_empty() {
        None
    } else {
        Some(labels.join("."))
    }
}

pub fn qname_matches_config(raw: &[u8], config: &AnswerConfig) -> bool {
    normalize_qname(raw)
        .map(|name| name == config.domain)
        .unwrap_or(false)
}

fn qname_wire_length(bytes: &[u8]) -> Option<usize> {
    let mut idx = 0;
    while idx < bytes.len() {
        let len = *bytes.get(idx)? as usize;
        idx += 1;
        if len == 0 {
            return Some(idx);
        }
        idx = idx.checked_add(len)?;
    }
    None
}

fn encode_domain(domain: &str) -> Vec<u8> {
    let labels: Vec<&str> = domain.split('.').collect();
    encode_labels(&labels)
}

pub fn build_codecrafters_answer(config: &AnswerConfig) -> Vec<u8> {
    let mut buffer = BytesMut::with_capacity(32);
    let name = encode_domain(config.domain);
    buffer.put_slice(&name);
    buffer.put_u16(1); // TYPE A
    buffer.put_u16(1); // CLASS IN
    buffer.put_u32(config.ttl_seconds);
    buffer.put_u16(4); // RDLENGTH
    buffer.put_slice(&config.ipv4);
    buffer.to_vec()
}

pub fn build_dns_response(request: &[u8]) -> Vec<u8> {
    build_dns_response_with_config(request, default_answer_config())
        .unwrap_or_else(build_canonical_response_packet)
}

fn build_dns_response_with_config(request: &[u8], config: AnswerConfig) -> Option<Vec<u8>> {
    if request.len() < HEADER_LEN {
        return None;
    }
    let (question, _) = DnsQuestion::from_packet(request, HEADER_LEN)?;
    let should_answer = question.matches_target(&config);
    let answer_bytes = should_answer.then(|| build_codecrafters_answer(&config));
    let answer_len = answer_bytes.as_ref().map_or(0, |bytes| bytes.len());

    let mut response = Vec::with_capacity(request.len() + answer_len);
    response.extend_from_slice(request);
    response[2] |= 0x80; // Set QR bit to signal response

    let ancount = answer_bytes.as_ref().map_or(0u16, |_| 1u16);
    response[6..8].copy_from_slice(&ancount.to_be_bytes());
    response[8..10].copy_from_slice(&0u16.to_be_bytes());
    response[10..12].copy_from_slice(&0u16.to_be_bytes());

    if let Some(answer) = answer_bytes {
        response.extend_from_slice(&answer);
    }
    Some(response)
}

fn build_canonical_response_packet() -> Vec<u8> {
    let question = DnsQuestion::canonical_codecrafters();
    let question_bytes = question.to_bytes();
    let mut packet = Vec::with_capacity(HEADER_LEN + question_bytes.len());
    packet.extend_from_slice(STANDARD_DNS_HEADER.bytes());
    packet.extend_from_slice(&question_bytes);
    packet
}
