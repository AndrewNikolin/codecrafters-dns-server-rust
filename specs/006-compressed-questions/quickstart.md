# Quickstart - DNS Question Compression Handling

## Prerequisites
- Rust 1.80+ toolchain installed
- `cargo` available on PATH
- UDP port 2053 free locally

## Run the server
```bash
cargo run --bin main
```
Keep it running to handle `dig` and custom packets.

## Manual verification
1. **Compressed follow-up**: Craft a packet where question 2 uses a pointer to question 1 (use the integration helper script or Python) and ensure the response QUESTION section lists both names fully expanded (no `0xC0` pointers).
2. **Multi-question answers**: Send a three-question packet and confirm ANCOUNT=3 and each ANSWER’s NAME/TTL/IP match expectations (`8.8.8.8`).
3. **Invalid pointer**: Create a packet whose pointer jumps beyond the packet length; confirm the server drops it (no response).
4. **Pointer loop**: Create two questions pointing to each other; verify the hop limit triggers a drop.

## Automated verification
```bash
cargo test
cargo clippy --all-targets
```
Integration tests under `tests/integration/` should cover compression success paths, invalid pointer drops, and multi-question responses.
