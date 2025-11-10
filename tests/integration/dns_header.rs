use super::support::{ResolverBehavior, ServerHarness, HEADER_LEN};
use std::convert::TryInto;
use std::net::UdpSocket;
use std::sync::{Mutex, OnceLock};

pub fn skip_if_udp_forbidden() -> bool {
    UdpSocket::bind("127.0.0.1:0").is_err()
}

fn test_mutex() -> &'static Mutex<()> {
    static TEST_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();
    TEST_MUTEX.get_or_init(|| Mutex::new(()))
}

#[test]
fn echoes_transaction_id_and_counts() {
    if skip_if_udp_forbidden() {
        eprintln!("Skipping UDP integration test: binding is not permitted in this environment");
        return;
    }

    let _guard = test_mutex().lock().expect("failed to acquire test mutex");
    let harness =
        ServerHarness::spawn(ResolverBehavior::Answering).expect("server failed to start");
    let packet = build_query_with_counts(0xBEEF, 0x0100, 2, 3, 4, 5, "codecrafters.io");
    let response = harness
        .send_probe(&packet)
        .expect("failed to receive DNS response bytes");

    let header = parse_header(&response).expect("response header missing");
    assert_eq!(header.id, 0xBEEF);
    assert_eq!(header.qdcount, 2);
    assert_eq!(
        header.ancount, 3,
        "counts should mirror incoming values at this stage"
    );
    assert_eq!(header.nscount, 4);
    assert_eq!(header.arcount, 5);
    assert_eq!(header.flags & 0x8000, 0x8000, "QR bit must be set");
}

#[test]
fn mirrors_opcode_and_rd_flags() {
    if skip_if_udp_forbidden() {
        eprintln!("Skipping UDP integration test: binding is not permitted in this environment");
        return;
    }

    let _guard = test_mutex().lock().expect("failed to acquire test mutex");
    let harness =
        ServerHarness::spawn(ResolverBehavior::Answering).expect("server failed to start");
    let opcode: u8 = 0b0011;
    let flags = ((opcode as u16) << 11) | 0x0100; // OPCODE=3, RD=1
    let packet = build_query_with_counts(0x4444, flags, 1, 0, 0, 0, "codecrafters.io");
    let response = harness
        .send_probe(&packet)
        .expect("failed to receive DNS response bytes");

    let header = parse_header(&response).expect("response header missing");
    let mirrored_opcode = ((header.flags & 0x7800) >> 11) as u8;
    assert_eq!(mirrored_opcode, opcode, "OPCODE must be mirrored");
    assert_eq!(header.flags & 0x0100, 0x0100, "RD flag must be mirrored");
    assert_eq!(header.flags & 0x0400, 0, "AA must be forced to 0");
    assert_eq!(header.flags & 0x0200, 0, "TC must be forced to 0");
    assert_eq!(header.flags & 0x0080, 0, "RA must be forced to 0");
}

#[test]
fn sets_rcode_based_on_opcode() {
    if skip_if_udp_forbidden() {
        eprintln!("Skipping UDP integration test: binding is not permitted in this environment");
        return;
    }

    let _guard = test_mutex().lock().expect("failed to acquire test mutex");
    let harness =
        ServerHarness::spawn(ResolverBehavior::Answering).expect("server failed to start");

    // Standard query (OPCODE 0) should yield RCODE 0.
    let standard_packet = build_query_with_counts(0x2222, 0x0100, 1, 0, 0, 0, "codecrafters.io");
    let standard_response = harness
        .send_probe(&standard_packet)
        .expect("failed to receive DNS response bytes");
    let standard_header = parse_header(&standard_response).expect("response header missing");
    assert_eq!(
        standard_header.flags & 0x000F,
        0,
        "RCODE must be 0 for OPCODE 0"
    );

    // Non-standard OPCODE should yield RCODE 4.
    let opcode: u8 = 0b0010;
    let exotic_flags = (opcode as u16) << 11;
    let exotic_packet =
        build_query_with_counts(0x3333, exotic_flags, 1, 0, 0, 0, "codecrafters.io");
    let exotic_response = harness
        .send_probe(&exotic_packet)
        .expect("failed to receive DNS response bytes");
    let exotic_header = parse_header(&exotic_response).expect("response header missing");
    assert_eq!(
        exotic_header.flags & 0x000F,
        4,
        "RCODE must be 4 for unsupported opcodes"
    );
}

#[test]
fn drops_packets_shorter_than_header() {
    if skip_if_udp_forbidden() {
        eprintln!("Skipping UDP integration test: binding is not permitted in this environment");
        return;
    }
    let _guard = test_mutex().lock().expect("failed to acquire test mutex");
    let harness =
        ServerHarness::spawn(ResolverBehavior::Answering).expect("server failed to start");
    let truncated = vec![0x01, 0x02, 0x03];
    harness
        .expect_no_response(&truncated)
        .expect("server should drop malformed packets without responding");
}

