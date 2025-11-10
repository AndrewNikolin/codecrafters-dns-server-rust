use super::support::{ResolverBehavior, ServerHarness, HEADER_LEN};
use std::convert::TryInto;
use std::net::UdpSocket;
use std::sync::{Mutex, OnceLock};

#[test]
fn forwards_single_question_successfully() {
    if skip_if_udp_forbidden() {
        eprintln!("Skipping UDP integration test: binding not permitted");
        return;
    }

    let _guard = test_mutex().lock().expect("mutex poisoned");
    let harness =
        ServerHarness::spawn(ResolverBehavior::Answering).expect("server failed to start");

    let query = build_single_question_query(0xAA55, "codecrafters.io");
    let response = harness
        .send_probe(&query)
        .expect("failed to receive forwarded response");

    let header = parse_header(&response).expect("response header missing");
    assert_eq!(header.id, 0xAA55, "transaction ID must be preserved");
    assert_eq!(
        header.rcode, 0,
        "successful forward should retain upstream rcode"
    );
    assert_eq!(
        header.ancount, 1,
        "upstream returns exactly one answer per question"
    );

    let ip = parse_first_answer_ip(&response).expect("answer missing");
    assert_eq!(ip, [8, 8, 8, 8], "answer body must match upstream payload");
}

#[test]
fn returns_servfail_when_upstream_times_out() {
    if skip_if_udp_forbidden() {
        eprintln!("Skipping UDP integration test: binding not permitted");
        return;
    }

    let _guard = test_mutex().lock().expect("mutex poisoned");
    let harness = ServerHarness::spawn(ResolverBehavior::Silent).expect("server failed to start");

    let query = build_single_question_query(0x0F0F, "codecrafters.io");
    let response = harness
        .send_probe(&query)
        .expect("expected SERVFAIL response");

    let header = parse_header(&response).expect("response header missing");
    assert_eq!(header.id, 0x0F0F);
    assert_eq!(
        header.rcode, 2,
        "timeout should result in SERVFAIL (rcode=2)"
    );
    assert_eq!(
        header.ancount, 0,
        "SERVFAIL must not contain answer records"
    );
}

#[test]
fn splits_multi_question_packets_and_merges_answers() {
    if skip_if_udp_forbidden() {
        eprintln!("Skipping UDP integration test: binding not permitted");
        return;
    }

    let _guard = test_mutex().lock().expect("mutex poisoned");
    let harness = ServerHarness::spawn(ResolverBehavior::sequence(vec![
        [10, 0, 0, 1],
        [10, 0, 0, 2],
    ]))
    .expect("server failed to start");

    let packet =
        build_multi_question_query(0xBBBB, &["alpha.codecrafters.io", "beta.codecrafters.io"]);
    let response = harness
        .send_probe(&packet)
        .expect("failed to receive merged response");

    let header = parse_header(&response).expect("response header missing");
    assert_eq!(header.id, 0xBBBB);
    assert_eq!(header.ancount, 2, "should return one answer per question");

    let ips = parse_all_answer_ips(&response).expect("answers missing");
    assert_eq!(ips, vec![[10, 0, 0, 1], [10, 0, 0, 2]]);
    assert_eq!(
        harness.request_count(),
        2,
        "server should forward one request per question"
    );
}

fn build_single_question_query(id: u16, domain: &str) -> Vec<u8> {
    let mut packet = Vec::new();
    let flags = 0x0100u16; // RD=1
    packet.extend_from_slice(&id.to_be_bytes());
    packet.extend_from_slice(&flags.to_be_bytes());
    packet.extend_from_slice(&1u16.to_be_bytes()); // QDCOUNT
    packet.extend_from_slice(&0u16.to_be_bytes()); // ANCOUNT
    packet.extend_from_slice(&0u16.to_be_bytes()); // NSCOUNT
    packet.extend_from_slice(&0u16.to_be_bytes()); // ARCOUNT
    packet.extend_from_slice(&encode_domain(domain));
    packet.extend_from_slice(&1u16.to_be_bytes()); // QTYPE=A
    packet.extend_from_slice(&1u16.to_be_bytes()); // QCLASS=IN
    packet
}

