# WORK-0007 — Project State model

Status: PLANNED
Owner agent: Core Engineer
Milestone: M1
Branch: `work/0007-project-state-model`

## Objective

Represent one complete immutable logical Project State using stable Project identity and immutable Component State references, while preserving the DAW-independent boundary.

## Normative requirements

- Glossary: Project State.
- Core Specification, sections 5.1, 7, 12–13, 24, 56, and 76–77.
- Core Invariants: INV-HIST-003, INV-RES-004, INV-RES-008, INV-PROJ-004, INV-DAW-004.

## Dependencies

- WORK-0001, WORK-0002, WORK-0003, WORK-0005, and WORK-0006.
- ADR-0004 resolves the Adapter State representation: Project State references exactly one canonical Adapter State metadata object, which may reference opaque native-state Resources.
- ADR-0007 defines the Resource Reference fields used by Adapter State resource entries and excludes names and physical storage/reconstruction data from those references.
- ADR-0008 defines the exact accepted integer range for each Resource Reference `byte_length`.
- ADR-0009 defines the versioned schema/Adapter validation authority and historical admission boundary for Resource Reference `properties`.
- Hashed JSON object maps follow RFC 8785 member ordering only, and duplicate member names are rejected before hashing/canonical serialization (ADR-0005).
- `component_bindings` is keyed by Creative Component Identifier and map values carry adapter-specific binding records; the key is not duplicated in the value (ADR-0006).
- DEC-PLATFORM-016 remains open. Do not add licensing fields to the M1 Project State model unless its ownership/location is decided first.

## Allowed scope

- `crates/omvcs-model/`
- Focused Project State validation/identity tests and conformance vectors explicitly permitted by this package.

## Deliverables

- Complete Project State references, not a change list.
- Storage-independent Project and Component State references.
- Typed reference to exactly one canonical Adapter State metadata object; native-state Resources remain referenced through that object.
- Adapter State `component_bindings` maps Creative Component Identifiers to adapter-specific binding records without repeating the key in the value.

## Acceptance tests

- A Project State identifies a complete logical state, not merely a delta.
- Project State references exactly one Adapter State Identifier; a native Resource Identifier is invalid as the complete Adapter State reference.
- Adapter State is canonical OMVCS metadata and may reference opaque native-state Resource Objects.
- Each Resource Reference embedded in Adapter State has a typed Resource Identifier and a `byte_length` equal to the complete Resource's byte count, represented as an integer in `0 ..= 9007199254740991`; reject negative, fractional, greater-than-maximum, string, and other alternate representations (ADR-0008).
- Embedded Resource Reference length vectors accept `0`, `1`, `9007199254740991` and reject `-1`, `1.5`, `9007199254740992`, and `"1"`.
- Every Adapter State Resource Reference with `properties`, including an empty map, is validated under the exact applicable versioned Adapter context before historical admission; unknown, unavailable, or non-unique context cannot yield valid Adapter State history.
- Unchecked candidates, if preserved, cannot be embedded in a valid Project State/Revision or used to produce a valid historical identity; validation evidence is not part of canonical metadata identity.
- Adapter schema validation rejects ADR-0007-excluded semantics under alternate keys and nested values; Core performs generic structure, duplicate-name, canonicalization, and nested collection validation without interpreting Adapter-specific values.
- Adapter State `component_bindings` uses a Creative-Component-Identifier-keyed object map whose values contain adapter-specific binding records; values do not repeat the key merely to restate the identifier.
- Project identity and Component State references are explicit and stable.
- Storage locations, platform URLs, local paths, and availability do not enter Project State identity.
- Different creative-object references yield different Project State identities.
- Component, Adapter State binding, and project-metadata map insertion-order permutations yield identical canonical bytes/identities; duplicate member names are rejected; no additional entry sorting is applied.

## Explicit non-goals

- Working State, materialisation, DAW restoration, Resource retrieval, or storage maps.
- Inferring missing component states or choosing defaults not defined by the Specs.
- DAW-specific fields in the Core model.

## Known Design Gaps

- DG-0006, DG-0007, DG-0008, and DG-0009 are resolved by ADR-0006, ADR-0007, ADR-0008, and ADR-0009; apply the generic binding-map, Resource Reference, and property-admission contracts consistently.

## Implementation plan

1. Apply the resolved Component State and Adapter State reference contracts.
2. Implement the complete-state model without adding adapter/provider fields.
3. Verify identity independence and complete-state—not-delta—semantics.

## Verification requirements

The Verifier must compare Project State references and invariants to the original sections and attempt invalid/partial-state cases without assuming an unspecified default.

## Completion criteria

Formatting, focused tests, coverage-map update, independent verification, handover, and clean Git state.
