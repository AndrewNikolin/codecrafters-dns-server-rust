# Implementation Plan: Lesson 2 DNS Question Section

**Branch**: `002-dns-question-section` | **Date**: 2025-11-08 | **Spec**: [specs/002-dns-question-section/spec.md](specs/002-dns-question-section/spec.md)
**Input**: Feature specification from `/specs/002-dns-question-section/spec.md`

## Summary

Extend the Lesson 1 UDP responder so every reply includes a canonical DNS question section for `codecrafters.io` (QDCOUNT=1, Type=1, Class=1) appended after the header. Work centers on serializing label-encoded domain names, updating header counts, and ensuring deterministic output regardless of incoming payloads. Supporting tasks include updated integration tests, documentation, and tooling so learners can inspect the new bytes locally.

## Technical Context

**Language/Version**: Rust 1.80 (Edition 2021)  
**Primary Dependencies**: `std::net::UdpSocket`, `bytes` for buffer manipulation, `anyhow`/`thiserror` for ergonomics  
**Storage**: N/A (stateless UDP responder)  
**Testing**: `cargo test` (unit + integration), Codecrafters CLI, smoke script  
**Target Platform**: Localhost Linux container invoked by Codecrafters  
**Project Type**: Single binary application (`src/main.rs`)  
**Performance Goals**: Reply to every probe in <200 ms; sustain 5+ sequential probes without restart  
**Constraints**: Always bind `127.0.0.1:2053`, responses capped at header+question only, deterministic output no matter the request  
**Scale/Scope**: Lesson-sized change (single question, single branch)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- The constitution file remains a placeholder with no enforceable directives; default engineering hygiene (tests + docs) applies. **Pre-Phase Gate: PASS**
- After completing Phase 0/1 artifacts (research, data model, contracts, quickstart) no new constraints emerged, so compliance still holds. **Post-Phase Gate: PASS**

## Project Structure

### Documentation (this feature)

```text
specs/002-dns-question-section/
├── plan.md
├── research.md          # created in Phase 0
├── data-model.md        # created in Phase 1
├── quickstart.md        # created in Phase 1
└── contracts/           # created in Phase 1
```

### Source Code (repository root)

```text
src/
├── main.rs              # UDP loop and logging
└── dns.rs               # shared DNS header/question helpers (to be extended)

tests/
└── integration/
    └── dns_header.rs    # UDP integration tests (will be expanded for questions)

scripts/
└── smoke_probe.sh       # learner-facing probe helper
```

**Structure Decision**: Keep the single-binary layout. Extend existing modules (`src/dns.rs`, `tests/integration/dns_header.rs`, `scripts/smoke_probe.sh`) rather than adding new crates to minimize complexity for Codecrafters learners.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| _None_ |  |  |
