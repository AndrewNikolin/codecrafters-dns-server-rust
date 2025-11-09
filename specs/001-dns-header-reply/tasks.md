# Tasks: Lesson 1 DNS Header Reply

**Input**: Design documents from `/specs/001-dns-header-reply/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Integration tests are included where they materially verify user stories; additional tests optional if more coverage desired.

**Organization**: Tasks are grouped by phase to keep each user story independently implementable and testable.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm environment readiness and understand the existing Rust scaffold before making changes.

- [X] T001 Verify Rust 1.80 toolchain and cargo availability per specs/001-dns-header-reply/quickstart.md prerequisites
- [X] T002 Run the current UDP stub in src/main.rs to capture baseline logs and ensure the binary starts without responding yet

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Establish shared modules and test harnesses required by every user story.

- [X] T003 Create src/dns.rs with `DnsHeaderResponse` struct and constant 12-byte header serialization per data-model.md
- [X] T004 Establish tests/integration/dns_header.rs harness with helpers to spawn `cargo run` and exchange UDP probes for reuse across stories

---

## Phase 3: User Story 1 – Codecrafters grader gets a reply (Priority: P1) 🎯 MVP

**Goal**: Ensure every probe from the grader receives the exact 12-byte DNS header with the specified field values within 200 ms.

**Independent Test**: Run `cargo test --test dns_header` or Codecrafters CLI to confirm a single probe gets the canonical header bytes (no question/record sections).

### Implementation

- [X] T005 [US1] Implement the blocking UDP loop in src/main.rs so each received packet triggers sending the static header from src/dns.rs within 200 ms
- [X] T006 [P] [US1] Add integration test `responds_with_fixed_header` in tests/integration/dns_header.rs asserting the returned bytes equal `04 d2 80 00 00 00 00 00 00 00 00 00`
- [X] T007 [US1] Document the Codecrafters grader command sequence in README.md so others can reproduce the acceptance test flow

**Parallel Example (US1)**: One contributor can code the UDP loop (T005) while another authors the integration test (T006) using the shared harness; both converge before updating documentation (T007).

---

## Phase 4: User Story 2 – Learner smoke-tests locally (Priority: P2)

**Goal**: Provide immediate feedback for learners running probes locally via logging and tooling.

**Independent Test**: Run `scripts/smoke_probe.sh` (created below) against `cargo run` and confirm logs show inbound/outbound events plus the expected hexdump.

### Implementation

- [X] T008 [US2] Enhance logging in src/main.rs to print inbound byte counts and when the standard header is sent so manual testers see confirmation
- [X] T009 [P] [US2] Create scripts/smoke_probe.sh that sends an empty UDP packet to 127.0.0.1:2053 and hexdumps the response for manual validation
- [X] T010 [US2] Extend specs/001-dns-header-reply/quickstart.md with explicit instructions for running scripts/smoke_probe.sh and interpreting the logs

**Parallel Example (US2)**: Logging improvements in src/main.rs (T008) can proceed while another contributor writes the smoke probe script (T009); documentation (T010) updates once both artifacts exist.

---

## Phase 5: User Story 3 – System handles malformed probes safely (Priority: P3)

**Goal**: Guarantee zero-length or oversized payloads still receive the canonical header without crashing the server.

**Independent Test**: Execute integration tests that send empty and >512-byte packets; verify both produce the same 12-byte header and the server keeps running.

### Implementation

- [X] T011 [US3] Harden src/main.rs receive loop to treat zero-length and oversized packets as valid triggers for reusing the standard header without panics
- [X] T012 [P] [US3] Add integration tests `responds_to_empty_payload` and `responds_to_large_payload` in tests/integration/dns_header.rs that cover the clarified edge cases

**Parallel Example (US3)**: While one developer updates the runtime behavior (T011), another can author the edge-case tests (T012) using the harness from Phase 2.

---

## Final Phase: Polish & Cross-Cutting Concerns

**Purpose**: Ensure quality gates (formatting, linting, documentation) are satisfied before submission.

- [X] T013 [P] Run `cargo fmt` and `cargo clippy -- -D warnings` at repo root to enforce Rust style and lint expectations
- [X] T014 Update specs/001-dns-header-reply/quickstart.md with troubleshooting tips (e.g., port already in use) observed during manual verification

---

## Dependencies & Execution Order

1. **Phase 1 → Phase 2**: Environment verification (T001–T002) must finish before creating shared modules/tests (T003–T004).
2. **Phase 2 → User Stories**: The dns module and integration harness are prerequisites for all user stories.
3. **User Story Priority**: US1 (T005–T007) delivers the MVP; US2 (T008–T010) and US3 (T011–T012) can run after Phase 2 and, if staffing permits, in parallel once US1 code paths stabilize.
4. **Polish Phase**: T013–T014 run after story-specific work completes.

## Parallel Execution Opportunities

- **US1**: T005 (implementation) and T006 (tests) can progress concurrently thanks to the shared harness; documentation T007 follows.
- **US2**: T008 (logging) and T009 (smoke script) are independent; T010 documents their combined workflow.
- **US3**: T011 (runtime guardrails) and T012 (edge-case tests) can be built in parallel, coordinating on expected behavior through spec clarifications.
- **Cross-Story**: Once Phase 2 is complete, US2 and US3 teams can work simultaneously provided they avoid touching the same sections of src/main.rs at the same time (coordinate via feature flags or sequencing).

## Implementation Strategy

1. **MVP First**: Complete US1 (T005–T007) to satisfy the grader and establish automated verification.
2. **Enhance Developer Experience**: Layer US2 improvements (logging + smoke script) to streamline local validation.
3. **Hardening**: Finish US3 to cover malformed inputs and keep the server stable.
4. **Polish**: Conclude with formatting, linting, and documentation tweaks (T013–T014) before submission.

tasks.md generated on 2025-11-08.
