# WORK-0017 — Resource and Chunk verification model

Status: READY — semantic gates resolved and implementation authorized by the
human-approved decisions recorded in ADR-0036/0037/0038. Not started.
Owner agent: Core Engineer
Milestone: M3
Branch: `work/0017-resource-chunk-verification`
Required review: Storage Engineer + Verifier

## Objective

Implement the approved verification model for available Resource and Chunk
Replicas, including reconstruction verification evidence and the distinction
between unavailable, corrupt, unverified, and verified copies.

## Normative requirements

- Core Specification §§8–8.2, 32, 47–55.
- Storage Adapter Specification §§30–36, 73–76, 109–112, 183, 192,
  204–205, and 240–243.
- Glossary: Resource Object, Chunk, Chunk Manifest, Resource Replica,
  Availability State, Corrupt Replica, Content Verification, Verification
  Strength, Verification Method, Verification Evidence, Verification Result,
  and Chunking Policy.
- Core Invariants: INV-RES-001–007, INV-STOR-003–005, INV-INT-001–003.
- ADR-0036/0037/0038 and resolved DEC-CORE-001, DEC-STORAGE-002,
  DEC-STORAGE-004, and DEC-STORAGE-005.

## Dependencies

- WORK-0015 and the independently reviewed WORK-0016 Replica
  model/locator and generation/CAS subdeliverables; verified-promotion
  integration code remains a bounded WORK-0017 integration deliverable or a
  small WORK-0016 follow-up; it is not a prerequisite for verification types
  and algorithms.
- Use the approved Replica identity, representation, locator, generation,
  and CAS contracts from ADR-0032/0033/0034/0035.
- Chunking, verification-strength/evidence, and destination assurance
  decisions are resolved by ADR-0036/0037/0038.
- DG-0034 remains open only for syntax-specific ProviderLocator
  schema-identifier validation and does not block this work package.
- DEC-STORAGE-011, deletion, GC, retention, and shared-namespace semantics
  remain outside scope.

## Allowed scope

- Verification domain types and Core orchestration in
  `crates/omvcs-model/` and `crates/omvcs-core/`.
- Generic Adapter verification result boundary in `crates/omvcs-storage/`.
- Focused conformance and failure-injection tests.

## Deliverables

- Typed verification results using the approved strength/evidence model.
- Resource verification over complete reconstructed bytes and independent
  Chunk identity verification.
- Deterministic OMVCS 0.1 fixed-size sequential chunking and reconstruction.
- Replica state transitions that do not conflate availability and integrity.
- Evidence model that records the approved method without entering
  historical hash preimages.
- Promotion-eligibility result requiring destination-applicable
  `resource_identity` assurance, consumable by the WORK-0016 registration
  boundary.

## Acceptance tests

- Identical Resource bytes yield identical Chunk IDs and ordered manifests
  under the exact 8,388,608-byte target policy.
- Deterministic boundaries are tested for below-target, exact-target,
  target-plus-one-byte, multi-Chunk, and zero-byte Resources.
- Complete-object and chunked representations preserve ResourceId; offsets
  are contiguous, ordered, gap-free, and non-overlapping.
- A zero-byte chunked representation uses one zero-length Chunk; a
  complete-object representation remains independently valid.
- Correct Chunks reconstruct and verify the complete Resource identity;
  wrong Chunk bytes fail `chunk_identity`.
- Reordered Chunks fail Resource verification; omitted or unavailable
  Chunks produce the specified failed or indeterminate result, not
  corruption when bytes were unavailable.
- One corrupt Replica does not invalidate another valid Replica or Resource
  identity.
- Direct full Resource verification can establish `resource_identity`.
- Deterministic destination Chunk verification establishes
  `resource_identity` only with the complete approved manifest/identity
  binding, exact ordered lengths, every required destination Chunk verified,
  and no missing, stale, or indeterminate evidence; a monolithic buffer
  reread is not required solely to hash the same bytes again.
- Provider checksum evidence is accepted only when equivalent to OMVCS
  SHA-256 over the exact Chunk or complete Resource bytes for the strength
  claimed. Non-equivalent checksums, generic provider success, ETags, CRC,
  object version, existence, and matching length are rejected as verification.
- Provider upload/copy success alone does not verify content.
- Destination copy does not inherit source assurance automatically;
  destination-applicable assurance is required before registration.
- Promotion eligibility is false for unverified, incomplete, failed, or
  indeterminate candidates and true only for accepted
  `resource_identity` assurance.
- Unavailable bytes are not corrupt; checked wrong bytes are corrupt for
  that Replica only.
- Verification evidence/method/strength/outcome do not change Resource,
  Component State, Project State, Revision identity, or creative provenance.

## Explicit non-goals

- Selecting a chunking algorithm or verification-strength taxonomy.
- Automatic Resource download, repair, GC, or Replica removal.
- Replication or migration orchestration.
- Publication durability policy, M4 transactions, encryption, grants,
  Platform authority, or DAW behavior.

## Known Design Gaps

- DG-0032 is resolved by ADR-0034. Map-mutating integration tests must
  follow the approved Project-scoped CAS contract.
- DG-0034 remains open but its syntax-specific locator grammar is outside
  this package and does not block it.
- DEC-CORE-001, DEC-STORAGE-002, DEC-STORAGE-004, and DEC-STORAGE-005 are
  resolved by ADR-0036/0037/0038; no remaining semantic blocker applies to
  the bounded WORK-0017 scope.

## Implementation plan

1. Define typed outcomes, strengths, methods, and operational evidence.
2. Implement deterministic chunking and direct or approved reconstructed
   verification.
3. Add direct identity, corruption, unavailability, provider-equivalence,
   promotion, and failure-injection tests.
4. Integrate promotion eligibility with the WORK-0016 registration boundary
   without changing Replica identity, Storage Map, or CAS semantics.
5. Obtain Storage Engineer review and independent Verifier acceptance.

## Verification requirements

Verifier must test every acceptance case above, including wrong bytes,
reordered/omitted chunks, the exact 8 MiB boundaries, zero bytes,
false/non-equivalent provider checksums, unavailable endpoints, stale or
insufficient evidence, source-to-destination assurance, and atomic candidate
registration eligibility.

## Completion criteria

Relevant tests, rustfmt, warnings-denied Clippy, coverage update, handover,
and clean diff checks pass. No verification taxonomy or acceptance rule is
left for implementation discretion.
