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
- Core Invariants: INV-HIST-001–003, INV-HIST-006, INV-RES-004, INV-PROJ-002, INV-PROV-003.
- ADR-0007 for the generic Resource Reference schema and identity boundary.

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
- A changed Resource reference produces a distinct Component State identity.
- Resource references use immutable Resource identifiers.
- Each embedded Resource Reference has a typed Resource Identifier and a `byte_length` equal to the complete Resource's byte count; optional fields and prohibited presentation/physical fields follow ADR-0007.
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

- None affecting this work package. DG-0007 is resolved by ADR-0007.

## Implementation plan

1. Implement the optional parentage semantics resolved in ADR-0003 without inventing operation-specific mandatory rules.
2. Model the resolved Component State references and immutable hash preimage.
3. Add identity, reference-integrity, lineage, and permutation tests from the resolved rules.

## Verification requirements

The Verifier must reject any parentage or collection-order behavior not authorized by resolved Specs and must test immutability and Resource identity references.

## Completion criteria

Formatting, focused tests, coverage-map update, independent verification, handover, and clean Git state.
