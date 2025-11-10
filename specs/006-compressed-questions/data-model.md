# Data Model - DNS Question Compression Handling

## CompressedQuestion
| Field | Type | Description | Validation |
|-------|------|-------------|------------|
| raw_offset | usize | Offset where the question begins | Must be within packet bounds |
| name | Vec<u8> | Decoded label sequence (no compression) | Total length ≤255, each label ≤63 |
| qtype | u16 | Record type | Must equal 1 (A) |
| qclass | u16 | Record class | Must equal 1 (IN) |

## DnsAnswerRecord
| Field | Type | Description | Validation |
|-------|------|-------------|------------|
| name | Vec<u8> | Label sequence copied from question | Must match corresponding question name |
| rtype | u16 | Record type | Fixed to 1 |
| rclass | u16 | Record class | Fixed to 1 |
| ttl | u32 | Time to live seconds | Fixed to 60 |
| rdlength | u16 | Length of RDATA | Fixed to 4 |
| rdata | [u8;4] | IPv4 payload | Fixed to `8.8.8.8` |

## DnsPacketResponse
| Field | Type | Description | Validation |
|-------|------|-------------|------------|
| header | DnsHeader | Mirrors ID/opcode/RD/QDCOUNT; ANCOUNT = number of questions | Must keep NSCOUNT/ARCOUNT = 0 |
| questions | Vec<CompressedQuestion> | Parsed question list | Must equal QDCOUNT |
| answers | Vec<DnsAnswerRecord> | Synthesized answers (one per question) | Must equal questions.len() |

### Relationships & Flow
- `CompressedQuestion` objects are parsed sequentially, resolving pointers as needed.
- Each question generates exactly one `DnsAnswerRecord` using the decoded name.
- `DnsPacketResponse` assembles header → questions → answers, keeping counts and ordering aligned.
