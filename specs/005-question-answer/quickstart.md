# Quickstart - DNS Question & Answer Echo

## Prerequisites
- Rust 1.80+ toolchain installed
- `cargo` available on PATH
- UDP port 2053 free locally

## Run the server
```bash
cargo run --bin main
```
Keep it running to accept `dig` and crafted packets.

## Manual verification
1. **Echo question**
   ```bash
   dig @127.0.0.1 -p 2053 codecrafters.io A +noedns +ignore
   ```
   Ensure the response `;; QUESTION SECTION:` matches the request exactly.
2. **Custom domain**: Repeat with `dig example.test A` and confirm the mirrored question and answer use `example.test` with IP `8.8.8.8`.
3. **Custom packet**: Craft a raw packet (e.g., Python) that sets QDCOUNT=2 but provides a single question; verify the server drops it (no response) due to mismatch.
4. **Unsupported QTYPE**: Modify the crafted packet to set QTYPE=0x0002 and confirm the server drops it.

## Automated verification
```bash
cargo test
cargo clippy --all-targets
```
Integration tests under `tests/integration.rs` will assert question mirroring, answer construction, header counters, and malformed packet handling.
