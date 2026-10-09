# DG-0022 — DeleteLine concurrency precondition

Status: RESOLVED
Classification: BLOCKS-FEATURE
Discovered by: Spec Guardian
Discovered during: DG-0016 resolution
Date: 2026-10-09

## Relevant specifications

- `Specs/OMVCS Core Specification.md` §§16–17, 58, 74, 82–83.
- `docs/decisions/ADR-0016-line-object-and-update-contract.md`.

## Problem

ADR-0016 defines DeleteLine as removal of the mutable Line record without
deleting historical objects, but does not select whether deletion must
compare an expected Line generation. The approved contract says that if
optimistic concurrency is required, the operation uses `line_id` and
expected generation with atomic conflict behavior; it does not state whether
that precondition is mandatory.

## Why the current specifications are insufficient

Applying MoveLine's compare-and-swap rule to deletion would broaden a
movement-specific contract. Omitting a concurrency precondition would
permit deletion after an unseen rename or move. Either choice changes
observable operation behavior.

## Affected work

- WORK-0010 DeleteLine inputs, stale-update behavior, retry contract, and
  deletion conformance tests.

## Can unaffected work continue?

Yes. The Line record, create/move/rename contracts, and the rule that
deletion does not itself delete historical objects are settled. DeleteLine
implementation must wait for this decision. Automatic pin/retention remains
separate under DEC-CORE-008.

## Candidate directions

The following are discussion material only and are NOT approved:

- require the current `generation` alongside `line_id`;
- delete atomically by `line_id` without a generation precondition.

## Required decision

Decide whether DeleteLine requires an expected generation. If required, the
operation must compare `line_id` and expected generation atomically and
report stale generation as a conflict without removing the Line.

## Resolution

Resolved by [ADR-0022](../decisions/ADR-0022-delete-line-concurrency.md)
and corresponding updates to Core §§17, 58, and 74; INV-HIST-010; ADR-0016;
WORK-0010; the decision register; and specification coverage.

DeleteLine requires `line_id` and `expected_generation`, and atomically
checks existence and generation with removal. A stale-generation conflict
leaves the Line unchanged and is distinguishable from a missing Line.
Success removes only the Line record and creates no next generation,
tombstone, or Line-mutation history. The caller must obtain and reason from
current state before retrying. DEC-CORE-008 alone governs automatic
retention/pinning.
