# ADR-0018 — Working State lifecycle and Base Revision contract

Status: ACCEPTED
Date: 2026-10-09
Decision owner: Human
Related Design Gap: DG-0018

## Context

The Specs distinguish mutable local Working State from immutable Project State
and Revision history, but did not define its executable Core lifecycle,
persistence boundary, Base Revision behavior, optional Line association,
custom component sources, or Core change status. The human approved the
decisions below for OMVCS 0.1. DEC-INTERACTION-004 remains a separate open
decision about temporary local safety checkpoints.

## Decision

### Classification and identity

1. Working State is mutable local Project operational state, distinct from
   immutable Project State and Revision history. It is not Platform-owned,
   is not DAW-native dirty/unsaved state, and is not itself content-addressed
   historical metadata.
2. The existence or mutation of Working State MUST NOT create or mutate a
   Revision. Materialisation and local changes do not alter immutable
   Project State, Component States, or Resources.
3. OMVCS 0.1 does not require a globally stable content-derived
   `WorkingStateId`. An internal implementation handle, if needed, is not
   historical identity or provenance.
4. Working State is not a repository reachability root merely by existing.
   Explicit Working State safety references remain distinct roots under Core
   §62; their reachability policy is outside this ADR and belongs to later
   reachability work.

### Persistence and conceptual Core representation

5. The Core Working State representation is persistent local repository
   operational metadata and MAY survive process or DAW restart. Ephemeral
   runtime details that do not affect recovery semantics are not required in
   that record. Working State MUST NOT be stored as immutable historical
   objects. Durable provider mechanics remain outside WORK-0012.
6. The conceptual Core record identifies its Project and records:
   - an optional `base_revision_id`;
   - an optional `line_id`;
   - a component-source mapping from each represented `CreativeComponentId`
     to an optional `ComponentStateId`; and
   - references or metadata needed by the defined Core/Adapter boundary to
     identify the current mutable working content.
   This is an operational representation, not a closed, content-addressed
   JSON schema.
7. Mutable Adapter/project-level Working State data is referenced through
   the opaque `AdapterWorkingStateRef` defined by ADR-0025. The Adapter owns
   its representation and durable state; Core owns the committed reference
   and its atomic coordination with other Core Working State metadata.

### Base Revision and initial state

8. A present `base_revision_id` identifies the Revision from which this
   Working State was initially materialised or explicitly rematerialised.
   An absent value means the Project has no Base Revision, including before
   its first Revision. Base Revision is not inferred from the current Line
   head and does not advance when that Line moves.
9. A Project MAY have Working State before any Revision exists. That state
   has no Base Revision, no null or implicit initial Revision, and remains
   unpublished until a later publication workflow creates the first
   Revision.
10. A successful full materialisation from an admitted Revision sets that
    Revision as Base Revision, establishes the represented components'
    source mapping from its Project State, and starts Core change status as
    `unchanged`. Materialisation does not require Resource bytes to be
    available where existing selective/lazy materialisation rules permit
    their absence. It does not mutate its source historical objects.
11. Explicit component-source selection from another admitted Revision or
    Component State changes only the relevant component-source mapping; it
    does not implicitly change the Working State Base Revision. A locally
    created component has no historical source Component State until one is
    created by a later publication/admission workflow.
12. Publication completion alone does not advance or replace the current
    Working State Base Revision. A change of Base Revision requires an
    explicit successful rematerialisation operation. No Git rebase, reset,
    stash, staging, detached-HEAD, or branch-switch semantics are implied.
13. A Line association is optional. It means only that the Working State is
    currently being worked in the context of that stable Line identity. A
    valid Working State may have no Line association. Changing the
    association changes only Working State operational metadata; it does
    not move the Line, change Base Revision, create history, or transfer
    ownership. A later Line move does not change the association, silently
    rematerialise state, or discard local changes. Core MAY report that the
    Line target differs from Base Revision.

### Change status

14. Core Working State comparison status is the derived tri-state
    `unchanged`, `changed`, or `unknown`. `unchanged` means Core can
    establish semantic equality with the applicable recorded source/base
    comparison state; `changed` means Core can establish a relevant
    difference; `unknown` means the available evidence is insufficient.
    This result is not immutable persisted truth. Implementations MAY cache
    evidence operationally, but a stale cached result MUST NOT override the
    current comparison.
15. DAW-native dirty/unsaved state remains separate from Core Working State
    comparison status. Neither status may silently substitute for the other.
    Adapter evidence may contribute to Core comparison only under the
    existing explicit Core/Adapter comparison boundary.

### Operations and atomicity

16. The logical operations in Core §82 cover initial creation, full
    materialisation, custom/selective materialisation, inspection/change
    detection, and optional Line association. Existing §83 operation
    contract requirements apply. Successful Working State metadata updates
    are atomic at the Core contract boundary; failure MUST NOT leave a
    partially updated Base Revision, source map, or Line association.
17. Existing unpublished work MUST NOT be silently discarded. Replacing a
    `changed` or `unknown` Working State requires operation-scoped explicit
    discard authorization under ADR-0024; the default is `preserve`.
18. Temporary safety checkpoints, their UX, creation, retention, and
    cleanup are excluded and remain open under DEC-INTERACTION-004.
    WORK-0012 MUST NOT define their behavior.
19. Operation-specific failure, retry, and idempotency behavior follows
    ADR-0026 and the updated Core §83 contract.

## Specification impact

- Core Specification §§3, 19–25, 62, 82–83.
- Glossary: Working State, Base Revision, Materialisation, Selective
  Materialisation, Custom Working State, Local Modification,
  AdapterWorkingStateRef, and Working State recovery condition.
- Core Invariants: INV-PROJ-004–005 and INV-WORK-001–006.
- DAW Adapter Specification §§9, 12–13, 30, 35–44, 65–69, 119,
  159–163, and 183.
- Interaction Specification §§13–14, 43, 47, 50, and 308.
- Ardour Reference Adapter Design §133.
- Platform and Storage Adapter specifications were cross-searched; no
  change to their ownership or provider boundaries is made.
- WORK-0012 and WORK-0013 plans and the specification coverage/register.

## Test impact

WORK-0012 conformance planning must cover the accepted lifecycle, absence of
history effects, persistence boundary, optional Base Revision and Line
association, source mapping, Line movement, derived Core change status, and
separation from DAW-native dirty state, as well as the contracts in
ADR-0024–ADR-0026. Temporary checkpoint tests remain outside WORK-0012
under DEC-INTERACTION-004.

## Compatibility and scope

This decision narrows Working State semantics without importing Git behavior
or changing immutable historical object identity. It does not define
temporary checkpoint policy under DEC-INTERACTION-004, provider-specific
durability mechanics, or an Adapter-owned persistence encoding.
