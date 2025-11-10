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

fn skip_if_udp_forbidden() -> bool {
    UdpSocket::bind("127.0.0.1:0").is_err()
}

fn test_mutex() -> &'static Mutex<()> {
    static TEST_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();
    TEST_MUTEX.get_or_init(|| Mutex::new(()))
}

#[test]
fn parses_compressed_questions() {
    if skip_if_udp_forbidden() {
        eprintln!("Skipping UDP integration test: binding is not permitted in this environment");
        return;
    }

    let _guard = test_mutex().lock().unwrap();
    let harness = TestHarness::spawn().expect("server failed to start");

    let packet =
        build_packet_with_specs(&[("codecrafters.io", None), ("codecrafters.io", Some(0))]);
    let response = harness
        .send_probe(&packet)
        .expect("failed to receive DNS response bytes");

    let names = parse_uncompressed_questions(&response).expect("response questions invalid");
    assert_eq!(names.len(), 2);
    assert_eq!(names[0], encode_domain("codecrafters.io"));
    assert_eq!(names[1], encode_domain("codecrafters.io"));
}

#[test]
fn mirrors_questions_uncompressed() {
    if skip_if_udp_forbidden() {
        eprintln!("Skipping UDP integration test: binding is not permitted in this environment");
        return;
    }

    let _guard = test_mutex().lock().unwrap();
    let harness = TestHarness::spawn().expect("server failed to start");

    let packet = build_packet_with_specs(&[
        ("alpha.codecrafters.io", None),
        ("alpha.codecrafters.io", Some(0)),
    ]);
    let response = harness
        .send_probe(&packet)
        .expect("failed to receive DNS response bytes");

    let question_bytes = extract_question_bytes(&response).expect("could not isolate questions");
    assert!(
        !question_bytes.iter().any(|byte| byte & 0xC0 == 0xC0),
        "response question section should be uncompressed"
    );
}

#[test]
fn answers_each_question() {
    if skip_if_udp_forbidden() {
        eprintln!("Skipping UDP integration test: binding is not permitted in this environment");
        return;
    }

    let _guard = test_mutex().lock().unwrap();
    let harness = TestHarness::spawn().expect("server failed to start");

    let packet = build_packet_with_specs(&[
        ("one.codecrafters.io", None),
        ("two.codecrafters.io", None),
        ("one.codecrafters.io", Some(0)),
    ]);
    let response = harness
        .send_probe(&packet)
        .expect("failed to receive DNS response bytes");

    let header = parse_header(&response).expect("header invalid");
    assert_eq!(header.qdcount, 3);
    assert_eq!(header.ancount, 3);

    let names = parse_uncompressed_questions(&response).expect("questions invalid");
    let answers = parse_answers(&response).expect("answers invalid");
    assert_eq!(answers.len(), 3);
    for (answer, expected_name) in answers.iter().zip(names.iter()) {
        assert_eq!(answer.name, *expected_name);
        assert_eq!(answer.rtype, 1);
        assert_eq!(answer.rclass, 1);
        assert_eq!(answer.ttl, 60);
        assert_eq!(answer.rdlength, 4);
        assert_eq!(answer.rdata.as_slice(), &[8, 8, 8, 8]);
    }
}

