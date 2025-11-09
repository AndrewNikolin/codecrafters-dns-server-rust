#!/usr/bin/env bash
set -euo pipefail

echo "Sending UDP probe to 127.0.0.1:2053"
python3 - <<'PY'
import socket
import sys

HEADER_LEN = 12
LABEL_HINT = "0c 63 6f 64 65 63 72 61 66 74 65 72 73 02 69 6f 00"
server = ("127.0.0.1", 2053)
sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
sock.settimeout(1.0)
try:
    sock.sendto(b"", server)
    data, addr = sock.recvfrom(512)
except OSError as exc:
    print(f"Probe failed: {exc}")
    sys.exit(1)
else:
    total = len(data)
    if total < HEADER_LEN:
        print(f"Received only {total} bytes (< 12-byte header); is the server running?")
        sys.exit(1)

    header = data[:HEADER_LEN]
    question = data[HEADER_LEN:]

    header_hex = ' '.join(f"{byte:02x}" for byte in header)
    question_hex = ' '.join(f"{byte:02x}" for byte in question)

    print(f"Received {total} bytes from {addr}")
    print(f"  Header  (12 bytes): {header_hex}")
    print(f"  Question({len(question)} bytes): {question_hex}")
    print(f"    Expected labels : {LABEL_HINT}")
finally:
    sock.close()
PY
