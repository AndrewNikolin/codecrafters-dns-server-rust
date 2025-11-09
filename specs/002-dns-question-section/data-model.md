# Data Model

## Entity: DnsQuestion
- **Name** (`LabelSequence`): canonical `codecrafters.io` encoded as `[0x0c, "codecrafters", 0x02, "io", 0x00]`.
- **Type** (`u16`): fixed to `0x0001` (A record) in network byte order.
- **Class** (`u16`): fixed to `0x0001` (IN) in network byte order.
- **Bytes** (`Vec<u8>` or `[u8; 18]`): contiguous serialization appended immediately after the 12-byte DNS header.

**Validation Rules**
- Must always contain exactly two labels plus the terminating null byte.
- Type and Class must remain `0x0001` regardless of incoming request contents.

## Entity: DnsResponsePacket
- **Header** (`DnsHeaderResponse`): lesson 1 header with `QDCOUNT=1`, other section counts zero.
- **Question** (`DnsQuestion`): serialized after header; no answers/authority/additional records.
- **Bytes** (`Vec<u8>`): header bytes (12) + question bytes (length determined by label sequence + 4 bytes for Type/Class).

**Validation Rules**
- Total packet length must equal 12 + length(Name) + 4.
- Packet must be emitted within 200 ms of receiving a probe.

## Supporting Type: LabelSequence
- **Labels**: ordered list of `(length: u8, content: ascii string)` entries, terminated by `0x00`.
- **Length Constraints**: each label length must fit within `u8`; cumulative encoded string must remain under UDP buffer (512 bytes).

**State Transitions**
1. `Initialized` (compile-time constant) → `Serialized` (converted to bytes once at startup).
2. `Serialized` → `Sent` each time `send_to` succeeds.
