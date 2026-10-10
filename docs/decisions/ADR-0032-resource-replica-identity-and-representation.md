# ADR-0032 — Resource Replica identity and representation

Status: ACCEPTED
Date: 2026-10-10
Decision owner: Human
Related Design Gap: DG-0029

## Context

The Specs defined Resource Replicas as verified physical copies and required
chunked representations to reconstruct the Resource, but did not define
Replica record identity, multiplicity at one Endpoint, or the binding
between a Replica and its complete physical representation.

## Decision

1. A Resource Replica is an operational record for one complete recoverable
   physical representation of one Resource at one Storage Endpoint. A
   registered, valid Replica has passed the applicable Content Verification
   requirements.
2. Each Replica has an assigned, stable `ReplicaId`, represented as a
   UUIDv7 in canonical lowercase textual form. It is independent of
   ResourceId, EndpointId, provider locator, representation layout,
   verification evidence, and availability state.
3. A Resource may have zero, one, or multiple Replicas. Multiple distinct,
   independently addressable representations of the same Resource may be
   recorded at one Endpoint.
4. A Replica binds exactly one complete representation. OMVCS 0.1 MUST
   support at least these representation kinds:
   - `complete-object`: its locator identifies a provider object whose
     bytes are the complete Resource;
   - `chunked`: its record binds an ordered Chunk Manifest under Core §8.2,
     and its locator lets the Storage Adapter retrieve every required Chunk
     representation at that Endpoint.
5. A chunked Replica is fully available only while every required Chunk
   representation for that Replica is available. Its manifest and
   reconstruction description remain operational metadata and do not alter
   Resource or historical identity. Chunk identity and reconstruction order
   are unchanged.
6. A new independently addressable physical representation receives a new
   ReplicaId. Updating operational facts about the same representation,
   including a provider-side locator change within the same Endpoint/provider
   context, retains its ReplicaId. Moving a representation that creates a
   distinct independently addressable copy creates a new Replica.
7. The Storage Map maps each ResourceId to a set of ReplicaIds and each
   ReplicaId to one Replica record. Endpoint identity and provider locator
   are not part of the ResourceId key. Individual Chunk copies are not
   Resource Replicas.
8. An incomplete or unverified candidate is not a registered Resource
   Replica. Registration MUST NOT mark a representation verified until the
   applicable Content Verification requirements succeed. This decision
   does not define verification-strength labels, evidence, or upload
   assurance. Removal changes Storage Map metadata only and does not imply
   physical deletion, GC eligibility, retention expiry, or permission to
   remove the last valid copy.
9. This decision makes no claim about stable shared-namespace identity,
   cross-Project physical-object identity or deduplication guarantees, or
   shared-namespace GC. DEC-STORAGE-011 remains open.

## Rationale

An assigned operational identifier distinguishes independently addressable
physical representations without coupling Resource identity to location.
Binding one record to one complete-object or chunked reconstruction
representation makes map cardinality and availability understandable while
preserving the already normative Resource and Chunk identity contracts.
Keeping Chunk locations within their owning Resource Replica avoids
introducing a separate Chunk-level Replica or shared-ownership contract.

## Alternatives considered

- Derive ReplicaId from ResourceId, EndpointId, or locator: rejected because
  those values can change or can identify multiple independent
  representations at one Endpoint.
- Require exactly one Replica per Resource and Endpoint: rejected because
  distinct independently addressable copies or reconstruction layouts may
  coexist there.
- Make each Chunk copy a Resource Replica: rejected because the Storage Map
  represents complete Resource availability, not independent Chunk copies.
- Put representation or location into Resource identity or historical
  metadata: rejected because this would reopen settled identity and
  reconstruction rules.

## Specification impact

- `Specs/OMVCS Core Specification.md` §§32–35.
- `Specs/OMVCS Core Specification.md` §51, separating availability from
  corruption/integrity.
- `Specs/OMVCS Core Invariants Specification.md`, INV-RES-003, INV-RES-005,
  and INV-RES-006.
- `Specs/OMVCS Glossary.md`: Replica Identifier, Resource Replica, Storage
  Location, Storage Map, Availability State, Replica Addition, and Replica
  Removal.
- `Specs/OMVCS Storage Adapter Specification.md` §§29–36, 73–75, and
  161–164.
- Storage Map guarded-update mechanics remain subject to DG-0032.

## Test impact

WORK-0016 conformance tests must cover canonical UUIDv7 ReplicaId
representation and stability; incomplete/unverified candidates remaining
outside the registered Storage Map; zero, one, and multiple Replicas; multiple
independently addressable representations at one Endpoint; new-versus-same
Replica behavior; complete-object and chunked binding; manifest ResourceId,
order, and reconstruction consistency; per-Replica chunk availability; and
identity/history independence from Endpoint, locator, preference,
availability, and verification metadata. Tests must ensure that a Chunk
copy is not registered as a Resource Replica. Guarded-update tests must
follow DG-0032 and must not copy Line-generation rules.

## Implementation impact

WORK-0016 may implement the Replica model and representation binding once
DEC-STORAGE-007 is reflected in the Specs. Guarded Storage Map mutations
remain gated by DG-0032. Registration-to-verified promotion remains gated
by the approved verification result contract in WORK-0017 and DEC-STORAGE-
004/005. WORK-0017–WORK-0021 must use these identity and representation
rules without adding shared-namespace claims.

## Compatibility / migration impact

No Resource, Chunk, Component State, Project State, or Revision identity
changes. Existing examples using abbreviated Replica identifiers are
illustrative only; conforming assigned ReplicaIds use canonical UUIDv7
text. No implementation or persisted Replica records are known to exist.

## Notes

This ADR does not choose a chunking algorithm, shared Chunk-location
ownership model, verification taxonomy, upload assurance, deletion policy,
or Storage Map generation domain.
