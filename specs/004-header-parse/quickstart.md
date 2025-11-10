# Quickstart - DNS Header Parsing & Echo

## Prerequisites
- Rust 1.80+ toolchain installed
- `cargo` available on PATH
- UDP port 2053 free locally

## Run the server
```bash
cargo run --bin main
```
Leave it running; logs will show packet summaries.

## Manual verification steps
1. **ID echo**: Send a basic DNS query with `dig` and inspect the transaction ID.
   ```bash
   dig @127.0.0.1 -p 2053 codecrafters.io A +noedns +ignore
   ```
   Confirm that the `;; ->>HEADER<<-` block shows the same `id` value in the reply and that `qr` is set to `1`.
2. **RD/OPCODE mirroring**: Toggle `+rd`/`+nord` flags and ensure the response mirrors the requested RD bit. For non-standard OPCODEs (not supported by `dig`), use a crafted payload:
   ```bash
   python - <<'PY'
   import socket
   pkt = bytearray(12)
   pkt[0:2] = (0x55AA).to_bytes(2, 'big')  # ID
   pkt[2] = 0b00110000  # QR=0, OPCODE=3, RD=0
   pkt[5] = 1  # QDCOUNT = 1
   sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
   sock.sendto(pkt, ("127.0.0.1", 2053))
   data, _ = sock.recvfrom(512)
   print("ID", int.from_bytes(data[0:2], 'big'))
   print("Flags", data[2:4].hex())
   PY
   ```
   Verify the response keeps ID `0x55AA`, sets `QR` to 1, and sets `RCODE` to 4 (check the lower four bits of byte 3).
3. **Malformed packets**: Send fewer than 12 bytes and ensure the server logs a drop or sends no response.
4. **Counter mirroring**: Modify the header counts (e.g., set `QDCOUNT=2`) in your crafted payload and confirm the response header mirrors the same 16-bit values, proving the parser is reusing the original counts.

## Automated verification
```bash
cargo test
cargo clippy --all-targets
```
Integration tests under `tests/integration.rs` will validate ID echoing, RD mirroring, RCODE handling, and malformed packet safety.
