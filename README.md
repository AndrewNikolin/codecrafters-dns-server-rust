[![progress-banner](https://backend.codecrafters.io/progress/dns-server/7dfc8df9-7ba4-49e9-a6f8-ee38a337771e)](https://app.codecrafters.io/users/codecrafters-bot?r=2qF)

This is a starting point for Rust solutions to the
["Build Your Own DNS server" Challenge](https://app.codecrafters.io/courses/dns-server/overview).

In this challenge, you'll build a DNS server that's capable of parsing and
creating DNS packets, responding to DNS queries, handling various record types
and doing recursive resolve. Along the way we'll learn about the DNS protocol,
DNS packet format, root servers, authoritative servers, forwarding servers,
various record types (A, AAAA, CNAME, etc) and more.

**Note**: If you're viewing this repo on GitHub, head over to
[codecrafters.io](https://codecrafters.io) to try the challenge.

# Passing the first stage

The entry point for your `your_program.sh` implementation is in `src/main.rs`.
Study and uncomment the relevant code, and push your changes to pass the first
stage:

```sh
git commit -am "pass 1st stage" # any msg
git push origin master
```

Time to move on to the next stage!

# Stage 2 & beyond

Note: This section is for stages 2 and beyond.

1. Ensure you have `cargo (1.87)` installed locally
1. Run `./your_program.sh` to run your program, which is implemented in
   `src/main.rs`. This command compiles your Rust project, so it might be slow
   the first time you run it. Subsequent runs will be fast.
1. Commit your changes and run `git push origin master` to submit your solution
   to CodeCrafters. Test output will be streamed to your terminal.

## Codecrafters grader command sequence

When you're ready for automated verification, use the Codecrafters CLI:

```bash
codecrafters test --stage 1
```

This command builds the project, launches the UDP server on `127.0.0.1:2053`,
and sends probe packets that expect the 12-byte DNS header response. Keep the
process running until the grader finishes streaming results; you'll see log
lines from `src/main.rs` for each inbound probe along with confirmation that
the fixed header was sent.

## Lesson 2: Question section expectations

- Every reply now includes a single question for `codecrafters.io`, so `QDCOUNT`
  in the header is set to `0x0001`.
- The encoded labels appear immediately after the 12-byte header and should
  read `0c 63 6f 64 65 63 72 61 66 74 65 72 73 02 69 6f 00` followed by
  `00 01 00 01` for Type and Class.
- If your local probes or grader output show fewer bytes, double-check that
  `build_response_packet()` (see `src/dns.rs`) is used everywhere the server
  responds.

## Manual header verification checklist

Use these steps while working on the header parsing/echo stages:

1. Run the server locally (`cargo run --bin main`) so it listens on `127.0.0.1:2053`.
2. With `dig`, send a standard query and confirm the response echoes the same
   transaction ID while `qr = 1`:
   ```bash
   dig @127.0.0.1 -p 2053 codecrafters.io A +noedns +ignore
   ```
3. Craft custom packets to toggle OPCODE/RD bits or send non-standard opcodes.
   The following Python snippet sends OPCODE 3 (“not implemented”) and prints the echoed header:
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
   print("ID:", int.from_bytes(data[0:2], 'big'))
   print("Flags:", data[2:4].hex())  # QR bit set, RCODE=4 (Not Implemented)
   PY
   ```
4. Finally, send fewer than 12 bytes (e.g., `printf '\\x00\\x01' | nc -u 127.0.0.1 2053`)
   and ensure the server drops the packet without crashing or emitting a reply.
