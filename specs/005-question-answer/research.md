# Phase 0 Research - DNS Question & Answer Echo

## Decision 1: QTYPE/QCLASS handling
- **Decision**: Accept only QTYPE=1 (A) and QCLASS=1 (IN); drop packets with any other combination.
- **Rationale**: Stage requirements explicitly limit the scope; dropping unsupported combos keeps behavior deterministic and prevents forging incorrect records.
- **Alternatives considered**: Mirror unsupported types (would require additional serialization rules not covered yet); return SERVFAIL (overkill for Codecrafters harness).

## Decision 2: Non-compressed QNAME assumption
- **Decision**: Assume all QNAMEs are raw label sequences without compression pointers; if compression bits are detected, drop the packet.
- **Rationale**: Codecrafters stages introduce compression later; rejecting pointers avoids premature complexity and potential security issues.
- **Alternatives considered**: Implement RFC-compliant compression parsing now (more error-prone, beyond scope).

## Decision 3: Answer payload policy
- **Decision**: Always build a single A record using the requested NAME with TTL 60 and IPv4 `8.8.8.8`, regardless of domain.
- **Rationale**: Matches stage instructions and keeps tests predictable.
- **Alternatives considered**: Return NXDOMAIN for unknown names (not aligned with exercise), or use configurable IP (no requirement yet).

## Decision 4: Header/section counter alignment
- **Decision**: Mirror incoming QDCOUNT, force ANCOUNT=1, NSCOUNT=0, ARCOUNT=0 once an answer is appended.
- **Rationale**: Ensures serialized sections match header counters and simplifies future stages where additional records are added deliberately.
- **Alternatives considered**: Mirror ANCOUNT from request (risks lying about answer count), or leave counts unchanged (causes inconsistencies).
