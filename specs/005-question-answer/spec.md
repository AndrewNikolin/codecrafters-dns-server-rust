# Feature Specification: DNS Question & Answer Echo

**Feature Branch**: `005-question-answer`  
**Created**: 2025-11-09  
**Status**: Draft  
**Input**: User description: "this spec should have number 5. the task is to extend DNS server to parse the question section of the DNS message you receive. The question type will always be A for this stage and the question class will always be IN. So your parser only needs to account for those record types for now.

Your program will need to respond with a DNS reply packet that contains:

a header section (same as in stage #5)
a question section (new in this stage)
an answer section (new in this stage)
Expected values for the question section:

Field Expected value
Name Mimic the domain name (as label sequence)
Type 1 encoded as a 2-byte big-endian int (corresponding to the \"A\" record type)
Class 1 encoded as a 2-byte big-endian int (corresponding to the \"IN\" record class)
Expected values for the answer section:

Field Expected Value
Name Mimic the domain name (as label sequence)
Type 1 encoded as a 2-byte big-endian int (corresponding to the \"A\" record type)
Class 1 encoded as a 2-byte big-endian int (corresponding to the \"IN\" record class)
TTL Any value, encoded as a 4-byte big-endian int. For example: 60.
Length 4, encoded as a 2-byte big-endian int (corresponds to the length of the RDATA field)
Data Any IP address, encoded as a 4-byte big-endian int. For example: \x08\x08\x08\x08 (that's 8.8.8.8 encoded as a 4-byte integer)"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Parse and echo question (Priority: P1)

Codecrafters test harness sends a query for any domain, and expects the server to parse the question section (NAME/QTYPE/QCLASS) and return the exact same bytes in the reply.

**Why this priority**: Without mirroring the question, downstream answer logic cannot rely on the client knowing which domain was resolved.

**Independent Test**: Send a query for `example.test` and assert the response question bytes match the request exactly.

**Acceptance Scenarios**:

1. **Given** a standard DNS packet with NAME `codecrafters.io`, QTYPE=1, QCLASS=1, **When** the server replies, **Then** the response question section matches byte-for-byte and QDCOUNT stays in sync with the mirrored header.
2. **Given** a query whose QTYPE/QCLASS are not (1,1), **When** it is parsed, **Then** the server drops or ignores it (documented assumption) to avoid producing unsupported answers.

---

### User Story 2 - Construct deterministic answer record (Priority: P2)

The client expects an A-record answer for the parsed domain using the configured IPv4 and TTL while mirroring the NAME encoding.

**Why this priority**: Producing a valid Resource Record proves the server can synthesize data beyond the static stub from earlier stages.

**Independent Test**: After User Story 1, send a query for `codecrafters.io` and confirm the answer record exists with TTL 60, RDLENGTH 4, and RDATA `8.8.8.8`.

**Acceptance Scenarios**:

1. **Given** a query for `codecrafters.io`, **When** the answer is inspected, **Then** NAME matches the label sequence from the question, TYPE=1, CLASS=1, TTL=60, RDLENGTH=4, and RDATA equals the configured IPv4.
2. **Given** a query for any other domain, **When** the answer logic runs, **Then** the server still produces a single A record using the requested NAME (per Codecrafters stage instructions).

---

### User Story 3 - Keep header, question, and answer counts consistent (Priority: P3)

Testers send packets with different QDCOUNTs to ensure the counts and sections align.

**Why this priority**: Prevents malformed replies where header counters do not match the serialized sections, which would break resolvers.

**Independent Test**: Craft a query with QDCOUNT=2 and confirm the response still embeds the first question, keeps QDCOUNT mirrored at 2, and sets ANCOUNT=1.

**Acceptance Scenarios**:

1. **Given** any incoming packet, **When** the response is serialized, **Then** header counters reflect the actual number of serialized sections (QDCOUNT copied from request, ANCOUNT set to 1, NSCOUNT/ARCOUNT zero).
2. **Given** a malformed packet with inconsistent counters vs. payload length, **When** the server detects the mismatch, **Then** it drops the packet instead of emitting a corrupted reply.

### Edge Cases

- Packets declaring QDCOUNT>1 but containing fewer encoded questions must be rejected to avoid underflow/over-read errors.
- NAME labels longer than 63 bytes or total domain length >255 bytes should trigger a drop (never send truncated labels).
- If QTYPE/QCLASS deviate from (A, IN), document and apply a deterministic fallback (e.g., drop packet) rather than forging unsupported answers.
- Timeouts or parsing failures must not leave partially written responses in the socket buffer.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The server MUST parse the question section starting at byte 12, supporting label compression-free names and validating QTYPE=1 and QCLASS=1.
- **FR-002**: The serialized response MUST include a question section identical to the parsed request (NAME/QTYPE/QCLASS mirrors).
- **FR-003**: The response MUST include a single answer record whose NAME matches the question NAME, TYPE=1, CLASS=1, TTL=60 seconds, RDLENGTH=4, and RDATA encodes `8.8.8.8`.
- **FR-004**: The response header MUST set QR=1, ANCOUNT=1, NSCOUNT=0, ARCOUNT=0 while mirroring the original ID, OPCODE, RD, and QDCOUNT.
- **FR-005**: If parsing fails (short packet, invalid labels, unsupported QTYPE/QCLASS), the server MUST drop the packet or return a documented error without emitting malformed data.
- **FR-006**: Answer serialization MUST use big-endian encoding for all numeric fields (TTL, RDLENGTH, RDATA bytes) to satisfy RFC 1035.
- **FR-007**: The implementation MUST remain stateless; each response depends only on the current packet and configured IPv4/TTL.
- **FR-008**: Integration tests MUST cover both successful and failure paths (valid query, unsupported QTYPE, malformed labels).

### Key Entities *(include if feature involves data)*

- **DNS Question**: Structure with NAME (label sequence), QTYPE, QCLASS parsed from incoming packets.
- **DNS Answer Record**: Structure containing NAME, TYPE, CLASS, TTL, RDLENGTH, RDATA representing the synthesized A record.
- **Answer Config**: Static configuration storing TTL (60 seconds) and IPv4 bytes (`8.8.8.8`).

### Assumptions

1. Clients always send uncompressed QNAMEs; name compression support is deferred to later stages.
2. All queries arrive over UDP and fit within 512 bytes.
3. Returning an answer for any domain (even if not `codecrafters.io`) keeps grader expectations satisfied for this stage.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of integration tests confirm question sections in responses mirror the request bytes for at least 20 domains.
- **SC-002**: 100% of manual `dig` queries show ANCOUNT=1 with TTL=60 and RDATA=`8.8.8.8`.
- **SC-003**: Malformed or unsupported queries result in zero emitted bytes 100% of the time across 50 fuzzed packets (verified by observing no socket sends).
- **SC-004**: No header/question/answer mismatch is detected across automated tests; parsers consistently satisfy header counters and section lengths (0 reported mismatches in CI).
