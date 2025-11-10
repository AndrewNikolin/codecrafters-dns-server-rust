# Feature Specification: Forwarding DNS Server

**Feature Branch**: `007-forwarding-dns-server`  
**Created**: 2025-11-10  
**Status**: Draft  
**Input**: User description: “the next spec should be number 7. the task is to implement a forwarding DNS server.

In this stage the tester will execute your program like this:

./your_server --resolver <address>
where <address> will be of the form <ip>:<port>
It'll then send a UDP packet (containing a DNS query) to port 2053. Your program will be responsible for forwarding DNS queries to a specified DNS server, and then returning the response to the original requester (i.e. the tester).

Your program will need to respond with a DNS reply packet that contains:

a header section (same as in stage #5)
a question section (same as in stage #6)
an answer section (new in this stage) mimicing what you received from the DNS server to which you forwarded the request.
Here are a few assumptions you can make about the tester -

It will always send you queries for A record type. So your parsing logic only needs to take care of this.
Here are few assumptions you can make about the DNS server you are forwarding the requests to -

It will always respond with an answer section for the queries that originate from the tester.
It will not contain other sections like (authority section and additional section)
It will only respond when there is only one question in the question section. If you send multiple questions in the question section, it will not respond at all. So when you receive multiple questions in the question section you will need to split it into two DNS packets and then send them to this resolver then merge the response in a single packet.
Remember to mimic the packet identifier value sent by the tester in your response.”

## Clarifications

### Session 2025-11-10

- Q: What timeout should be enforced per forwarded question before treating the upstream lookup as failed? → A: 200 ms per question.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Forward single-question packets (Priority: P0)

When the tester sends a single-question packet, the server must forward that packet to the configured resolver, relay the resolver’s response back to the tester, and preserve the DNS header fields that the tester expects (ID, flags, counts).

**Why this priority**: Without the basic forward-and-relay behavior, the stage’s happy path fails completely.

**Independent Test**: Start a mock upstream server that mirrors requests; send one-question packets to our server and assert the response matches the upstream packet byte-for-byte except for UDP/IP headers.

**Acceptance Scenarios**:

1. **Given** a DNS request listening on port 2053 with QDCOUNT=1, **When** the resolver replies, **Then** the server returns that answer to the tester with identical question and answer sections and matching header fields (QR=1, RCODE same as upstream, ANCOUNT from upstream).
2. **Given** a forwarded request that times out, **When** the timeout expires, **Then** the server replies with SERVFAIL (or drops the request) and logs diagnostics without crashing or hanging future requests.

---

### User Story 2 - Split multi-question requests (Priority: P1)

The tester may send multiple questions in one packet while the upstream resolver insists on a single question per request; the server must split the incoming questions into distinct forwarded packets, gather each response, and reassemble them into one response preserving order.

**Why this priority**: Without splitting, the upstream never answers, so forwarding silently stalls.

**Independent Test**: Craft a packet with two A questions. Verify the server issues two upstream queries (one per question) and responds with both question entries followed by two answers aggregated from upstream.

**Acceptance Scenarios**:

1. **Given** a request whose QDCOUNT>1, **When** the server processes it, **Then** it sends exactly one upstream query per question with that question as the sole entry in the forwarded packet.
2. **Given** multiple upstream responses, **When** the server composes the final reply, **Then** it concatenates the original question section (matching order) and merges answers so ANCOUNT equals the number of upstream packets processed; NSCOUNT/ARCOUNT remain zero unless upstream provided otherwise (which is out of scope per assumptions).

---

### User Story 3 - Resolver configuration & CLI (Priority: P2)

Operators need to specify the upstream resolver at runtime so the server can target different IP:port pairs without recompilation.

**Why this priority**: The tester invokes the binary via `./your_server --resolver <addr>`; without CLI parsing the binary cannot connect to the correct upstream.

**Independent Test**: Run `./your_server --resolver 1.1.1.1:53` and verify it binds to port 2053, opens a UDP socket to the resolver, and fails fast with a descriptive error if the flag is missing or malformed.

**Acceptance Scenarios**:

1. **Given** the CLI flag `--resolver <ip:port>`, **When** parsing succeeds, **Then** the server stores the parsed socket address and uses it for all forwarded packets.
2. **Given** the flag is missing or invalid, **When** the server starts, **Then** it prints usage guidance and exits with a non-zero status rather than running with a default.

### Edge Cases

- Upstream timeout or socket error must produce a controlled SERVFAIL (RCODE=2) or silent drop, never panicking.
- Requests whose total size exceeds 512 bytes should still be proxied verbatim; the server must not reserialize unless splitting multi-question packets.
- If upstream returns a different transaction ID, the server must overwrite it with the tester’s original ID before responding.
- When splitting multi-question packets, each sub-request must include identical header flags (RD, OPCODE) and the original ID so upstream responses are still matchable; internally we must correlate responses via question index, not by ID alone.
- Responses containing only header+question but zero answers (e.g., NXDOMAIN) must be forwarded as-is while still merging multiple upstream results.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The server MUST listen on UDP port 2053, parse inbound DNS packets, and extract header, question, and answer sections as defined in stages #5 and #6.
- **FR-002**: For each inbound packet, the server MUST forward the questions to the configured resolver(s) using UDP, preserving the tester’s transaction ID and relevant header flags (RD, OPCODE, etc.).
- **FR-003**: If the inbound packet has QDCOUNT=1, the forwarded packet MUST be byte-identical to the request (except possibly socket address metadata) to maximize upstream compatibility.
- **FR-004**: If QDCOUNT>1, the server MUST create one forwarding packet per question, each containing that question as the sole entry, send them sequentially or in parallel, and track which answer corresponds to which question.
- **FR-005**: Once all necessary upstream responses are collected (or time out), the server MUST construct a single response packet to the tester that mirrors the original header (ID, OPCODE, RD), sets QR=1, sets QDCOUNT to the original value, sets ANCOUNT equal to the number of answers included, and encodes the question section exactly as received.
- **FR-006**: The answer section in the tester-facing response MUST be a concatenation of the upstream answer RRs, preserving their NAME/TYPE/CLASS/TTL/RDLENGTH/RDATA bytes; no additional records may be fabricated beyond those returned by the resolver.
- **FR-007**: If any upstream query fails (timeout, format error), the server MUST either drop the entire tester request without replying or send a SERVFAIL response to the tester, and MUST NOT send partial, mixed-success responses; timeouts MUST be enforced at 200 ms per forwarded question.
- **FR-008**: The CLI MUST require a `--resolver <ip:port>` argument; failure to provide it results in a descriptive error and non-zero exit.
- **FR-009**: Logging and metrics (if any) MUST avoid leaking raw packet contents but should record high-level events: request received, forwarded, response assembled, error cases.

### Key Entities

- **ForwardingResolver**: Abstraction owning the upstream socket/timeout logic, capable of sending a DNS packet and awaiting a response.
- **QuestionSplitter**: Utility that converts a multi-question packet into a set of single-question sub-requests while keeping mapping metadata (question index, offsets).
- **ResponseAssembler**: Component that takes the original request + gathered upstream responses and produces the final tester-facing packet.
- **CliConfig**: Struct capturing parsed `--resolver` and future flags (e.g., timeout).

### Assumptions

1. Upstream resolver returns only question and answer sections, no authority or additional records, per tester guarantee.
2. All inbound questions are `TYPE=A`, `CLASS=IN`; we may drop or log anything else.
3. UDP payloads remain ≤512 bytes; we don’t need to handle TCP fallback or EDNS.
4. The upstream expects (and responds to) the same transaction ID we send; we can reuse the tester’s ID safely for each split sub-request because the upstream responds sequentially.
5. The tester sends at most a “handful” of questions per packet, so serial forwarding is acceptable with a fixed 200 ms per question timeout budget.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Integration tests show ≥99% success rate when proxying 100 single-question packets via a mock resolver; responses match upstream payloads byte-for-byte.
- **SC-002**: Multi-question test (≥3 questions) proves the server issues one upstream request per question and merges answers into a single response with correct QDCOUNT/ANCOUNT; verified via packet capture or assertions.
- **SC-003**: CLI parsing rejects missing or malformed `--resolver` values and prints actionable error text; covered by unit tests over argument parsing.
- **SC-004**: Timeout handling test confirms that if the upstream is unreachable, the server responds with SERVFAIL (RCODE=2) within the 200 ms per question timeout and continues serving subsequent requests.
- **SC-005**: Manual `dig @127.0.0.1 -p 2053 example.com A` with `--resolver 8.8.8.8:53` returns the same answers as querying 8.8.8.8 directly, demonstrating end-to-end fidelity.
