# Feature Specification: DNS Question Compression Handling

**Feature Branch**: `006-compressed-questions`  
**Created**: 2025-11-09  
**Status**: Draft  
**Input**: User description: "the next spec should be numbered 6. the task is to parse the DNS question section which has compressed the question label sequences. You will be sent multiple values in the question section and you have to parse the queries and respond with the same question section (no need for compression) in the response along with answers for them. As for the answer section, respond with an A record type for each question. The values for these A records can be anything of your choosing.

As we already know how the Question Section and Answer Section look like from the previous stages, we will just give high level details of the packet that you are sent and what the tester expects.

Here is what the tester will send you:

| ------------------------------------------ |
| Header                                     |
| ------------------------------------------ |
| Question 1 (un-compressed label sequence)  |
| ------------------------------------------ |
| Question 2 (compressed label sequence)     |
| ------------------------------------------ |
What the tester expects in response:

| ------------------------------------------ |
| Header                                     |
| ------------------------------------------ |
| Question 1 (un-compressed label sequence)  |
| ------------------------------------------ |
| Question 2 (un-compressed label sequence)  |
| ------------------------------------------ |
| Answer 1 (un-compressed label sequence)    |
| ------------------------------------------ |
| Answer 2 (un-compressed label sequence)    |
| ------------------------------------------ |
You don't need to compress your response. We will never ask you to do something that will overflow the buffer size restriction of UDP, so compressing your response packet is not something you have to worry about. Though if you like an extra challenge feel free to compress the DNS packet, the tester will work with it too.

The question type will always be A and the question class will always be IN."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Decode compressed questions (Priority: P1)

The Codecrafters tester sends packets whose question section mixes normal label sequences and RFC-1035 compression pointers; the server must parse every question into canonical labels without crashing.

**Why this priority**: Without correctly understanding compressed questions we cannot know which domains to answer, so this is the MVP.

**Independent Test**: Send a packet with two questions where the second uses a pointer to the first’s labels and confirm parsing returns two decoded domain strings.

**Acceptance Scenarios**:

1. **Given** a DNS packet whose QDCOUNT=2 with the second question using a compression pointer, **When** the parser runs, **Then** both questions are decoded into explicit label sequences with the same textual names as the client intended.
2. **Given** a question containing a pointer loop or out-of-range offset, **When** parsing attempts to follow it, **Then** the server aborts processing and drops the packet safely.

---

### User Story 2 - Mirror question section uncompressed (Priority: P2)

Clients expect the outbound packet to include the exact same questions (in order) but encoded without compression to simplify grading.

**Why this priority**: The graded stage asserts that question sections in responses match the uncompressed textual names, so failing here breaks interop even if parsing succeeds.

**Independent Test**: After parsing, serialize the response and confirm each question is present with length-prefixed labels only (no pointers) and QTYPE/QCLASS are still 1.

**Acceptance Scenarios**:

1. **Given** a mixed compressed/uncompressed request, **When** a response is sent, **Then** each question is serialized with fresh label bytes (no `0xC0` pointers) yet still matches the original domain string and ordering.
2. **Given** QDCOUNT>1, **When** the response is inspected, **Then** QDCOUNT equals the request and all question records appear contiguously before answers.

---

### User Story 3 - Emit one A-record per question (Priority: P3)

Each parsed question should yield a deterministic A-record answer (TYPE=1, CLASS=1, TTL configurable) echoing the question name in uncompressed form.

**Why this priority**: Producing an answer per question demonstrates the server can service multi-question packets and respects header counters.

**Independent Test**: Generate a three-question packet (some compressed, some not) and confirm the response contains three A answers, each referencing the respective question’s NAME and the configured IPv4.

**Acceptance Scenarios**:

1. **Given** any decoded question, **When** the answer is serialized, **Then** NAME equals the uncompressed question labels, TYPE=1, CLASS=1, TTL=chosen constant (e.g., 60), RDLENGTH=4, and RDATA matches the configured IPv4.
2. **Given** multiple questions, **When** the header is written, **Then** ANCOUNT equals QDCOUNT, and NSCOUNT/ARCOUNT remain zero unless future stages change them.

### Edge Cases

- Compression pointers that reference offsets outside the packet bounds must cause the packet to be dropped.
- Pointer loops (e.g., two offsets pointing to each other) must be detected within a finite hop limit to avoid infinite recursion.
- Mixed compressed/uncompressed questions with total QDCOUNT larger than available bytes should result in a drop rather than partial processing.
- Names exceeding 255 bytes after expansion or individual labels >63 bytes should be rejected.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The server MUST parse each question, resolving RFC-1035 compression pointers into explicit label sequences while enforcing label length and total name length limits.
- **FR-002**: If any question fails to parse (pointer loop, out-of-range offset, unsupported QTYPE/QCLASS), the entire packet MUST be dropped without emitting a response.
- **FR-003**: The response MUST include a question section with the same QDCOUNT and question order as the request, encoded purely as length-prefixed labels (no compression), with QTYPE=1 and QCLASS=1.
- **FR-004**: For every parsed question, the server MUST generate a single A-record answer whose NAME equals the question NAME, TYPE=1, CLASS=1, TTL=60 seconds (or documented constant), RDLENGTH=4, and RDATA equals the configured IPv4.
- **FR-005**: The response header MUST mirror ID, OPCODE, RD, and QDCOUNT from the request, set QR=1, set ANCOUNT equal to the number of answers generated (one per question), and zero NSCOUNT/ARCOUNT.
- **FR-006**: Serialization MUST maintain packet integrity by writing header → questions → answers in order and keeping counts aligned; integration tests should fail if mismatched.
- **FR-007**: All parsing and serialization MUST remain stateless and bounded to 512-byte UDP payloads to avoid buffer overruns.

### Key Entities

- **CompressedQuestion**: Represents a parsed question including raw bytes, resolved name, and validation state.
- **DnsAnswerRecord**: Mirrors prior stages but now created per question.
- **AnswerConfig**: Stores TTL and IPv4 used for every synthesized answer.

### Assumptions

1. Requests may contain up to a handful of questions but always fit inside a standard 512-byte UDP message.
2. QTYPE and QCLASS remain fixed at A/IN for this stage.
3. Responses need not use compression; uncompressed output always satisfies the tester.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of integration tests confirm responses reproduce all questions uncompressed and in order for at least 10 mixed-compression packets.
- **SC-002**: 100% of questions parsed during fuzz testing resolve within a bounded pointer-follow depth (e.g., ≤10 hops) with zero infinite-loop incidents.
- **SC-003**: For every accepted request, ANCOUNT equals QDCOUNT and each answer encodes TTL 60 with IPv4 `8.8.8.8`; manual inspection via crafted scripts shows no mismatches.
- **SC-004**: Malformed or unsupported packets produce zero responses in 100 fuzzed trials (verified by the absence of `send_to` calls).
