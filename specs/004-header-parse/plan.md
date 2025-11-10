# Implementation Plan: DNS Header Parsing & Echo

**Branch**: `004-header-parse` | **Date**: 2025-11-09 | **Spec**: [`specs/004-header-parse/spec.md`](specs/004-header-parse/spec.md)
**Input**: Feature specification from `/specs/004-header-parse/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Implement full DNS header parsing so every inbound UDP packet has its ID, OPCODE, RD, and counter fields mirrored in the response while QR is forced to 1, AA/TC/RA/Z are zeroed, and RCODE is either 0 (standard query) or 4 (not implemented). The plan focuses on extracting the first 12 bytes safely, reusing them when legal, and guarding against malformed or truncated packets before later stages add question or answer sections.

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: Rust 1.80 (Edition 2021)  
**Primary Dependencies**: `std::net::UdpSocket`, `bytes` for buffer helpers, `anyhow`, `thiserror`  
**Storage**: N/A (stateless UDP responder)  
**Testing**: `cargo test`, integration harness under `tests/integration.rs`, optional fuzz harness later via `cargo test -- --ignored`  
**Target Platform**: POSIX-style environments (Codecrafters runner, macOS/Linux)  
**Project Type**: Single binary crate with integration tests  
**Performance Goals**: Parse and respond within 5 ms locally; avoid allocations beyond 512-byte working buffer  
**Constraints**: Must not mutate packet counts except when future specs require; header parsing must tolerate truncated payloads without panics  
**Scale/Scope**: Handles a single in-flight packet at a time; throughput bounded by UDP socket loop

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

The project constitution file contains only placeholders with no enforceable principles or gates, so there are no constraints to validate. Gate status: **PASS (no active constitutional requirements)**.

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
└── dns.rs           # packet parsing & serialization helpers

tests/
├── integration.rs   # shared test harness
└── integration/     # scenario-specific helpers

specs/
└── 004-header-parse/
```

**Structure Decision**: Continue with the single Rust crate layout; header parsing logic lives in `src/dns.rs` while `src/main.rs` orchestrates socket I/O. Integration coverage remains under `tests/integration.rs`.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None | n/a | n/a |

## Phase 0 – Research

- Verified DNS RFC 1035 header bit semantics for QR, OPCODE, RD, and RCODE to ensure mirroring rules align with standard behavior.
- Documented safe-handling approach for sub-12-byte payloads (drop silently; no attempt to echo ID).
- Outcome: No open unknowns; proceed with deterministic parsing strategy summarized in `research.md`.

## Phase 1 – Design & Contracts

### Data Model
- Captures request/response header structures, flag fields, and constraints in `data-model.md`.

### Contracts
- Logical UDP contract documented in `contracts/dns-header.yaml`, describing header expectations for round-trips.

### Quickstart
- Step-by-step validation instructions (manual `dig`, crafted payloads) captured in `quickstart.md`.

### Agent Context
- `.specify/scripts/bash/update-agent-context.sh codex` will keep the AGENTS briefing synced once design artifacts are finalized.

## Constitution Check (Post-Design)

No new constitutional rules emerged during Phase 1, so the gate remains **PASS**.
