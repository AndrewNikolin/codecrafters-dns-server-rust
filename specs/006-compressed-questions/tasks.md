# Tasks: DNS Question Compression Handling

**Input**: Design documents from `/specs/006-compressed-questions/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Integration assertions per user story ensure each slice is independently verifiable.

**Organization**: Tasks are grouped by user story to preserve incremental delivery.

## Phase 1: Setup (Shared Infrastructure)

- [x] T001 Confirm `Cargo.toml` already contains the required crates (`bytes`, `anyhow`, `thiserror`) for compression parsing helpers (Cargo.toml)
- [x] T002 Document the compression-focused verification workflow from `quickstart.md` inside `README.md` (README.md)

---

## Phase 2: Foundational (Blocking Prerequisites)

- [x] T003 Implement a reusable label decoder that can follow RFC-1035 compression pointers with hop/bounds checks (src/dns.rs)
- [x] T004 Add pointer loop/out-of-range detection (max 10 hops, drop on violation) and expose errors up the call stack (src/dns.rs)
- [x] T005 [P] Extend the integration harness with utilities to craft compressed/uncompressed questions and assert drops (tests/integration/dns_question.rs)

**Checkpoint**: Compression-safe parsing utilities exist; tests can craft scenarios.

---

## Phase 3: User Story 1 - Decode compressed questions (Priority: P1) 🎯 MVP

**Goal**: Parse every question (compressed or not) into canonical label sequences.

**Independent Test**: Integration sends a mixed packet and asserts decoded domains match expectations; malformed pointers cause a drop.

- [x] T006 [US1] Wire the compression-aware decoder into the packet parsing loop so all questions are collected into `CompressedQuestion` structs (src/dns.rs)
- [x] T007 [P] [US1] Add integration test `dns_question::parses_compressed_questions` covering a pointer to previous labels plus invalid pointer cases (tests/integration/dns_question.rs)

**Checkpoint**: Parser handles compressed inputs without crashing; drop policy validated.

---

## Phase 4: User Story 2 - Mirror question section uncompressed (Priority: P2)

**Goal**: Serialize the response question section using fresh label bytes (no pointers) while preserving order/QTYPE/QCLASS.

**Independent Test**: Integration confirms response questions are uncompressed and identical to decoded names.

- [x] T008 [US2] Implement a serializer that writes each question (NAME/QTYPE/QCLASS) from the parsed structs using length-prefixed labels (src/dns.rs)
- [x] T009 [P] [US2] Add integration test `dns_question::mirrors_questions_uncompressed` verifying QDCOUNT and serialized bytes (tests/integration/dns_question.rs)

**Checkpoint**: Responses mirror questions exactly; ready for answering logic.

---

## Phase 5: User Story 3 - Emit one A-record per question (Priority: P3)

**Goal**: Produce a deterministic A-record answer for every parsed question and keep header counts aligned.

**Independent Test**: Integration sends multi-question packets and asserts ANCOUNT=QDCOUNT with correct TTL/IP.

- [x] T010 [US3] Generate one A-record per parsed question (NAME from question, TYPE=1, CLASS=1, TTL=60, RDATA=8.8.8.8) and append to the response (src/dns.rs)
- [x] T011 [US3] Update header-writing logic so ANCOUNT equals number of answers while NSCOUNT/ARCOUNT stay zero (src/dns.rs)
- [x] T012 [P] [US3] Add integration test `dns_question::answers_each_question` covering multi-question packets (tests/integration/dns_question.rs)

**Checkpoint**: Multi-question packets are fully serviced with aligned header counters.

---

## Phase 6: Polish & Cross-Cutting Concerns

- [x] T013 [P] Run `cargo fmt`, `cargo clippy --all-targets`, and `cargo test` to ensure formatting, lint, and integration suites pass (.)
- [x] T014 Update `specs/006-compressed-questions/quickstart.md` with any new troubleshooting notes discovered during implementation (specs/006-compressed-questions/quickstart.md)

---

## Dependencies & Execution Order

1. Setup (T001–T002)
2. Foundational (T003–T005)
3. User Story 1 (T006–T007) → MVP
4. User Story 2 (T008–T009)
5. User Story 3 (T010–T012)
6. Polish (T013–T014)

## Parallel Execution Examples

- T003 and T005 can proceed in parallel (parser vs. tests) once T002 is done.
- During US1, T006 (implementation) and T007 (tests) can work concurrently with coordination on expected structures.
- US2’s serializer (T008) can start as soon as T006 completes; T009 can begin once serialization format is defined.
- In US3, T010 and T011 touch `src/dns.rs` sequentially, while T012 can begin drafting tests based on the planned answer format.

## Implementation Strategy

1. Deliver MVP by finishing US1 (compression parsing & safety).
2. Mirror questions uncompressed (US2) to satisfy response requirements.
3. Emit deterministic answers for every question (US3) and finalize with polish tasks.
