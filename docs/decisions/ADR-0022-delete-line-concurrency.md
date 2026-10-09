# ADR-0022 — DeleteLine concurrency precondition

Status: ACCEPTED
Date: 2026-10-09
Decision owner: Human
Related Design Gap: DG-0022

## Context

ADR-0016 established that DeleteLine removes the mutable Line record without
deleting its referenced historical objects, but left open whether deletion
must guard against an unseen mutation. Omitting a generation check would
permit deletion after an unseen rename or move, which is the race recorded
in DG-0022.

## Decision

1. `DeleteLine` requires `line_id` and `expected_generation`. The expected
   value uses the exact JSON-number domain for Line generation defined by
   ADR-0020.
2. The identified Line MUST exist, and its current generation MUST equal
   `expected_generation`. The existence check and generation comparison
   MUST be part of the same atomic deletion decision.
3. If the generation differs, deletion MUST fail as a concurrency conflict.
   The stale-generation failure MUST leave the Line intact and unchanged.
   A missing Line MUST be distinguishable from a stale-generation conflict.
4. Successful deletion removes exactly that Line record. It MUST NOT delete
   any Revision, Project State, Component State, or Resource merely because
   the Line was deleted.
5. Successful deletion does not need to create a new generation because the
   record ceases to exist. No tombstone, reflog, or Line-mutation history is
   introduced.
6. Retry behavior requires the caller to obtain and reason from current
   state. An implementation MUST NOT silently retry deletion against a
   newly observed generation on the caller's behalf.
7. Automatic retention or pinning after deletion remains entirely separate
   under DEC-CORE-008. This ADR does not choose or imply any retention,
   archival, or garbage-collection policy.

## Rationale

The required generation comparison makes deletion conditional on the exact
Line state the caller observed. Without it, a caller could delete a Line
after another actor had renamed or moved it without seeing that change.
Atomic comparison distinguishes a valid current-state deletion from that
stale request while preserving the Line on conflict.

## Alternatives considered

- Deleting by `line_id` alone was rejected because it permits deletion after
  an unseen rename or move.
- Automatically retrying against the newly observed generation was rejected
  because it would bypass the caller's concurrency decision.
- Adding a tombstone, operation-history record, or retention period here was
  rejected; these are not part of the approved DeleteLine contract, and
  automatic pin/retention remains under DEC-CORE-008.

## Specification impact

- `Specs/OMVCS Core Specification.md` §§17, 58, 74, and 82–83.
- `Specs/OMVCS Core Invariants Specification.md`, INV-HIST-010.
- `docs/decisions/ADR-0016-line-object-and-update-contract.md`.

## Test impact

WORK-0010 must verify that deletion with the current generation succeeds;
stale generation conflicts and leaves the Line unchanged; a missing Line is
distinguishable from a stale-generation conflict; success removes only the
Line record and leaves historical objects unaffected; and neither automatic
pin/retention nor other DEC-CORE-008 behavior is inferred.

## Implementation impact

- WORK-0010 implements DeleteLine as an atomic compare-and-delete using
  `line_id` and `expected_generation`.
- Reachability, retention, pinning, and garbage collection remain outside
  this operation contract.

## Compatibility / migration impact

No production implementation or persisted Line record is changed by this
documentation decision. DeleteLine callers must provide the generation they
observed; blind deletion by identifier alone is non-conforming.

## Notes

No Line tombstone or mutation history is introduced.
