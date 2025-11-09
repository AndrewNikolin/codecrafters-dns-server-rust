# Research

## Decision: Represent DNS question as reusable struct alongside header helper
- **Rationale**: Keeping `DnsHeaderResponse` and the new `DnsQuestion` struct together (e.g., `src/dns.rs`) centralizes serialization logic and ensures both header and question bytes stay consistent across server loop and tests.
- **Alternatives considered**: Building question bytes inline in `main.rs` (harder to test, risks duplication); introducing a full DNS packet builder (overkill for a single question and would obscure learning goals).

## Decision: Hard-code canonical `codecrafters.io` label sequence
- **Rationale**: Lesson instructions demand deterministic replies regardless of incoming query, so precomputing the label bytes (`0c codecrafters 02 io 00`) avoids re-parsing payloads and makes manual verification easier for learners.
- **Alternatives considered**: Echoing the requester’s domain (would contradict requirements and complicate malformed input handling); adding a domain parser (unnecessary until later lessons introduce dynamic queries).

## Decision: Serialize Type/Class as big-endian constants appended after labels
- **Rationale**: Using `u16::to_be_bytes(1)` for both Type and Class guarantees the correct `0x00 01` sequence and keeps the send buffer contiguous with header+question data, satisfying the <200 ms budget.
- **Alternatives considered**: Writing bytes manually each time (duplicative, error-prone); deferring to external DNS crates (adds dependencies and hides protocol understanding).
