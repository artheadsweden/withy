# WORK-0020 — Resource replication

Status: BLOCKED — DG-0038, Core §69 resumability boundary
Owner agent: Core Engineer
Milestone: M3
Branch: `work/0020-resource-replication`
Required review: Storage Engineer + Verifier

## Objective

Copy Resource/Chunk content between Endpoints and add a destination Replica
only after the approved verification and registration requirements pass;
never remove or invalidate the source as part of replication.

## Normative requirements

- Core Specification §§32–36, 47–55, and 69.
- Storage Adapter Specification §§54–55, 59–63, 73–76, 109–112,
  183, 204–205, 240–243, and 252–253.
- Core Specification §83 and ADR-0043, Per-Resource replication outcomes.
- Glossary: Resource Replica, Resource Replication Outcome, Storage Map,
  Verification, Availability State.
- Core Invariants: INV-RES-001–007, INV-STOR-003–005, INV-INT-001–004,
  INV-GC-001–003.

## Dependencies

- WORK-0015 through WORK-0017 are VERIFIED / INTEGRATED.
- ADR-0032/0033 Replica identity and locator contracts reflected in
  approved interfaces; Storage Map registration uses the Project-wide
  guarded mutation contract in ADR-0034.
- ADR-0036/0037/0038 define chunking, verification, and destination
  assurance. Replication MUST NOT automatically rechunk an existing OMVCS
  0.1 representation because an Endpoint prefers another physical layout.
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

- Success reports `success`, the destination `ReplicaId`, and
  `registration=committed` only after destination `resource_identity`
  verification and known-committed Storage Map CAS registration.
- Destination registration occurs only after the approved verification
  result succeeds.
- Failed, interrupted, incomplete, unavailable, and corrupt destinations
  are not registered as valid Replicas.
- Source remains registered, retrievable, and otherwise untouched on
  success and on all copy/verify/register failures; replication never
  invalidates or demotes it.
- Provider-native copy success alone is not treated as verification.
- Chunked replication preserves the approved OMVCS 0.1 Chunk boundaries;
  provider-internal segmentation is not exposed as different OMVCS ChunkIds.
- Destination `resource_identity` assurance follows direct full Resource
  verification or ADR-0038's complete deterministic verified-reconstruction
  path. Source assurance does not transfer automatically.
- A failure reports the observed operation stage explicitly without claiming
  that unfinished work succeeded.
- Every non-success reports the furthest relevant stage, one of
  `source_retrieval`, `destination_transfer`,
  `destination_verification`, or `storage_map_registration`; destination
  effects are exactly `none`, `may_exist`, `complete_unverified`, or
  `verified_unregistered`; registration is exactly `not_attempted`,
  `not_committed`, `unknown`, or `committed`.
- `unknown` registration status is used only when the applicable atomic CAS
  outcome cannot establish whether it committed. Known CAS non-commit after
  verified transfer reports `incomplete`/`verified_unregistered` with
  `registration=not_committed`, not `failed`.
- Failures with no persistent destination effect may report `failed` with
  destination effect `none`; any known or possible destination effect
  requires `incomplete`. Indeterminate verification is not corruption and
  prevents registration.
- Core does not retry the composed operation after it returns non-success.
  A caller reinvocation re-observes the authoritative map/evidence; no
  global replay idempotency or physical deduplication is promised.
- Adapter retry failure injection proves non-idempotent provider actions
  are not blindly repeated when duplicate effects are possible.
- Replication changes only operational Storage Map data; ResourceId,
  ChunkId, Component State, Project State, Revision, and history remain
  unchanged.

## Explicit non-goals

- Migration cutover, source removal, physical deletion, GC, retention, or
  orphan cleanup.
- Durable operation-log format and cross-step retry/recovery orchestration
  beyond the applicable approved Core §69 contract. DG-0038 must resolve
  whether that existing resumability requirement applies to this package;
  no §69 requirement is waived here.
- Encryption/key-management semantics, Platform grants, FFI, or DAW behavior.

## Known Design Gaps

- DG-0037 is resolved by ADR-0043. DG-0038 is OPEN and blocks implementation
  until the Core §69 long-running/resumability boundary for single-Resource
  replication is decided. See
  `docs/gaps/DG-0038-resource-replication-resumability-boundary.md`.
- Destination map registration uses ADR-0034's atomic CAS contract.
- DEC-CORE-005/008 and DEC-STORAGE-010/012 affect deletion/cleanup only,
  all of which are excluded.

## Implementation plan

1. Resolve DG-0038 without weakening Core §69 or inventing a duration bound.
2. Implement copy/stream orchestration over approved contracts.
3. Verify then register the destination; retain source on every failure path.
4. Add direct, per-operation, and failure-injection tests for ADR-0043.
5. Obtain Storage Engineer review and independent Verifier acceptance.

## Verification requirements

Verifier must inject destination failures at every operation boundary and
prove there is no false registration, source loss, history mutation, or
success-shaped partial result.

## Completion criteria

Relevant workspace tests, rustfmt, warnings-denied Clippy, coverage update,
handover, and clean diff checks pass.
