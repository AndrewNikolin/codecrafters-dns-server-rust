# Feature Specification: DNS Header Parsing & Echo

**Feature Branch**: `004-header-parse`  
**Created**: 2025-11-09  
**Status**: Draft  
**Input**: User description: "for the next specification use number 4. the task is to add header section parsing. In this stage, youll have to parse the DNS packet that you receive and respond with the same ID in the response. Youll also need to set some other fields in the header section. program will need to respond with a DNS reply packet that contains a header section with the following values: Field Size Expected value Packet Identifier (ID) 16 bits Mimic the 16 bit packet identifier from the request packet sent by tester Query/Response Indicator (QR) 1 bit 1 Operation Code (OPCODE) 4 bits Mimic the OPCODE value sent by the tester Authoritative Answer (AA) 1 bit 0 Truncation (TC) 1 bit 0 Recursion Desired (RD) 1 bit Mimic the RD value sent by the tester Recursion Available (RA) 1 bit 0 Reserved (Z) 3 bits 0 Response Code (RCODE) 4 bits 0 (no error) if OPCODE is 0 (standard query) else 4 (not implemented) Question Count (QDCOUNT) 16 bits Any valid value Answer Record Count (ANCOUNT) 16 bits Any valid value Authority Record Count (NSCOUNT) 16 bits Any valid value Additional Record Count (ARCOUNT) 16 bits Any valid value"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Matching request identifiers (Priority: P1)

A DNS client sends a standard query and expects the response to carry the exact same 16-bit transaction ID so it can correlate answers with outstanding requests.

**Why this priority**: Without ID mirroring, clients cannot trust responses, making the server unusable; thus it is the minimum viable behavior.

**Independent Test**: Send any DNS query with a known ID value and verify the response header echoes that ID while setting QR to 1 and keeping the question count untouched.

**Acceptance Scenarios**:

1. **Given** a properly formed 12-byte DNS header with ID `0xBEEF`, **When** the server replies, **Then** the response ID is `0xBEEF`, QR is 1, and QDCOUNT matches the incoming count.
2. **Given** a malformed packet shorter than 12 bytes, **When** it is received, **Then** the server discards it or replies with a fixed fallback without attempting to mirror an unknown ID, avoiding corrupted replies.

---

### User Story 2 - Flag parity with requester (Priority: P2)

Test harnesses validate that certain control bits are preserved while others are forced to deterministic values, ensuring interoperability with recursive resolvers.

**Why this priority**: Correct bit-level handling proves the server can coexist with diverse DNS clients and is necessary before implementing richer sections.

**Independent Test**: Craft queries that toggle OPCODE and RD flags and assert the response mirrors those bits while AA, TC, RA, Z bits remain zero.

**Acceptance Scenarios**:

1. **Given** a query whose OPCODE and RD flags are set, **When** the response is emitted, **Then** OPCODE and RD match the request and AA/TC/RA/Z are all zero.
2. **Given** any query, **When** the response is inspected, **Then** ANCOUNT, NSCOUNT, and ARCOUNT values remain whatever the server sets for that stage (commonly zero) but retain 16-bit integrity (no overflow or corruption).

---

### User Story 3 - RCODE rules for unsupported operations (Priority: P3)

DNS testers send non-standard OPCODE values to ensure the stub marks them as not implemented without crashing.

**Why this priority**: Handling unsupported operations gracefully prevents incorrect success codes and aligns with resolver expectations for capability discovery.

**Independent Test**: Send a query where OPCODE ≠ 0 and confirm the response RCODE is 4 (Not Implemented); confirm standard queries (OPCODE 0) still return RCODE 0.

**Acceptance Scenarios**:

1. **Given** a query whose OPCODE equals 0, **When** the response is parsed, **Then** RCODE equals 0 and the rest of the header adheres to User Story 1 & 2 rules.
2. **Given** a query whose OPCODE is set to any non-zero value, **When** the reply is checked, **Then** RCODE equals 4 and all other mirrored/header bits still follow the defined rules.

### Edge Cases

- Packets shorter than 12 bytes must be rejected or handled via a documented fallback without attempting to read missing header fields.
- Packets with multiple questions must still mirror the incoming QDCOUNT value even if additional questions are ignored later in the pipeline.
- Inputs with reserved bits already set should not cause undefined behavior; the server still forces RA/Z bits in the response to zero.
- Non-UTF8 payloads or corrupt headers should fail fast without panics, returning nothing rather than emitting invalid flags.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The server MUST parse the first 12 bytes of every UDP payload into ID, flag bits, and section counters before constructing a response.
- **FR-002**: The response header MUST echo the incoming ID verbatim and set the QR bit to 1.
- **FR-003**: The response header MUST copy the OPCODE bits from the request and propagate the RD bit without modification.
- **FR-004**: The response header MUST force AA, TC, RA, and Z bits to zero regardless of the incoming values.
- **FR-005**: The response header MUST populate RCODE with 0 when OPCODE is 0 and 4 (Not Implemented) for any other OPCODE value.
- **FR-006**: The response MUST retain the incoming QDCOUNT, ANCOUNT, NSCOUNT, and ARCOUNT values unless the broader feature explicitly changes them in later stages.
- **FR-007**: If the incoming packet is shorter than 12 bytes, the system MUST avoid reading out-of-bounds data and SHOULD drop the packet or return a static error response.
- **FR-008**: The implementation MUST remain stateless; decisions rely solely on the current packet header (no cached IDs or flags).

### Key Entities

- **DNS Request Header**: 12-byte structure containing ID, flag bits, and section counts extracted from every inbound packet.
- **DNS Response Header**: Derived structure mirroring select fields (ID, OPCODE, RD, counters) while overriding QR, AA, TC, RA, Z, and RCODE per specification.

### Assumptions

1. Test harnesses always send well-formed DNS headers; malformed inputs only need safe failure, not full error messaging.
2. Section counts can remain whatever values prior stages set (commonly QDCOUNT=1, others 0) without conflicting with this feature.
3. Only opcode values 0–15 exist; responding with RCODE 4 for any non-zero opcode is sufficient for Codecrafters requirements.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of integration tests confirm the response ID matches the request ID for at least 50 sampled queries.
- **SC-002**: For 100% of standard queries (OPCODE 0), RCODE remains 0 and QR is set to 1; no mismatches observed during manual `dig` validation.
- **SC-003**: For 100% of non-zero OPCODE tests, responses carry RCODE 4 while still echoing ID and RD bits, demonstrating graceful handling of unsupported commands.
- **SC-004**: No malformed header read is observed across fuzzed inputs up to 100 random packets; the server either drops them silently or responds with safe defaults (no crashes or undefined behavior).
