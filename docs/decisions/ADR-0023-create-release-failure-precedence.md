# ADR-0023 — CreateRelease overlapping failure precedence

Status: ACCEPTED
Date: 2026-10-09
Decision owner: Human
Related Design Gap: DG-0023

## Context

Core §18 and ADR-0017 classify a stored Release identifier/body mismatch as an
integrity violation and a Project/name binding to another Release identifier
as a name conflict. Neither specified which result `CreateRelease` returns
when both conditions are observed for the same request. Core §83 requires
operation failure modes and results to be specified.

The human explicitly approved the narrow overlap decision below on
2026-10-09.

## Decision

When one `CreateRelease` request simultaneously encounters an existing
same-ReleaseId/different-canonical-body integrity violation and a
Project/name binding conflict, Core MUST return the integrity violation.
The name conflict MUST NOT mask it.

For this overlap, failure MUST remain atomic and MUST leave all stored
objects and Project/name bindings unchanged.

This decision specifies only the precedence of these two simultaneous
conditions. It does not alter either isolated failure category, other
precondition ordering, retry behavior, identity, or name semantics, and does
not establish a general failure-precedence framework.

## Rationale

The integrity condition concerns disagreement between a content-derived
identifier and immutable body bytes. Returning the secondary name conflict
instead would obscure that detected object-corruption condition. Reporting
the integrity failure first is a narrow deterministic response rule, while
preserving the no-mutation-on-failure guarantee.

## Specification impact

- Core Specification §18, `CreateRelease` failure wording.
- No other Spec behavior is changed by this bounded precedence decision.

## Test impact

WORK-0011's conformance plan requires the overlapping-condition test to
assert that the returned result is the integrity violation, not the name
conflict, and that all stored objects and Project/name bindings are unchanged.

## Implementation impact

The Core `CreateRelease` operation must return the integrity violation for
this overlap without changing stored objects or Project/name bindings. This
decision specifies the result and atomicity, not a general validation order.

## Compatibility

This defines only the previously unspecified outcome when both listed
conditions occur simultaneously. It does not change the individual outcome
for an isolated integrity violation or name conflict.