fn build_multi_question_query(id: u16, domains: &[&str]) -> Vec<u8> {
    let mut packet = Vec::new();
    let flags = 0x0100u16;
    packet.extend_from_slice(&id.to_be_bytes());
    packet.extend_from_slice(&flags.to_be_bytes());
    packet.extend_from_slice(&(domains.len() as u16).to_be_bytes());
    packet.extend_from_slice(&0u16.to_be_bytes()); // ANCOUNT
    packet.extend_from_slice(&0u16.to_be_bytes()); // NSCOUNT
    packet.extend_from_slice(&0u16.to_be_bytes()); // ARCOUNT
    for domain in domains {
        packet.extend_from_slice(&encode_domain(domain));
        packet.extend_from_slice(&1u16.to_be_bytes());
        packet.extend_from_slice(&1u16.to_be_bytes());
    }
    packet
}

fn parse_header(packet: &[u8]) -> Option<HeaderView> {
    if packet.len() < HEADER_LEN {
        return None;
    }
    let id = u16::from_be_bytes([packet[0], packet[1]]);
    let flags = u16::from_be_bytes([packet[2], packet[3]]);
    let ancount = u16::from_be_bytes([packet[6], packet[7]]);
    let rcode = (flags & 0x000F) as u8;
    Some(HeaderView { id, ancount, rcode })
}

fn parse_first_answer_ip(packet: &[u8]) -> Option<[u8; 4]> {
    let question_end = question_end(packet)?;
    let mut idx = question_end;
    let name_len = qname_wire_length(&packet[idx..])?;
    idx += name_len;
    let type_end = idx.checked_add(2)?;
    let class_end = type_end.checked_add(2)?;
    let ttl_end = class_end.checked_add(4)?;
    let rdlength_end = ttl_end.checked_add(2)?;
    if rdlength_end > packet.len() {
        return None;
    }
    let rdlength = u16::from_be_bytes(packet[ttl_end..rdlength_end].try_into().ok()?) as usize;
    let rdata_end = rdlength_end.checked_add(rdlength)?;
    if rdata_end > packet.len() || rdlength != 4 {
        return None;
    }
    let mut ip = [0u8; 4];
    ip.copy_from_slice(&packet[rdlength_end..rdata_end]);
    Some(ip)
}

fn parse_all_answer_ips(packet: &[u8]) -> Option<Vec<[u8; 4]>> {
    let mut ips = Vec::new();
    let mut offset = question_end(packet)?;
    for _ in 0..u16::from_be_bytes([packet[6], packet[7]]) {
        let remaining = packet.get(offset..)?;
        let name_len = qname_wire_length(remaining)?;
        offset += name_len;
        let type_end = offset.checked_add(2)?;
        let class_end = type_end.checked_add(2)?;
        let ttl_end = class_end.checked_add(4)?;
        let rdlength_end = ttl_end.checked_add(2)?;
        if rdlength_end > packet.len() {
            return None;
        }
        let rdlength = u16::from_be_bytes(packet[ttl_end..rdlength_end].try_into().ok()?) as usize;
        let rdata_end = rdlength_end.checked_add(rdlength)?;
        if rdata_end > packet.len() || rdlength != 4 {
            return None;
        }
        let mut ip = [0u8; 4];
        ip.copy_from_slice(&packet[rdlength_end..rdata_end]);
        ips.push(ip);
        offset = rdata_end;
    }
    Some(ips)
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
    encoded.push(0);
    encoded
}

fn skip_if_udp_forbidden() -> bool {
    UdpSocket::bind("127.0.0.1:0").is_err()
}

fn test_mutex() -> &'static Mutex<()> {
    static TEST_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();
    TEST_MUTEX.get_or_init(|| Mutex::new(()))
}

struct HeaderView {
    id: u16,
    ancount: u16,
    rcode: u8,
}
