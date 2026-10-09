# DG-0027 — Working State safety-reference reachability roots

Status: OPEN
Classification: BLOCKS-FEATURE
Discovered by: Spec Guardian
Discovered during: WORK-0013 preflight
Date: 2026-10-09

## Relevant specifications

- `Specs/OMVCS Core Specification.md` §§19, 62–64, 82–83.
- `Specs/OMVCS Glossary.md`, Working State, Base Revision, Adapter Working
  State Reference, and Reachability.
- `Specs/OMVCS Core Invariants Specification.md`, INV-WORK-001–006 and
  INV-GC-001–003.
- `Specs/OMVCS Interaction Specification.md` §§13–14, 33–36, 123, and 308,
  item 4.
- `Specs/OMVCS DAW Adapter Specification.md` §§118–119 and 183.
- `docs/decisions/ADR-0018-working-state-lifecycle-and-base-contract.md`,
  Decisions 4–6 and 18.
- `docs/decisions/ADR-0024-working-state-destructive-replacement-authorization.md`,
  Decisions 5 and 8.
- `docs/decisions/ADR-0025-adapter-working-state-persistence-boundary.md`,
  Decisions 2, 7–11.
- `docs/decisions/ADR-0026-working-state-operation-results-retry-and-partial-failure.md`,
  Decisions 9–10 and 15.
- `docs/plans/WORK-0013-repository-reachability.md`.

## Problem

Core §62 names “Working State safety references” as minimum roots and says
that only explicit safety references, not Working State merely by existing,
are roots. The Core Working State record is separately described in §19 and
ADR-0018 Decision 6, but neither defines which of its references, if any,
are safety references for reachability. ADR-0018 Decision 4 defers their
reachability policy. ADR-0025 expressly says the `AdapterWorkingStateRef`
is not itself a root, but does not identify the remaining root set.

No exact Working State safety-reference field/reference set or corresponding
reachability edges is identified for WORK-0013. DEC-INTERACTION-004 remains
open for temporary safety-checkpoint treatment, but checkpoint guidance does
not specify the complete Core reachability root set.

## Why the current specifications are insufficient

Treating a Base Revision, a component-source mapping entry, a checkpoint
reference, or another operational reference as a root—or omitting one that
is intended to be protected—changes which historical objects the reachability
calculation reports as protected. Core §62 does not identify those
references or define their root edges. ADR-0018 and ADR-0025 expressly leave
that policy unresolved, and DEC-INTERACTION-004 does not supply it.
Selecting a field set or edge would therefore add semantics rather than
implement an approved contract.

## Affected work

- WORK-0013 Working State-root enumeration and corresponding conformance
  tests.
- Any later operation that consumes WORK-0013 reachability results for
  Working State protection.

## Can unaffected work continue?

Yes. WORK-0013 may implement and test the independently specified Line and
Release roots, their approved edges, Revision ancestry traversal, and
metadata reachability independent of Resource-byte availability. It must not
claim complete Working State-root coverage or infer roots from the Working
State record or `AdapterWorkingStateRef` until this gap is resolved. This gap
does not define checkpoint creation, UX, retention, or cleanup.

## Candidate directions

None recorded. Any exact root-reference set or edge would be a semantic
choice.

## Required decision

Identify the exact explicit Working State safety references that are
reachability roots and the historical objects reached from each, sufficient
for WORK-0013 to implement and test that root class. Do not treat Working
State existence or `AdapterWorkingStateRef` itself as a root. This decision
does not settle temporary-checkpoint UX, creation, retention, or cleanup
under DEC-INTERACTION-004.

## Resolution

UNRESOLVED
