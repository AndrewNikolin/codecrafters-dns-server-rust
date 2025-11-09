const HEADER_LEN: usize = 12;

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

pub fn build_response_packet() -> Vec<u8> {
    let question = DnsQuestion::canonical_codecrafters();
    let question_bytes = question.to_bytes();
    let mut packet = Vec::with_capacity(HEADER_LEN + question_bytes.len());
    packet.extend_from_slice(STANDARD_DNS_HEADER.bytes());
    packet.extend_from_slice(&question_bytes);
    packet
}
