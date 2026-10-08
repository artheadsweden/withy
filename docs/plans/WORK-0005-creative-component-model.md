# WORK-0005 — Creative Component identity model

Status: PLANNED
Owner agent: Core Engineer
Milestone: M1
Branch: `work/0005-creative-component-model`

## Objective

Represent a Creative Component as a stable logical identity anchor, distinct from filenames and current Resource Objects, with Project association expressed through Project State membership.

## Normative requirements

- Glossary: Project and Creative Component.
- Core Specification, sections 4, 9, 10, and 13.
- Core Invariants: INV-PROJ-001–003.
- ADR-0010: Creative Component identity and object boundary.

## Dependencies

- WORK-0001 for assigned Project and Component identifiers.
- WORK-0004 for the distinction between Components and Resources and the resolved generic Resource Reference model (ADR-0007).
- ADR-0010 for the one-field generic object and Project State membership association.
- The actual Component State relationship is addressed by WORK-0006.

## Allowed scope

- `crates/omvcs-model/`
- Focused Creative Component identity tests and conformance vectors explicitly permitted by this package.

## Deliverables

- Generic Creative Component object containing exactly a required typed assigned `component_id`.
- Project association represented by Project State membership/reference, not a Component `project_id` field.
- Model distinction between semantic Component identity, Component State history, descriptive names, and current Resources.
- No unapproved taxonomy or lifecycle semantics.

## Acceptance tests

- Replacing a file/Resource used by a Component does not change that Component's identifier.
- Creative Component contains a required typed `component_id` and generic model/serialization contains no `project_id`, `name`, `kind`, or `created_at`.
- Component identity is stable across Resource replacement/re-recording and independent of Project membership, Component State parentage, DAW-native identifiers, names, kind/classification, timestamps, filenames, storage, Platform accounts, and locations.
- Project association is expressed through Project State membership/reference, not Component object data.
- A presentation rename does not create a new Component Identifier and does not by itself require a new Component State.
- No clone, fork, copy, import, move, ownership, or cross-Project reuse semantics are inferred or implemented.
- Distinct Components may coexist without Core requiring a generic `kind` field or vocabulary.

## Explicit non-goals

- Inventing a mandatory `kind` vocabulary, naming rules, rename/delete lifecycle, or DAW-to-Component mapping.
- Component State lineage or Project State composition beyond consuming the Project State membership contract.
- Clone, fork, copy, import, move, ownership, or cross-Project reuse semantics.
- User-interface labels or adapter-specific grouping.

## Known Design Gaps

- DG-0010 is resolved by ADR-0010. Implement only the one-field generic Core Component object and the Project State membership association; do not add cross-Project ownership or lifecycle behavior.
- WORK-0006 parentage semantics are resolved by ADR-0003; hashed-map identity follows the resolved RFC 8785 rule in ADR-0005.

## Implementation plan

1. Define the stable assigned Component identity object from normative requirements.
2. Reuse assigned identifier types from WORK-0001.
3. Add negative tests against Project membership, filename, Resource, DAW-native identity, descriptive metadata, storage, Platform, and location coupling.

## Verification requirements

The Verifier must check stable semantic identity independently of filename, physical storage, and adapter-specific entity names.

## Completion criteria

Formatting, focused tests, coverage-map update, independent verification, handover, and clean Git state.
