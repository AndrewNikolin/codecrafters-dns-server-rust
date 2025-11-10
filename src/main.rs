mod dns;
mod forwarder;

use crate::dns::{parse_questions, DnsHeaderRequest};
use crate::forwarder::{ForwardResult, ForwardingJob, ForwardingResolver, ResponseAssembler};
use anyhow::{Context, Result};
use clap::Parser;
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;

const LISTEN_ADDR: &str = "0.0.0.0:2053";
const FORWARD_TIMEOUT_MS: u64 = 200;

#[derive(Parser, Debug)]
#[command(
    name = "codecrafters-dns-server",
    about = "Codecrafters DNS forwarding server"
)]
struct CliConfig {
    #[arg(long = "resolver", value_name = "IP:PORT")]
    resolver: SocketAddr,
}

fn main() {
    env_logger::init();
    let config = CliConfig::parse();
    if let Err(err) = run(&config) {
        log::error!("DNS server exited with error: {err:#}");
        std::process::exit(1);
    }
}

fn run(config: &CliConfig) -> Result<()> {
    let socket = UdpSocket::bind(LISTEN_ADDR)
        .with_context(|| format!("failed to bind UDP socket on {LISTEN_ADDR}"))?;
    let resolver =
        ForwardingResolver::new(config.resolver, Duration::from_millis(FORWARD_TIMEOUT_MS))
            .context("failed to initialize forwarding resolver")?;
    log::info!(
        "Listening on {LISTEN_ADDR} and forwarding queries to {}",
        config.resolver
    );

    let mut buf = [0u8; 512];
    loop {
        match socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                let payload = &buf[..size];
                log::debug!(
                    "Received {} packet from {} bytes={} payload={}",
                    payload_label(size, buf.len()),
                    source,
                    size,
                    hex_preview(payload)
                );
                match handle_packet(&resolver, payload) {
                    Ok(Some(response)) => {
                        if let Err(err) = socket.send_to(&response, source) {
                            log::error!("Failed to send DNS packet to {source}: {err}");
                        } else {
                            log::debug!(
                                "Sent DNS response ({} bytes) to {}",
                                response.len(),
                                source
                            );
                        }
                    }
                    Ok(None) => log::debug!("Dropped unsupported DNS packet from {}", source),
                    Err(err) => log::error!("Failed to process DNS packet from {source}: {err}"),
                }
            }
            Err(err) => {
                log::error!("Error receiving data: {err}");
            }
        }
    }
}

fn handle_packet(resolver: &ForwardingResolver, packet: &[u8]) -> Result<Option<Vec<u8>>> {
    let header = match DnsHeaderRequest::parse(packet) {
        Ok(header) => header,
        Err(_) => return Ok(None),
    };
    let questions = match parse_questions(packet, header.qdcount) {
        Some(questions) => questions,
        None => return Ok(None),
    };

    if header.opcode() != 0 {
        let assembler = ResponseAssembler::new(&header, &questions);
        return Ok(Some(assembler.assemble(&[], 0, 4)));
    }

    if header.qdcount == 1 {
        let assembler = ResponseAssembler::new(&header, &questions);
        let question = &questions[0];
        let response = match resolver.send_and_recv(packet, header.id, question) {
            ForwardResult::Success {
                upstream_header,
                answer_section,
            } => assembler.assemble(
                &answer_section,
                upstream_header.ancount,
                upstream_header.rcode,
            ),
            ForwardResult::Failure(err) => {
                log::error!("Forwarding error: {err}");
                assembler.assemble_servfail()
            }
        };
        return Ok(Some(response));
    }

    let mut job = ForwardingJob::from_request(header, questions);
    let split_jobs = job.split_jobs.clone();
    for split in split_jobs {
        match resolver.send_and_recv(&split.forward_packet, job.header.id, &split.question) {
            ForwardResult::Success {
                upstream_header,
                answer_section,
            } => job.push_response(ForwardResult::success(upstream_header, answer_section)),
            ForwardResult::Failure(err) => {
                log::error!("Forwarding error for question #{}: {err}", split.index + 1);
                return Ok(Some(
                    ResponseAssembler::new(&job.header, &job.questions).assemble_servfail(),
                ));
            }
        }
    }

    let (answer_section, total_answers, rcode) = job.aggregate_answers();
    Ok(Some(
        ResponseAssembler::new(&job.header, &job.questions).assemble(
            &answer_section,
            total_answers,
            rcode,
        ),
    ))
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
