# ADR-0033 — Provider locator envelope

Status: ACCEPTED
Date: 2026-10-10
Decision owner: Human
Related Design Gap: N/A (resolves DEC-STORAGE-007)

## Context

Storage Adapter §73 required Replica records to retain enough information
to identify a physical copy, but the provider locator's generic
representation, persistence boundary, and relationship to Endpoint identity
were not fixed. DEC-STORAGE-007 records this question.

## Decision

1. Provider locator data is an opaque provider-owned operational value
   persisted in a Replica record through a generic typed envelope
   equivalent to:

   ```text
   ProviderLocator {
       schema: string identifying the exact versioned locator schema,
       value: canonical JSON data governed by that schema
   }
   ```

2. `schema` is a string using the existing versioned schema-identifier
   convention and identifies the exact provider locator schema and version.
   `value` is serialized as canonical JSON using RFC 8785/JCS; the
   provider-owned schema defines its valid shape and meaning. Generic Core
   validates the envelope and canonical representation but does not
   interpret or semantically validate provider-specific locator fields.
3. Generic Core owns the envelope structure, canonical representation,
   persistence, association with the Replica, and exact pass-through to the
   applicable Storage Adapter. The Storage Adapter/provider owns locator
   meaning, validation under the identified schema, and translation to
   provider operations.
4. ProviderLocator is operational metadata. It is not Resource identity,
   Replica identity, Chunk identity, or historical metadata.
5. A ProviderLocator MUST NOT contain credentials, access tokens,
   expiring signed URLs, or other secrets; depend on process-local handles;
   or prevent rediscovery of the physical representation after restart.
   Temporary access grants remain separate.
6. EndpointId and ProviderLocator together identify where a Replica
   representation can be accessed. A locator is interpreted only in its
   Endpoint and Storage Adapter context. The same locator value at two
   Endpoints does not imply the same physical object or Replica.
7. Locator changes that merely move or rekey the same logical physical
   representation within the same Endpoint/provider context retain
   ReplicaId. Creating a distinct independently addressable copy creates
   a new ReplicaId under ADR-0032.

## Rationale

The envelope lets Core durably preserve and return provider locator data
without importing provider path/key semantics into Core or historical
identity. Versioned schema association makes provider-specific interpretation
explicit, while canonical JSON supports deterministic persistence and
comparison.

## Alternatives considered

- Store provider-specific fields directly in generic Core records: rejected
  because this leaks provider semantics into Core.
- Store only an Endpoint and rediscover all locators after every restart:
  rejected because a Replica record must retain enough durable operational
  information to identify its physical representation.
- Use a process-local provider handle or temporary URL as locator: rejected
  because it cannot provide durable rediscovery and may expire or expose
  secrets.
- Include locator data in Resource or Revision identity: rejected because
  storage location is operational.

## Specification impact

- `Specs/OMVCS Storage Adapter Specification.md` §§73–75, §252, and §270,
  item 7.
- `Specs/OMVCS Core Specification.md` §§32–33, for the generic Replica
  record boundary and example.
- `Specs/OMVCS Glossary.md`, Storage Location and Storage Map.
- `Specs/OMVCS Core Invariants Specification.md`, INV-RES-003 and
  INV-RES-005.
- ADR-0032 defines the associated Replica identity and locator-mutation
  rules.

## Test impact

WORK-0016 and Storage Adapter conformance tests must cover exact envelope
round-trip and pass-through; exact schema/version association; canonical
JSON persistence and comparison; provider rejection of invalid values under
the named schema; locator durability across restart; separation from
Endpoint/Resource/Replica/Chunk identity; same-value locators at different
Endpoints not implying shared identity; locator mutation for the same
Replica versus a new independently addressable copy; and rejection of
credentials, expiring grants, and process-local handles.

## Implementation impact

WORK-0016 owns the Core Replica/Storage Map model and its opaque locator
envelope. Storage Adapters validate and interpret their own locator schemas.
WORK-0018–WORK-0021 may use the envelope only under this provider-neutral
boundary. No production implementation is authorized by this ADR alone.

## Compatibility / migration impact

The provider-specific example shape in Storage Adapter §73 is replaced by
the generic envelope. No persisted OMVCS 0.1 Replica records are known to
exist. Locator schema evolution is provider-owned and must retain exact
version association.

## Notes

DEC-STORAGE-011 remains open. This ADR does not establish Endpoint namespace
identity, shared-object identity, cross-Project deduplication guarantees,
or shared-namespace GC.
