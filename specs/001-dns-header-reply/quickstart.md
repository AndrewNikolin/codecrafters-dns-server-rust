# Quickstart

## Prerequisites
- Rust toolchain 1.80+ (Edition 2021)
- `cargo` available on PATH
- Optional: Codecrafters CLI (`codecrafters test`) for remote grading

## Run the UDP server
```bash
cargo run
```
- Binds `127.0.0.1:2053` and prints diagnostic logs.

## Smoke-test the DNS header reply

```bash
# In a second terminal
printf '' | nc -u 127.0.0.1 2053 | hexdump -C
```
- Expect a single 12-byte response: `04 d2 80 00 00 00 00 00 00 00 00 00`.
- Repeat rapidly to confirm multiple probes are handled without restarting.

Or run the helper script (wraps send/receive logging):

```bash
./scripts/smoke_probe.sh
```

- Prints the received byte count plus a hex dump.
- Fails fast with a helpful error if the port is not reachable.

## Run automated tests
```bash
cargo test -- --nocapture
```
- Includes forthcoming integration tests that send UDP probes and assert on the 12-byte header.

## Submit to Codecrafters
```bash
git add -A
git commit -m "Implement lesson 1 header reply"
git push origin 001-dns-header-reply
```
- Wait for streamed grader output; ensure 100% of probes pass the <200 ms requirement.

## Troubleshooting
- **Port already in use**: Run `lsof -i UDP:2053` (macOS/Linux) to find the process holding the port, stop it, then rerun `cargo run`.
- **No UDP permissions**: Some environments block local UDP binds. Retry on a host where you can bind to `127.0.0.1:2053` or run within the Codecrafters CLI container.
- **No response**: Ensure `cargo run` is still active and watch the logs emitted from `src/main.rs` for each probe.
