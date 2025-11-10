use std::collections::VecDeque;
use std::convert::TryInto;
use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub const SERVER_ADDR: &str = "127.0.0.1:2053";
pub const STARTUP_DELAY: Duration = Duration::from_millis(60);
pub const RESPONSE_TIMEOUT: Duration = Duration::from_millis(400);
pub const HEADER_LEN: usize = 12;

pub struct ServerHarness {
    child: Child,
    resolver: MockResolver,
}

impl ServerHarness {
    pub fn spawn(behavior: ResolverBehavior) -> io::Result<Self> {
        let resolver = MockResolver::start(behavior)?;
        let child = Command::new(env!("CARGO_BIN_EXE_codecrafters-dns-server"))
            .arg("--resolver")
            .arg(resolver.addr.to_string())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        thread::sleep(STARTUP_DELAY);
        Ok(Self { child, resolver })
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
                format!("expected no response but received {len} bytes"),
            )),
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => Ok(()),
            Err(err) => Err(err),
        }
    }

    pub fn request_count(&self) -> usize {
        self.resolver.request_count()
    }
}

impl Drop for ServerHarness {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub struct MockResolver {
    pub addr: SocketAddr,
    shutdown: Arc<AtomicBool>,
    requests: Arc<AtomicUsize>,
    thread: Option<thread::JoinHandle<()>>,
}

impl MockResolver {
    fn start(behavior: ResolverBehavior) -> io::Result<Self> {
        let socket = UdpSocket::bind("127.0.0.1:0")?;
        socket.set_read_timeout(Some(Duration::from_millis(50)))?;
        let addr = socket.local_addr()?;
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_flag = shutdown.clone();
        let requests = Arc::new(AtomicUsize::new(0));
        let request_counter = requests.clone();

        let mut behavior_state = behavior.into_state();

        let thread = thread::spawn(move || {
            let mut buf = [0u8; 512];
            while !shutdown_flag.load(Ordering::Relaxed) {
                match socket.recv_from(&mut buf) {
                    Ok((len, peer)) => {
                        request_counter.fetch_add(1, Ordering::Relaxed);
                        if let Some(response) = behavior_state.handle_packet(&buf[..len]) {
                            let _ = socket.send_to(&response, peer);
                        }
                    }
                    Err(err)
                        if err.kind() == io::ErrorKind::WouldBlock
                            || err.kind() == io::ErrorKind::TimedOut =>
                    {
                        continue;
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            addr,
            shutdown,
            requests,
            thread: Some(thread),
        })
    }

    pub fn request_count(&self) -> usize {
        self.requests.load(Ordering::Relaxed)
    }
}

impl Drop for MockResolver {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        let _ = UdpSocket::bind("127.0.0.1:0").and_then(|socket| socket.send_to(&[0u8], self.addr));
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}

pub enum ResolverBehavior {
    Answering,
    Silent,
    Sequence(Vec<[u8; 4]>),
}

impl ResolverBehavior {
    pub fn sequence<I: Into<Vec<[u8; 4]>>>(ips: I) -> Self {
        Self::Sequence(ips.into())
    }

    fn into_state(self) -> ResolverState {
        match self {
            ResolverBehavior::Answering => ResolverState::Answering,
            ResolverBehavior::Silent => ResolverState::Silent,
            ResolverBehavior::Sequence(list) => ResolverState::Sequence(VecDeque::from(list)),
        }
    }
}

enum ResolverState {
    Answering,
    Silent,
    Sequence(VecDeque<[u8; 4]>),
}

impl ResolverState {
    fn handle_packet(&mut self, packet: &[u8]) -> Option<Vec<u8>> {
        match self {
            ResolverState::Silent => None,
            ResolverState::Answering => build_response(packet, [8, 8, 8, 8]),
            ResolverState::Sequence(queue) => {
                let ip = queue.pop_front().unwrap_or([8, 8, 8, 8]);
                build_response(packet, ip)
            }
        }
    }
}

fn build_response(request: &[u8], ipv4: [u8; 4]) -> Option<Vec<u8>> {
    if request.len() < HEADER_LEN {
        return None;
    }
    let header = parse_header(request)?;
    if header.qdcount == 0 {
        return None;
    }
    let questions = parse_questions(request, header.qdcount)?;
    let mut response = Vec::new();
    let mut flags: u16 = 0;
    flags |= 1 << 15; // QR
    flags |= header.flags & 0x7800; // OPCODE
    if header.flags & 0x0100 != 0 {
        flags |= 0x0100;
    }

    response.extend_from_slice(&header.id.to_be_bytes());
    response.extend_from_slice(&flags.to_be_bytes());
    response.extend_from_slice(&header.qdcount.to_be_bytes());
    response.extend_from_slice(&header.qdcount.to_be_bytes());
    response.extend_from_slice(&0u16.to_be_bytes()); // NSCOUNT
    response.extend_from_slice(&0u16.to_be_bytes()); // ARCOUNT

    let question_bytes = encode_questions(&questions);
    response.extend_from_slice(&question_bytes);
    for question in &questions {
        response.extend_from_slice(&question.name);
        response.extend_from_slice(&question.qtype.to_be_bytes());
        response.extend_from_slice(&question.qclass.to_be_bytes());
        response.extend_from_slice(&60u32.to_be_bytes());
        response.extend_from_slice(&4u16.to_be_bytes());
        response.extend_from_slice(&ipv4);
    }
    Some(response)
}

fn parse_header(packet: &[u8]) -> Option<RawHeader> {
    if packet.len() < HEADER_LEN {
        return None;
    }
    Some(RawHeader {
        id: u16::from_be_bytes([packet[0], packet[1]]),
        flags: u16::from_be_bytes([packet[2], packet[3]]),
        qdcount: u16::from_be_bytes([packet[4], packet[5]]),
    })
}

fn parse_questions(packet: &[u8], qdcount: u16) -> Option<Vec<ParsedQuestion>> {
    let mut offset = HEADER_LEN;
    let mut questions = Vec::with_capacity(qdcount as usize);
    for _ in 0..qdcount {
        let (question, next_offset) = decode_question(packet, offset)?;
        questions.push(question);
        offset = next_offset;
    }
    Some(questions)
}

fn decode_question(packet: &[u8], offset: usize) -> Option<(ParsedQuestion, usize)> {
    let (name, name_end) = decode_name(packet, offset)?;
    let qtype_end = name_end.checked_add(2)?;
    let qclass_end = qtype_end.checked_add(2)?;
    if qclass_end > packet.len() {
        return None;
    }
    let qtype = u16::from_be_bytes(packet[name_end..qtype_end].try_into().ok()?);
    let qclass = u16::from_be_bytes(packet[qtype_end..qclass_end].try_into().ok()?);
    Some((
        ParsedQuestion {
            name,
            qtype,
            qclass,
        },
        qclass_end,
    ))
}

fn decode_name(packet: &[u8], offset: usize) -> Option<(Vec<u8>, usize)> {
    let mut cursor = offset;
    let mut labels: Vec<Vec<u8>> = Vec::new();
    let mut consumed: Option<usize> = None;
    let mut hops = 0;
    while cursor < packet.len() {
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
            if hops > 10 {
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
            if end > packet.len() || label_len == 0 || label_len > 63 {
                return None;
            }
            labels.push(packet[start..end].to_vec());
            cursor = end;
        }
    }
    let mut name = Vec::new();
    for label in labels {
        name.push(label.len() as u8);
        name.extend_from_slice(&label);
    }
    name.push(0);
    Some((name, consumed.unwrap_or(cursor)))
}

fn encode_questions(questions: &[ParsedQuestion]) -> Vec<u8> {
    let mut buffer = Vec::new();
    for question in questions {
        buffer.extend_from_slice(&question.name);
        buffer.extend_from_slice(&question.qtype.to_be_bytes());
        buffer.extend_from_slice(&question.qclass.to_be_bytes());
    }
    buffer
}

#[derive(Clone)]
struct ParsedQuestion {
    name: Vec<u8>,
    qtype: u16,
    qclass: u16,
}

struct RawHeader {
    id: u16,
    flags: u16,
    qdcount: u16,
}

fn bind_client_socket() -> io::Result<UdpSocket> {
    let socket = UdpSocket::bind("127.0.0.1:0")?;
    socket.set_read_timeout(Some(RESPONSE_TIMEOUT))?;
    Ok(socket)
}
