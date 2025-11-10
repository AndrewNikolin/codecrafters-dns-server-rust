# Implementation Plan: DNS Question & Answer Echo

**Branch**: `005-question-answer` | **Date**: 2025-11-09 | **Spec**: [`specs/005-question-answer/spec.md`](specs/005-question-answer/spec.md)
**Input**: Feature specification from `/specs/005-question-answer/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Extend the DNS stub so it fully parses the question section (NAME/QTYPE/QCLASS), mirrors it in responses, and appends a deterministic A-record answer (TTL 60, RDATA 8.8.8.8) for whatever domain was requested while keeping header counts consistent.

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: Rust 1.80 (Edition 2021)  
**Primary Dependencies**: `std::net::UdpSocket`, `bytes`, `anyhow`, `thiserror`  
**Storage**: N/A (stateless UDP responder)  
**Testing**: `cargo test`, integration suite under `tests/integration.rs`, manual `dig` verification  
**Target Platform**: POSIX environments (Codecrafters runner, macOS/Linux)  
**Project Type**: Single binary crate with integration tests  
**Performance Goals**: Keep per-packet parsing under 5 ms and avoid allocations beyond local buffers  
**Constraints**: Only uncompressed QNAMEs (per assumption); QTYPE/QCLASS limited to A/IN; maintain header-question-answer alignment  
**Scale/Scope**: Handles single in-flight UDP packet; low QPS expected in challenge harness

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Constitution file is placeholder-only (no enforceable principles), so there are no gates to satisfy. Status: **PASS**.

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)
<!--
  ACTION REQUIRED: Replace the placeholder tree below with the concrete layout
  for this feature. Delete unused options and expand the chosen structure with
  real paths (e.g., apps/admin, packages/something). The delivered plan must
  not include Option labels.
-->

```text
src/
├── main.rs          # UDP entrypoint
└── dns.rs           # parsing & serialization helpers

tests/
├── integration.rs
└── integration/     # DNS scenario modules

specs/
└── 005-question-answer/
```

**Structure Decision**: Continue with the single-crate layout; new parsing and serialization code lives in `src/dns.rs`, while integration tests stay under `tests/integration.rs`.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None | n/a | n/a |

## Phase 0 – Research

- Documented approach for decoding label sequences, validating QTYPE/QCLASS, and ensuring header counters align.
- Decided to drop any packet whose QTYPE/QCLASS is not (1,1) to keep behavior deterministic for this stage.
- Outcome captured in `research.md`.

## Phase 1 – Design & Contracts

### Data Model
- Defined `DnsQuestion`, `DnsAnswerRecord`, and supporting structs with validation rules in `data-model.md`.

### Contracts
- Added `contracts/dns-question-answer.yaml` describing the expected header/question/answer layout for UDP interactions.

### Quickstart
- Authored manual + automated verification steps (dig, crafted payloads) in `quickstart.md`.

### Agent Context
- `.specify/scripts/bash/update-agent-context.sh codex` will keep `AGENTS.md` in sync after design artifacts are finalized.

## Constitution Check (Post-Design)

No new governance rules were introduced during Phase 1; gate remains **PASS**.
