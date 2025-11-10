use bytes::{BufMut, BytesMut};
use std::convert::TryInto;

pub const HEADER_LEN: usize = 12;

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
pub struct DnsHeaderRequest {
    pub id: u16,
    pub flags: u16,
    pub qdcount: u16,
    pub ancount: u16,
    pub nscount: u16,
    pub arcount: u16,
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
            ancount: u16::from_be_bytes([packet[6], packet[7]]),
            nscount: u16::from_be_bytes([packet[8], packet[9]]),
            arcount: u16::from_be_bytes([packet[10], packet[11]]),
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
            ancount: request.ancount,
            nscount: request.nscount,
            arcount: request.arcount,
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
        // AA, TC, RA, Z bits are intentionally left as zero
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
    _qtype: u16,
    qclass: u16,
}

impl DnsQuestion {
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
                _qtype: qtype,
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

pub fn build_dns_response(request: &[u8]) -> Option<Vec<u8>> {
    build_dns_response_with_config(request, default_answer_config())
}

fn build_dns_response_with_config(request: &[u8], config: AnswerConfig) -> Option<Vec<u8>> {
    let header = DnsHeaderRequest::parse(request).ok()?;
    let mut response = Vec::with_capacity(request.len() + 64);
    response.extend_from_slice(request);

    let mut header_response = DnsHeaderResponse::from_request(&header);

    if let Some(answer) = maybe_build_answer(request, &config) {
        if header_response.ancount == u16::MAX {
            header_response.ancount = u16::MAX;
        } else {
            header_response.ancount = header_response.ancount.saturating_add(1);
        }
        response.extend_from_slice(&answer);
    }

    header_response.write_into(&mut response[..HEADER_LEN]);
    Some(response)
}

fn maybe_build_answer(packet: &[u8], config: &AnswerConfig) -> Option<Vec<u8>> {
    if packet.len() <= HEADER_LEN {
        return None;
    }
    let (question, _) = DnsQuestion::from_packet(packet, HEADER_LEN)?;
    if question.matches_target(config) {
        Some(build_codecrafters_answer(config))
    } else {
        None
    }
}
