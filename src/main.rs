mod dns;

use crate::dns::build_response_packet;
use std::net::UdpSocket;

fn main() {
    if let Err(err) = run() {
        eprintln!("DNS server exited with error: {err}");
    }
}

fn run() -> std::io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:2053")?;
    println!("Listening on 127.0.0.1:2053 for UDP probes");

    let mut buf = [0u8; 512];
    loop {
        match socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                println!(
                    "Received {} packet from {} bytes={} payload={}",
                    payload_label(size, buf.len()),
                    source,
                    size,
                    hex_preview(&buf[..size])
                );
                let response = build_response_packet();
                if let Err(err) = socket.send_to(&response, source) {
                    eprintln!("Failed to send DNS packet to {source}: {err}");
                } else {
                    println!(
                        "Sent DNS response (header=12 bytes, question={} bytes) to {}",
                        response.len().saturating_sub(12),
                        source
                    );
                }
            }
            Err(err) => {
                eprintln!("Error receiving data: {err}");
            }
        }
    }
}

fn payload_label(size: usize, buffer_capacity: usize) -> &'static str {
    if size == 0 {
        "empty"
    } else if size >= buffer_capacity {
        "truncated-or-oversized"
    } else {
        "standard"
    }
}

fn hex_preview(bytes: &[u8]) -> String {
    const MAX_BYTES: usize = 8;
    if bytes.is_empty() {
        return String::from("empty");
    }
    let mut parts: Vec<String> = bytes
        .iter()
        .take(MAX_BYTES)
        .map(|b| format!("{b:02x}"))
        .collect();
    if bytes.len() > MAX_BYTES {
        parts.push("…".into());
    }
    parts.join(" ")
}
