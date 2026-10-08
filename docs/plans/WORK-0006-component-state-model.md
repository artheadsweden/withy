# WORK-0006 — Component State model

Status: PLANNED
Owner agent: Core Engineer
Milestone: M1
Branch: `work/0006-component-state-model`

## Objective

Represent an immutable, content-addressed state of one Creative Component, including its stable Component reference and Resource dependencies, with optional parentage that preserves known lineage without fabricating unknown ancestry.

## Normative requirements

- Glossary: Component State.
- Core Specification, sections 5.1, 7, 10–11, and 23.
- Core Invariants: INV-HIST-001–003, INV-HIST-006, INV-RES-004, INV-RES-008, INV-PROJ-002, INV-PROV-003, INV-DAW-004.
- ADR-0007 for the generic Resource Reference schema and identity boundary.
- ADR-0008 for the exact `byte_length` integer range and rejection rules.
- ADR-0009 for applicable schema/Adapter validation and historical admission of Resource Reference `properties`.
- ADR-0010 for the one-field Creative Component object and separation of Component State from Project State membership.

## Dependencies

- WORK-0001 through WORK-0005.
- Hashed metadata maps follow RFC 8785 object-member ordering; duplicate member names are rejected before canonicalization/hashing (ADR-0005).

## Allowed scope

- `crates/omvcs-model/`
- Focused Component State validation/identity tests and conformance vectors explicitly permitted by this package.

## Deliverables

- Immutable Component State identity referencing its Creative Component and required Resource Objects.
- Optional parentage representation and validation; distinguish known initial state from unknown/unasserted lineage.
- Canonical identity tests use resolved array collection rules and RFC 8785 map handling.

## Acceptance tests

- A published Component State cannot be changed in place.
- Each Component State identifies the Creative Component whose state it represents by its typed `component_id`; Component State creative fields do not expand the generic Creative Component object.
- A changed Resource reference produces a distinct Component State identity.
- Resource references use immutable Resource identifiers.
- Each embedded Resource Reference has a typed Resource Identifier and a `byte_length` equal to the complete Resource's byte count, represented as an integer in `0 ..= 9007199254740991`; reject negative, fractional, greater-than-maximum, string, and other alternate representations (ADR-0008).
- Embedded Resource Reference length vectors accept `0`, `1`, `9007199254740991` and reject `-1`, `1.5`, `9007199254740992`, and `"1"`.
- Every embedded Resource Reference without `properties` passes generic Core validation; any present `properties`, including an empty map, is admitted only after validation under the exact versioned context governing its Component State use.
- Rejected, unknown, unavailable, or non-unique property validation context prevents admission of that Component State as valid history; preserved unchecked candidates cannot be hashed or committed as valid history.
- Component State identity contains property values after successful validation but not validation status/evidence; context-driven shapes, nested array declarations, and recursive duplicate-member rejection are enforced.
- Changing a canonical Resource Reference field value changes Component State identity but does not change Resource Identifier for unchanged raw bytes; reordering `properties` map insertion does not change identity.
- An initial state with zero parents is valid; known derived states SHOULD record one or more parents.
- An explicit empty `parents` array identifies a known initial state; an omitted field does not imply initial state.
- Unknown historical lineage is never fabricated.
- An operation/provenance rule requiring derivation preservation MUST enforce parent recording.
- Parent/resource set-like permutations produce identical identities; duplicate set elements are rejected.
- Metadata map insertion-order permutations produce identical canonical bytes/identities; duplicate member names are rejected; map entries receive no additional element-byte sorting.

## Explicit non-goals

- Defining provenance beyond the specified Component State lineage relation.
- Adapter-specific interpretation, Component mapping, or DAW-state operations.
- Storage chunking, replica tracking, or publication transactions.

## Known Design Gaps

- DG-0007 and DG-0008 are resolved by ADR-0007 and ADR-0008; apply the same Resource Reference schema and safe-integer validation to embedded references.
- DG-0009 is resolved by ADR-0009; apply the same exact-context properties validation and unchecked-to-historical admission boundary to every embedded Resource Reference.

## Implementation plan

1. Implement the optional parentage semantics resolved in ADR-0003 without inventing operation-specific mandatory rules.
2. Model the resolved Component State references and immutable hash preimage.
3. Add identity, reference-integrity, lineage, and permutation tests from the resolved rules.

## Verification requirements

The Verifier must reject any parentage or collection-order behavior not authorized by resolved Specs and must test immutability and Resource identity references.

## Completion criteria

Formatting, focused tests, coverage-map update, independent verification, handover, and clean Git state.
