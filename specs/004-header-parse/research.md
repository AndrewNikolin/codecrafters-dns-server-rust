# Phase 0 Research - DNS Header Parsing & Echo

## Decision 1: Handling malformed headers
- **Decision**: Drop packets shorter than 12 bytes without emitting a response.
- **Rationale**: Without a full header we cannot safely mirror the ID or flags; dropping avoids corrupt replies and mirrors common DNS server behavior.
- **Alternatives considered**:
  - Return a static failure response (risk: sends incorrect IDs, may confuse clients).
  - Attempt partial parsing with zero padding (risk: undefined behavior, violates spec).

## Decision 2: RCODE policy for unsupported OPCODEs
- **Decision**: Emit RCODE 4 (Not Implemented) whenever OPCODE ≠ 0; standard queries keep RCODE 0.
- **Rationale**: Matches RFC 1035 guidance and Codecrafters instructions; keeps behavior deterministic for future stages.
- **Alternatives considered**:
  - Respond with SERVFAIL (RCODE 2) for unknown opcodes (less precise, implies transient errors).
  - Drop packet without responding (client would retry endlessly).

## Decision 3: Section counter strategy
- **Decision**: Mirror QDCOUNT/ANCOUNT/NSCOUNT/ARCOUNT exactly as received for this stage.
- **Rationale**: Maintains compatibility with earlier Codecrafters stages where counts are pre-set; defers ownership of counter mutations to future features (e.g., answer section injection).
- **Alternatives considered**:
  - Force ANCOUNT/NSCOUNT/ARCOUNT to zero regardless of input (could hide upstream issues, diverges from instructions allowing "any valid value").
  - Normalize QDCOUNT to 1 (breaks tests that purposely send different values).

## Decision 4: Flag mirroring granularity
- **Decision**: Copy OPCODE and RD bits exactly; override QR to 1 and clear AA/TC/RA/Z to 0 using bit masks after parsing.
- **Rationale**: Aligns with spec and ensures resolvers can rely on RD echoing, while AA/TC/RA/Z remain deterministic.
- **Alternatives considered**:
  - Rebuild header from scratch without mirroring (harder to keep counters in sync, more error-prone).
  - Mirror AA/TC if set by client (contradicts exercise requirement to force them to zero).
