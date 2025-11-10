# Feature Specification: DNS Answer Section Response

**Feature Branch**: `003-answer-section`  
**Created**: 2025-11-09  
**Status**: Draft  
**Input**: User description: "the next spec with number 3 will be for Writing answer section. The task is to extend DNS server to respond with the answer section, the third section of a DNS message. The answer section contains a list of RRs (Resource Records), which are answers to the questions asked in the question section. Each RR has the following structure: Field Type Description Name Label Sequence The domain name encoded as a sequence of labels. Type 2-byte Integer 1 for an A record, 5 for a CNAME record etc., full list here Class 2-byte Integer Usually set to 1 (full list here) TTL (Time-To-Live) 4-byte Integer The duration in seconds a record can be cached before requerying. Length (RDLENGTH) 2-byte Integer Length of the RDATA field in bytes. Data (RDATA) Variable Data specific to the record type. Section 3.2.1 of the RFC covers the answer section format in detail. In this stage, well only deal with the \"A\" record type, which maps a domain name to an IPv4 address. The RDATA field for an \"A\" record type is a 4-byte integer representing the IPv4 address. The answer section should contain a single RR, with the following values:

Field\tExpected Value
Name\t\x0ccodecrafters\x02io followed by a null byte (thats codecrafters.io encoded as a label sequence)
Type\t1 encoded as a 2-byte big-endian int (corresponding to the A record type)
Class\t1 encoded as a 2-byte big-endian int (corresponding to the IN record class)
TTL\tAny value, encoded as a 4-byte big-endian int. For example: 60.
Length\t4, encoded as a 2-byte big-endian int (corresponds to the length of the RDATA field)
Data\tAny IP address, encoded as a 4-byte big-endian int. For example: x08x08x08x08"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Authoritative answer for codecrafters.io (Priority: P1)

A DNS client queries the server for `codecrafters.io` (type A) and receives a single-answer response containing a valid A record constructed per RFC 1035.

**Why this priority**: Returning a correct resource record is the minimal observable value for users and unlocks subsequent stages.

**Independent Test**: Send a standard DNS query for `codecrafters.io` and confirm the response includes one answer record with expected field values regardless of client.

**Acceptance Scenarios**:

1. **Given** a DNS query for `codecrafters.io` type A, **When** the server replies, **Then** the answer section contains exactly one RR whose NAME matches the queried domain and whose TYPE/CLASS are both set to 1.
2. **Given** the same query, **When** the response is parsed, **Then** the TTL, RDLENGTH (4), and RDATA (IPv4) fields comply with the specified values and ordering.

---

### User Story 2 - Cache-friendly TTL behavior (Priority: P2)

Caching resolvers reuse the returned record for the duration announced in TTL before requerying the server.

**Why this priority**: Predictable TTL ensures upstream resolvers do not overload the stub server and demonstrates correct interpretation of the TTL field.

**Independent Test**: Inspect the TTL value in the answer and verify that resolvers respect it by refraining from requerying until the TTL expires.

**Acceptance Scenarios**:

1. **Given** repeated queries from a caching resolver, **When** TTL seconds have not elapsed, **Then** the resolver continues serving the cached record without recontacting the server.
2. **Given** the TTL has expired, **When** a resolver sends a fresh query, **Then** the server issues a new answer with the same TTL value.

---

### User Story 3 - Non-target queries remain unaffected (Priority: P3)

Queries for domains other than `codecrafters.io` continue to receive whatever minimal response existed previously (e.g., header + question only) without an incorrect answer section.

**Why this priority**: Ensures the new answer logic does not leak to unrelated domains, keeping system behavior predictable for unsupported queries.

**Independent Test**: Issue a query for any other domain or record type and confirm that no answer section is appended.

**Acceptance Scenarios**:

1. **Given** a DNS query whose QNAME is not `codecrafters.io`, **When** the server responds, **Then** the ANSWER count stays zero and no RR is added.

### Edge Cases

- Queries where `codecrafters.io` is requested with a non-A QTYPE should still receive the single configured A RR to keep behavior deterministic for the exercise.
- Requests containing multiple questions should only be answered when the first question matches `codecrafters.io`, and additional questions remain unanswered.
- Malformed or truncated queries must continue to be rejected or ignored without emitting an answer section.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The server MUST return an answer section containing exactly one RR whenever the question section includes `codecrafters.io` as QNAME.
- **FR-002**: The response header MUST set the answer count to 1 only for responses that include the RR, and to 0 otherwise.
- **FR-003**: The RR NAME MUST encode `codecrafters.io` exactly as the label sequence `\x0c codecrafters \x02 io \x00`.
- **FR-004**: The RR TYPE field MUST be set to decimal 1 (A record) encoded as a two-byte big-endian integer.
- **FR-005**: The RR CLASS field MUST be set to decimal 1 (IN) encoded as a two-byte big-endian integer.
- **FR-006**: The RR TTL field MUST be a positive 32-bit big-endian integer (default 60 seconds) applied consistently across responses.
- **FR-007**: The RR RDLENGTH MUST always equal 4 and be encoded as a two-byte big-endian integer.
- **FR-008**: The RR RDATA MUST contain a fixed IPv4 address expressed as four bytes (default `8.8.8.8`) that matches the documented expectation for the stage.
- **FR-009**: The server MUST preserve previously implemented behavior for unsupported domains, returning no answer records for them.

### Key Entities *(include if feature involves data)*

- **DNS Query Packet**: Contains the transaction ID, flags, and one question describing QNAME (`codecrafters.io`), QTYPE, and QCLASS that trigger answer generation.
- **DNS Answer Record**: Holds NAME, TYPE, CLASS, TTL, RDLENGTH, and RDATA fields that together describe the IPv4 mapping for `codecrafters.io`.
- **Configured IPv4 Target**: The canonical IPv4 address (default `8.8.8.8`) that serves as the payload for RDATA and can be inspected or changed in future specs.

### Assumptions

1. Only `codecrafters.io` requires an answer section during this stage; other domains remain unsupported.
2. A static IPv4 address of `8.8.8.8` is acceptable unless a later spec demands configurability.
3. Clients send well-formed DNS queries over UDP, and transport-level error handling remains unchanged from prior stages.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of manual DNS queries for `codecrafters.io` receive a response whose ANSWER count is 1 and whose RR fields match the specification.
- **SC-002**: Observed TTL in responses equals the documented value (60 seconds) with a variance of 0 seconds across 20 consecutive replies.
- **SC-003**: DNS responses including the answer are generated within 100 ms in a local environment so that resolvers perceive no additional latency.
- **SC-004**: 100% of queries for domains other than `codecrafters.io` continue to produce responses with zero answer records, demonstrating no regression in scope.
