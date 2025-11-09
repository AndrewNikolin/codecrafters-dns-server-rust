# Tasks: Lesson 2 DNS Question Section

**Input**: Design documents from `/specs/002-dns-question-section/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Integration tests are used to validate UDP behavior; additional tests can be added if needed.

**Organization**: Tasks are grouped by phase to keep each user story independently implementable and testable.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm the existing lesson-1 baseline before layering the question section.

- [X] T001 Run baseline integration suite from `./` using `cargo test --test integration` to capture current header-only behavior
- [ ] T002 Execute `scripts/smoke_probe.sh` to record the existing hexdump output prior to updating the question section

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Create shared serialization helpers used by every story.

- [X] T003 Add canonical `DnsQuestion` struct and label encoding helpers to `src/dns.rs`
- [X] T004 Introduce parsing/assertion helpers in `tests/integration/dns_header.rs` for reusing header/question byte checks across stories

---

## Phase 3: User Story 1 – Grader validates reply question (Priority: P1) 🎯 MVP

**Goal**: Ensure every DNS reply sets `QDCOUNT=1` and appends the canonical `codecrafters.io` question so Codecrafters grading passes.

**Independent Test**: Run `cargo test --test integration` and verify the new test asserting the question bytes plus the Codecrafters CLI for end-to-end confirmation.

### Implementation

- [X] T005 [US1] Update `src/dns.rs` to expose a `build_response_packet()` (or equivalent) that concatenates the header with the canonical question bytes and sets `QDCOUNT=1`
- [X] T006 [P] [US1] Wire `src/main.rs` to send the combined header+question packet for every probe and log the appended question length
- [X] T007 [P] [US1] Add an integration test `responds_with_question_section` in `tests/integration/dns_header.rs` asserting the header fields, domain labels, and Type/Class bytes
- [X] T008 [US1] Update `README.md` to mention Lesson 2’s question-section requirement and how the grader validates it

**Parallel Example (US1)**: While one contributor wires `src/main.rs` (T006), another can implement the integration test (T007) using the helpers from T004; documentation (T008) lands once behavior is verified.

---

## Phase 4: User Story 2 – Learner inspects question payload locally (Priority: P2)

**Goal**: Provide tooling and docs so learners can view the question section without relying solely on the grader.

**Independent Test**: Run `scripts/smoke_probe.sh` and confirm the script output highlights the question labels; follow quickstart instructions to compare expected bytes.

### Implementation

- [X] T009 [US2] Enhance `scripts/smoke_probe.sh` to highlight header vs question bytes (e.g., annotate label boundaries) in the printed hexdump
- [X] T010 [US2] Extend `specs/002-dns-question-section/quickstart.md` with explicit instructions for spotting the question section when using the smoke script or `netcat`

**Parallel Example (US2)**: Script updates (T009) and documentation (T010) can progress independently once the canonical bytes from US1 are known.

---

## Phase 5: User Story 3 – Server stays deterministic on malformed inputs (Priority: P3)

**Goal**: Guarantee the canonical question section is returned even for empty, oversized, or alternate-domain probes.

**Independent Test**: Run the updated integration tests that send malformed payloads and confirm the canonical question still appears without panics.

### Implementation

- [X] T011 [US3] Expand `tests/integration/dns_header.rs` with cases for empty payloads and alternate-domain probes that assert the question bytes remain canonical
- [X] T012 [US3] Review `src/main.rs` receive loop to ensure malformed payloads still call the canonical packet builder and improve log messages for these scenarios

**Parallel Example (US3)**: Additional tests (T011) can be authored while logging/loop updates (T012) are implemented, provided both agree on the expected canonical response.

---

## Final Phase: Polish & Cross-Cutting Concerns

**Purpose**: Keep the repository tidy and ready for submission.

- [X] T013 [P] Run `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test --test integration` from `./` to ensure style and test gates pass
- [X] T014 Capture the final hexdump (post-question) in `specs/002-dns-question-section/quickstart.md` or `README.md` as a reference screenshot/snippet for future lessons

---

## Dependencies & Execution Order

1. Complete Phase 1 (baseline capture) before editing code so regressions are clear.
2. Phase 2 foundational helpers (T003–T004) are prerequisites for all user stories.
3. US1 (P1) must be implemented before US2/US3 because it delivers the canonical question bytes the other stories rely on.
4. US2 and US3 can run in parallel after US1 stabilizes, as long as they avoid editing the same lines simultaneously.
5. Final polish (T013–T014) runs after all story work is merged.

## Parallel Execution Opportunities

- **US1**: T006 (server wiring) and T007 (integration test) can happen concurrently once T005 lands.
- **US2**: T009 (script) and T010 (docs) are independent and can be split across contributors.
- **US3**: T011 (tests) and T012 (runtime/log updates) can proceed in parallel, coordinating on expected outcomes.

## Implementation Strategy

1. **MVP First**: Deliver US1 to satisfy the grader with the canonical question section.
2. **Developer Experience**: Layer US2 improvements so learners can inspect responses locally without guesswork.
3. **Hardening**: Complete US3 to keep behavior deterministic under malformed inputs.
4. **Polish**: Finish with formatting, linting, and refreshed documentation/hexdumps before submitting to Codecrafters.

Tasks generated on 2025-11-08.
