# WORK-0004 — Resource and Resource Reference model

Status: BLOCKED ON DG-0007
Owner agent: Core Engineer
Milestone: M1
Branch: `work/0004-resource-model`

## Objective

Model logical Resources, immutable Resource Objects, and historical Resource References using content identity independent of physical storage.

## Normative requirements

- Glossary: Resource, Resource Object, Resource Identifier, Resource Manifest, Friendly Name.
- Core Specification, sections 6–8.
- Core Invariants: INV-RES-001–007 and INV-PROJ-003.

## Dependencies

- WORK-0001 and WORK-0003 for Resource identifiers and raw-byte hashing.
- WORK-0002 for any hashed metadata representation.
- Chunk representation details are not required for the logical M1 Resource identity.

## Allowed scope

- `crates/omvcs-model/`
- Focused Resource model tests and conformance vectors explicitly permitted by this package.

## Deliverables

- Explicit distinction between logical Resource metadata and immutable Resource Object bytes/identity.
- Resource References identify content by Resource Identifier; descriptive fields do not become identity or locators.
- Model boundary documenting that physical replicas/locations are operational, not historical Resource identity.

## Acceptance tests

- A Resource Object is immutable by identity; changed bytes require a different Resource Identifier.
- Renaming a Friendly Name does not change Resource identity.
- Resource References use the content identifier and do not require a storage URL/path.
- Physical location, replica count, and availability do not alter Resource identity or historical references.
- Two physical copies with identical bytes share the same Resource Identifier.

## Explicit non-goals

- Chunk sizes, chunk boundaries, chunk manifests, transfer, replicas, or Storage Adapter behavior.
- Inventing required media-type, role, or format-specific metadata where not stated as mandatory.
- Implementing filesystem materialisation or local cache behavior.

## Known Design Gaps

- DG-0007 blocks defining the Resource Manifest / Resource Reference schema boundary and which descriptive fields participate in containing hashed historical objects. Do not implement those fields or identities until the required human decision is recorded in an ADR and the affected Specs are updated.
- Physical Chunk Manifest representation remains outside this package as stated in the explicit non-goals.

## Implementation plan

1. Define the minimal logical Resource/Resource Reference model from explicit normative requirements.
2. Connect Resource identity to WORK-0003.
3. Test byte identity, names, location independence, and immutable references.

## Verification requirements

The Verifier must attempt to disprove any coupling between Resource identity and filename, physical locator, provider, or replica state.

## Completion criteria

Formatting, focused tests, coverage-map update, independent verification, handover, and clean Git state.
