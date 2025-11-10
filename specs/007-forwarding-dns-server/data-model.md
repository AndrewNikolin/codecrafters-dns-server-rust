# Data Model — Forwarding DNS Server (Phase 1)

## Entity: `CliConfig`
- **Fields**
  - `resolver_addr: SocketAddr` — required IPv4/IPv6 + port for upstream resolver.
  - `bind_addr: SocketAddr` — default `0.0.0.0:2053`, configurable for future stages.
  - `per_question_timeout: Duration` — fixed at 200 ms.
- **Validation Rules**
  - `resolver_addr` must parse via `SocketAddr::from_str`; reject hostnames.
  - `bind_addr.port` locked to `2053` unless a future flag overrides it (not in scope now).
  - `per_question_timeout` is constant; disallow overriding via CLI to keep tester expectations stable.

## Entity: `DnsPacket`
- **Fields**
  - `header: DnsHeader`
  - `questions: Vec<DnsQuestion>`
  - `answers: Vec<DnsAnswer>`
- **Relationships**
  - Each `DnsQuestion` and `DnsAnswer` references the same `name` encoding abstraction (`DnsName`).
  - `header.qdcount == questions.len()`, `header.ancount == answers.len()`.
- **Validation**
  - Total serialized size must stay ≤512 bytes.
  - Parser enforces single-question packets when forwarding upstream; multi-question packets are only emitted toward the tester.

## Entity: `ForwardingJob`
- **Description**: Represents the workflow for a single inbound tester packet.
- **Fields**
  - `original_packet: DnsPacket`
  - `split_jobs: Vec<SplitQuestion>`
  - `responses: Vec<ForwardResult>`
- **Relationships**
  - `split_jobs[i]` corresponds to `original_packet.questions[i]`.
  - `responses` stored in the same order to simplify merge logic.

## Entity: `SplitQuestion`
- **Fields**
  - `question: DnsQuestion`
  - `index: usize` — original order in the tester packet.
  - `forward_packet: Vec<u8>` — single-question DNS packet bytes reused for transmission.
- **Validation**
  - Only one question per `forward_packet`; header counters set to 1.
  - Retains the tester’s transaction ID and RD flag exactly.

## Entity: `ForwardResult`
- **Variants**
  - `Success { answer_section: Vec<DnsAnswer>, upstream_header: DnsHeader }`
  - `Failure { error: ForwardError }`
- **Rules**
  - On `Failure`, the enclosing `ForwardingJob` triggers SERVFAIL and discards other successes.
  - `Success` copies upstream `DnsAnswer` objects without mutation (except ID normalization).

## Entity: `ForwardingResolver`
- **Responsibilities**
  - Owns the UDP socket connected to the upstream resolver.
  - Provides `send_and_recv(packet: &[u8], timeout: Duration) -> ForwardResult`.
- **Fields**
  - `socket: UdpSocket`
  - `resolver_addr: SocketAddr`
  - `timeout: Duration` (200 ms)
- **Validation**
  - Socket must be non-blocking or use per-call timeouts (`set_read_timeout`).
  - Retries are out-of-scope; a single attempt per question is sufficient per spec.

## Entity: `ResponseAssembler`
- **Inputs**
  - `original_packet: DnsPacket`
  - `forward_results: Vec<ForwardResult>`
- **Outputs**
  - `response_bytes: Vec<u8>` targeting the tester socket.
- **Logic**
  - If any `ForwardResult` is `Failure`, emit SERVFAIL with zero answers and mirrored question section.
  - Otherwise, aggregate answers in question order and set header counts accordingly.

## Supporting Types
- `DnsName`
  - Stores uncompressed label sequence (Vec<Vec<u8>>).
  - Validates ≤63 bytes per label and ≤255 bytes total.
- `ForwardError`
  - Enumerates timeout, socket I/O, malformed response, and mismatched question errors for logging/metrics.

## State Transitions
1. **Received** → parse to `DnsPacket`.
2. **Validated** → build `ForwardingJob` & `SplitQuestion` packets.
3. **Forwarded** → each `SplitQuestion` transitions to `ForwardResult`.
4. **Assembled** → `ResponseAssembler` builds final DNS packet.
5. **Completed** → send response or drop (if serialization fails).
