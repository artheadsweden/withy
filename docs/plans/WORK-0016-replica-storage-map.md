# WORK-0016 — Replica model and Storage Map

Status: VERIFIED — bounded Replica/representation/locator model, guarded/CAS map updates, and
authoritative persisted-map reconstruction accepted by Storage Engineer and
independent Verifier. ProviderLocator schema-identifier lexical validation
remains excluded under DG-0034. Promotion semantics are defined by
ADR-0037/0038; the verification-result integration was not part of the
accepted WORK-0016 baseline. The promotion integration is now implemented
within WORK-0017 on `work/0017-resource-chunk-verification`; it remains
pending WORK-0017's required independent reviews and is not integrated.
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
- Approved decisions: ADR-0032, ADR-0033, ADR-0034, and ADR-0035. ADR-0037
  and ADR-0038 now define the verification gate for a follow-up promotion
  integration. DEC-STORAGE-011 remains open and is explicitly excluded.

## Dependencies

- WORK-0015.
- ADR-0032 resolves DG-0029 and defines Replica identity, cardinality, and
  representation binding.
- ADR-0033 resolves DEC-STORAGE-007 and defines the provider locator
  envelope.
- ADR-0034 resolves DG-0032 and defines the Project-wide generation and
  guarded mutation contract. Replica, representation, locator, Storage Map,
  generation, and CAS mechanics may proceed under the approved contracts.
- ADR-0035 resolves DG-0033: reconstruction from authoritative Repository
  Home persistence continues prior successful registration after structural
  validation; deserialization alone does not register a record, and map
  reconstruction does not re-verify Resource bytes solely because it is
  loaded. Repository Home implementations must uphold this authority
  contract.
- DG-0034 remains OPEN and blocks only syntax-specific validation of
  ProviderLocator schema identifiers. WORK-0016 MUST NOT invent a lexical
  version convention; it may implement the opaque envelope, canonical value,
  and syntax checks directly established by the Specs.
- DEC-STORAGE-011 gates stable namespace identity and shared-object
  semantics only. This package explicitly excludes stable namespace,
  cross-Project shared-object/deduplication, and shared-namespace GC claims.
- During the implementation window, DEC-STORAGE-004/005 were open.
  WORK-0016 therefore did not invent verification evidence, strength labels,
  or upload assurance. These decisions were subsequently resolved by
  ADR-0037/0038.
- The integrated WORK-0016 code does not include verified-promotion
  integration. Under ADR-0037/0038, promotion requires destination-applicable
  `resource_identity` assurance. Its registration integration may be
  implemented as a small WORK-0016 follow-up or as the integration portion
  of WORK-0017, without reopening Replica identity, Storage Map, or CAS.

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
- A chunked Replica is fully available only when exactly one current
  availability result per ordered manifest entry is present and every result
  is `Available`; unavailable, unknown, missing, or extra results do not
  establish full availability.
- Canonical Replica persistence bytes use RFC 8785/JCS, including nested
  ProviderLocator values. Provider-specific secret rejection and
  restart-durability conformance remain responsibilities of concrete
  Storage Adapter implementations.
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
- Decoding/serializing a Replica record alone does not register it; loading
  an authoritative persisted map structurally reconstructs prior registered
  state without re-verifying Resource bytes. Non-authoritative candidates
  remain subject to the new-registration gate.
- Public Core callers cannot directly build a registered Storage Map from
  arbitrary deserialized Replica records; Core reconstructs the map only
  from records returned by the authoritative persistence interface.
- Structural reconstruction rejects duplicate ReplicaIds and inconsistent
  map/generation state. It restores map and generation together and does not
  expose a partial state.
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
- Implementing verification result/evidence types or promotion eligibility;
  these are assigned to WORK-0017 under ADR-0037/0038.
- Physical deletion, GC, orphan cleanup, retention, or automatic repair.
- Replication/migration orchestration, Repository Home conformance, M4
  transactions, Platform policy, or DAW behavior.

## Known Design Gaps

- DG-0032 is resolved by ADR-0034; guarded map mutations are within scope.
- DG-0033 is resolved by ADR-0035; authoritative persisted-map
  reconstruction is distinct from candidate registration.
- DG-0034 blocks additional lexical ProviderLocator schema-identifier
  validation until its grammar is approved; no version syntax is inferred
  from examples.
- ADR-0037/0038 resolve DEC-STORAGE-004/005. The previously excluded
  promotion integration is now semantically unblocked once WORK-0017's
  verification-result API is implemented.
- DEC-STORAGE-011 remains open but does not block the explicitly isolated
  Endpoint/Replica scope.
- DEC-CORE-005, DEC-CORE-008, and DEC-STORAGE-010/012 block deletion and
  cleanup behavior, which is excluded.

## Implementation plan

1. Implement the approved Core-owned Replica identity, representation,
   Storage Map structure, `StorageMapGeneration`, guarded mutation contract,
   and ProviderLocator envelope.
2. Keep verified-promotion/registration in the WORK-0017 integration
   deliverable or a bounded WORK-0016 follow-up; incomplete, failed, or
   indeterminate candidates are not registered Replicas.
3. Treat decoded Replica records as data only. Reconstruct records for the
   Storage Map only from the authoritative persistence boundary, validating
   structure and map-generation consistency without re-verifying bytes.
4. Submit independently reviewable model/locator and generation/CAS
   subdeliverables so WORK-0017 can proceed once its own semantic gates are
   resolved.
5. Exclude DEC-STORAGE-011 shared namespace claims, deletion, GC,
   retention, and physical cleanup.
6. Add direct conformance and failure/concurrency tests for the executable
   scope, then complete the gated tests after the remaining contracts are
   approved.
7. Obtain Storage Engineer review and independent Verifier acceptance for
   each executable subdeliverable.

## Verification requirements

Verifier must trace record identity/cardinality and the generation/CAS
contract to the resolved Spec text; attempt stale, invalid, no-op,
multi-entry, exhausted, unsupported, and provider-failure updates; and prove
that changing operational location cannot change content or history
identity. Verify authoritative persisted-map reconstruction separately from
new registration; verify that canonical Replica persistence bytes preserve
the nested locator; and verify the full-availability condition across every
manifest entry. DG-0034 is an explicit exclusion: no syntax-specific
ProviderLocator schema-identifier grammar is inferred or claimed.

## Completion criteria

The Replica/representation/locator model and guarded Storage Map CAS
mechanics are complete at the bounded scope accepted by Storage Engineer
and independent Verifier. Lexical ProviderLocator schema-identifier
validation remains outside this acceptance under DG-0034. The approved
promotion contract is now defined by ADR-0037/0038; implementation requires
the WORK-0017 result API and must preserve the integrated WORK-0016 identity
and CAS behavior. Relevant
workspace tests, rustfmt, warnings-denied Clippy, coverage update, handover,
and clean diff checks must pass.
