# Implementation Plan: Forwarding DNS Server

**Branch**: `007-forwarding-dns-server` | **Date**: 2025-11-10 | **Spec**: [`specs/007-forwarding-dns-server/spec.md`](specs/007-forwarding-dns-server/spec.md)
**Input**: Feature specification from `/specs/007-forwarding-dns-server/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Extend the Codecrafters DNS server to operate as a configurable forwarder: bind to UDP port 2053, accept DNS queries (possibly containing multiple questions), forward each question to an upstream resolver supplied via `--resolver <ip:port>`, and relay the upstream answers back to the tester while preserving header/question integrity. The implementation will reuse existing parsing/serialization modules, add a forwarding pipeline with deterministic timeouts (200 ms per question), and aggregate the upstream responses into a single reply packet that mirrors the original request ordering.

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: Rust 1.80 (Edition 2021)  
**Primary Dependencies**: `std::net::UdpSocket`, `bytes` for buffer helpers, `anyhow` + `thiserror` for ergonomics, `clap` for CLI parsing (new)  
**Storage**: N/A (stateless UDP responder)  
**Testing**: `cargo test`, integration harness under `tests/integration`, manual `dig`/mock resolver runs  
**Target Platform**: POSIX environments (Codecrafters runner, macOS/Linux)  
**Project Type**: Single binary crate (`src/main.rs`)  
**Performance Goals**: Forward single-question queries with ≤5 ms added latency and keep total turnaround ≤200 ms per upstream question  
**Constraints**: Fixed UDP payload ≤512 bytes, enforce 200 ms timeout per forwarded question, no authority/additional sections expected  
**Scale/Scope**: Handles “handful” (≤5) questions per packet, sequential forwarding acceptable thanks to tight timeout budget

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

The constitution file only contains placeholders with no enforceable principles or gates, so there are no active constraints blocking this plan. Status: **PASS**.

## Project Structure

### Documentation (this feature)

```text
specs/007-forwarding-dns-server/
├── plan.md              # This file (/speckit.plan output)
├── research.md          # Phase 0 research findings
├── data-model.md        # Phase 1 data contracts
├── quickstart.md        # Phase 1 verification steps
├── contracts/           # Phase 1 API/CLI & protocol contracts
└── tasks.md             # Phase 2 task decomposition (future)
```

### Source Code (repository root)

```text
src/
├── main.rs              # CLI + UDP entry point
└── dns.rs               # DNS header/question/answer parsing helpers

tests/
├── integration.rs       # Harness entry point
└── integration/
    ├── dns_header.rs    # prior stage tests
    └── dns_question.rs  # question/answer validation

your_program.sh          # helper script used by Codecrafters runner
```

**Structure Decision**: Continue with the single binary crate; forwarding-specific modules will live in `src/dns.rs` (parsers) and new helper modules (e.g., `src/forwarder.rs`) if needed, with new integration cases added under `tests/integration/`.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No constitution violations are anticipated, so complexity tracking is not required for this feature.

## Phase 0 – Research & Unknown Resolution

- **Focus Areas**: timeout policy, forwarding strategy for multi-question packets, CLI flag parsing, response aggregation, and error/SERVFAIL handling.
- **Method**: Reviewed DNS forwarding best practices and Codecrafters tester guarantees; see [`research.md`](research.md) for detailed decisions/rationales/alternatives.
- **Key Outcomes**:
  1. Hard 200 ms per-question timeout with SERVFAIL fallback.
  2. Sequential forwarding of split questions using shared transaction IDs.
  3. Adoption of `clap` for robust CLI parsing/validation.
  4. Pass-through of upstream answer bytes with mirrored question sections.
  5. Drop-only when serialization fails; otherwise emit SERVFAIL on upstream issues.

## Phase 1 – Design, Data Model & Contracts

- **Data Model**: Captured primary entities (`CliConfig`, `ForwardingJob`, `SplitQuestion`, `ForwardResult`, `ForwardingResolver`, `ResponseAssembler`, etc.) plus validation invariants in [`data-model.md`](data-model.md).
- **Contracts**: Authored [`contracts/dns-forwarder.yaml`](contracts/dns-forwarder.yaml) to document CLI requirements and DNS packet semantics via an OpenAPI-style schema, covering SERVFAIL behavior and answer mirroring.
- **Quickstart**: Documented build/run/test steps, including manual `dig` verification and timeout testing in [`quickstart.md`](quickstart.md).
- **Agent Context**: Will run `.specify/scripts/bash/update-agent-context.sh codex` after design artifacts are finalized to keep `AGENTS.md` aligned.

## Constitution Re-Check

No new constraints were introduced during design; the constitution remains placeholder-only. Status: **PASS**.

## Phase 2 – Implementation Outline (for `/speckit.tasks`)

1. **CLI & Config**: Integrate `clap` parsing for `--resolver`, validate socket addresses, and initialize shared `ForwardingResolver`.
2. **Packet Splitting**: Extend `dns.rs` (or new module) with helpers to clone headers/questions, enforce single-question packets, and track indices.
3. **Forwarding Loop**: Implement sequential forwarding with 200 ms deadlines, normalizing upstream IDs and collecting `ForwardResult`s.
4. **Response Assembly**: Build final DNS response (success or SERVFAIL), ensuring counts and sections align; add integration tests covering single/multi-question and timeout paths.
