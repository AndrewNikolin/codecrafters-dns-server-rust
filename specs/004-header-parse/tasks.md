# Tasks: DNS Header Parsing & Echo

**Input**: Design documents from `/specs/004-header-parse/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Integration assertions are added per user story to keep each slice independently verifiable.

**Organization**: Tasks are grouped by user story so each increment can be delivered and tested independently.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm baseline dependencies and test harness alignment before modifying packet logic.

- [x] T001 Verify `Cargo.toml` already lists `bytes`, `anyhow`, and `thiserror` so header parsing helpers can reuse them without new deps (Cargo.toml)
- [x] T002 Document the `dig` + crafted-payload steps from the quickstart in README for broader team visibility (README.md)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared utilities that every story relies on.

- [x] T003 Define `DNSHeaderRequest` and `DNSHeaderResponse` structs described in the data model, plus conversion helpers (src/dns.rs)
- [x] T004 Implement a safe `parse_header` function that validates packet length (>=12 bytes) and returns an error for shorter payloads (src/dns.rs)
- [x] T005 [P] Extend the integration harness with helpers to craft raw DNS headers for future tests (tests/integration.rs)

**Checkpoint**: Parsing infrastructure exists and tests can easily craft packets; user story work can start.

---

## Phase 3: User Story 1 - Matching request identifiers (Priority: P1) 🎯 MVP

**Goal**: Echo the incoming 16-bit ID, preserve question counts, and set QR=1 for every valid packet.

**Independent Test**: Send a query with a known ID via integration test and assert the response ID matches and QR=1; send <12-byte payload and ensure no response.

### Implementation & Tests

- [x] T006 [P] [US1] Add builder logic that copies ID/QDCOUNT/ANCOUNT/NSCOUNT/ARCOUNT from `DNSHeaderRequest` into the response struct (src/dns.rs)
- [x] T007 [US1] Wire `parse_header` + builder into the UDP runtime, dropping malformed packets before reply (src/main.rs)
- [x] T008 [P] [US1] Add integration test `dns_header::echoes_transaction_id` validating ID/QR mirroring and malformed packet handling (tests/integration.rs)

**Checkpoint**: Server can safely parse headers and echo IDs—declared MVP.

---

## Phase 4: User Story 2 - Flag parity with requester (Priority: P2)

**Goal**: Mirror OPCODE and RD while forcing AA/TC/RA/Z to zero.

**Independent Test**: Integration tests flip OPCODE/RD bits and confirm only those bits are mirrored while AA/TC/RA/Z remain zero.

### Implementation & Tests

- [x] T009 [US2] Implement bitmask helpers to copy OPCODE/RD then set QR=1 and clear AA/TC/RA/Z before serializing (src/dns.rs)
- [x] T010 [P] [US2] Extend integration coverage with `dns_header::mirrors_opcode_and_rd` asserting correct bit behavior (tests/integration.rs)

**Checkpoint**: Responses now align with requester control bits while remaining deterministic elsewhere.

---

## Phase 5: User Story 3 - RCODE rules for unsupported operations (Priority: P3)

**Goal**: Return RCODE 0 for standard queries and 4 (Not Implemented) for any other OPCODE.

**Independent Test**: Crafted packets where OPCODE≠0 trigger RCODE 4; standard queries still yield 0.

### Implementation & Tests

- [x] T011 [US3] Add RCODE selection logic conditioned on parsed OPCODE (src/dns.rs)
- [x] T012 [P] [US3] Add integration test `dns_header::sets_rcode_for_unsupported_opcodes` covering both OPCODE paths (tests/integration.rs)

**Checkpoint**: Unsupported operations respond gracefully without breaking ID mirroring.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Repository-wide validation and documentation sync.

- [x] T013 [P] Run `cargo fmt`, `cargo clippy --all-targets`, and `cargo test` to ensure formatting, lint, and integration coverage pass (.)
- [x] T014 Update `specs/004-header-parse/quickstart.md` with any insights from manual testing (specs/004-header-parse/quickstart.md)

---

## Dependencies & Execution Order

1. Phase 1 completes baseline verification/documentation.
2. Phase 2 creates reusable parsing helpers and test utilities; user stories depend on it.
3. User stories proceed in priority order (US1 → US2 → US3). Later stories rely on the parsing/serialization hooks from earlier phases.
4. Polish tasks run after all stories to validate the codebase.

## Parallel Execution Examples

- T003 and T005 can run in parallel once T002 is done (one on parser, one on tests).
- Within US1, T006 (builder) and T008 (integration test skeleton) can progress simultaneously while T007 waits for T006 to land.
- US2’s masking logic (T009) can start as soon as T006 is merged, while US3’s test (T012) can be prepared in parallel with T011 once opcode handling is drafted.

## Implementation Strategy

1. Deliver MVP by finishing US1 (T006–T008); this unlocks correct ID mirroring and validates the parsing pipeline.
2. Harden interoperability via US2 flag parity tasks (T009–T010).
3. Add unsupported-opcode handling (US3) for graceful degradation.
4. Conclude with polish tasks to ensure repository health and updated docs.