#[test]
fn still_returns_answer_section_for_codecrafters() {
    if skip_if_udp_forbidden() {
        eprintln!("Skipping UDP integration test: binding is not permitted in this environment");
        return;
    }

    let _guard = test_mutex().lock().expect("failed to acquire test mutex");
    let harness =
        ServerHarness::spawn(ResolverBehavior::Answering).expect("server failed to start");
    let query = build_query_with_counts(0x7777, 0x0100, 1, 0, 0, 0, "codecrafters.io");
    let response = harness
        .send_probe(&query)
        .expect("failed to receive DNS response bytes");

    let answer = parse_answer_record(&response).expect("missing answer record");
    assert_eq!(answer.name().as_deref(), Some("codecrafters.io"));
    assert_eq!(answer.rdata.as_slice(), &[8, 8, 8, 8]);
}

#[derive(Debug)]
struct HeaderView {
    id: u16,
    flags: u16,
    qdcount: u16,
    ancount: u16,
    nscount: u16,
    arcount: u16,
}

fn parse_header(packet: &[u8]) -> Option<HeaderView> {
    if packet.len() < HEADER_LEN {
        return None;
    }
    Some(HeaderView {
        id: u16::from_be_bytes(packet[0..2].try_into().ok()?),
        flags: u16::from_be_bytes(packet[2..4].try_into().ok()?),
        qdcount: u16::from_be_bytes(packet[4..6].try_into().ok()?),
        ancount: u16::from_be_bytes(packet[6..8].try_into().ok()?),
        nscount: u16::from_be_bytes(packet[8..10].try_into().ok()?),
        arcount: u16::from_be_bytes(packet[10..12].try_into().ok()?),
    })
}

fn build_query_with_counts(
    id: u16,
    flags: u16,
    qdcount: u16,
    ancount: u16,
    nscount: u16,
    arcount: u16,
    domain: &str,
) -> Vec<u8> {
    let mut packet = Vec::new();
    packet.extend_from_slice(&id.to_be_bytes());
    packet.extend_from_slice(&flags.to_be_bytes());
    packet.extend_from_slice(&qdcount.to_be_bytes());
    packet.extend_from_slice(&ancount.to_be_bytes());
    packet.extend_from_slice(&nscount.to_be_bytes());
    packet.extend_from_slice(&arcount.to_be_bytes());
    packet.extend_from_slice(&encode_domain(domain));
    packet.extend_from_slice(&1u16.to_be_bytes());
    packet.extend_from_slice(&1u16.to_be_bytes());
    packet
}

fn parse_answer_record(response: &[u8]) -> Option<AnswerRecord> {
    if response.len() <= HEADER_LEN {
        return None;
    }
    let question_end = question_end(response)?;
    if question_end >= response.len() {
        return None;
    }

    let name_len = qname_wire_length(&response[question_end..])?;
    let name_end = question_end + name_len;
    let type_start = name_end;
    let type_end = type_start.checked_add(2)?;
    let class_end = type_end.checked_add(2)?;
    let ttl_end = class_end.checked_add(4)?;
    let rdlength_end = ttl_end.checked_add(2)?;
    if rdlength_end > response.len() {
        return None;
    }
    let rdlength = u16::from_be_bytes(response[ttl_end..rdlength_end].try_into().ok()?) as usize;
    let rdata_end = rdlength_end.checked_add(rdlength)?;
    if rdata_end > response.len() {
        return None;
    }

    Some(AnswerRecord {
        name: response[question_end..name_end].to_vec(),
        rtype: u16::from_be_bytes(response[type_start..type_end].try_into().ok()?),
        rclass: u16::from_be_bytes(response[type_end..class_end].try_into().ok()?),
        ttl: u32::from_be_bytes(response[class_end..ttl_end].try_into().ok()?),
        rdata: response[rdlength_end..rdata_end].to_vec(),
    })
}

fn question_end(packet: &[u8]) -> Option<usize> {
    if packet.len() <= HEADER_LEN {
        return None;
    }
    let mut idx = HEADER_LEN;
    while idx < packet.len() {
        let len = *packet.get(idx)? as usize;
        idx += 1;
        if len == 0 {
            break;
        }
        idx = idx.checked_add(len)?;
    }
    let qtype_end = idx.checked_add(2)?;
    let qclass_end = qtype_end.checked_add(2)?;
    if qclass_end <= packet.len() {
        Some(qclass_end)
    } else {
        None
    }
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
    let mut encoded = Vec::new();
    for label in domain.split('.') {
        encoded.push(label.len() as u8);
        encoded.extend_from_slice(label.as_bytes());
    }
    encoded.push(0x00);
    encoded
}

#[allow(dead_code)]
struct AnswerRecord {
    name: Vec<u8>,
    rtype: u16,
    rclass: u16,
    ttl: u32,
    rdata: Vec<u8>,
}

impl AnswerRecord {
    fn name(&self) -> Option<String> {
        normalize_qname(&self.name)
    }
}

fn normalize_qname(raw: &[u8]) -> Option<String> {
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
