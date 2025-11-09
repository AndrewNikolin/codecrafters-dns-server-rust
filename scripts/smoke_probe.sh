#!/usr/bin/env bash
set -euo pipefail

echo "Sending UDP probe to 127.0.0.1:2053"
python3 - <<'PY'
import binascii
import socket
import sys

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
    hex_bytes = ' '.join(f"{byte:02x}" for byte in data)
    print(f"Received {len(data)} bytes from {addr}: {hex_bytes}")
finally:
    sock.close()
PY
