# Tasks: DNS Answer Section Response

**Input**: Design documents from `/specs/003-answer-section/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Only include when they materially de-risk a requirement. Here, integration assertions are added for each user story to keep the stories independently verifiable.

**Organization**: Tasks are grouped by user story so each increment can be delivered and tested independently.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm repository dependencies and documentation match the planned stack.

- [x] T001 Verify `Cargo.toml` declares `bytes`, `anyhow`, and `thiserror` dependencies required for the DNS answer work (Cargo.toml)
- [x] T002 Align manual verification steps in `specs/003-answer-section/quickstart.md` with the dig/cargo workflow to be run after implementation (specs/003-answer-section/quickstart.md)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core helpers shared by all stories.

- [x] T003 Introduce `AnswerConfig` (domain, IPv4, TTL) and expose an accessor in `src/dns.rs` so later tasks can reuse a single source of truth (src/dns.rs)
- [x] T004 Add a QNAME normalization helper (label decoding + lowercase compare) in `src/dns.rs` to ensure all stories can match `codecrafters.io` consistently (src/dns.rs)

**Checkpoint**: Domain constants and normalization helpers exist; user story work can begin.

---

## Phase 3: User Story 1 - Authoritative answer for codecrafters.io (Priority: P1) 🎯 MVP

**Goal**: Return a single RFC-compliant A record for `codecrafters.io` queries.

**Independent Test**: Send `dig @127.0.0.1 -p 2053 codecrafters.io A` and confirm `ANSWER: 1` with IPv4 `8.8.8.8`.

### Implementation

- [x] T005 [P] [US1] Implement `build_codecrafters_answer()` using `bytes::BufMut` to append NAME/TYPE/CLASS/RDLENGTH/RDATA per spec (src/dns.rs)
- [x] T006 [US1] Update the UDP handler to call the builder, append the RR to the response buffer, and bump ANCOUNT to 1 for matching queries (src/main.rs)
- [x] T007 [P] [US1] Extend `tests/integration.rs` to assert that a `codecrafters.io` query yields ANCOUNT=1 and contains the expected IPv4 payload (tests/integration.rs)

**Checkpoint**: `codecrafters.io` queries now return a valid answer and pass the integration assertion.

---

## Phase 4: User Story 2 - Cache-friendly TTL behavior (Priority: P2)

**Goal**: Ensure TTL stays fixed (60s) so caching resolvers behave predictably.

**Independent Test**: Issue consecutive `dig` queries for `codecrafters.io` and confirm TTL remains `60` every time.

### Implementation

- [x] T008 [P] [US2] Wire `AnswerConfig.ttl_seconds` (60) into the answer builder so TTL is encoded once in big-endian form for every record (src/dns.rs)
- [x] T009 [US2] Add/extend integration coverage to assert TTL stays 60 across multiple responses, guarding against accidental drift (tests/integration.rs)

**Checkpoint**: TTL is stable and verified, allowing caching resolvers to reuse responses for 60 seconds.

---

## Phase 5: User Story 3 - Non-target queries remain unaffected (Priority: P3)

**Goal**: Keep responses for other domains unchanged (no accidental answers).

**Independent Test**: Send `dig @127.0.0.1 -p 2053 example.com A` and confirm `ANSWER: 0`.

### Implementation

- [x] T010 [US3] Guard the response path so only normalized `codecrafters.io` questions trigger answer construction while others leave ANCOUNT at zero (src/main.rs)
- [x] T011 [P] [US3] Extend integration coverage to assert non-target domains and multi-question packets still produce zero answers (tests/integration.rs)

**Checkpoint**: Non-target queries behave exactly as before, proven by integration assertions.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Wrap-up tasks spanning multiple stories.

- [x] T012 [P] Run `cargo fmt`, `cargo clippy --all-targets`, and `cargo test` from repo root to ensure style, lint, and integration health (.)
- [x] T013 Update `README.md` or `specs/003-answer-section/quickstart.md` with any final verification notes discovered during implementation (README.md / specs/003-answer-section/quickstart.md)

---

## Dependencies & Execution Order

1. Setup (Phase 1) → ensure dependencies/docs ready.
2. Foundational (Phase 2) → required before any user story begins.
3. User Story phases follow priority order but can proceed in parallel after Phase 2 if staffing allows:
   - US1 (P1) must complete before declaring MVP.
   - US2 (P2) depends on US1’s builder existing but can start once T005 is in review.
   - US3 (P3) depends on normalization helper and server guard logic but can run alongside US2.
4. Polish (Phase 6) runs after desired stories finish.

## Parallel Execution Examples

- While T005 ([US1] builder) is in progress, another dev can tackle T007 ([US1] integration test) because it only depends on the expected contract, not on the code landing.
- After Phase 2, one engineer can finish US1 (T005–T006) while another starts US2’s TTL wiring (T008) since both touch `src/dns.rs` but in distinct functions.
- US3’s integration coverage (T011) can run in parallel with US2’s TTL checks as soon as the guard logic stub exists.

## Implementation Strategy

1. **MVP**: Complete US1 (T005–T007) so `codecrafters.io` serves a correct A record.
2. **Hardening**: Layer in TTL stability (US2) to satisfy caching behavior expectations.
3. **Regression Protection**: Finish US3 to guarantee other domains remain untouched.
4. **Polish**: Run toolchain commands and refresh docs to reflect the final behavior.
