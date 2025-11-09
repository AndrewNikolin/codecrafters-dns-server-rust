# Research

## Decision: Blocking `UdpSocket` loop without async runtime
- **Rationale**: Lesson scope only needs a single-threaded listener that immediately responds with a fixed 12-byte header. Using the standard library’s blocking `recv_from` keeps dependencies minimal, matches Codecrafters’ bootstrap, and meets the <200 ms response requirement given the tiny payload.
- **Alternatives considered**: Tokio/async runtimes (rejected as overkill and would add setup complexity for no concurrency benefit); spawning threads per packet (rejected because it increases context switching and risks race conditions without improving latency for this workload).

## Decision: Precompute static DNS header bytes
- **Rationale**: All required fields are constant for Lesson 1, so constructing a `[u8; 12]` array once and copying it into the send buffer ensures correctness, avoids manual bit masking on every packet, and keeps response serialization under microseconds.
- **Alternatives considered**: Building the header dynamically for each request (adds unnecessary CPU work and more room for bit-order bugs); using a DNS crate (brings extra dependencies and hides learning goals).

## Decision: Use `anyhow` + structured logging for diagnostics
- **Rationale**: `anyhow` is already in Cargo.toml and provides easy context on fatal errors while still letting us log non-fatal issues (e.g., malformed packets) and continue looping. This aligns with the requirement to keep the server running for multiple probes.
- **Alternatives considered**: Bare `Result` with `expect` (would crash the server on recoverable issues); adding a full logging framework (unnecessary for the small scope and would complicate lesson onboarding).
