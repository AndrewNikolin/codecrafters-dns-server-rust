# Data Model - DNS Answer Section Response

## DNSQuery
| Field | Type | Description | Validation |
|-------|------|-------------|------------|
| transaction_id | u16 | Identifier echoed in response header | Must be echoed unchanged |
| flags | u16 | Header flag bits | QR bit flipped to response; opcode preserved |
| questions | Vec<Question> | Question list (Codecrafters harness sends 1) | First question inspected for `codecrafters.io` |

### Question (embedded)
| Field | Type | Description | Validation |
|-------|------|-------------|------------|
| qname | Vec<Label> | Domain encoded as length-prefixed labels | Must decode to ASCII, compare case-insensitively to `codecrafters.io` |
| qtype | u16 | Requested RR type | Regardless of value, triggers fixed A answer when qname matches |
| qclass | u16 | Requested RR class | Must equal 1 to keep response consistent; otherwise echo request but answer omitted |

## DNSAnswerRecord
| Field | Type | Description | Validation |
|-------|------|-------------|------------|
| name | Vec<u8> | Pre-encoded label sequence for `codecrafters.io` | Always `\x0ccodecrafters\x02io\x00`
| rtype | u16 | Record type | Fixed value 1 (A) |
| rclass | u16 | Record class | Fixed value 1 (IN) |
| ttl | u32 | Cache duration in seconds | Fixed at 60 seconds |
| rdlength | u16 | Length of RDATA | Fixed at 4 |
| rdata | [u8;4] | IPv4 payload | Fixed bytes `8.8.8.8` |

## AnswerConfig
| Field | Type | Description | Validation |
|-------|------|-------------|------------|
| target_domain | String | Domain eligible for auto-answer | Exact match `codecrafters.io` |
| ipv4 | [u8;4] | Address inserted into A record | Defaults to `8,8,8,8`; allow override behind future config |
| ttl_seconds | u32 | TTL applied to every answer | Defaults to 60 |

### Relationships & State
- `DNSQuery` → `DNSAnswerRecord` is 0..1: an answer record is produced only when `qname` matches `target_domain` and `qclass` is IN.
- `AnswerConfig` supplies constants used when manufacturing the `DNSAnswerRecord` each time; there is no persisted state.
- Multiple questions per query are ignored beyond the first, so no state machine is required beyond checking the leading question.
