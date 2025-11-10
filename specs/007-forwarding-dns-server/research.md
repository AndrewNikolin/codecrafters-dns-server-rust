# Research Findings — Forwarding DNS Server (Phase 0)

## Decision 1: Upstream timeout policy
- **Decision**: Enforce a 200 ms timeout per forwarded question before treating the upstream lookup as failed.
- **Rationale**: Matches spec clarification, keeps overall latency bounded (<1 s even for 5 questions), and aligns with common recursive resolver defaults (100–300 ms). Prevents the tester from waiting indefinitely if the upstream misbehaves.
- **Alternatives considered**:
  - 500 ms per question — adds unnecessary latency budget and could slow retries.
  - Adaptive timeout based on RTT measurements — overkill for the short-lived runner environment and increases complexity.

## Decision 2: Sequential forwarding for multi-question packets
- **Decision**: Split multi-question requests and forward them sequentially (one after another) while reusing the tester’s transaction ID for each sub-request.
- **Rationale**: Tester guarantees only “a handful” of questions; sequential forwarding avoids juggling multiple outstanding sockets or correlators, and still finishes within the cumulative timeout budget (≤1 s). Simplifies mapping question index → upstream response.
- **Alternatives considered**:
  - Parallel forwarding on separate sockets — adds concurrency complexity and potential ID collisions; unnecessary for ≤5 questions.
  - Reusing a single forwarded packet with multiple questions — invalid per upstream constraint (would drop responses).

## Decision 3: CLI parsing with `clap`
- **Decision**: Use the `clap` crate to parse `--resolver <ip:port>` and future flags.
- **Rationale**: `clap` is idiomatic in Rust CLI projects, already used widely, and provides validation for socket addresses with minimal boilerplate. Keeps argument handling declarative and testable.
- **Alternatives considered**:
  - Manual `std::env::args` parsing — more error-prone, lacks automatic help/version output.
  - `argh`/`structopt` — smaller surface but less feature-rich; `clap` subsumes them and remains actively maintained.

## Decision 4: Response assembly strategy
- **Decision**: Mirror the original question section verbatim, then append upstream answer RRs in the order responses were received (matching split question order). Preserve upstream TTL/RDATA but overwrite header counts.
- **Rationale**: Guarantees QDCOUNT/ANCOUNT consistency, keeps question ordering stable for the tester, and avoids re-encoding names unnecessarily. Maintaining upstream answer bytes ensures fidelity (NXDOMAIN, TTL, etc.).
- **Alternatives considered**:
  - Regenerate answers manually from parsed structs — risks deviating from upstream-provided TTL/flags.
  - Reorder answers alphabetically — would violate tester expectations tied to original ordering.

## Decision 5: Error handling & SERVFAIL policy
- **Decision**: On any forwarding failure (timeout, socket error, malformed response), return a SERVFAIL response with zero answers while preserving the original question section; if serialization fails, drop the packet.
- **Rationale**: Keeps tester aware of upstream issues, matches DNS error semantics, and satisfies spec requirement to avoid partial responses. Dropping only occurs when we cannot form a valid DNS packet.
- **Alternatives considered**:
  - Always drop failing requests — gives the tester no feedback and complicates grading.
  - Partial answers (respond with successes only) — explicitly disallowed by spec and could mislead clients.
