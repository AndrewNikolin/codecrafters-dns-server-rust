# Quickstart - DNS Answer Section Response

## Prerequisites
- Rust toolchain 1.80+ (Edition 2021)
- `cargo` available on PATH
- UDP port 2053 (default Codecrafters port) available locally

## Run the server
```bash
cargo run --bin main
```
The program binds to the port defined by the Codecrafters harness (usually `UDP:2053`). Keep it running in a terminal.

## Manual verification
1. Send a DNS query for `codecrafters.io`:
   ```bash
   dig @127.0.0.1 -p 2053 codecrafters.io A
   ```
2. Confirm the response contains:
   - `ANSWER: 1`
   - TTL `60`
   - `codecrafters.io. 60 IN A 8.8.8.8`
3. Query a different domain (e.g., `example.com`) and confirm `ANSWER: 0`.
4. Re-run the `codecrafters.io` query and ensure the TTL remains `60`, indicating caching clients can reuse the record without requerying early.

## Automated tests
```bash
cargo test
cargo clippy --all-targets
```
This runs the Codecrafters integration tests plus any new unit tests, then lint checks to prevent regressions.
