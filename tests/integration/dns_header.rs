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

const EXPECTED_HEADER: [u8; 12] = [
    0x04, 0xD2, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

#[test]
fn responds_with_fixed_header() {
    if skip_if_udp_forbidden() {
        eprintln!("Skipping UDP integration test: binding is not permitted in this environment");
        return;
    }

    let _guard = test_mutex().lock().expect("failed to acquire test mutex");
    let harness = TestHarness::spawn().expect("server failed to start");
    let response = harness
        .send_probe(&[0xAA, 0xBB, 0xCC])
        .expect("failed to receive DNS header bytes");
    assert_eq!(response, EXPECTED_HEADER);
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
        .expect("failed to receive DNS header bytes");
    assert_eq!(response, EXPECTED_HEADER);
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
        .expect("failed to receive DNS header bytes");
    assert_eq!(response, EXPECTED_HEADER);
}
