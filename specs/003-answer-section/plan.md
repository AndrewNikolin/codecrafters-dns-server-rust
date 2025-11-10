# Implementation Plan: DNS Answer Section Response

**Branch**: `003-answer-section` | **Date**: 2025-11-09 | **Spec**: [`specs/003-answer-section/spec.md`](specs/003-answer-section/spec.md)
**Input**: Feature specification from `/specs/003-answer-section/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Extend the existing UDP DNS stub so every query for `codecrafters.io` receives a standards-compliant answer section containing a single A record (NAME=`codecrafters.io`, TYPE=1, CLASS=1, TTL=60, RDLENGTH=4, RDATA=`8.8.8.8`). The plan focuses on deterministic packet construction using the current Rust networking stack (`UdpSocket`, `bytes`) while preserving legacy behavior for all other query names.

## Technical Context

**Language/Version**: Rust 1.80 (Edition 2021)  
**Primary Dependencies**: `std::net::UdpSocket`, `bytes` (buffer helpers), `anyhow`, `thiserror`  
**Storage**: N/A (stateless UDP responder)  
**Testing**: `cargo test` + `cargo clippy` integration checks  
**Target Platform**: POSIX-style environments (Codecrafters runner, macOS/Linux)  
**Project Type**: Single binary crate with integration tests  
**Performance Goals**: Respond within 100 ms locally; maintain zero-copy writes where possible  
**Constraints**: One-answer responses only, fixed TTL/IP, must not alter behavior for non-target domains  
**Scale/Scope**: Single domain (`codecrafters.io`) and single RR; single-threaded event loop with low request volume

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

The constitution file currently contains placeholder sections with no enforceable principles or gates. Documented governance is effectively undefined, so no constitutional violations are possible at this time. Gate status: **PASS (no active rules to satisfy)**.

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
├── main.rs          # entry point wiring UDP listener
└── dns.rs           # packet parsing/encoding helpers

tests/
├── integration.rs
└── integration/     # Codecrafters harness fixtures

specs/
└── 003-answer-section/  # Spec, plan, research, etc.
```

**Structure Decision**: Continue using the single-crate layout under `src/` with integration tests in `tests/`. No additional packages or services are required for this stage.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None | n/a | n/a |

## Phase 0 – Research

- Confirmed TTL, IPv4 payload, buffer strategy, and QTYPE handling in [`research.md`](specs/003-answer-section/research.md).
- Outcome: no open questions; implementation will hard-code TTL=60, IP=`8.8.8.8`, and always emit the A record when QNAME matches regardless of QTYPE while preserving existing behavior for other domains.

## Phase 1 – Design & Contracts

### Data Model
- Documented `DNSQuery`, embedded `Question`, `DNSAnswerRecord`, and `AnswerConfig` entities with validation rules and relationships in [`data-model.md`](specs/003-answer-section/data-model.md).

### Contracts
- Captured the logical interaction contract for the UDP handler using an OpenAPI-style description in [`contracts/dns-answer.yaml`](specs/003-answer-section/contracts/dns-answer.yaml), including payload expectations and deterministic header values.

### Quickstart
- Authored [`quickstart.md`](specs/003-answer-section/quickstart.md) covering prerequisites, run instructions, manual verification via `dig`, and automated test commands.

### Agent Context
- Ran `.specify/scripts/bash/update-agent-context.sh codex` to record Rust 1.80 + DNS stack details for future agents (see updated `AGENTS.md`).

## Constitution Check (Post-Design)

No new governance rules were introduced during Phase 1, so the earlier PASS status remains valid.
