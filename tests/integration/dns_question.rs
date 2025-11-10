use std::convert::TryInto;
use std::io;
use std::net::UdpSocket;
use std::process::{Child, Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Duration;

const SERVER_ADDR: &str = "127.0.0.1:2053";
const STARTUP_DELAY: Duration = Duration::from_millis(50);
const RESPONSE_TIMEOUT: Duration = Duration::from_millis(400);
const HEADER_LEN: usize = 12;

pub struct TestHarness {
    child: Child,
}

impl TestHarness {
    pub fn spawn() -> io::Result<Self> {
        let child = Command::new(env!("CARGO_BIN_EXE_codecrafters-dns-server"))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        wait_for_port()?;
        Ok(Self { child })
    }

    pub fn send_probe(&self, payload: &[u8]) -> io::Result<Vec<u8>> {
        let socket = bind_client_socket()?;
        socket.send_to(payload, SERVER_ADDR)?;

        let mut buf = [0u8; 512];
        let (len, _) = socket.recv_from(&mut buf)?;
        Ok(buf[..len].to_vec())
    }

    pub fn expect_no_response(&self, payload: &[u8]) -> io::Result<()> {
        let socket = bind_client_socket()?;
        socket.send_to(payload, SERVER_ADDR)?;

        let mut buf = [0u8; 512];
        match socket.recv_from(&mut buf) {
            Ok((len, _)) => Err(io::Error::new(
                io::ErrorKind::Other,
                format!("expected drop but received {} bytes", len),
            )),
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => Ok(()),
            Err(err) => Err(err),
        }
    }
}

impl Drop for TestHarness {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn bind_client_socket() -> io::Result<UdpSocket> {
    let socket = UdpSocket::bind("127.0.0.1:0")?;
    socket.set_read_timeout(Some(RESPONSE_TIMEOUT))?;
    Ok(socket)
}

fn wait_for_port() -> io::Result<()> {
    thread::sleep(STARTUP_DELAY);
    Ok(())
}

fn test_mutex() -> &'static Mutex<()> {
    static TEST_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();
    TEST_MUTEX.get_or_init(|| Mutex::new(()))
}

fn skip_if_udp_forbidden() -> bool {
    UdpSocket::bind("127.0.0.1:0").is_err()
}

#[test]
fn mirrors_question_section() {
    if skip_if_udp_forbidden() {
        eprintln!("Skipping UDP integration test: binding is not permitted in this environment");
        return;
    }

    let _guard = test_mutex().lock().unwrap();
    let harness = TestHarness::spawn().expect("server failed to start");
    let domain = "example.test";
    let packet = build_query(vec![domain]);
    let response = harness
        .send_probe(&packet)
        .expect("failed to receive DNS response bytes");

    let question_len = question_section_len(&packet).expect("request question invalid");
    let response_question_len = question_section_len(&response).expect("response question invalid");
    assert_eq!(
        response_question_len, question_len,
        "response should include same number of questions"
    );
    assert_eq!(
        &response[HEADER_LEN..HEADER_LEN + question_len],
        &packet[HEADER_LEN..HEADER_LEN + question_len],
        "question bytes must mirror request"
    );
}

#[test]
fn returns_deterministic_answer() {
    if skip_if_udp_forbidden() {
        eprintln!("Skipping UDP integration test: binding is not permitted in this environment");
        return;
    }

    let _guard = test_mutex().lock().unwrap();
    let harness = TestHarness::spawn().expect("server failed to start");
    let packet = build_query(vec!["custom.domain"]);
    let response = harness
        .send_probe(&packet)
        .expect("failed to receive DNS response bytes");

    let question_len = question_section_len(&packet).expect("request invalid");
    let answer_offset = HEADER_LEN + question_len;
    let answer = parse_answer(&response[answer_offset..]).expect("missing answer");
    assert_eq!(answer.name, decode_labels("custom.domain"));
    assert_eq!(answer.rtype, 1);
    assert_eq!(answer.rclass, 1);
    assert_eq!(answer.ttl, 60);
    assert_eq!(answer.rdlength, 4);
    assert_eq!(answer.rdata.as_slice(), &[8, 8, 8, 8]);
}

#[test]
fn rejects_inconsistent_counts_and_unsupported_questions() {
    if skip_if_udp_forbidden() {
        eprintln!("Skipping UDP integration test: binding is not permitted in this environment");
        return;
    }

    let _guard = test_mutex().lock().unwrap();
    let harness = TestHarness::spawn().expect("server failed to start");

    // Header qdcount=2 but only one encoded question => expect drop.
    let mut invalid_packet = Vec::new();
    invalid_packet.extend_from_slice(&0xAAAAu16.to_be_bytes());
    invalid_packet.extend_from_slice(&0x0100u16.to_be_bytes());
    invalid_packet.extend_from_slice(&2u16.to_be_bytes()); // QDCOUNT
    invalid_packet.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    invalid_packet.extend_from_slice(&encode_labels(&["only", "one"]));
    invalid_packet.extend_from_slice(&1u16.to_be_bytes());
    invalid_packet.extend_from_slice(&1u16.to_be_bytes());

    harness
        .expect_no_response(&invalid_packet)
        .expect("server should drop inconsistent QDCOUNT packet");

    // Unsupported QTYPE should also be dropped.
    let mut unsupported = build_query(vec!["codecrafters.io"]);
    let question_start = HEADER_LEN;
    let question_len = question_section_len(&unsupported).unwrap();
    let qtype_pos = question_start + question_len - 4; // qtype before qclass
    unsupported[qtype_pos] = 0x00;
    unsupported[qtype_pos + 1] = 0x02; // TYPE=NS
    harness
        .expect_no_response(&unsupported)
        .expect("server should drop unsupported QTYPE packets");

    // Valid packet with QDCOUNT=2 should be answered.
    let packet = build_query(vec!["codecrafters.io", "example.com"]);
    let response = harness
        .send_probe(&packet)
        .expect("failed to receive DNS response bytes");
    let header = parse_header(&response).expect("response header missing");
    assert_eq!(header.qdcount, 2, "QDCOUNT must mirror request");
    assert_eq!(header.ancount, 1, "ANCOUNT must equal number of answers");
}

#[derive(Debug)]
struct AnswerView {
    name: Vec<u8>,
    rtype: u16,
    rclass: u16,
    ttl: u32,
    rdlength: u16,
    rdata: Vec<u8>,
}

fn parse_answer(bytes: &[u8]) -> Option<AnswerView> {
    if bytes.is_empty() {
        return None;
    }
    let (name, after_name) = read_qname(bytes, 0)?;
    let type_start = after_name;
    let type_end = type_start.checked_add(2)?;
    let class_end = type_end.checked_add(2)?;
    let ttl_end = class_end.checked_add(4)?;
    let rdlength_end = ttl_end.checked_add(2)?;
    if rdlength_end > bytes.len() {
        return None;
    }
    let rdlength = u16::from_be_bytes(bytes[ttl_end..rdlength_end].try_into().ok()?);
    let data_end = rdlength_end.checked_add(rdlength as usize)?;
    if data_end > bytes.len() {
        return None;
    }
    Some(AnswerView {
        name,
        rtype: u16::from_be_bytes(bytes[type_start..type_end].try_into().ok()?),
        rclass: u16::from_be_bytes(bytes[type_end..class_end].try_into().ok()?),
        ttl: u32::from_be_bytes(bytes[class_end..ttl_end].try_into().ok()?),
        rdlength,
        rdata: bytes[rdlength_end..data_end].to_vec(),
    })
}

#[derive(Debug)]
struct HeaderView {
    qdcount: u16,
    ancount: u16,
}

fn parse_header(packet: &[u8]) -> Option<HeaderView> {
    if packet.len() < HEADER_LEN {
        return None;
    }
    Some(HeaderView {
        qdcount: u16::from_be_bytes(packet[4..6].try_into().ok()?),
        ancount: u16::from_be_bytes(packet[6..8].try_into().ok()?),
    })
}

fn question_section_len(packet: &[u8]) -> Option<usize> {
    let qdcount = u16::from_be_bytes(packet[4..6].try_into().ok()?);
    let mut offset = HEADER_LEN;
    for _ in 0..qdcount {
        let (_, next) = read_qname(packet, offset)?;
        let qtype_end = next.checked_add(2)?;
        let qclass_end = qtype_end.checked_add(2)?;
        if qclass_end > packet.len() {
            return None;
        }
        offset = qclass_end;
    }
    Some(offset - HEADER_LEN)
}

fn read_qname(packet: &[u8], offset: usize) -> Option<(Vec<u8>, usize)> {
    let mut idx = offset;
    let mut total = 0usize;
    while idx < packet.len() {
        let len = *packet.get(idx)? as usize;
        idx += 1;
        if len == 0 {
            break;
        }
        let end = idx.checked_add(len)?;
        total = total.checked_add(len + 1)?;
        idx = end;
    }
    if idx > packet.len() {
        return None;
    }
    Some((packet[offset..idx].to_vec(), idx))
}

fn build_query(domains: Vec<&str>) -> Vec<u8> {
    let mut packet = Vec::new();
    packet.extend_from_slice(&0x1234u16.to_be_bytes());
    packet.extend_from_slice(&0x0100u16.to_be_bytes());
    packet.extend_from_slice(&(domains.len() as u16).to_be_bytes());
    packet.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    for domain in domains {
        packet.extend_from_slice(&encode_domain(domain));
        packet.extend_from_slice(&1u16.to_be_bytes());
        packet.extend_from_slice(&1u16.to_be_bytes());
    }
    packet
}

fn encode_domain(domain: &str) -> Vec<u8> {
    let labels: Vec<&str> = domain.split('.').collect();
    encode_labels(&labels)
}

fn encode_labels(labels: &[&str]) -> Vec<u8> {
    let mut out = Vec::new();
    for label in labels {
        out.push(label.len() as u8);
        out.extend_from_slice(label.as_bytes());
    }
    out.push(0);
    out
}

fn decode_labels(domain: &str) -> Vec<u8> {
    encode_domain(domain)
}
