#[derive(Clone, Copy, Debug)]
pub struct DnsHeaderResponse {
    bytes: [u8; 12],
}

impl DnsHeaderResponse {
    pub const fn new() -> Self {
        Self {
            bytes: [
                0x04, 0xD2, // ID = 1234
                0x80, 0x00, // QR=1, Opcode=0, AA=0, TC=0, RD=0, RA=0, Z=0, RCODE=0
                0x00, 0x00, // QDCOUNT = 0
                0x00, 0x00, // ANCOUNT = 0
                0x00, 0x00, // NSCOUNT = 0
                0x00, 0x00, // ARCOUNT = 0
            ],
        }
    }

    pub const fn bytes(&self) -> &[u8; 12] {
        &self.bytes
    }
}

pub const STANDARD_DNS_HEADER: DnsHeaderResponse = DnsHeaderResponse::new();
