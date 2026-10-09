# ADR-0031 — Working State root boundary classification

Status: ACCEPTED
Date: 2026-10-09
Decision owner: Human
Related Design Gap: DG-0028

## Context

ADR-0030 defines the current Working State Base Revision and present
component-source Component States as reachability roots. Core §56 requires
declared history boundaries to match the referring object Identifier,
normative edge kind, and missing target Identifier. A Working State is
operational metadata with no historical object Identifier, and its root
references are not edges from immutable historical metadata.

## Decision

1. A missing Working State safety-reference target is `unresolved`.
   Validation MUST NOT classify it as `declared_incomplete` by matching a
   declared history boundary.
2. The exact declared-boundary lookup in Core §56 applies to required
   references from identified historical metadata objects. It does not apply
   to Working State safety-reference roots.
3. Core MUST NOT substitute a ProjectId, ComponentId, AdapterWorkingStateRef,
   or another Identifier for a nonexistent Working State Identifier in a
   boundary tuple, and MUST NOT invent a Working State edge kind for that
   lookup.
4. The unresolved root target remains included in the reachability result as
   a referenced identifier; its unknown outgoing edges cannot be traversed.
   Validation reports the unresolved Working State root finding, and history
   completeness is `unresolved`, not `declared_incomplete`.
5. This rule does not prevent a later, independently specified repository
   import contract from defining boundary metadata for operational roots.
   No such contract is introduced here.

## Rationale

The existing boundary identity contract requires a referring Identifier.
Working State intentionally has no assigned historical identifier, and
substituting another object's Identifier would change the meaning and
uniqueness of the declared tuple. Treating an absent root target as
unresolved is the only result available without broadening the boundary
contract or pretending the missing history has been intentionally declared.

## Alternatives considered

- Use the ProjectId as the boundary referrer and add edge kinds for Base
  Revision/component sources. Rejected: ProjectId identifies the Project, not
  the Working State record, and the existing Specs do not authorize this
  alias or tuple shape.
- Use a component key or AdapterWorkingStateRef as the referrer. Rejected:
  neither identifies the complete persisted Working State record; the
  Adapter reference is explicitly operational and opaque.
- Permit a declared boundary to match without a referring Identifier or edge
  kind. Rejected: this would weaken the exact-tuple contract in Core §56.

## Specification impact

- Core Specification §§19, 56, and 62.
- Glossary: Declared History Boundary and Reachability.
- Core Invariants: INV-WORK-007 and INV-GC-003.
- ADR-0027–0029 cross-references; ADR-0030.

## Test impact

WORK-0014 must verify that missing Working State Base Revision and
component-source targets remain `unresolved` even when operational boundary
metadata could otherwise appear relevant, and that no boundary lookup is
issued with a fabricated referrer or edge kind. Resolved targets are
traversed normally. Reachability retains unresolved identifiers and does not
infer outgoing edges.

## Implementation impact

WORK-0013 may report unresolved Working State-root targets with their root
context but performs no declared-boundary lookup. WORK-0014 reports these
targets as unresolved and reduces history completeness accordingly.
Contributions, archival pins, publication transactions, temporary
checkpoints, and import persistence remain outside this decision.

## Compatibility / migration impact

No immutable object schema or historical identity changes. Existing
declared-boundary records and their exact tuple semantics are unchanged.

## Notes

This decision resolves only whether existing declared-history-boundary
metadata can classify absent Working State safety-root targets.
