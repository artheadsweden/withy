# WORK-0005 — Creative Component identity model

Status: BLOCKED ON DG-0010 — Creative Component object schema and metadata semantics
Owner agent: Core Engineer
Milestone: M1
Branch: `work/0005-creative-component-model`

## Objective

Represent a Creative Component as a stable, semantically identified Project entity distinct from filenames and current Resource Objects.

## Normative requirements

- Glossary: Project and Creative Component.
- Core Specification, sections 4 and 9.
- Core Invariants: INV-PROJ-001–003.

## Dependencies

- WORK-0001 for assigned Project and Component identifiers.
- WORK-0004 for the distinction between Components and Resources and the resolved generic Resource Reference model (ADR-0007).
- The actual Component State relationship is addressed by WORK-0006.

## Allowed scope

- `crates/omvcs-model/`
- Focused Creative Component identity tests and conformance vectors explicitly permitted by this package.

## Deliverables

- Stable assigned Component identity linked to its Project.
- Model distinction between semantic Component identity and names/current files.
- No unapproved taxonomy or lifecycle semantics.

## Acceptance tests

- Replacing a file/Resource used by a Component does not change that Component's identifier.
- Distinct Components of a similar kind may coexist.
- Filenames and display names are not authoritative Component identity.
- Component identity remains stable across storage-provider and DAW changes.

## Explicit non-goals

- Inventing a mandatory `kind` vocabulary, naming rules, rename/delete lifecycle, or DAW-to-Component mapping.
- Component State lineage or Project State composition.
- User-interface labels or adapter-specific grouping.

## Known Design Gaps

- DG-0010 blocks this package because the Core Specs do not normatively define the Creative Component object fields or the historical/mutable/out-of-object semantics of the conceptual `project_id`, `kind`, `name`, and `created_at` fields in Core §9.
- WORK-0006 parentage semantics are resolved by ADR-0003; hashed-map identity follows the resolved RFC 8785 rule in ADR-0005. These resolutions do not resolve DG-0010.

## Implementation plan

1. Define the stable Project-scoped Component identity relation from normative requirements.
2. Reuse assigned identifier types from WORK-0001.
3. Add negative tests against filename and Resource identity substitution.

## Verification requirements

The Verifier must check stable semantic identity independently of filename, physical storage, and adapter-specific entity names.

## Completion criteria

Formatting, focused tests, coverage-map update, independent verification, handover, and clean Git state.
