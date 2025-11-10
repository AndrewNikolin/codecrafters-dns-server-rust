# Tasks: DNS Question & Answer Echo

**Input**: Design documents from `/specs/005-question-answer/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Integration assertions accompany each user story to prove independent verification.

**Organization**: Tasks are grouped by user story so each slice can ship independently.

## Phase 1: Setup (Shared Infrastructure)

- [x] T001 Confirm `Cargo.toml` already lists `bytes`, `anyhow`, and `thiserror` so new parsing/serialization helpers reuse existing deps (Cargo.toml)
- [x] T002 Add the question+answer verification steps from the quickstart into `README.md` so contributors know how to test manually (README.md)

---

## Phase 2: Foundational (Blocking Prerequisites)

- [x] T003 Implement reusable label decoding/encoding helpers and the `DnsQuestion` structure from the data model (src/dns.rs)
- [x] T004 Add a validator that enforces QTYPE=1 and QCLASS=1, returning an error or drop signal otherwise (src/dns.rs)
- [x] T005 [P] Extend the integration harness with utilities to craft arbitrary headers/questions for future stories (tests/integration.rs)

**Checkpoint**: Parsing utilities exist; tests can craft packets confidently.

---

## Phase 3: User Story 1 - Parse and echo question (Priority: P1) 🎯 MVP

**Goal**: Parse the request question and mirror it byte-for-byte in the response.

**Independent Test**: Integration test sends a query for `example.test` and asserts the response question equals the request.

- [x] T006 [P] [US1] Parse the first question into `DnsQuestion` and store it alongside the header context (src/dns.rs)
- [x] T007 [US1] Serialize the parsed question back into the response right after the header, respecting the original label bytes (src/dns.rs)
- [x] T008 [P] [US1] Add integration test `dns_question::mirrors_question_section` ensuring NAME/QTYPE/QCLASS are unchanged and malformed (<12-byte) packets are dropped (tests/integration.rs)

**Checkpoint**: Question mirroring works; MVP achieved.

---

## Phase 4: User Story 2 - Construct deterministic answer record (Priority: P2)

**Goal**: Build a single A record using the parsed NAME, TTL 60, and IP `8.8.8.8`.

**Independent Test**: Integration test confirms the answer RR fields match spec for any requested domain.

- [x] T009 [US2] Implement `build_answer_record(question_name)` that fills TYPE=1, CLASS=1, TTL=60, RDLENGTH=4, RDATA=8.8.8.8 (src/dns.rs)
- [x] T010 [US2] Append the serialized answer to the response when parsing succeeds, ensuring domains other than `codecrafters.io` still use the requested NAME (src/dns.rs)
- [x] T011 [P] [US2] Add integration test `dns_question::returns_deterministic_answer` verifying TTL, RDLENGTH, and RDATA bytes (tests/integration.rs)

**Checkpoint**: Responses now include the deterministic answer.

---

## Phase 5: User Story 3 - Keep header/question/answer counts consistent (Priority: P3)

**Goal**: Ensure header counters (QDCOUNT/ANCOUNT/NSCOUNT/ARCOUNT) reflect serialized sections; drop packets with mismatched counts.

**Independent Test**: Crafted packet with QDCOUNT=2 but single question should be dropped; valid packets show mirrored QDCOUNT and ANCOUNT=1.

- [x] T012 [US3] Update header-writing logic so QDCOUNT mirrors the request, ANCOUNT=1, NSCOUNT=0, ARCOUNT=0, and inconsistent payloads trigger a drop (src/dns.rs)
- [x] T013 [P] [US3] Add integration test `dns_question::rejects_inconsistent_counts` covering both mirrored-count success and drop-on-mismatch scenarios (tests/integration.rs)

**Checkpoint**: Header counters and sections stay aligned under all scenarios.

---

## Phase 6: Polish & Cross-Cutting Concerns

- [x] T014 [P] Run `cargo fmt`, `cargo clippy --all-targets`, and `cargo test` to ensure formatting, lint, and integration suites pass (.)
- [x] T015 Update `specs/005-question-answer/quickstart.md` with any new troubleshooting notes discovered during testing (specs/005-question-answer/quickstart.md)

---

## Dependencies & Execution Order

1. Setup (T001–T002)
2. Foundational (T003–T005)
3. User Story 1 (T006–T008) → MVP
4. User Story 2 (T009–T011)
5. User Story 3 (T012–T013)
6. Polish (T014–T015)

## Parallel Execution Examples

- T003 and T005 can proceed in parallel once T002 completes (one implements parsers, the other enhances the test harness).
- Within US1, T006 (parser) and T008 (test skeleton) can start concurrently; T007 waits for T006.
- US2’s test (T011) can begin while T009 is under review, as long as the contract is defined.
- US3’s integration test (T013) can start as soon as T012’s API is drafted.

## Implementation Strategy

1. Deliver MVP by finishing US1 (T006–T008) to ensure reliable question parsing/echoing.
2. Layer deterministic answer serialization (US2) to satisfy stage requirements.
3. Harden header/section consistency (US3) to prevent malformed replies.
4. Run polish tasks (T014–T015) to keep the repo healthy and docs current.
