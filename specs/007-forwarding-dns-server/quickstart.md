# Quickstart — Forwarding DNS Server (Phase 1)

## Prerequisites
1. Rust toolchain 1.80+ installed (`rustup override set 1.80.0` recommended).
2. Mock upstream resolver for local testing (e.g., `named`, `coredns`, or `dnsdist`). For quick checks you can run `python3 -m dns.resolver` alternatives or tunnel to `8.8.8.8:53`.
3. UDP port 2053 available on localhost (kill any previous `your_server` process).

## Build & Run
```bash
cargo build
./target/debug/codecrafters-dns-server-rust --resolver 8.8.8.8:53
```
- The binary listens on `0.0.0.0:2053/udp`.
- The `--resolver <ip:port>` flag is mandatory; use any upstream (public or mocked).
- Use `RUST_LOG=debug` to inspect forwarding steps.

## Integration Tests
```bash
cargo test -- --nocapture
```
- Existing integration suites will continue to run; add new cases under `tests/integration/forwarding.rs` (planned).

## Manual Verification
1. Start the server pointing to a known resolver (e.g., `1.1.1.1:53`).
2. From another terminal:
   ```bash
   dig @127.0.0.1 -p 2053 example.com A
   dig @127.0.0.1 -p 2053 codecrafters.io A
   ```
   - Responses should match direct queries to `1.1.1.1`.
3. Multi-question scenario:
   ```bash
   python3 scripts/send_multi_question.py
   ```
   (Script to be added; it should craft two-question packets and verify the merged response.)
4. Timeout path:
   - Run the server with `--resolver 203.0.113.1:53` (blackhole IP). Verify that the client receives SERVFAIL within ~200 ms and the server keeps accepting subsequent queries.

## Troubleshooting
- **No response**: Confirm upstream resolver is reachable (`nc -u 8.8.8.8 53`), check firewall.
- **Mismatched IDs**: Ensure response assembler overwrites upstream IDs with the tester’s original.
- **Multi-question drops**: Confirm the splitter sends exactly one question per forwarded packet and that upstream replies are received before the 200 ms timeout.
