# ADR-0020 — Line generation numeric domain

Status: ACCEPTED
Date: 2026-10-09
Decision owner: Human
Related Design Gap: DG-0020

## Context

ADR-0016 established Line `generation` as an unsigned monotonic concurrency
token that starts at zero, increments once for every successful Line-record
mutation other than deletion, never wraps, and causes an atomic failure when
the next value cannot be represented. It left the interoperable numeric
domain and serialized representation open.

ADR-0008 independently establishes the same safe-integer bound for Resource
`byte_length`, but that decision is specifically Resource-length scoped and
does not define a general OMVCS integer profile.

## Decision

1. In OMVCS 0.1, a Line's `generation` is encoded as a JSON number and MUST
   represent an exact non-negative integer in the inclusive range
   `0 ..= 9007199254740991` (`0 ..= 2^53 - 1`).
2. Negative values, fractional values, values greater than
   `9007199254740991`, and string or other alternate representations are
   invalid. Host-language integer width MUST NOT change the accepted or
   emitted interoperable domain. Implementations MAY use a wider internal
   integer if all externally visible values obey this decision.
3. Where canonical JSON serialization is applied, the `generation` JSON
   number MUST use the RFC 8785/JCS number serialization rules in Core §5.
4. A newly created Line has generation zero. Every successful mutation of
   the Line record that leaves it existing increments generation by exactly
   one as required by ADR-0016. Failed mutations leave the record unchanged.
5. If the current generation is `9007199254740991`, any operation requiring
   another increment MUST fail atomically. The Line record remains unchanged.
   Generation MUST NOT wrap, reset, silently saturate, or reuse an earlier
   value.
6. Successful DeleteLine removes the record and therefore does not create a
   next generation; its expected-generation check is defined separately by
   ADR-0022.
7. This is a Line-specific numeric decision. It independently selects the
   same boundary as ADR-0008 because it is exactly interoperable under the
   OMVCS RFC 8785/JCS JSON model. ADR-0008 does not establish a general
   OMVCS integer profile.

## Rationale

The JCS safe-integer boundary gives every implementation one exact JSON
number domain and stable canonical number representation. Making the
selection explicitly Line-specific avoids extending ADR-0008 beyond Resource
`byte_length` while preventing host-dependent generation ranges.

## Alternatives considered

- Reusing the Resource-specific bound by implication was rejected because
  ADR-0008 is explicitly scoped to Resource `byte_length`.
- Host-language integer widths were rejected because they would allow
  different interoperable Line records and exhaustion points.
- String-encoded integers or a wider alternate JSON encoding were rejected
  for OMVCS 0.1 because the approved decision selects an exact JSON-number
  domain.

## Specification impact

- `Specs/OMVCS Core Specification.md` §§5, 16–17, 56, and 58.
- `Specs/OMVCS Glossary.md`, Line.
- `Specs/OMVCS Core Invariants Specification.md`, INV-HIST-010.
- `docs/decisions/ADR-0016-line-object-and-update-contract.md`.

## Test impact

WORK-0010 must accept `0`, ordinary positive values, and
`9007199254740991`; reject `9007199254740992`, negative, fractional, and
string values; verify canonical serialization stability; verify successful
mutations increment by exactly one; and verify each increment-requiring
mutation at the maximum fails atomically without wrapping, resetting, or
saturating.

## Implementation impact

- WORK-0010 validates and serializes the approved Line generation domain
  and applies the terminal-value failure to successful MoveLine and
  RenameLine mutations.
- DeleteLine checks its expected generation and removes the record without
  incrementing it under ADR-0022.

## Compatibility / migration impact

No production implementation or persisted record is changed by this
documentation decision. Any Line record using a generation outside this
domain or a non-number representation is invalid in OMVCS 0.1.

## Notes

This ADR defines only the Core Line record's `generation`. Platform Mirror
`mirror_generation` and other operational generations remain governed by
their respective contracts.
