# DG-0028 — Declared-boundary matching for Working State roots

Status: RESOLVED
Classification: BLOCKS-MILESTONE
Discovered by: Spec Guardian
Discovered during: M2 specification impact analysis
Date: 2026-10-09

## Relevant specifications

- `Specs/OMVCS Core Specification.md` §§19, 56, and 62.
- `Specs/OMVCS Glossary.md`, Working State, Declared History Boundary, and
  Reachability.
- `Specs/OMVCS Core Invariants Specification.md`, INV-WORK-007 and
  INV-GC-001–003.
- ADR-0027, ADR-0028, ADR-0029, ADR-0030, and ADR-0031.
- WORK-0013 and WORK-0014.

## Original problem (resolved)

ADR-0030 defines a present Base Revision and each present component-source
`ComponentStateId` as Working State safety-reference roots. If such a target
is absent, Core §56 requires validation to query for a declared history
boundary using the exact referring object Identifier, normative edge kind,
and target Identifier.

The persisted Working State is operational metadata, not a historical
object, and has no WorkingStateId. The Specs do not say whether its root
references may have declared-history-boundary declarations, what Identifier
would identify the referrer, or which normative edge kind identifies each
Working State root reference.

## Why the original specifications were insufficient

The missing-target classification changes history-completeness results:
matching a declaration produces `declared_incomplete`, while no match
produces `unresolved`. Using the ProjectId as a surrogate referrer, defining
new Working State edge kinds, or excluding these roots from boundary
matching each required a semantic decision not determined by ADR-0030 or
ADR-0027–0029. ADR-0031 resolves this question.

## Affected work

- WORK-0014 Working State-root validation and history-completeness findings.
- WORK-0013 unresolved-root result context.
- M2 closeout requires implementation and independent verification of the
  root and validation contracts.

## Can unaffected work continue?

Resolved. WORK-0013 may enumerate the approved Working State roots, traverse
resolved targets, retain unresolved-root context, and preserve partial
Core §62 coverage. WORK-0014 reports a missing Working State
safety-reference target as `unresolved`, with no declared-boundary lookup.

## Candidate directions

The alternatives below were non-normative; ADR-0031 selected no boundary
lookup and the `unresolved` finding.

## Required decision

The human decision was whether a missing Base Revision or component-source
safety-root target could match a declared history boundary without inventing
an Identifier or weakening the exact-tuple contract.

## Resolution

Resolved by human-approved [ADR-0031](../decisions/ADR-0031-working-state-root-boundary-classification.md).
Because a Working State has no historical Identifier, an absent Base
Revision or component-source safety-root target is reported as `unresolved`.
Core MUST NOT fabricate a referring Identifier or edge kind to query the
existing declared-history-boundary provider. This does not prevent a future
separately specified import contract from defining such a mechanism.
