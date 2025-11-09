# Quickstart: Lesson 2 DNS Question Section

## Prerequisites
- Rust toolchain 1.80+
- `cargo` on PATH
- Optional: Codecrafters CLI for end-to-end grading

## Run the server
```bash
cargo run
```
- Binds `127.0.0.1:2053` and logs each probe plus the canonical question encoding.

## Verify the question section with netcat
```bash
printf '' | nc -u 127.0.0.1 2053 | hexdump -C
```
- Expect header bytes followed by `0c 63 6f 64 65 63 72 61 66 74 65 72 73 02 69 6f 00 00 01 00 01`.
- Sample output:
  ```
  00000000  04 d2 80 00 00 01 00 00  00 00 00 00 0c 63 6f 64  |.............cod|
  00000010  65 63 72 61 66 74 65 72  73 02 69 6f 00 00 01 00  |ecrafter s.io...|
  00000020  01                                                |.|
  ```

## Use the helper script (if available)
```bash
./scripts/smoke_probe.sh
```
- Shows the first 12 bytes (header) and the remaining question bytes separately so you can quickly confirm the canonical label sequence and the trailing `00 01 00 01` Type/Class fields.

## Run automated tests
```bash
cargo test --test integration
```
- Ensures all integration cases (normal, empty payload, oversized payload) still pass with the appended question section.

## Troubleshooting
- If the port is busy, stop other services bound to UDP 2053 before retrying.
- If you see fewer than 18 bytes after the header, confirm the question builder is appending the label sequence and Type/Class fields.
