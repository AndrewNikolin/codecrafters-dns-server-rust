# Feature Specification: Lesson 2 DNS Question Section

**Feature Branch**: `002-dns-question-section`  
**Created**: 2025-11-08  
**Status**: Draft  
**Input**: User description: "The next task is to extend your DNS server to respond with the question section...Expected value Name x0ccodecraftersx02io followed by a null byte"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Grader validates reply question (Priority: P1)

The Codecrafters grader sends a UDP probe and inspects the reply to confirm it now includes a single-question section whose name, type, and class match the lesson’s expected values (`codecrafters.io`, Type=1, Class=1) and whose header counts show exactly one question.

**Why this priority**: Passing the lesson hinges on the grader detecting the correctly encoded question section; without it, progression halts.

**Independent Test**: Run the Codecrafters CLI (or automated integration tests) and assert that the response bytes contain the canonical header followed by the encoded domain name, Type, and Class fields with no additional records.

**Acceptance Scenarios**:

1. **Given** the server is running on port 2053, **When** the grader sends a probe, **Then** the reply contains a question count of 1 and includes the bytes `0c 63 6f 64 65 63 72 61 66 74 65 72 73 02 69 6f 00`.
2. **Given** the grader inspects the next four bytes after the domain name, **When** it parses them as 16-bit integers, **Then** both Type and Class equal `0x0001`.

---

### User Story 2 - Learner inspects question payload locally (Priority: P2)

As a learner, I can trigger the server locally, capture the UDP response, and see the encoded domain labels + Type/Class fields so I gain confidence before pushing to Codecrafters.

**Why this priority**: Quick local confirmation avoids repeated grader runs and clarifies how DNS questions are encoded.

**Independent Test**: Use `scripts/smoke_probe.sh` or `netcat` plus `hexdump` to verify the response structure byte-by-byte without needing the official grader.

**Acceptance Scenarios**:

1. **Given** the learner runs the smoke script, **When** the script prints the hex bytes, **Then** the output shows the correct label lengths (`0c` + `02`) and the terminating `00`.

---

### User Story 3 - Server stays deterministic on malformed inputs (Priority: P3)

If a probe arrives with a different hostname, truncated payload, or no question section at all, the server still returns the canonical question section for `codecrafters.io` so automated tests remain deterministic.

**Why this priority**: Guarding against malformed input prevents confusing failures and mirrors real DNS resilience expectations.

**Independent Test**: Send probes with empty payloads or alternate domain strings and verify the response still contains the lesson’s required question section without crashing the server.

**Acceptance Scenarios**:

1. **Given** a probe containing a different domain name, **When** the server responds, **Then** the outgoing packet still includes `codecrafters.io` with Type=1 and Class=1.
2. **Given** a probe with zero bytes, **When** it is processed, **Then** the response remains well-formed (header + canonical question) and no panic occurs.

---

### Edge Cases

- Incoming packet omits a question section or encodes a different domain; the server must still emit a canonical question with `codecrafters.io`.
- Requests containing uppercase or compressed labels should not influence the encoded response; reply must always use length-prefixed lowercase labels ending with `0x00`.
- Oversized or zero-length payloads still require the same question output and must not alter the QDCOUNT or Type/Class fields.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The DNS reply MUST keep the Lesson 1 header defaults but update `QDCOUNT` to `0x0001` to signal one question.
- **FR-002**: Every UDP response MUST append exactly one question section immediately after the 12-byte header.
- **FR-003**: The question name MUST encode `codecrafters.io` using length-prefixed labels: `0x0c "codecrafters" 0x02 "io" 0x00`.
- **FR-004**: The question Type field MUST be `0x0001` (A record) in network byte order for all responses.
- **FR-005**: The question Class field MUST be `0x0001` (IN) in network byte order for all responses.
- **FR-006**: Responses MUST continue to be transmitted within 200 ms of receiving the probe so the grader does not time out.

### Key Entities

- **DnsQuestion**: Represents the single question included in each response; contains Name (label sequence), Type (u16), and Class (u16).
- **DomainLabelSequence**: Ordered list of `<length, content>` pairs ending with a null byte; for this lesson it is fixed to `codecrafters.io` but establishes the pattern for future domains.

### Assumptions

- Codecrafters expects exactly one question per reply, so multiple-question responses are out of scope for this lesson.
- Even if the incoming probe specifies another domain, the canonical response remains `codecrafters.io`.
- Type and Class are both `0x0001` unless future lessons specify otherwise.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of automated grader probes confirm `QDCOUNT=1` and detect the canonical question section appended to the header.
- **SC-002**: 100% of manual smoke tests display the exact domain label encoding `0c 63 6f 64 65 63 72 61 66 74 65 72 73 02 69 6f 00`.
- **SC-003**: 100% of replies set Type=1 and Class=1 regardless of incoming payload content.
- **SC-004**: The server still replies to at least 5 sequential malformed probes in a single run without exceeding the 200 ms response budget.
