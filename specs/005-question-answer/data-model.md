# Data Model - DNS Question & Answer Echo

## DNSQuestion
| Field | Type | Description | Validation |
|-------|------|-------------|------------|
| name | Vec<u8> | Label sequence copied from request | Must decode to ASCII labels, total length ≤255 |
| qtype | u16 | Record type | Must equal 1 (A) |
| qclass | u16 | Record class | Must equal 1 (IN) |

## DNSAnswerRecord
| Field | Type | Description | Validation |
|-------|------|-------------|------------|
| name | Vec<u8> | Label sequence reused from question | Same bytes as DNSQuestion.name |
| rtype | u16 | Record type | Fixed to 1 |
| rclass | u16 | Record class | Fixed to 1 |
| ttl | u32 | Time to live seconds | Fixed to 60 |
| rdlength | u16 | Length of RDATA | Fixed to 4 |
| rdata | [u8;4] | IPv4 payload | Fixed to `8.8.8.8` |

## DNSPacket
| Field | Type | Description | Validation |
|-------|------|-------------|------------|
| header | DnsHeader | Mirrors Stage #4 response requirements | ID/opcode/RD mirrored, counts updated (QDCOUNT mirrored, ANCOUNT=1, NSCOUNT=ARCOUNT=0) |
| question | DNSQuestion | Parsed question from request | Required exactly once |
| answer | DNSAnswerRecord | Synthesized A record | Present exactly once |

### Relationships & Flow
- `DNSQuestion` parsed from request drives both the mirrored question section and the answer NAME.
- `DNSAnswerRecord` reuses the question name and static config values (TTL/IP) to build the RR.
- `DNSPacket` assembles header + question + answer ensuring counters align with serialized sections.
