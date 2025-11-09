# Implementation Plan: Lesson 1 DNS Header Reply

**Branch**: `001-dns-header-reply` | **Date**: 2025-11-08 | **Spec**: [specs/001-dns-header-reply/spec.md](specs/001-dns-header-reply/spec.md)  
**Input**: Feature specification from `/specs/001-dns-header-reply/spec.md`

## Summary

Lesson 1 requires a Rust binary that listens on UDP port 2053 and immediately returns a 12-byte DNS header with fixed field values (ID 1234, QR=1, OPCODE=0, flags zeroed, section counts zero) for every probe. The plan focuses on wiring the existing `src/main.rs` bootstrap to bind the socket, loop on `recv_from`, craft the static header, and echo it back quickly enough (<200 ms) while handling malformed packets without crashing.

## Technical Context

**Language/Version**: Rust 1.80 (Edition 2021 per Cargo.toml)  
**Primary Dependencies**: `std::net::UdpSocket`, `bytes` for buffer helpers, `anyhow` + `thiserror` for ergonomic error surfacing  
**Storage**: N/A (stateless UDP responder)  
**Testing**: `cargo test` (unit + integration), Codecrafters CLI harness, manual `netcat` probes  
**Target Platform**: Linux container invoked by Codecrafters (localhost networking)  
**Project Type**: Single CLI/daemon binary (`src/main.rs`)  
**Performance Goals**: Respond to every probe in <200 ms; sustain at least 5 sequential probes per run without restart  
**Constraints**: Must always bind `127.0.0.1:2053`, keep memory footprint small (<10 MB) and avoid blocking the loop between packets  
**Scale/Scope**: Single-lesson scope (one binary, header-only responses)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- The project constitution file currently contains placeholders with no enforceable principles. Adopt standard engineering hygiene (tests, docs) and proceed. **Pre-Phase Gate: PASS**
- Post-Phase 1 re-evaluation: no new principles introduced; design artifacts remain compliant. **Post-Phase Gate: PASS**

## Project Structure

### Documentation (this feature)

```text
specs/001-dns-header-reply/
├── spec.md
├── plan.md
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── contracts/           # Phase 1 output
```

### Source Code (repository root)

```text
src/
└── main.rs            # UDP server entry point

tests/
└── integration/
    └── dns_header.rs  # planned integration tests for UDP behavior

specs/
└── 001-dns-header-reply/
    ├── spec.md
    ├── plan.md
    └── ...
```

**Structure Decision**: Maintain the single-binary Rust layout; add an integration test target under `tests/integration/` to exercise UDP behavior without affecting the Codecrafters harness.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| _None_ |  |  |
