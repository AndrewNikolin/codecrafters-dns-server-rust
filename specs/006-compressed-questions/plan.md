# Implementation Plan: DNS Question Compression Handling

**Branch**: `006-compressed-questions` | **Date**: 2025-11-09 | **Spec**: [`specs/006-compressed-questions/spec.md`](specs/006-compressed-questions/spec.md)
**Input**: Feature specification from `/specs/006-compressed-questions/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Add RFC-1035 compression support for the question section so incoming packets can mix uncompressed labels and pointer-based names, mirror all questions back uncompressed in the response, and emit one deterministic A-record answer per question while keeping header counts consistent.

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: Rust 1.80 (Edition 2021)  
**Primary Dependencies**: `std::net::UdpSocket`, `bytes`, `anyhow`, `thiserror`  
**Storage**: N/A (stateless UDP responder)  
**Testing**: `cargo test`, integration harness under `tests/integration/`, manual `dig` + crafted packets  
**Target Platform**: POSIX environments (Codecrafters runner, macOS/Linux)  
**Project Type**: Single binary crate with integration tests  
**Performance Goals**: Resolve compressed names within ≤10 pointer hops and respond within 5 ms locally  
**Constraints**: Must stay within 512-byte UDP payloads; no response compression required; QTYPE/QCLASS fixed to A/IN  
**Scale/Scope**: Multi-question packets (up to a handful) processed sequentially in memory

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

The constitution file contains only placeholders, so there are no active gates. Status: **PASS**.

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
└── dns.rs           # packet parsing + serialization (including compression decoder)

tests/
├── integration.rs   # entry point for scenario modules
└── integration/
    ├── dns_header.rs
    └── dns_question.rs (new compression-specific tests)

specs/
└── 006-compressed-questions/
```

**Structure Decision**: Continue using the single Rust crate; compression-aware parsing is built in `src/dns.rs` and new integration scenarios live under `tests/integration/`.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None | n/a | n/a |

## Phase 0 – Research

- Clarified pointer-resolution rules (max hop count, bounds checking) and decided to drop packets on invalid pointers.
- Documented policy for deterministic answers per question even when requests include duplicates.
- See `research.md` for decisions and alternatives.

## Phase 1 – Design & Contracts

### Data Model
- Captures `CompressedQuestion`, `DnsAnswerRecord`, and response envelope in `data-model.md`.

### Contracts
- Added `contracts/dns-question-compression.yaml` to describe expected request/response layout.

### Quickstart
- Manual verification steps (dig + crafted compressed packets) in `quickstart.md`.

### Agent Context
- Run `.specify/scripts/bash/update-agent-context.sh codex` after finalizing plan artifacts to keep `AGENTS.md` current.

## Constitution Check (Post-Design)

No constitutional changes; gate remains **PASS**.
