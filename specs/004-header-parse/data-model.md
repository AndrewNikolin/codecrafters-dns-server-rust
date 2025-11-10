# Data Model - DNS Header Parsing & Echo

## DNSHeaderRequest
| Field | Type | Description | Validation |
|-------|------|-------------|------------|
| id | u16 | Transaction identifier from client | Required for correlation; drop packet if missing |
| flags | DnsFlags | Raw 16-bit flags from bytes 2–3 | Must preserve OPCODE+RD bits |
| qdcount | u16 | Question count | Echoed verbatim in response |
| ancount | u16 | Answer count | Echoed verbatim |
| nscount | u16 | Authority count | Echoed verbatim |
| arcount | u16 | Additional count | Echoed verbatim |

## DNSHeaderResponse
| Field | Type | Description | Validation |
|-------|------|-------------|------------|
| id | u16 | Mirrors request ID | Must exactly equal request ID |
| flags | DnsFlags | Flag bits after masks applied | QR forced to 1; AA/TC/RA/Z forced to 0; OPCODE & RD mirrored |
| qdcount | u16 | Question count | Copy of request.qdcount |
| ancount | u16 | Answer count | Copy of request.ancount |
| nscount | u16 | Authority count | Copy of request.nscount |
| arcount | u16 | Additional count | Copy of request.arcount |

## DnsFlags
| Field | Type | Description | Validation |
|-------|------|-------------|------------|
| qr | bool | Query/Response indicator | Always true in responses |
| opcode | u8 | 4-bit operation code | Copied from request; range 0–15 |
| aa | bool | Authoritative Answer | Always false |
| tc | bool | Truncation | Always false |
| rd | bool | Recursion Desired | Copied from request |
| ra | bool | Recursion Available | Always false |
| z | u8 | 3-bit reserved segment | Always zero |
| rcode | u8 | 4-bit response code | 0 when opcode==0 else 4 |

### Relationships & Flow
- `DNSHeaderRequest` → `DNSHeaderResponse`: 1-to-1 transformation per packet; no persisted state.
- `DnsFlags` is embedded within both header structs to keep masking logic centralized.
- Malformed packets that cannot populate `DNSHeaderRequest` are discarded before a response header is created.
