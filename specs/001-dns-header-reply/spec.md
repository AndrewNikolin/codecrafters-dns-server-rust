# Feature Specification: Lesson 1 DNS Header Reply

**Feature Branch**: `001-dns-header-reply`  
**Created**: 2025-11-08  
**Status**: Draft  
**Input**: User description: "This project is a learning exercise during which a small implementation of DNS server should be built...Additional Record Count (ARCOUNT) 16 bits Number of records in the Additional section. Expected value: 0."

## Clarifications

### Session 2025-11-08

- Q: How should the server handle malformed probes such as empty or >512 byte payloads? → A: Respond to every probe with the same standard DNS header.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Codecrafters grader gets a reply (Priority: P1)

The Codecrafters grader (or automated tests) starts the learner’s DNS binary, sends a UDP probe to port 2053, and expects an immediate DNS response packet whose header mirrors the lesson’s required values.

**Why this priority**: Without a valid reply, the lesson cannot be marked complete and the learning progression stalls.

**Independent Test**: Run the Codecrafters test suite (or a manual UDP probe) and confirm the first packet produces a compliant DNS header response without inspecting other functionality.

**Acceptance Scenarios**:

1. **Given** the server process is running and listening on port 2053, **When** the grader sends a single UDP packet, **Then** a response is sent within 200 ms and contains QR=1, OPCODE=0, AA=0, TC=0, RD=0, RA=0, Z=0, RCODE=0, ID=1234, and all section counts set to 0.
2. **Given** the grader sends a packet with arbitrary payload bytes, **When** the server responds, **Then** the payload length of the response matches the DNS header length (no question or record sections are appended).

---

### User Story 2 - Learner smoke-tests locally (Priority: P2)

As a learner, I can run a simple script (`netcat`, `dig`, or Codecrafters CLI) to send a UDP packet to localhost:2053 and observe that the server responds with the fixed DNS header so I gain confidence before submitting.

**Why this priority**: Quick local verification keeps iteration fast and avoids relying solely on remote grading.

**Independent Test**: Execute a manual UDP probe locally and inspect the response bytes (via hexdump) to ensure each header bit matches the lesson’s expectations.

**Acceptance Scenarios**:

1. **Given** the learner manually sends two probes back-to-back, **When** the server receives them, **Then** both responses carry identical header values and are returned in order without restarting the process.

---

### User Story 3 - System handles malformed probes safely (Priority: P3)

If a probe arrives with zero bytes or more than 512 bytes, the server still returns the same well-formed header without crashing, so the learning environment stays stable.

**Why this priority**: Defensive handling prevents confusing crashes during experimentation and mirrors real DNS robustness expectations.

**Independent Test**: Send intentionally short or oversized UDP payloads and verify the server either responds with the standard header or cleanly logs/ignores the packet without exiting.

**Acceptance Scenarios**:

1. **Given** a zero-length UDP payload, **When** it is received, **Then** the server replies with the standard header and keeps the socket open for future probes.
2. **Given** a payload longer than 512 bytes, **When** it is received, **Then** the server replies with the standard header without truncating fields and continues listening for new packets.

---

### Edge Cases

- Packet arrives before the socket is fully bound (process start-up race). The server must bind before processing and queue/ignore early packets without crashing.
- Consecutive probes arrive faster than they can be processed. The server must handle at least two sequential packets without data races or dropped sockets.
- Malformed packets (empty payload, >512 bytes, random bytes) still trigger the standard header response and must not corrupt header fields or terminate the process.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The server MUST bind a UDP socket on port 2053 (localhost) within 100 ms of process start to accept grader probes.
- **FR-002**: For every UDP packet received, the server MUST send a DNS response packet whose header length is 12 bytes and contains no question, answer, authority, or additional sections.
- **FR-003**: The response header MUST set ID=1234, QR=1, OPCODE=0, AA=0, TC=0, RD=0, RA=0, Z=0, RCODE=0, QDCOUNT=0, ANCOUNT=0, NSCOUNT=0, and ARCOUNT=0 regardless of query contents.
- **FR-004**: Responses MUST be transmitted within 200 ms of receiving the probe so the Codecrafters grader does not time out.
- **FR-005**: The server MUST remain running and continue listening after handling each packet so multiple probes in the same session are answered consistently.
- **FR-006**: The server MUST return the same standard 12-byte DNS header for malformed probes (empty payloads, >512 bytes, or random data) instead of dropping them.

### Key Entities

- **Inbound Probe Packet**: Any UDP datagram received on port 2053; treated as opaque payload whose arrival triggers a response workflow.
- **DNS Header Response**: Fixed 12-byte structure containing the lesson’s required field values; sent as the entirety of each outbound packet.

### Assumptions

- Codecrafters tests send probes from localhost, so IPv4 loopback handling is sufficient for this lesson.
- Returning only the DNS header (no question or record sections) satisfies the grader for Lesson 1.
- A fixed ID value of 1234 is acceptable even though real DNS echoes the request ID; future lessons may relax or change this.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of UDP probes sent by the Codecrafters grader receive a reply within 200 ms while the lesson binary is running.
- **SC-002**: 100% of replies inspected by automated tests show all specified header bits and counts set exactly to their expected values.
- **SC-003**: The server successfully handles at least 5 consecutive probes in a single run without needing to restart or rebind the socket.
- **SC-004**: Learners can manually trigger a probe (via Codecrafters CLI or `netcat`) and observe the expected header in fewer than 2 trial runs, indicating the behavior is deterministic.
