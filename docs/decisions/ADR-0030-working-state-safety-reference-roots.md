# ADR-0030 — Working State safety-reference roots

Status: ACCEPTED
Date: 2026-10-09
Decision owner: Human
Related Design Gap: DG-0027

## Context

Core §62 requires Working State safety-reference roots but did not identify
which references in the persisted Working State record protect immutable
history. ADR-0018 establishes that Working State is operational state, not a
root merely because it exists, and defers the safety-reference policy.

## Decision

1. Reachability uses the currently persisted Working State record. Working
   State itself remains operational metadata and is not a historical root
   object.
2. A present `base_revision_id` is a Working State safety-reference root.
   The root edge is from the Working State record to that Revision, and
   traversal follows the normal historical metadata edges from that
   Revision.
3. Every present `ComponentStateId` in the component-source mapping is a
   Working State safety-reference root, including a source not reachable
   from the Base Revision. An absent source contributes no root.
4. The Working State record has no separate immutable historical
   `AdapterStateId` reference. No Adapter State root is inferred from
   `AdapterWorkingStateRef`.
5. `line_id` is not an additional Working State safety root. The associated
   Line is rooted by the Line contract; its target neither replaces nor
   supplements the Working State Base Revision as a Working State-derived
   root.
6. The recovery condition and Core change status do not produce roots.
   Temporary checkpoints receive no root semantics from this decision;
   DEC-INTERACTION-004 remains separate.
7. Root traversal is set-based and deduplicated when roots or traversals
   converge. Reachability reflects the current persisted record: changing
   its Base Revision or source mapping changes the next calculation.
   Line movement alone does not change the Working State safety roots.
   Publication completion alone does not change them unless an explicit
   Working State operation changes the persisted record.
8. A pre-first-Revision Working State with no Base Revision and no historical
   component sources contributes zero historical safety roots. A custom
   Working State may root its Base Revision and any present source Component
   States from other Revisions; no synthetic Revision is created.
9. These roots protect references into immutable history. They create no
   provenance, mutate no historical object, and grant no deletion or
   retention authority. Contributions, configured archival pins, and
   pending publication transactions remain separate Core §62 root classes.

## Rationale

The Base Revision is the persisted historical comparison/recovery reference
for unpublished work. Component-source references may preserve selected
historical states outside that Revision's graph. Only these explicit
historical references are needed to protect the current Working State
composition; operational state, association, and status fields do not
themselves identify historical objects.

## Alternatives considered

- Rooting only the Base Revision would omit source Component States selected
  from other Revisions.
- Rooting `AdapterWorkingStateRef`, the associated Line target, recovery
  status, or comparison status would treat operational data as additional
  historical references and is not approved.
- Synthesizing a Revision for a custom Working State would invent history.

## Specification impact

- Core Specification §§19, 56, 62, and 64.
- Glossary: Working State, Base Revision, and Reachability.
- Core Invariants: new INV-WORK-007; INV-GC-001 remains applicable to these
  explicit roots.
- ADR-0018 cross-reference only; its accepted lifecycle semantics are
  unchanged.
- ADR-0029 validation-coverage cross-reference.

## Test impact

The WORK-0013 extension must test Base-only roots, absent Base Revision,
direct source roots outside Base ancestry, absent sources, deduplication,
custom Working State, Line movement, excluded operational fields,
persisted-record changes, convergence with Line/Release roots, and
Resource-byte independence.

WORK-0014 must report Working State safety-root coverage while preserving
partial Core §62 coverage. ADR-0031 resolves declared-boundary matching for
missing Working State safety references: they are reported as unresolved
without a declared-boundary lookup.

## Implementation impact

Extend WORK-0013 and its reachability provider/result to include persisted
Working State safety references while retaining an explicitly partial
Core §62 result. WORK-0014 reports absent Working State root targets as
unresolved under ADR-0031. No production code is authorized by this ADR
alone.

## Compatibility / migration impact

No historical schema or immutable object identity changes. Existing
Working State records already contain the optional Base Revision and
component-source mapping; a pre-first-Revision record may validly contribute
zero roots.

## Notes

This decision does not make reachability complete for Contributions,
configured archival pins, or pending publication transactions. It does not
resolve temporary checkpoint policy or determine what happens when a
Working State safety-reference target is absent.
