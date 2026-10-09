# DG-0018 — Working State lifecycle and Base Revision contract

Status: RESOLVED
Classification: BLOCKS-FEATURE
Discovered by: Spec Guardian
Discovered during: M2 preflight
Date: 2026-10-09

## Relevant specifications

- `Specs/OMVCS Core Specification.md` §§3, 19–23, 25, 56, 62, 65, 82–83.
- `Specs/OMVCS Glossary.md`, Working State, Base Revision, Materialisation,
  Selective Materialisation, Custom Working State, and Local Modification.
- `Specs/OMVCS Core Invariants Specification.md`, INV-PROJ-004–005,
  INV-WORK-001–006, and INV-GC-001–002.
- `Specs/OMVCS DAW Adapter Specification.md` §§9, 11–13, 30, 35–44, 65–69,
  119, 153, 159–163, and 183.
- `Specs/OMVCS Interaction Specification.md` §§13–14, 33–36, 43, 121–124,
  and 262–267.
- Existing `DEC-INTERACTION-004` in `docs/decision-register.md` (temporary
  local safety checkpoint UX/lifecycle).

## Problem

The Specs settle important boundaries: Working State is mutable local state,
distinct from immutable Project State; materialisation does not create a
Revision; unpublished changes remain distinct from the Base Revision; the
Base Revision is the Revision from which the state was initially
materialised; and custom combinations do not become history automatically.
Core §19 gives an explicitly conceptual structure, while §§20–23 describe
materialisation and custom states. Adapter and Interaction requirements
distinguish native dirty state from OMVCS change detection and require
protection against silently discarding unpublished work.

The Specs do not define an executable Working State record/lifecycle or
which portions persist across process/session restart versus remain
ephemeral operational state. They do not define the initial Working State
for a Project before its first Revision, whether its Base Revision is absent
or represented another way, how a Working State relates to a Line (if at
all), what happens to the Base Revision when a Line head moves, or whether a
state with no current Line association has a defined Core status. They also
do not define which saved/current Project State and Component State
references form a custom state's source mapping, how component modifications
and newly created resources are represented, or how a persisted
“modified”/dirty indicator relates to adapter-reported `changed`,
`unchanged`, or `unknown`.

Temporary safety checkpoints are a narrower existing question under
DEC-INTERACTION-004 and are not duplicated here.

## Why the current specifications are insufficient

The conceptual JSON uses a non-null `base_revision` and a Boolean `modified`,
but does not state that those are normative field types or values. A Project
may have no Revision yet, and a custom Working State may combine states from
different Revisions while retaining one Base Revision. Choosing how to
represent that condition, persist it, associate it with Lines, or update it
after a head movement changes what the Core can report and recover. Calling
such a state “detached” or assigning Git detached-HEAD behavior would also
invent semantics.

## Affected work

- M2 Working State model, lifecycle, persistence, first-publication
  preparation, materialisation/update, and Base Revision tracking.
- M2 dirty/change-state reporting and Working State reference validation.
- Reachability of persisted Working State safety references, without
  deciding the separate temporary-checkpoint policy in DEC-INTERACTION-004.

## Can unaffected work continue?

Yes. Immutable historical Project State and Revision admission, Resource
identity, and materialisation's requirement not to alter history can
continue. Adapter-native dirty-state reporting may implement its explicitly
defined `unchanged`/`changed`/`unknown` contract independently, but the Core
must not turn that into an invented persistent Working State schema or
Base-advance rule.

## Candidate directions

None were selected when this gap was opened. The human-approved decisions
made to resolve it are recorded in ADR-0018; the remaining subcontracts are
tracked by DG-0024–DG-0026.

## Required decision

Define the OMVCS 0.1 Working State lifecycle and Core representation,
including what is persistent versus ephemeral; the pre-first-Revision case;
Base Revision presence and change rules; any Line association and behavior
when the Line head moves; custom Component State/source tracking; and the
meaning and persistence of dirty/change status. Keep DAW-native unsaved
changes distinct from Core's comparison with the Base Revision. Resolve
temporary checkpoint behavior separately under DEC-INTERACTION-004.

## Resolution

Resolved by human-approved
[ADR-0018](../decisions/ADR-0018-working-state-lifecycle-and-base-contract.md)
and its approved subordinate contracts:
[ADR-0024](../decisions/ADR-0024-working-state-destructive-replacement-authorization.md),
[ADR-0025](../decisions/ADR-0025-adapter-working-state-persistence-boundary.md),
and
[ADR-0026](../decisions/ADR-0026-working-state-operation-results-retry-and-partial-failure.md).
The affected Specs define the Core Working State classification, Base
Revision and optional Line association, pre-first-Revision state,
component-source mapping, derived Core change status, destructive
replacement authorization, AdapterWorkingStateRef persistence/recovery, and
operation failure/retry/idempotency behavior. Temporary checkpoints remain
separate under DEC-INTERACTION-004.
