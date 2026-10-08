# WORK-0006 — Component State model

Status: BLOCKED ON DG-0002 AND DG-0003
Owner agent: Core Engineer
Milestone: M1
Branch: `work/0006-component-state-model`

## Objective

Represent an immutable, content-addressed state of one Creative Component, including its stable Component reference and Resource dependencies, without choosing unresolved lineage or collection-order semantics.

## Normative requirements

- Glossary: Component State.
- Core Specification, sections 10–11 and 89, question 10.
- Core Invariants: INV-HIST-001–003, INV-RES-004, INV-PROJ-002, INV-PROV-003.

## Dependencies

- WORK-0001 through WORK-0005.
- DG-0002 must resolve Component State parentage requirements.
- DG-0003 must resolve canonical ordering of hashed collection fields.

## Allowed scope

- `crates/omvcs-model/`
- Focused Component State validation/identity tests and conformance vectors explicitly permitted by this package.

## Deliverables

- Immutable Component State identity referencing its Creative Component and required Resource Objects.
- Parentage representation and validation only after DG-0002 is resolved.
- Canonical identity tests only after DG-0003 is resolved.

## Acceptance tests

- A published Component State cannot be changed in place.
- A changed Resource reference produces a distinct Component State identity.
- Resource references use immutable Resource identifiers.
- Parentless/parented-state validity tests remain blocked until DG-0002 is resolved.
- Permutations of parent/resource collections are tested only after DG-0003 defines their order semantics.

## Explicit non-goals

- Defining provenance beyond the specified Component State lineage relation.
- Adapter-specific interpretation, Component mapping, or DAW-state operations.
- Storage chunking, replica tracking, or publication transactions.

## Known Design Gaps

- DG-0002 — Component State parentage requirement.
- DG-0003 — canonical order of hashed collection fields.

## Implementation plan

1. Hold implementation of normative parentage validation until DG-0002 is approved and Specs/tests are updated.
2. Model the resolved Component State references and immutable hash preimage.
3. Add identity, reference-integrity, lineage, and permutation tests from the resolved rules.

## Verification requirements

The Verifier must reject any parentage or collection-order behavior not authorized by resolved Specs and must test immutability and Resource identity references.

## Completion criteria

Formatting, focused tests, coverage-map update, independent verification, handover, and clean Git state.
