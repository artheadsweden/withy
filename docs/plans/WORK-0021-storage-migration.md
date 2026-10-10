# WORK-0021 — Storage migration

Status: PLANNED — blocked on Replica and verification contracts
Owner agent: Core Engineer
Milestone: M3
Branch: `work/0021-storage-migration`
Required review: Storage Engineer + Verifier

## Objective

Implement Core-owned storage migration for a Resource by copying to a
destination Endpoint, verifying it, and registering the new Replica while
retaining the source. Source removal and M4-level interruption/recovery
orchestration are explicitly excluded.

## Normative requirements

- Core Specification §§30, 32–36, and 174–176.
- Storage Adapter Specification §§55–58, 73–75, 111–115, 127–129,
  187–188, 204, and 241.
- Glossary: Resource Replica, Storage Map, Storage Migration, Verification.
- Core Invariants: INV-RES-001–007, INV-STOR-003–005, INV-INT-001–004,
  INV-GC-001–003, INV-REC-001–006.

## Dependencies

- WORK-0015 through WORK-0017 and WORK-0020.
- DG-0029 resolved.
- DEC-STORAGE-004/005 and joint chunking decision resolved for migration's
  verification and reconstruction behavior.
- DEC-CORE-005/008 and DEC-STORAGE-010/012 do not block this retained-source
  subset because no deletion or cleanup occurs.

## Allowed scope

- Core-owned single-Resource migration orchestration in
  `crates/omvcs-core/`.
- Provider-neutral copy/verification coordination in
  `crates/omvcs-storage/`.
- Deterministic tests using mock/failure-injected Endpoints.

## Deliverables

- Bounded migration operation: validate source, copy, verify destination,
  register destination Replica, and report completion with source retained.
- Explicit phase/result reporting sufficient for a caller to observe
  operation outcome; no durable M4 operation state is claimed.
- Proof that migration changes mutable storage metadata only.

## Acceptance tests

- A failed source check, copy, destination verification, or registration
  leaves the source intact and does not falsely report completed migration.
- Destination is registered only after approved verification succeeds.
- Both Replicas remain present after successful migration in this package's
  retained-source scope.
- ResourceId, ChunkId, Component State, Project State, Revision, and
  historical bytes remain unchanged.
- Restart/retry, duplicate calls, and partial side-effect recovery are not
  assigned new semantics; operation reports the observed local outcome
  without claiming durable orchestration.

## Explicit non-goals

- Removing/retiring the old Replica or changing minimum replica policy.
- GC, retention, provider orphan cleanup, or delete semantics.
- Repository Home migration, publication transactions, M4 operation logs,
  durable retry/recovery orchestration, or Platform/DAW behavior.

## Known Design Gaps

- DG-0029 blocks Replica identity and registration semantics.
- DEC-CORE-005/008 and DEC-STORAGE-010/012 continue to block any future
  source-removal/cleanup extension.

## Implementation plan

1. Reuse WORK-0020 copy/verification/registration behavior.
2. Add Core-owned source validation and migration outcome reporting only to
   the extent stated in the Specs.
3. Preserve the source across every path; add failure injection.
4. Obtain Storage Engineer review and independent Verifier acceptance.

## Verification requirements

Verifier must inject failure at each migration phase, check source retention
and destination registration, and attempt to detect hidden delete, history
mutation, or M4 transaction semantics.

## Completion criteria

Relevant tests, rustfmt, warnings-denied Clippy, coverage update, handover,
and clean diff checks pass. No source removal is included without a later
approved package and all applicable deletion decisions.
