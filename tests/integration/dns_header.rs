use std::convert::TryInto;
use std::io;
use std::net::UdpSocket;
use std::process::{Child, Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Duration;

const SERVER_ADDR: &str = "127.0.0.1:2053";
const STARTUP_DELAY: Duration = Duration::from_millis(50);
const RESPONSE_TIMEOUT: Duration = Duration::from_millis(500);
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

pub fn skip_if_udp_forbidden() -> bool {
    UdpSocket::bind("127.0.0.1:0").is_err()
}

fn test_mutex() -> &'static Mutex<()> {
    static TEST_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();
    TEST_MUTEX.get_or_init(|| Mutex::new(()))
}

#[test]
fn returns_answer_section_for_codecrafters() {
    if skip_if_udp_forbidden() {
        eprintln!("Skipping UDP integration test: binding is not permitted in this environment");
        return;
    }

    let _guard = test_mutex().lock().expect("failed to acquire test mutex");
    let harness = TestHarness::spawn().expect("server failed to start");
    let query = build_query("codecrafters.io");
    let response = harness
        .send_probe(&query)
        .expect("failed to receive DNS response bytes");

    let ancount = u16::from_be_bytes([response[6], response[7]]);
    assert_eq!(ancount, 1, "expected a single answer record");

    let answer = parse_answer_record(&response).expect("missing answer record");
    assert_eq!(
        answer.name().as_deref(),
        Some("codecrafters.io"),
        "answer NAME should match queried domain"
    );
    assert_eq!(answer.rtype, 1, "answer TYPE should be A");
    assert_eq!(answer.rclass, 1, "answer CLASS should be IN");
    assert_eq!(
        answer.rdata.as_slice(),
        &[8, 8, 8, 8],
        "A record payload must return 8.8.8.8"
    );
}

#[test]
fn ttl_remains_constant_across_responses() {
    if skip_if_udp_forbidden() {
        eprintln!("Skipping UDP integration test: binding is not permitted in this environment");
        return;
    }

    let _guard = test_mutex().lock().expect("failed to acquire test mutex");
    let harness = TestHarness::spawn().expect("server failed to start");
    let query = build_query("codecrafters.io");

    let response_one = harness.send_probe(&query).expect("first response failed");
    let response_two = harness.send_probe(&query).expect("second response failed");

    let ttl_one = parse_answer_record(&response_one)
        .expect("missing answer")
        .ttl;
    let ttl_two = parse_answer_record(&response_two)
        .expect("missing answer")
        .ttl;
    assert_eq!(ttl_one, 60, "TTL should default to 60 seconds");
    assert_eq!(
        ttl_one, ttl_two,
        "TTL should not drift across consecutive responses"
    );
}

#[test]
fn non_target_queries_do_not_receive_answers() {
    if skip_if_udp_forbidden() {
        eprintln!("Skipping UDP integration test: binding is not permitted in this environment");
        return;
    }

    let _guard = test_mutex().lock().expect("failed to acquire test mutex");
    let harness = TestHarness::spawn().expect("server failed to start");
    let query = build_query("example.com");
    let response = harness
        .send_probe(&query)
        .expect("failed to receive DNS response bytes");

    let ancount = u16::from_be_bytes([response[6], response[7]]);
    assert_eq!(ancount, 0, "non-target queries must not include answers");
    assert!(
        parse_answer_record(&response).is_none(),
        "no answer bytes should be appended for other domains"
    );
}

fn build_query(domain: &str) -> Vec<u8> {
    let mut packet = Vec::new();
    packet.extend_from_slice(&[0x12, 0x34]); // ID
    packet.extend_from_slice(&[0x01, 0x00]); // standard flags
    packet.extend_from_slice(&[0x00, 0x01]); // QDCOUNT
    packet.extend_from_slice(&[0x00, 0x00]); // ANCOUNT
    packet.extend_from_slice(&[0x00, 0x00]); // NSCOUNT
    packet.extend_from_slice(&[0x00, 0x00]); // ARCOUNT
    packet.extend_from_slice(&encode_domain(domain));
    packet.extend_from_slice(&1u16.to_be_bytes()); // QTYPE A
    packet.extend_from_slice(&1u16.to_be_bytes()); // QCLASS IN
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
