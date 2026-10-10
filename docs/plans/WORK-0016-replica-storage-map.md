# WORK-0016 — Replica model and Storage Map

Status: PLANNED — Replica record semantics blocked on named semantic gates
Owner agent: Core Engineer
Milestone: M3
Branch: `work/0016-replica-storage-map`
Required review: Storage Engineer + Verifier

## Objective

Define Core-owned operational Replica records and the mutable Storage Map
that locate immutable Resources without changing their identities or
historical references.

## Normative requirements

- Core Specification §§29–36, 47–51, and 55.
- Storage Adapter Specification §§30–38, 49–51, 59–63, 73–75,
  109–110, 129–134, 183–184, and 204.
- Glossary: Resource Replica, Storage Location, Storage Map, Availability
  State, Corrupt Replica, Replica Addition, and Replica Removal.
- Core Invariants: INV-RES-005–007, INV-STOR-001–005, INV-INT-001–003,
  INV-GC-001–003.

## Dependencies

- WORK-0015.
- DG-0029 must be resolved before defining Replica identity/cardinality and
  representation binding.
- DEC-STORAGE-007 must define the provider locator representation.
- DEC-STORAGE-011 gates stable namespace identity and shared-object
  semantics only; this package must explicitly exclude those claims unless
  that decision is resolved.
- DEC-STORAGE-004/005 remain separate gates for verification evidence and
  upload assurance; this package does not decide them.

## Allowed scope

- Core operational Replica and Storage Map model in `crates/omvcs-model/`
  and `crates/omvcs-core/`.
- Provider-neutral Storage Adapter integration boundary only where needed
  to read/write Replica operational data.
- Focused tests for registration and mutable map updates.

## Deliverables

- Core-owned typed Replica and Storage Map models reflecting approved
  identity, representation, and locator decisions.
- Guarded/versioned map updates consistent with the specified mutable
  operational-pointer rules.
- Explicit separation of availability from integrity and verification.
- Tests proving moves and map changes do not alter immutable object IDs.

## Acceptance tests

- A Resource can map to multiple records according to the human-approved
  Replica identity/cardinality rule.
- Storage Map updates are guarded/versioned and stale updates are rejected
  without partial mutation.
- Endpoint, locator, preference, and availability changes do not alter
  ResourceId, ChunkId, RevisionId, or historical bytes.
- Unavailable is not corrupt; existence/availability is not verification.
- Registration/removal cannot assert verified state without the applicable
  approved verification evidence.
- No last-Replica deletion, GC, or retention behavior is inferred.

## Explicit non-goals

- Choosing Replica identity/cardinality before DG-0029 resolution.
- Provider-specific locator encoding before DEC-STORAGE-007 resolution.
- Namespace-sharing guarantees before DEC-STORAGE-011 resolution.
- Verification-strength taxonomy or upload assurance.
- Physical deletion, GC, orphan cleanup, retention, or automatic repair.
- Replication/migration orchestration, Repository Home conformance, M4
  transactions, Platform policy, or DAW behavior.

## Known Design Gaps

- DG-0029 blocks this package until resolved.
- DEC-STORAGE-007 and DEC-STORAGE-011 are unresolved decisions, not
  substitutes for DG-0029.
- DEC-CORE-005, DEC-CORE-008, and DEC-STORAGE-010/012 block deletion and
  cleanup behavior, which is excluded.

## Implementation plan

1. Resume Replica identity/cardinality and locator work only after DG-0029
   and DEC-STORAGE-007 are incorporated into the Specs and this plan.
   Exclude stable namespace identity/shared-object semantics unless
   DEC-STORAGE-011 is also resolved.
2. Implement Core-owned records and Storage Map guarded updates.
3. Add direct conformance and failure/concurrency tests.
4. Obtain Storage Engineer review and independent Verifier acceptance.

## Verification requirements

Verifier must trace record identity/cardinality to the resolved Spec text,
attempt stale and conflicting map updates, and prove that changing
operational location cannot change content or history identity.

## Completion criteria

Relevant workspace tests, rustfmt, warnings-denied Clippy, coverage update,
handover, and clean diff checks pass. The package must not begin while its
semantic gates remain open.
