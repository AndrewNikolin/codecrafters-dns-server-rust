# Data Model

## Entity: InboundProbe
- **SourceAddress** (`SocketAddr`): IPv4 loopback + dynamic port from Codecrafters grader. Used for response routing.
- **Payload** (`[u8; <=512]` but may exceed): Raw bytes received; treated as opaque for this lesson.
- **ReceivedAt** (`Instant`): Timestamp to enforce <200 ms turnaround budget.
- **SizeBytes** (`usize`): Actual number of bytes read; must be `>=0` and typically `<=512`, but oversized packets are tolerated.

**Validation Rules**
- Socket must be bound to `127.0.0.1:2053` before receiving.
- Packets larger than the buffer get truncated by the OS; record the reported size for logging but still produce a response.

**State Transitions**
1. `Pending` (awaiting socket bind) → `Received` once `recv_from` completes.
2. `Received` → `Responded` after the DNS header is queued for send.

## Entity: DnsHeaderResponse
- **PacketId** (`u16`): Fixed value `1234` (`0x04D2`).
- **Flags** (`u16` bitfield): `QR=1`, `OPCODE=0`, `AA=0`, `TC=0`, `RD=0`, `RA=0`, `Z=0`, `RCODE=0` => combined constant `0x8000`.
- **QuestionCount/QdCount** (`u16`): `0`.
- **AnswerCount/AnCount** (`u16`): `0`.
- **AuthorityCount/NsCount** (`u16`): `0`.
- **AdditionalCount/ArCount** (`u16`): `0`.
- **Bytes** (`[u8; 12]`): Serialized header derived from the above fields; reused per response.

**Validation Rules**
- Serialization must follow big-endian network byte order for all 16-bit fields.
- Total length must remain exactly 12 bytes; no question or record sections appended.

**State Transitions**
1. `Initialized` (static constant) → `Encoded` upon converting to `[u8; 12]` once at startup.
2. `Encoded` → `Sent` each time `send_to` succeeds.

## Relationships
- Each `InboundProbe` maps 1:1 to a `DnsHeaderResponse` (always the same bytes, but tracked per probe for logging).
- Multiple probes reuse the same in-memory `DnsHeaderResponse` constant; no persistence layer is introduced.
