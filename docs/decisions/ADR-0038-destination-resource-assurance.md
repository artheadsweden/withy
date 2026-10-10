# ADR-0038 — Destination Resource assurance

Status: ACCEPTED
Date: 2026-10-10
Decision owner: Human
Related Design Gap: N/A
Related decision: DEC-STORAGE-005

## Context

The Specs require verification before registering a new Resource Replica but
did not decide whether every upload/copy must be read back and rehashed as a
single complete Resource, or whether verified deterministic Chunk
reconstruction may establish equivalent Resource assurance.

## Decision

1. A candidate may be registered as a verified Resource Replica only after
   it has applicable `resource_identity` assurance as defined by ADR-0037.
   Incomplete, unverified, failed, or indeterminate candidates remain outside
   the registered Storage Map.
2. Direct full Resource verification is sufficient: obtain or reconstruct
   the destination representation's complete byte sequence, hash it using
   the ResourceId algorithm, and require an exact ResourceId match.
3. For a chunked destination, Resource-level assurance may instead be
   established by deterministic verified reconstruction without physically
   reading the same bytes again into a monolithic buffer solely to hash them,
   only when all of the following hold:
   - the Chunk Manifest/reconstruction description is deterministic and
     complete;
   - its ordered Chunk sequence is the one defined for the intended Resource
     representation under the applicable OMVCS chunking policy;
   - every required destination Chunk has independently verified
     `chunk_identity` evidence for the exact ChunkId;
   - byte lengths and ordering reconstruct exactly the complete intended
     byte sequence;
   - an existing `resource_identity` result or equivalent approved proof
     binds that exact complete ordered manifest and its reconstruction to
     the expected ResourceId, and the destination's verified ChunkIds and
     order establish that it has the same complete bytes; and
   - no required Chunk or evidence is missing, stale, or indeterminate.
4. Provider upload/copy success, existence, matching length, a previously
   stored locator, and source verification alone MUST NOT establish
   destination `resource_identity`.
5. Previously verified evidence may be reused only while applicable to the
   exact immutable bytes/Chunk identity involved. Moving or copying content
   to a new destination does not transfer source assurance automatically.
   Destination-applicable verification must be established using direct
   full Resource verification, the deterministic verified-reconstruction
   path above, or an equivalent checksum method permitted by ADR-0037.
6. Replica registration does not imply publication, durability guarantees,
   or continued availability or integrity. A registered Replica may later
   become unavailable or corrupt under the ordinary operational model.

## Rationale

The direct path gives a straightforward end-to-end check. The deterministic
Chunk path avoids a redundant monolithic read when every exact destination
Chunk is independently verified and the complete identity is established
from the complete, ordered representation. Both paths require
destination-applicable assurance and neither treats provider operation
success as verification.

## Alternatives considered

- Require complete Resource readback and rehash after every upload/copy:
  rejected because the approved deterministic verified-reconstruction path
  can establish equivalent Resource identity without that redundant
  monolithic pass.
- Allow source verification or provider copy-success to transfer assurance
  to a destination: rejected because it does not establish the bytes at that
  destination.
- Register an unverified candidate and upgrade it later: rejected because
  registration requires Resource-level assurance.

## Specification impact

- `Specs/OMVCS Core Specification.md` §§32, 35, 48, 51, and 55.
- `Specs/OMVCS Storage Adapter Specification.md` §§30–36, 54–58, 73–76,
  109–112, 183, 192, 204–205, and 240–243.
- `Specs/OMVCS Glossary.md`, Resource Replica, Replica Addition, Content
  Verification, and Verification Strength.
- `Specs/OMVCS Core Invariants Specification.md`, INV-RES-005 and
  INV-STOR-005, INV-INT-001–003.
- WORK-0016 promotion integration, WORK-0017, WORK-0020, and WORK-0021.

## Test impact

WORK-0017 MUST test direct full-hash promotion; deterministic verified
Chunk-reconstruction promotion; missing, stale, or indeterminate Chunk
evidence; wrong ordering/length/identity; provider copy-success without
destination verification; source verification not transferring assurance;
and a destination registration attempt that atomically fails or is rejected
without Resource-level assurance.

## Implementation impact

WORK-0017 produces a typed promotion-eligibility result consumable by the
WORK-0016 registration boundary. The small integration may be delivered as a
WORK-0016 follow-up or as WORK-0017's integration portion; it MUST NOT
duplicate or alter the already integrated WORK-0016 identity, map, or CAS
semantics.

## Compatibility / migration impact

Authoritative persisted Storage Maps continue to be reconstructed under
ADR-0035 without re-verifying bytes solely on load. This decision applies to
new candidate registration, not persisted-map reconstruction. Existing
Resource/Chunk identifiers and Storage Map generation/CAS semantics are
unchanged.

## Notes

DEC-STORAGE-004 defines the strength/method contract used here. No
publication, durability, deletion, GC, retention, or shared-namespace
semantics are added.
