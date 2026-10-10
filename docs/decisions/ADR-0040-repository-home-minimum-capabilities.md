# ADR-0040 — Repository Home minimum capabilities

Status: ACCEPTED
Date: 2026-10-10
Decision owner: Human
Related decision: DEC-STORAGE-003

## Context

The Storage Adapter Specification required Repository Home metadata
capabilities but did not specify a minimum conformance profile. Resource
storage, repository authority, and Platform mirroring needed distinct
boundaries.

## Decision

1. A Repository Home is the durable authoritative repository-metadata home
   for one OMVCS Project. It is not synonymous with Resource storage, a
   particular Endpoint, a filesystem directory layout, or a Platform.
2. A conforming Home MUST durably store and retrieve locally admitted
   immutable historical metadata required by the Core repository model,
   subject to declared metadata-completeness/shallow-boundary state. Home
   status does not imply complete history.
3. A conforming Home MUST durably store and retrieve required operational
   repository metadata, including as applicable Project/repository
   identity, metadata completeness and declared-boundary records, Endpoint
   descriptors, Storage Map and `StorageMapGeneration`, and operational
   records required by implemented Core contracts.
4. A conforming Home MUST support guarded updates for mutable operational
   metadata and the conditional/logical atomicity required by Core,
   including the complete Project Storage Map CAS in ADR-0034. Provider
   tokens may implement this internally but do not replace Core
   `StorageMapGeneration`.
5. A conforming Home MUST persist and expose the marker/discovery
   information defined by ADR-0041. Successful durable writes MUST meet
   the durability semantics declared under the existing Storage Adapter
   contract.
6. Resource and Chunk byte storage are not required Home capabilities.
   A Home MAY provide them only as separately declared Resource Storage
   capabilities.
7. Platform metadata is not authoritative Home metadata merely because a
   Platform mirrors it. The existing requirement that every actively
   published Project designate a creator-controlled Repository Home is
   unchanged.
8. This decision does not define M4 publication-transaction durability,
   recovery, or operation-log semantics.

## Rationale

The profile defines the minimum durable and concurrency-safe repository
authority without conflating it with content custody or Platform services.

## Alternatives considered

- Require every Home to store Resource/Chunk bytes: rejected; content
  custody may be at separate Endpoints.
- Treat a Platform mirror as a Home: rejected; platform authority is
  explicitly separate.
- Claim Home status proves complete metadata history: rejected; declared
  completeness/boundary state remains authoritative.
- Define M4 publication and recovery semantics here: excluded by the
  approved decision.

## Specification impact

- `Specs/OMVCS Storage Adapter Specification.md` §§10, 13, 44–51, 124–126,
  and decision list §270.
- `Specs/OMVCS Core Specification.md` §29.
- `Specs/OMVCS Glossary.md`, Repository Home.
- `Specs/OMVCS Core Invariants Specification.md`, existing INV-STOR-001–005
  and history-completeness invariants remain controlling.

## Test impact

Home conformance tests MUST separately exercise historical metadata
durability/retrieval, completeness boundary preservation, required
operational metadata, Project-wide Storage Map CAS/atomicity, marker
availability, and declared durability. Tests MUST show Resource/Chunk
capability may be absent without invalidating Home conformance, and a
Platform mirror alone does not satisfy Home authority.

## Implementation impact

WORK-0019 may implement a filesystem Home only with the complete profile;
Resource-only filesystem capability remains independently reportable.
WORK-0020/0021 retain their existing boundaries and may consume the Home
contract without changing replication or retained-source migration
semantics.

## Compatibility / migration impact

This clarifies the already-required active-Project Home boundary; it does
not change Project, Resource, Revision, Replica, or Storage Map identity.

## Notes

DEC-STORAGE-013 remains OPEN. No official reference Storage Adapter is
designated.
