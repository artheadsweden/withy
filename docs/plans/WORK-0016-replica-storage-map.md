# WORK-0016 — Replica model and Storage Map

Status: READY — Replica model and guarded/CAS map updates are authorized;
verified promotion remains gated
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
- Approved decisions: ADR-0032, ADR-0033, and ADR-0034. Open gates:
  DEC-STORAGE-004/005 for verification promotion; DEC-STORAGE-011 is
  explicitly excluded.

## Dependencies

- WORK-0015.
- ADR-0032 resolves DG-0029 and defines Replica identity, cardinality, and
  representation binding.
- ADR-0033 resolves DEC-STORAGE-007 and defines the provider locator
  envelope.
- ADR-0034 resolves DG-0032 and defines the Project-wide generation and
  guarded mutation contract. Replica, representation, locator, Storage Map,
  generation, and CAS mechanics may proceed under the approved contracts.
- DEC-STORAGE-011 gates stable namespace identity and shared-object
  semantics only. This package explicitly excludes stable namespace,
  cross-Project shared-object/deduplication, and shared-namespace GC claims.
- DEC-STORAGE-004/005 remain open. WORK-0016 MUST NOT invent verification
  evidence, strength labels, or upload assurance. It MUST NOT register an
  incomplete or unverified candidate as a Resource Replica. Any verified
  promotion/registration integration requires the approved verification
  result contract from WORK-0017. WORK-0017 may consume independently
  reviewed WORK-0016 model/locator and generation/CAS subdeliverables;
  verified-promotion integration is not a prerequisite for implementing
  those subdeliverables.

## Allowed scope

- Core operational Replica and Storage Map model in `crates/omvcs-model/`
  and `crates/omvcs-core/`.
- Provider-neutral Storage Adapter integration boundary only where needed
  to read/write Replica operational data.
- Focused tests for registration and mutable map updates.

## Deliverables

- Core-owned typed Replica and Storage Map models reflecting approved
  identity, representation, and locator decisions.
- Project-scoped `StorageMapGeneration` and guarded/CAS updates consistent
  with Core §34 and ADR-0034.
- Explicit separation of availability from integrity and verification.
- Tests proving moves and map changes do not alter immutable object IDs.

## Acceptance tests

- A Resource can map to multiple records according to the human-approved
  Replica identity/cardinality rule, including distinct representations at
  the same Endpoint.
- Complete-object and chunked representations bind to their complete
  Resource bytes or ordered Chunk Manifest and Endpoint-scoped locator.
- Storage Map updates are guarded/versioned and stale updates are rejected
  atomically under ADR-0034.
- Generation parsing/serialization accepts only exact canonical JSON
  integers in `0 ..= 9007199254740991`, including rejection of negative
  zero, leading-zero, decimal, exponent, string, and out-of-range forms;
  initialization distinguishes the empty map at generation zero from
  absent/incomplete map metadata.
- A state-changing mutation affecting multiple entries validates against
  one pre-mutation map, commits atomically, and advances the Project map
  generation exactly once. A no-op does not advance it.
- Stale conflicts, validation failures, and generation exhaustion leave the
  complete map and generation unchanged; conflict reports the observed
  generation where available, and stale requests are not retried
  automatically.
- Core generation is distinct from provider conditional-write tokens;
  unsupported conditional atomicity cannot produce success.
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

- Verified promotion/registration without the approved WORK-0017 result.
- Stable namespace identity and shared-object guarantees before
  DEC-STORAGE-011 resolution.
- Verification-strength taxonomy or upload assurance.
- Physical deletion, GC, orphan cleanup, retention, or automatic repair.
- Replication/migration orchestration, Repository Home conformance, M4
  transactions, Platform policy, or DAW behavior.

## Known Design Gaps

- DG-0032 is resolved by ADR-0034; guarded map mutations are within scope.
- DEC-STORAGE-004/005 block verification-dependent promotion, not the
  operational record model.
- DEC-STORAGE-011 remains open but does not block the explicitly isolated
  Endpoint/Replica scope.
- DEC-CORE-005, DEC-CORE-008, and DEC-STORAGE-010/012 block deletion and
  cleanup behavior, which is excluded.

## Implementation plan

1. Implement the approved Core-owned Replica identity, representation,
   Storage Map structure, `StorageMapGeneration`, guarded mutation contract,
   and ProviderLocator envelope.
2. Keep verified-promotion/registration behind the approved WORK-0017
    result; incomplete/unverified candidates are not registered Replicas.
3. Submit independently reviewable model/locator and generation/CAS
    subdeliverables so WORK-0017 can proceed once its own semantic gates are
    resolved.
4. Exclude DEC-STORAGE-011 shared namespace claims, deletion, GC,
   retention, and physical cleanup.
5. Add direct conformance and failure/concurrency tests for the executable
   scope, then complete the gated tests after the remaining contracts are
   approved.
6. Obtain Storage Engineer review and independent Verifier acceptance for
   each executable subdeliverable.

## Verification requirements

Verifier must trace record identity/cardinality and the generation/CAS
contract to the resolved Spec text; attempt stale, invalid, no-op,
multi-entry, exhausted, unsupported, and provider-failure updates; and prove
that changing operational location cannot change content or history
identity.

## Completion criteria

The Replica/representation/locator model and guarded Storage Map CAS
mechanics may begin now. WORK-0016's model/CAS scope is complete when its
conformance tests pass review and independent verification. No verified
promotion is included before the approved WORK-0017 result. Relevant
workspace tests, rustfmt, warnings-denied Clippy, coverage update, handover,
and clean diff checks must pass.
