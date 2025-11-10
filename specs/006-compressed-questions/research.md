# Phase 0 Research - DNS Question Compression Handling

## Decision 1: Pointer resolution strategy
- **Decision**: Implement RFC-1035 pointer resolution with a strict hop limit (10) and bounds checks; drop packets on overflow or loops.
- **Rationale**: Prevents infinite recursion and guards against malicious offsets while covering the small pointer chains typical in tests.
- **Alternatives considered**: Unlimited recursion (risk: DoS); pre-flatten entire packet before parsing (more complex, unnecessary for small packets).

## Decision 2: Answer IP/TTL policy
- **Decision**: Continue returning TTL=60 and IPv4 `8.8.8.8` for every question.
- **Rationale**: Matches prior stages and keeps assertions deterministic.
- **Alternatives considered**: Domain-specific IPs (adds state/config not required), dynamic TTL (no added value).

## Decision 3: Response question encoding
- **Decision**: Always serialize questions uncompressed even if requests used pointers.
- **Rationale**: Simpler to reason about and satisfies grader expectations; compression would only reduce bytes marginally.
- **Alternatives considered**: Preserve original compression (harder to implement/test) or convert everything to text (would require additional encoding step anyway).

## Decision 4: Error handling policy
- **Decision**: Drop the entire packet (no response) when any question fails to parse or violates constraints.
- **Rationale**: Avoids partial responses that could confuse clients and aligns with earlier stage behavior.
- **Alternatives considered**: Send SERVFAIL/FORMERR responses (not required by Codecrafters, adds header logic).
