use std::convert::TryInto;

pub const HEADER_LEN: usize = 12;
const MAX_POINTER_HOPS: usize = 10;

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
    pub rcode: u8,
}

impl DnsHeaderResponse {
    pub fn with_counts_and_rcode(request: &DnsHeaderRequest, ancount: u16, rcode: u8) -> Self {
        Self {
            id: request.id,
            opcode: request.opcode(),
            rd: request.rd(),
            qdcount: request.qdcount,
            ancount,
            nscount: 0,
            arcount: 0,
            rcode,
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
        flags |= (self.rcode & 0x0F) as u16;
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

pub fn parse_questions(packet: &[u8], qdcount: u16) -> Option<Vec<DecodedQuestion>> {
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
    let mut consumed: Option<usize> = None;
    let mut hops = 0;
    let mut labels: Vec<Vec<u8>> = Vec::new();

    loop {
        if cursor >= packet.len() {
            return None;
        }
        let len = packet[cursor];
        if len & 0xC0 == 0xC0 {
            if cursor + 1 >= packet.len() {
                return None;
            }
            if consumed.is_none() {
                consumed = Some(cursor + 2);
            }
            let pointer = ((((len & 0x3F) as u16) << 8) | packet[cursor + 1] as u16) as usize;
            if pointer >= packet.len() {
                return None;
            }
            hops += 1;
            if hops > MAX_POINTER_HOPS {
                return None;
            }
            cursor = pointer;
            continue;
        } else if len == 0 {
            cursor += 1;
            if consumed.is_none() {
                consumed = Some(cursor);
            }
            break;
        } else {
            let label_len = len as usize;
            let start = cursor + 1;
            let end = start.checked_add(label_len)?;
            if label_len == 0 || label_len > 63 || end > packet.len() {
                return None;
            }
            labels.push(packet[start..end].to_vec());
            cursor = end;
        }
    }

    let mut name = Vec::new();
    for label in labels {
        if name.len() + label.len() + 1 > 255 {
            return None;
        }
        name.push(label.len() as u8);
        name.extend_from_slice(&label);
    }
    name.push(0);

    Some((name, consumed.unwrap_or(cursor)))
}

pub fn encode_questions(questions: &[DecodedQuestion]) -> Vec<u8> {
    let mut buffer = Vec::new();
    for question in questions {
        buffer.extend_from_slice(&question.name);
        buffer.extend_from_slice(&question.qtype.to_be_bytes());
        buffer.extend_from_slice(&question.qclass.to_be_bytes());
    }
    buffer
}
