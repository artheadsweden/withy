# WORK-0005 — Creative Component identity model

Status: VERIFIED
Owner agent: Core Engineer
Milestone: M1
Branch: `work/0005-creative-component`

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
- Creative Component contains a required typed assigned `component_id` and generic model/serialization contains no `project_id`, `name`, `kind`, `created_at`, Resource content, Component State parentage, DAW-native identifier, Platform, storage, or location field.
- If the generic model provides a decoding boundary, reject extra object fields rather than silently treating them as part of the generic Creative Component.
- Component identity is stable across Resource replacement/re-recording and independent of Project membership, Component State parentage, DAW-native identifiers, names, kind/classification, timestamps, filenames, storage, Platform accounts, and locations.
- Project association is expressed through Project State membership/reference, not Component object data.
- A presentation rename does not create a new Component Identifier and does not by itself require a new Component State.
- No clone, fork, copy, import, move, ownership, or cross-Project reuse semantics are inferred or implemented.
- Distinct Components may coexist without Core requiring a generic `kind` field or vocabulary.

## Explicit non-goals

- Inventing a mandatory `kind` vocabulary, naming rules, rename/delete lifecycle, or DAW-to-Component mapping.
- Component State modeling, historical parentage, or Project State modeling/composition/membership logic; consume the already-specified membership boundary only where needed to keep the Component object separate.
- Adapter State modeling, storage, repository operations, publication, or other repository behavior.
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

## Independent acceptance — 2026-10-08

The Verifier accepted remediation commit `947fa4e76ab9af6a2f5933b58c35aec843321e56`.
The original gate rejected positional JSON array decoding; the map-only visitor now
rejects arrays and other non-object forms while the exact one-field object round-trips.
The independent regression file `crates/omvcs-model/tests/creative_component_acceptance.rs`
is preserved without weakening its assertions.

- Focused model tests: 6 implementation tests and 3 independent acceptance tests passed.
- Full `cargo test --locked -p omvcs-model`: 62 tests passed, including 4 compile-fail doctests.
- `cargo fmt --package omvcs-model -- --check`: passed.
- `cargo clippy --locked -p omvcs-model --all-targets -- -D warnings`: passed.
- Working-tree and base-to-HEAD `git diff --check`: passed.
- Base `b1106ee6966351d57a448d90c6e2980ee4275549` and ADR-0010 merge
  `509dbba` are ancestors; no implementation-branch specification edits or unrelated changes.
- One-field typed identity, strict decoding, identity independence and namespace separation
  confirmed. No state/membership, parentage, Adapter State, storage/repository operations,
  or undefined lifecycle/cross-Project semantics were implemented.

Semantic decisions beyond the Specs/ADR-0010: `None`. WORK-0006 and all later
packages remain unstarted. This acceptance does not authorize their start.
No push or integration performed; the verification commit contains only tests/docs.
