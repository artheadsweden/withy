# WORK-0004 — Resource and Resource Reference model

Status: PLANNED
Owner agent: Core Engineer
Milestone: M1
Branch: `work/0004-resource-model`

## Objective

Model logical Resources, immutable Resource Objects, and historical Resource References using content identity independent of physical storage.

## Normative requirements

- Glossary: Resource, Resource Object, Resource Identifier, Resource Manifest, Friendly Name.
- Core Specification, sections 5.1, 6–8, and 56.
- Core Invariants: INV-RES-001–007 and INV-PROJ-003.
- ADR-0001, ADR-0005, and ADR-0007.
- ADR-0008 defines the accepted `byte_length` domain and conformance vectors.

## Dependencies

- WORK-0001 and WORK-0003 for Resource identifiers and raw-byte hashing.
- WORK-0002 for any hashed metadata representation.
- Chunk representation details are not required for the logical M1 Resource identity.

## Allowed scope

- `crates/omvcs-model/`
- Focused Resource model tests and conformance vectors explicitly permitted by this package.

## Deliverables

- Explicit distinction between logical Resource metadata and immutable Resource Object bytes/identity.
- Generic Resource References contain required typed `resource_id` and complete-resource `byte_length` integer in `0 ..= 9007199254740991`, with optional `role`, `media_type`, and schema/Adapter-supplied immutable `properties`.
- Every present Resource Reference field participates in the containing historical object's canonical identity, while Resource Identifier remains determined solely by complete raw Resource bytes.
- Resource References exclude friendly/logical filename, Chunk/Chunk Manifest data, storage/location/Replica/provider metadata, and credentials.
- Resource Manifest is descriptive Resource-oriented metadata/reconstruction terminology only, with no separate content-derived historical identifier in OMVCS 0.1.
- Model boundary documenting that physical replicas/locations are operational, not historical Resource identity.

## Acceptance tests

- A Resource Object is immutable by identity; changed bytes require a different Resource Identifier.
- Renaming a Friendly Name does not change Resource identity.
- Resource References require a typed Resource Identifier and non-negative byte length; optional fields are limited to role, media type, and schema/Adapter-supplied immutable interpretation properties.
- `byte_length` is REQUIRED and equals the complete Resource's byte count; it MUST be an integer in `0 ..= 9007199254740991`. Reject missing, negative, fractional/non-integral, greater-than-maximum, string, and alternate-encoded values regardless of host integer capacity.
- Changing the canonical value of a present Resource Reference field changes the containing historical object's identity but never changes the Resource Identifier for the same raw bytes.
- Reordering `properties` map insertion without changing its entries does not change the containing identity under RFC 8785/ADR-0005.
- Changing any Resource Reference descriptive field while keeping raw bytes fixed leaves the Resource Identifier unchanged.
- Resource References contain no logical/Friendly Name, filename, chunk layout, Chunk Manifest, storage/Replica/provider data, or credentials.
- A presentation/local rename, storage location or Replica change, and physical chunk-layout variation do not change Resource or historical identity.
- If exact DAW reconstruction requires a filename, that state is represented in Adapter State rather than Resource Reference.
- No Resource Manifest identifier type or separate Resource Manifest historical object is introduced for OMVCS 0.1.
- Physical location, replica count, and availability do not alter Resource identity or historical references.
- Two physical copies with identical bytes share the same Resource Identifier.
- Resource Manifest has no independent content-derived historical identifier in OMVCS 0.1.
- `byte_length` conformance vectors accept `0`, `1`, and `9007199254740991`; reject `-1`, `1.5`, `9007199254740992`, and `"1"`.

## Explicit non-goals

- Chunking algorithms, chunk sizes/boundaries, Chunk Manifest schemas, Chunk IDs, Storage Maps, Replicas, transfer, Storage Adapters, or other storage behavior.
- Repository Home, publication transactions, and recovery.
- Adapter-specific naming rules.
- A Resource Manifest identifier type or independent Resource Manifest identity.
- Inventing required media-type, role, or format-specific metadata where not stated as mandatory.
- Implementing filesystem materialisation or local cache behavior.

## Known Design Gaps

- DG-0007 is resolved by ADR-0007. Implement only the approved Resource Reference schema and Resource Manifest boundary; do not add fields or identities beyond that decision.
- DG-0008 is resolved by ADR-0008. Enforce the approved safe-integer range and reject alternate representations.
- Physical Chunk Manifest representation remains outside this package as stated in the explicit non-goals.

## Implementation plan

1. Define the minimal logical Resource/Resource Reference model from explicit normative requirements.
2. Connect Resource identity to WORK-0003.
3. Test byte identity, names, location independence, and immutable references.

## Verification requirements

The Verifier must attempt to disprove any coupling between Resource identity and filename, physical locator, provider, or replica state.

## Completion criteria

Formatting, focused tests, coverage-map update, independent verification, handover, and clean Git state.
