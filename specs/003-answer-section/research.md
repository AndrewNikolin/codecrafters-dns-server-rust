# Phase 0 Research - DNS Answer Section Response

## Decision 1: TTL policy
- **Decision**: Hard-code TTL to 60 seconds for every `codecrafters.io` response.
- **Rationale**: Matches spec guidance, keeps caching behavior deterministic, and avoids clock drift concerns in an educational stub.
- **Alternatives considered**:
  - Dynamic TTL based on config or flags (overkill for single record stage).
  - Extremely low TTL (<=5s) to force frequent refreshes, which would increase network noise without added value.

## Decision 2: Response buffer implementation
- **Decision**: Build DNS responses with `bytes::BytesMut`/`BufMut` so NAME, TYPE, CLASS, TTL, RDLENGTH, and RDATA are appended sequentially in big-endian order.
- **Rationale**: `BytesMut` avoids repeated reallocations, provides endianness helpers, and already underpins earlier stages, reducing risk.
- **Alternatives considered**:
  - Direct `Vec<u8>` manipulation (more manual index math, easier to make off-by-one mistakes).
  - Using `ByteOrder` crate (extra dependency for marginal gain).

## Decision 3: IPv4 payload
- **Decision**: Return `8.8.8.8` (Google Public DNS) as the A record payload.
- **Rationale**: Public, memorable address; already cited in the spec and easy to assert against in tests.
- **Alternatives considered**:
  - `1.1.1.1` (Cloudflare) – equally valid but not referenced in exercise text.
  - `127.0.0.1` – would loop back to client and diverge from provided expectation.

## Decision 4: QTYPE handling for codecrafters.io
- **Decision**: If QNAME is `codecrafters.io`, always append the single A record even when the question's QTYPE is not `A`.
- **Rationale**: Aligns with the spec's Edge Case requirement to keep behavior deterministic for the stage while other record types remain unsupported.
- **Alternatives considered**:
  - Strictly honor QTYPE and omit the answer for non-A questions (would contradict spec directive).
  - Return NOTIMP/unsupported response codes (unnecessary complexity for this milestone).
