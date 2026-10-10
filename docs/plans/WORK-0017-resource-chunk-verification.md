# WORK-0017 — Resource and Chunk verification model

Status: PLANNED — blocked on named semantic gates
Owner agent: Core Engineer
Milestone: M3
Branch: `work/0017-resource-chunk-verification`
Required review: Storage Engineer + Verifier

## Objective

Implement the approved verification model for available Resource and Chunk
Replicas, including reconstruction verification evidence and the distinction
between unavailable, corrupt, unverified, and verified copies.

## Normative requirements

- Core Specification §§8–8.2, 47–55.
- Storage Adapter Specification §§30–36, 73–76, 109–112, 183, 192,
  204, and 240–243.
- Glossary: Resource Object, Chunk, Chunk Manifest, Resource Replica,
  Availability State, Corrupt Replica, and Verification.
- Core Invariants: INV-RES-001–007, INV-STOR-003–005, INV-INT-001–003.

## Dependencies

- WORK-0015 and WORK-0016.
- DG-0029 resolved.
- DEC-CORE-001 and DEC-STORAGE-002 resolved together as one chunking-policy
  decision before chunk-policy-specific reconstruction tests or claims.
- DEC-STORAGE-004 verification-strength taxonomy resolved.
- DEC-STORAGE-005 post-upload full-hash versus verified reconstruction
  assurance resolved.

## Allowed scope

- Verification domain types and Core orchestration in
  `crates/omvcs-model/` and `crates/omvcs-core/`.
- Generic Adapter verification result boundary in `crates/omvcs-storage/`.
- Focused conformance and failure-injection tests.

## Deliverables

- Typed verification results using the approved strength/evidence model.
- Resource verification over complete reconstructed bytes and independent
  Chunk identity verification.
- Replica state transitions that do not conflate availability and integrity.
- Evidence model that records the approved method without entering
  historical hash preimages.

## Acceptance tests

- Reconstructed Resource bytes verify against complete Resource identity;
  each Chunk verifies against its own identity.
- Ordered manifest reconstruction preserves specified order and byte
  offsets/lengths.
- Provider checksum acceptance is limited to the approved equivalence
  criteria.
- Missing/unavailable bytes produce unavailable or indeterminate results,
  not corruption; checked bytes that fail identity become corrupt.
- One corrupt Replica does not invalidate another valid Replica or the
  Resource identity.
- Provider copy success alone does not imply verified content.
- Previously verified evidence is treated only as permitted by resolved
  DEC-STORAGE-004/005.
- Verification metadata does not change Resource, Component State, Project
  State, or Revision identity.

## Explicit non-goals

- Selecting a chunking algorithm or verification-strength taxonomy.
- Automatic Resource download, repair, GC, or Replica removal.
- Publication durability policy, M4 transactions, encryption, grants,
  Platform authority, or DAW behavior.

## Known Design Gaps

- DG-0029 blocks association of evidence with Replica identity.
- DEC-CORE-001 + DEC-STORAGE-002 and DEC-STORAGE-004/005 block the named
  semantics.

## Implementation plan

1. Wait for all listed semantic gates.
2. Define typed results and deterministic reconstruction verification.
3. Add direct identity, corruption, unavailability, and failure-injection
   tests.
4. Obtain Storage Engineer review and independent Verifier acceptance.

## Verification requirements

Verifier must test both positive verification and adversarial cases:
wrong bytes, reordered chunks, omitted chunks, false provider checksums,
unavailable endpoints, and stale/insufficient evidence.

## Completion criteria

Relevant tests, rustfmt, warnings-denied Clippy, coverage update, handover,
and clean diff checks pass. No taxonomy or acceptance rule may be chosen
while its decision remains open.
