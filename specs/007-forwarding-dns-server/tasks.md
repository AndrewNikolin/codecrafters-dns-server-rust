# Tasks: Forwarding DNS Server

**Input**: Design documents from `/specs/007-forwarding-dns-server/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: No dedicated test tasks were requested. Integration coverage is included within the story implementation steps.

**Organization**: Tasks are grouped by user story so each increment can be implemented and validated independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (changes different files, no blocking dependency)
- **[Story]**: User story label (US1, US2, US3). Setup/Foundational/Polish omit labels.
- Include exact file paths in every description.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Ensure the repository has the dependencies and scaffolding needed for forwarding work.

- [X] T001 Add the `clap` dependency (and features if required) to `Cargo.toml` and refresh `Cargo.lock`.
- [X] T002 Create `src/forwarder.rs` with module stubs and expose it via `mod forwarder;` in `src/main.rs`.
- [X] T003 [P] Add an integration test harness placeholder in `tests/integration/forwarding.rs` and register it inside `tests/integration.rs`.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core data structures shared by every user story.

- [X] T004 Define `ForwardResult`, `ForwardError`, `ForwardingJob`, and `SplitQuestion` structs in `src/forwarder.rs` following `data-model.md`.
- [X] T005 Implement the `ResponseAssembler` skeleton in `src/forwarder.rs` plus helper functions in `src/dns.rs` to serialize mirrored question sections.

**Checkpoint**: Shared forwarding primitives are ready; user stories can now build on them.

---

## Phase 3: User Story 1 – Forward Single-Question Packets (Priority: P0) 🎯 MVP

**Goal**: Forward single-question DNS packets to the upstream resolver and relay responses (or SERVFAIL on timeout) while mirroring tester headers/questions.

**Independent Test**: With the server running against a mock resolver, send a one-question packet and verify the reply matches the upstream answer byte-for-byte; simulate a blackhole resolver to confirm SERVFAIL after 200 ms.

### Implementation

- [X] T006 [US1] Implement `ForwardingResolver::send_and_recv` with a 200 ms read timeout and ID normalization in `src/forwarder.rs`.
- [X] T007 [US1] Wire the UDP server loop in `src/main.rs` to detect QDCOUNT=1, forward the packet via `ForwardingResolver`, and invoke `ResponseAssembler`.
- [X] T008 [US1] Extend `ResponseAssembler` in `src/forwarder.rs` to emit SERVFAIL responses (rcode=2, zero answers) when forwarding fails.
- [X] T009 [P] [US1] Add an integration scenario in `tests/integration/forwarding.rs` that covers successful single-question forwarding and SERVFAIL timeout behavior.

**Checkpoint**: MVP complete—the server can proxy single-question requests end-to-end.

---

## Phase 4: User Story 2 – Split Multi-Question Requests (Priority: P1)

**Goal**: Accept multi-question packets, split them into single-question upstream queries, forward sequentially, and merge the upstream answers in the tester response.

**Independent Test**: Craft a two-question packet (one compressed) and ensure the server issues two upstream lookups, then replies with both questions and both answers in order; confirm behavior when one upstream call times out (entire response becomes SERVFAIL).

### Implementation

- [X] T010 [US2] Implement `QuestionSplitter` logic in `src/forwarder.rs` that clones the request header/question into single-question DNS packets preserving IDs/flags.
- [X] T011 [US2] Extend `ForwardingJob` execution in `src/forwarder.rs` to sequentially forward each `SplitQuestion`, capturing `ForwardResult`s in original order.
- [X] T012 [US2] Update `ResponseAssembler` merge logic in `src/forwarder.rs` to concatenate question sections verbatim and append all upstream answers while keeping ANCOUNT/QDCOUNT consistent.
- [X] T013 [P] [US2] Add multi-question integration coverage (compressed + uncompressed labels) in `tests/integration/forwarding.rs`.

**Checkpoint**: Server correctly handles multiple questions per inbound packet.

---

## Phase 5: User Story 3 – Resolver Configuration & CLI (Priority: P2)

**Goal**: Require `--resolver <ip:port>` at startup, validate the socket address, and propagate configuration (including 200 ms timeout and bind address) through the server.

**Independent Test**: Run `./your_server --resolver 1.1.1.1:53` to confirm it binds to `0.0.0.0:2053`, rejects missing/invalid flags with non-zero exit, and uses the configured resolver for forwarding.

### Implementation

- [X] T014 [US3] Add a `CliConfig` struct plus `clap` derive/arg parsing in `src/main.rs`, ensuring `--resolver <ip:port>` is required.
- [X] T015 [US3] Validate resolver parsing (IPv4/IPv6) and emit helpful errors/usage strings in `src/main.rs`; plumb the parsed socket into `ForwardingResolver`.
- [X] T016 [P] [US3] Update `quickstart.md` and `README.md` with the new CLI flag usage and failure modes.

**Checkpoint**: Operators can configure the upstream resolver at runtime with defensive CLI validation.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final refinement across the feature.

- [X] T017 [P] Add logging/metrics hooks (e.g., `log` crate) around forwarding success/failure paths in `src/forwarder.rs` and `src/main.rs`.
- [X] T018 Run `cargo fmt && cargo clippy --all-targets` and resolve any warnings before final review.

---

## Dependencies & Execution Order

1. **Setup → Foundational**: Complete Phase 1 before Phase 2. Forwarding primitives rely on the new module scaffolding and dependencies.
2. **Foundational → User Stories**: Phases 3–5 must wait until shared structs/assemblers exist (Phase 2).
3. **User Story Priority Order**:
   - US1 (P0) is the MVP and should finish first.
   - US2 (P1) depends on US1’s forwarding path but can largely proceed in parallel once the resolver path is stable.
   - US3 (P2) can start after Setup since CLI parsing doesn’t depend on multi-question logic, but final validation requires US1 functionality.
4. **Polish** begins after desired user stories are complete.

---

## Parallel Execution Examples

- **US1**: T006 (resolver implementation) and T009 (integration test) can proceed in parallel once T004–T005 are done, because they touch different files.
- **US2**: T010 (splitter) and T013 (integration coverage) can run concurrently while T011–T012 focus on execution/assembly.
- **US3**: T014–T015 (code) can proceed while T016 updates documentation.
- **Setup**: T002 (module scaffolding) and T003 (test harness) can run in parallel after T001 updates dependencies.

---

## Implementation Strategy

### MVP First (User Story 1)
1. Finish Setup and Foundational phases.
2. Deliver US1 end-to-end (T006–T009) and validate via `dig`/mock resolver (per quickstart).
3. Ship MVP capable of forwarding single-question packets.

### Incremental Delivery
1. After MVP, implement US2 to support multi-question packets while keeping US1 green.
2. Add US3 to expose the CLI flag and configuration guardrails.
3. Conclude with Polish tasks (logging, linting) to stabilize the release.

### Parallel Team Strategy
- One developer can own US1 while another starts US3 CLI work (post-Setup). A third developer can begin US2 splitting logic once the resolver path is stable.
- Regularly merge via integration tests in `tests/integration/forwarding.rs` to ensure independent story validation.
