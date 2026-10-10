# WORK-0020 — Resource replication

Status: PLANNED — blocked on Replica and verification contracts
Owner agent: Core Engineer
Milestone: M3
Branch: `work/0020-resource-replication`
Required review: Storage Engineer + Verifier

## Objective

Copy Resource/Chunk content between Endpoints and add a destination Replica
only after the approved verification and registration requirements pass;
never remove or invalidate the source as part of replication.

## Normative requirements

- Core Specification §§32–36 and 47–55.
- Storage Adapter Specification §§54–55, 59–63, 73–76, 109–112,
  183, 204–205, and 240–243.
- Glossary: Resource Replica, Storage Map, Verification, Availability State.
- Core Invariants: INV-RES-001–007, INV-STOR-003–005, INV-INT-001–004,
  INV-GC-001–003.

## Dependencies

- WORK-0015 through WORK-0017.
- DG-0029 resolved.
- DEC-STORAGE-004/005 resolved for verification behavior. The joint
  DEC-CORE-001/DEC-STORAGE-002 decision is required only if this package
  selects policy-dependent chunk transfer/rechunking; complete-Resource
  byte streaming does not select a Chunking policy.
- No filesystem dependency: conformance runs against approved mock/fake
  Endpoints and other adapters as available.

## Allowed scope

- Provider-neutral replication orchestration in `crates/omvcs-core/`.
- Adapter transfer integration in `crates/omvcs-storage/`.
- Replication tests using mock/failure-injected Endpoints.

## Deliverables

- Resource replication flow that streams/copies bytes and verifies the
  destination before Replica registration.
- Explicit per-resource outcome for success, partial completion, and
  provider/verification failure.
- Source-preserving behavior on all destination failures.

## Acceptance tests

- Destination registration occurs only after the approved verification
  result succeeds.
- Failed, interrupted, incomplete, unavailable, and corrupt destinations
  are not registered as valid Replicas.
- Source remains registered and retrievable on all copy/verify/register
  failures.
- Provider-native copy success alone is not treated as verification.
- A failure reports the observed operation stage explicitly without claiming
  that unfinished work succeeded.
- Replication changes only operational Storage Map data; ResourceId,
  ChunkId, Component State, Project State, Revision, and history remain
  unchanged.

## Explicit non-goals

- Migration cutover, source removal, physical deletion, GC, retention, or
  orphan cleanup.
- M4 transaction/recovery state machine, durable operation-log format, or
  cross-step retry orchestration.
- Encryption/key-management semantics, Platform grants, FFI, or DAW behavior.

## Known Design Gaps

- DG-0029 blocks Replica registration semantics.
- DEC-CORE-005/008 and DEC-STORAGE-010/012 affect deletion/cleanup only,
  all of which are excluded.

## Implementation plan

1. Implement copy/stream orchestration over approved contracts.
2. Verify then register the destination; retain source on every failure path.
3. Add direct, per-operation, and failure-injection tests.
4. Obtain Storage Engineer review and independent Verifier acceptance.

## Verification requirements

Verifier must inject destination failures at every operation boundary and
prove there is no false registration, source loss, history mutation, or
success-shaped partial result.

## Completion criteria

Relevant workspace tests, rustfmt, warnings-denied Clippy, coverage update,
handover, and clean diff checks pass.
