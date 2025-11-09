use std::io;
use std::net::UdpSocket;
use std::process::{Child, Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Duration;

const SERVER_ADDR: &str = "127.0.0.1:2053";
const STARTUP_DELAY: Duration = Duration::from_millis(50);
const RESPONSE_TIMEOUT: Duration = Duration::from_millis(500);

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
    match UdpSocket::bind("127.0.0.1:0") {
        Ok(_) => false,
        Err(_) => true,
    }
}

fn test_mutex() -> &'static Mutex<()> {
    static TEST_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();
    TEST_MUTEX.get_or_init(|| Mutex::new(()))
}

const HEADER_LEN: usize = 12;
const EXPECTED_HEADER: [u8; HEADER_LEN] = [
    0x04, 0xD2, 0x80, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
const EXPECTED_QUESTION: [u8; 21] = [
    0x0c, b'c', b'o', b'd', b'e', b'c', b'r', b'a', b'f', b't', b'e', b'r', b's', 0x02, b'i', b'o',
    0x00, 0x00, 0x01, 0x00, 0x01,
];

fn assert_canonical_response(response: &[u8]) {
    assert!(
        response.len() >= HEADER_LEN,
        "response too short: expected at least {} bytes, got {}",
        HEADER_LEN,
        response.len()
    );
    let (header, rest) = response.split_at(HEADER_LEN);
    assert_eq!(
        header, EXPECTED_HEADER,
        "header bytes differed from expected canonical header"
    );
    assert_eq!(
        rest, EXPECTED_QUESTION,
        "question bytes differed from canonical codecrafters.io question"
    );
}

#[test]
fn responds_with_question_section() {
    if skip_if_udp_forbidden() {
        eprintln!("Skipping UDP integration test: binding is not permitted in this environment");
        return;
    }

    let _guard = test_mutex().lock().expect("failed to acquire test mutex");
    let harness = TestHarness::spawn().expect("server failed to start");
    let response = harness
        .send_probe(&[0xAA, 0xBB, 0xCC])
        .expect("failed to receive DNS response bytes");
    assert_canonical_response(&response);
}

#[test]
fn responds_to_empty_payload() {
    if skip_if_udp_forbidden() {
        eprintln!("Skipping UDP integration test: binding is not permitted in this environment");
        return;
    }

    let _guard = test_mutex().lock().expect("failed to acquire test mutex");
    let harness = TestHarness::spawn().expect("server failed to start");
    let response = harness
        .send_probe(&[])
        .expect("failed to receive DNS response bytes");
    assert_canonical_response(&response);
}

#[test]
fn responds_to_large_payload() {
    if skip_if_udp_forbidden() {
        eprintln!("Skipping UDP integration test: binding is not permitted in this environment");
        return;
    }

    let _guard = test_mutex().lock().expect("failed to acquire test mutex");
    let payload = vec![0u8; 600];
    let harness = TestHarness::spawn().expect("server failed to start");
    let response = harness
        .send_probe(&payload)
        .expect("failed to receive DNS response bytes");
    assert_canonical_response(&response);
}

#[test]
fn responds_with_canonical_question_for_alternate_domain() {
    if skip_if_udp_forbidden() {
        eprintln!("Skipping UDP integration test: binding is not permitted in this environment");
        return;
    }

    let _guard = test_mutex().lock().expect("failed to acquire test mutex");
    // Fake payload that resembles a different domain question (e.g., example.com)
    let payload = [
        0x07, b'e', b'x', b'a', b'm', b'p', b'l', b'e', 0x03, b'c', b'o', b'm', 0x00, 0x00, 0x01,
        0x00, 0x01,
    ];
    let harness = TestHarness::spawn().expect("server failed to start");
    let response = harness
        .send_probe(&payload)
        .expect("failed to receive DNS response bytes");
    assert_canonical_response(&response);
}
