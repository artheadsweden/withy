# WORK-0007 — Project State model

Status: PLANNED
Owner agent: Core Engineer
Milestone: M1
Branch: `work/0007-project-state-model`

## Objective

Represent one complete immutable logical Project State using stable Project identity and immutable Component State references, while preserving the DAW-independent boundary.

## Normative requirements

- Glossary: Project State.
- Core Specification, sections 12–13, 24, and 55.
- Core Invariants: INV-HIST-003, INV-RES-004, INV-PROJ-004.

## Dependencies

- WORK-0001, WORK-0002, WORK-0003, WORK-0005, and WORK-0006.
- ADR-0004 resolves the Adapter State representation: Project State references exactly one canonical Adapter State metadata object, which may reference opaque native-state Resources.
- Hashed JSON object maps follow RFC 8785 member ordering only, and duplicate member names are rejected before hashing/canonical serialization (ADR-0005).
- DEC-PLATFORM-016 remains open. Do not add licensing fields to the M1 Project State model unless its ownership/location is decided first.

## Allowed scope

- `crates/omvcs-model/`
- Focused Project State validation/identity tests and conformance vectors explicitly permitted by this package.

## Deliverables

- Complete Project State references, not a change list.
- Storage-independent Project and Component State references.
- Typed reference to exactly one canonical Adapter State metadata object; native-state Resources remain referenced through that object.

## Acceptance tests

- A Project State identifies a complete logical state, not merely a delta.
- Project State references exactly one Adapter State Identifier; a native Resource Identifier is invalid as the complete Adapter State reference.
- Adapter State is canonical OMVCS metadata and may reference opaque native-state Resource Objects.
- Project identity and Component State references are explicit and stable.
- Storage locations, platform URLs, local paths, and availability do not enter Project State identity.
- Different creative-object references yield different Project State identities.
- Component, Adapter State binding, and project-metadata map insertion-order permutations yield identical canonical bytes/identities; duplicate member names are rejected; no additional entry sorting is applied.

## Explicit non-goals

- Working State, materialisation, DAW restoration, Resource retrieval, or storage maps.
- Inferring missing component states or choosing defaults not defined by the Specs.
- DAW-specific fields in the Core model.

## Known Design Gaps

- None affecting this work package.

## Implementation plan

1. Apply the resolved Component State and Adapter State reference contracts.
2. Implement the complete-state model without adding adapter/provider fields.
3. Verify identity independence and complete-state—not-delta—semantics.

## Verification requirements

The Verifier must compare Project State references and invariants to the original sections and attempt invalid/partial-state cases without assuming an unspecified default.

## Completion criteria

Formatting, focused tests, coverage-map update, independent verification, handover, and clean Git state.