#[test]
fn drops_invalid_pointers() {
    if skip_if_udp_forbidden() {
        eprintln!("Skipping UDP integration test: binding is not permitted in this environment");
        return;
    }

    let _guard = test_mutex().lock().unwrap();
    let harness = TestHarness::spawn().expect("server failed to start");

    let out_of_range = build_invalid_pointer_packet(0x3FF0);
    harness
        .expect_no_response(&out_of_range)
        .expect("server should drop packets with out-of-range pointer");

    let loop_packet = build_pointer_loop_packet();
    harness
        .expect_no_response(&loop_packet)
        .expect("server should drop packets with pointer loops");
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

fn parse_uncompressed_questions(packet: &[u8]) -> Option<Vec<Vec<u8>>> {
    let qdcount = u16::from_be_bytes(packet[4..6].try_into().ok()?);
    let mut offset = HEADER_LEN;
    let mut names = Vec::with_capacity(qdcount as usize);
    for _ in 0..qdcount {
        let (name, next) = read_qname(packet, offset)?;
        let qtype_end = next.checked_add(2)?;
        let qclass_end = qtype_end.checked_add(2)?;
        if qclass_end > packet.len() {
            return None;
        }
        names.push(name);
        offset = qclass_end;
    }
    Some(names)
}

fn parse_answers(packet: &[u8]) -> Option<Vec<AnswerView>> {
    let header = parse_header(packet)?;
    let mut offset = HEADER_LEN;
    for _ in 0..header.qdcount {
        let (_, next) = read_qname(packet, offset)?;
        let qtype_end = next.checked_add(2)?;
        let qclass_end = qtype_end.checked_add(2)?;
        offset = qclass_end;
    }
    let mut answers = Vec::with_capacity(header.ancount as usize);
    for _ in 0..header.ancount {
        let (name, next) = read_qname(packet, offset)?;
        let type_start = next;
        let type_end = type_start.checked_add(2)?;
        let class_end = type_end.checked_add(2)?;
        let ttl_end = class_end.checked_add(4)?;
        let rdlength_end = ttl_end.checked_add(2)?;
        if rdlength_end > packet.len() {
            return None;
        }
        let rdlength = u16::from_be_bytes(packet[ttl_end..rdlength_end].try_into().ok()?);
        let rdata_end = rdlength_end.checked_add(rdlength as usize)?;
        if rdata_end > packet.len() {
            return None;
        }
        answers.push(AnswerView {
            name,
            rtype: u16::from_be_bytes(packet[type_start..type_end].try_into().ok()?),
            rclass: u16::from_be_bytes(packet[type_end..class_end].try_into().ok()?),
            ttl: u32::from_be_bytes(packet[class_end..ttl_end].try_into().ok()?),
            rdlength,
            rdata: packet[rdlength_end..rdata_end].to_vec(),
        });
        offset = rdata_end;
    }
    Some(answers)
}

fn extract_question_bytes(packet: &[u8]) -> Option<Vec<u8>> {
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
    Some(packet[HEADER_LEN..offset].to_vec())
}

fn read_qname(packet: &[u8], offset: usize) -> Option<(Vec<u8>, usize)> {
    let mut idx = offset;
    while idx < packet.len() {
        let len = *packet.get(idx)? as usize;
        idx += 1;
        if len == 0 {
            break;
        }
        let end = idx.checked_add(len)?;
        if end > packet.len() {
            return None;
        }
        idx = end;
    }
    if idx > packet.len() {
        return None;
    }
    Some((packet[offset..idx].to_vec(), idx))
}

fn build_packet_with_specs(specs: &[(&str, Option<usize>)]) -> Vec<u8> {
    let mut packet = Vec::new();
    packet.extend_from_slice(&0x4321u16.to_be_bytes());
    packet.extend_from_slice(&0x0100u16.to_be_bytes());
    packet.extend_from_slice(&(specs.len() as u16).to_be_bytes());
    packet.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

    let mut question_offsets = Vec::with_capacity(specs.len());
    let mut cursor = HEADER_LEN;
    for (idx, (domain, pointer_to)) in specs.iter().enumerate() {
        question_offsets.push(cursor);
        match pointer_to {
            Some(target_idx) => {
                let target_offset = question_offsets[*target_idx] as u16;
                let pointer_val = 0xC000 | (target_offset & 0x3FFF);
                packet.extend_from_slice(&pointer_val.to_be_bytes());
                packet.extend_from_slice(&1u16.to_be_bytes());
                packet.extend_from_slice(&1u16.to_be_bytes());
                cursor += 2 + 2 + 2;
            }
            None => {
                let encoded = encode_domain(domain);
                packet.extend_from_slice(&encoded);
                packet.extend_from_slice(&1u16.to_be_bytes());
                packet.extend_from_slice(&1u16.to_be_bytes());
                cursor += encoded.len() + 4;
            }
        }
        debug_assert_eq!(question_offsets.len(), idx + 1);
    }

    packet
}

fn build_invalid_pointer_packet(pointer: u16) -> Vec<u8> {
    let mut packet = Vec::new();
    packet.extend_from_slice(&0x9999u16.to_be_bytes());
    packet.extend_from_slice(&0x0100u16.to_be_bytes());
    packet.extend_from_slice(&1u16.to_be_bytes());
    packet.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    let pointer_val = 0xC000 | (pointer & 0x3FFF);
    packet.extend_from_slice(&pointer_val.to_be_bytes());
    packet.extend_from_slice(&1u16.to_be_bytes());
    packet.extend_from_slice(&1u16.to_be_bytes());
    packet
}

fn build_pointer_loop_packet() -> Vec<u8> {
    let mut packet = Vec::new();
    packet.extend_from_slice(&0xAAAAu16.to_be_bytes());
    packet.extend_from_slice(&0x0100u16.to_be_bytes());
    packet.extend_from_slice(&1u16.to_be_bytes());
    packet.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    let pointer_offset = HEADER_LEN as u16;
    let pointer_val = 0xC000 | (pointer_offset & 0x3FFF);
    packet.extend_from_slice(&pointer_val.to_be_bytes());
    packet.extend_from_slice(&1u16.to_be_bytes());
    packet.extend_from_slice(&1u16.to_be_bytes());
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
