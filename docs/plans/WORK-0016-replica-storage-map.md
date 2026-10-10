# WORK-0016 — Replica model and Storage Map

Status: PLANNED — Replica/locator model may proceed; guarded map updates
and verified promotion remain gated
Owner agent: Core Engineer
Milestone: M3
Branch: `work/0016-replica-storage-map`
Required review: Storage Engineer + Verifier

## Objective

Define Core-owned operational Replica records and the mutable Storage Map
that locate immutable Resources without changing their identities or
historical references.

## Normative requirements

- Core Specification §§8–8.2, 29–36, 47–51, 55, and 58.
- Storage Adapter Specification §§29–38, 44–45, 49–51, 59–63, 73–75,
  109–110, 129–134, 160–164, 183–184, 204, and 252.
- Glossary: Resource Replica, Storage Location, Storage Map, Availability
  State, Corrupt Replica, Replica Addition, and Replica Removal.
- Core Invariants: INV-RES-005–007, INV-STOR-001–005, INV-INT-001–003,
  INV-GC-001–003.
- Approved decisions: ADR-0032 and ADR-0033. Open gates: DG-0032 and
  DEC-STORAGE-004/005; DEC-STORAGE-011 is explicitly excluded.

## Dependencies

- WORK-0015.
- ADR-0032 resolves DG-0029 and defines Replica identity, cardinality, and
  representation binding.
- ADR-0033 resolves DEC-STORAGE-007 and defines the provider locator
  envelope.
- DG-0032 blocks implementation of guarded/versioned Storage Map mutations
  and completion of this package. Replica, representation, and locator
  models may proceed before that decision.
- DEC-STORAGE-011 gates stable namespace identity and shared-object
  semantics only. This package explicitly excludes stable namespace,
  cross-Project shared-object/deduplication, and shared-namespace GC claims.
- DEC-STORAGE-004/005 remain open. WORK-0016 may define records and the
  registration boundary, but MUST NOT invent verification evidence,
  strength labels, or upload assurance. Verified promotion/registration
  requires the approved verification result contract from WORK-0017.
  WORK-0017 may consume the independently reviewed WORK-0016
  model/locator subdeliverable before WORK-0016 is fully complete; its
  result then gates WORK-0016's promotion integration.

## Allowed scope

- Core operational Replica and Storage Map model in `crates/omvcs-model/`
  and `crates/omvcs-core/`.
- Provider-neutral Storage Adapter integration boundary only where needed
  to read/write Replica operational data.
- Focused tests for registration and mutable map updates.

## Deliverables

- Core-owned typed Replica and Storage Map models reflecting approved
  identity, representation, and locator decisions.
- Guarded/versioned map updates consistent with Core §34 once DG-0032
  defines the generation/CAS contract.
- Explicit separation of availability from integrity and verification.
- Tests proving moves and map changes do not alter immutable object IDs.

## Acceptance tests

- A Resource can map to multiple records according to the human-approved
  Replica identity/cardinality rule, including distinct representations at
  the same Endpoint.
- Complete-object and chunked representations bind to their complete
  Resource bytes or ordered Chunk Manifest and Endpoint-scoped locator.
- Storage Map updates are guarded/versioned and stale updates are rejected
  atomically, under the generation/CAS contract resolved from DG-0032.
- Endpoint, locator, preference, and availability changes do not alter
  ResourceId, ChunkId, RevisionId, or historical bytes.
- Unavailable is not corrupt; existence/availability is not verification.
- Incomplete or unverified candidates remain outside the registered
  Storage Map.
- Verified promotion/registration cannot occur without the applicable
  approved WORK-0017 verification result; no strength taxonomy or upload
  assurance is added here.
- Chunk copies are not separate Resource Replicas; no shared-object
  ownership or namespace guarantee is inferred.
- No last-Replica deletion, GC, or retention behavior is inferred.

## Explicit non-goals

- Resolving Storage Map generation domain/CAS behavior before DG-0032.
- Stable namespace identity and shared-object guarantees before
  DEC-STORAGE-011 resolution.
- Verification-strength taxonomy or upload assurance.
- Physical deletion, GC, orphan cleanup, retention, or automatic repair.
- Replication/migration orchestration, Repository Home conformance, M4
  transactions, Platform policy, or DAW behavior.

## Known Design Gaps

- DG-0032 blocks guarded/versioned Storage Map mutations and completion;
  Replica and locator model work may proceed.
- DEC-STORAGE-004/005 block verification-dependent promotion, not the
  operational record model.
- DEC-STORAGE-011 remains open but does not block the explicitly isolated
  Endpoint/Replica scope.
- DEC-CORE-005, DEC-CORE-008, and DEC-STORAGE-010/012 block deletion and
  cleanup behavior, which is excluded.

## Implementation plan

1. Implement the approved Core-owned Replica identity, representation,
   Storage Map record shape, and ProviderLocator envelope.
2. Do not implement guarded map mutations until DG-0032 is resolved.
3. Submit the model/locator subdeliverable for review so WORK-0017 can
    proceed once its own semantic gates are resolved; keep
    verified-promotion integration behind its approved result.
4. Exclude DEC-STORAGE-011 shared namespace claims, deletion, GC,
   retention, and physical cleanup.
5. Add direct conformance and failure/concurrency tests for the executable
   scope, then complete the gated tests after the remaining contracts are
   approved.
6. Obtain Storage Engineer review and independent Verifier acceptance for
    each executable subdeliverable.

## Verification requirements

Verifier must trace record identity/cardinality to the resolved Spec text,
attempt stale and conflicting map updates, and prove that changing
operational location cannot change content or history identity.

## Completion criteria

The independently executable model/locator scope may begin now. WORK-0016
is not complete until DG-0032 is resolved, guarded updates pass their
conformance tests, and verified-promotion integration uses the approved
WORK-0017 result. Relevant workspace tests, rustfmt, warnings-denied
Clippy, coverage update, handover, and clean diff checks must pass.
