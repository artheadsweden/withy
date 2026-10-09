# DG-0020 — Line generation numeric domain

Status: RESOLVED
Classification: BLOCKS-FEATURE
Discovered by: Spec Guardian
Discovered during: DG-0016 resolution
Date: 2026-10-09

## Relevant specifications

- `Specs/OMVCS Core Specification.md` §§5, 7, 16–17, 58.
- `docs/decisions/ADR-0008-resource-byte-length-range.md`.
- `docs/decisions/ADR-0016-line-object-and-update-contract.md`.

## Problem

ADR-0016 defines Line `generation` as an unsigned monotonic token beginning
at zero, incrementing exactly once on successful mutation, never wrapping,
and requiring rejection when no next value is representable. Its
interoperable numeric domain and serialized representation remain
unspecified.

Core §7 and ADR-0008 define the RFC 8785/JCS interoperable safe-integer range
for Resource `byte_length`, but explicitly scope that bound to one Resource.
No general OMVCS integer profile for operational metadata or generations is
defined.

## Why the current specifications are insufficient

Applying the Resource-specific bound to Line generations would extend a
rule expressly scoped to Resource size. Choosing a host-language integer
width or another JSON/string encoding would create incompatible observable
Line records and overflow behavior.

## Affected work

- WORK-0010 Line serialization, generation validation, increment, and
  overflow/failure conformance.
- Any later protocol that reads or updates Core Line generations.

## Can unaffected work continue?

Yes. The Line field, start-at-zero rule, exact increment rule, no-wrap rule,
and rejection-on-exhaustion rule are settled. Work unrelated to serialization
and the numeric bound may proceed only if it does not accept or emit a
non-conforming Line generation representation. WORK-0010 remains blocked
until a numeric domain is approved.

## Candidate directions

The following are discussion material only and are NOT approved:

- use non-negative RFC 8785/JCS safe integers through `2^53 - 1`;
- define a decimal-string or other explicitly tagged larger integer profile.

## Required decision

Select the interoperable OMVCS 0.1 serialized representation and inclusive
numeric domain for Line `generation`, including the terminal value at which
the next successful mutation must be rejected.

## Resolution

Resolved by [ADR-0020](../decisions/ADR-0020-line-generation-numeric-domain.md)
and corresponding updates to Core §§16–17, 56, and 58; Glossary Line;
INV-HIST-010; ADR-0016; WORK-0010; the decision register; and specification
coverage.

The human-approved OMVCS 0.1 Line-specific representation is a JSON number
encoding an exact integer in `0 ..= 9007199254740991`. Negative, fractional,
above-maximum, and string/alternate values are invalid. Host integer width
does not change the wire domain. Incrementing from the maximum fails
atomically without modifying the Line. The same numeric bound in ADR-0008
remains scoped to Resource `byte_length` and is not a general integer profile.
