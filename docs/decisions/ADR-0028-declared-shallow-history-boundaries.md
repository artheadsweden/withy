# ADR-0028 — Declared shallow-history boundaries

Status: ACCEPTED
Date: 2026-10-09
Decision owner: Human
Related Design Gap: DEC-CORE-009

## Context

Core permits shallow ancestry in principle but does not specify how an
intentional omission is distinguished from unexpectedly missing metadata.
The local-completeness decision in ADR-0027 requires a machine-readable
declaration without altering the historical reference.

## Decision

1. An import MAY intentionally omit historical metadata targets if each
   omitted required edge is represented in local operational metadata by a
   declared history boundary.
2. A boundary identifies at least:
   - the referring object Identifier;
   - the normative edge kind/reference;
   - the omitted target Identifier; and
   - the `intentional_omission` classification.
   A human-readable reason MAY also be recorded. The exact wire and storage
   representation is implementation-specific and belongs to the M4
   import/persistence work.
3. A boundary matches only the exact referring-object, edge-kind, and
   target-Identifier tuple. A provider lookup MUST distinguish a matching
   declaration, no matching declaration, and provider failure. Provider
   failure MUST NOT be treated as “no declaration”.
4. A matching boundary classifies an absent target as declared incomplete;
   it does not resolve, validate, fabricate, rewrite, or admit the target.
   Existing historical-object admission requirements remain in force for
   the referring object and any unavailable required target.
5. If the target later resolves, validation MUST validate the actual target
   and its references normally. The boundary MUST NOT mask an available
   target or permanently exclude it. `ValidateRepository` MUST NOT remove
   obsolete boundary records.
6. A missing target without a matching boundary is unresolved metadata.
   Neither a declared nor unresolved absence, by itself, establishes
   corruption or changes historical identity/provenance.

## Rationale

Exact edge matching lets a shallow import preserve the original immutable
objects while making intentional omissions distinguishable from unexpected
missing references. Keeping the declaration local and operational avoids
putting import-specific state into historical hashes.

## Alternatives considered

- Infer a boundary from any missing target. Rejected: it would conceal
  unexpected metadata loss.
- Treat a declaration as a substitute for the target or as proof that the
  target is valid. Rejected: it would fabricate resolution and could weaken
  admission checks.
- Permanently ignore a declared target if it later appears. Rejected: the
  declaration describes a current omission, not a permanent exclusion.
- Require a Core-wide serialized boundary format now. Rejected: M4 owns
  persistence and import transport mechanics; M2 needs only a testable
  Core-facing provider contract.

## Specification impact

- `Specs/OMVCS Core Specification.md` §§14, 29–30, 55–56, 66–67, and 83.
- `Specs/OMVCS Core Invariants Specification.md` INV-HIST-003,
  INV-HIST-009, INV-REC-002, and INV-REC-006.
- `Specs/OMVCS Glossary.md`, new Declared History Boundary term.

## Test impact

WORK-0014 must test exact matching and non-matching tuples, absent target
with and without a declaration, provider failure, and a later-resolved
target that must be validated despite a retained declaration. It must also
verify that declarations do not change a historical object’s bytes or
identity and do not waive its admission requirements.

## Implementation impact

WORK-0014 defines and contract-tests a provider-neutral query for an exact
boundary tuple. Body-derived Identifiers remain independent of target
resolution under ADR-0027. M4 defines durable representation and import
persistence.

## Compatibility / migration impact

No historical object body, hash, or identifier changes. Imports that
intentionally omit targets must record the corresponding local operational
declarations; an omission without one is unresolved.

## Notes

This decision does not define transaction mechanics for import, general
error precedence, or deletion/retention policy.
