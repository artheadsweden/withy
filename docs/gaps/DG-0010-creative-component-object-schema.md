# DG-0010 — Creative Component object schema and metadata semantics

Status: RESOLVED
Classification: BLOCKS-FEATURE
Discovered by: OMVCS Lead during WORK-0005 revalidation
Discovered during: WORK-0005 — Creative Component identity model
Date: 2026-10-08

## Relevant specifications

- `Specs/OMVCS Glossary.md`, Project, Creative Component, and Component State.
- `Specs/OMVCS Core Specification.md` §§4, 9–10, and 13.
- `Specs/OMVCS Core Invariants Specification.md`, INV-PROJ-001–003.
- `Specs/OMVCS DAW Adapter Specification.md` §§23–27.
- `Specs/OMVCS Interaction Specification.md` §§51–56.
- `Specs/OMVCS Platform Protocol.md` §31.

## Problem

Core §9 labels its example a “Conceptual Component record” and shows `component_id`, `project_id`, `kind`, `name`, and `created_at`. The surrounding normative text establishes that a Creative Component has a stable identifier, remains the same logical Component when its Resource material changes, and has Component States over time. It does not define the Component object's schema or make those example fields normative.

The specifications therefore do not determine:

- which fields, if any, belong to a Creative Component object besides its assigned identifier;
- whether and how a Component is associated with a Project as object data versus Project State membership;
- whether `kind`, `name`, or `created_at` are immutable historical fields, mutable descriptive metadata, Platform/UI metadata, or outside the M1 Component object;
- whether changes to such descriptive data belong to Component identity or any historical state.

The assigned Component Identifier is specified independently of Resource identity; it is not derived from Resource content, Component State parentage, DAW-native identifiers, or Project membership. DAW-native identifiers are used for Adapter mapping, not as OMVCS Component Identifiers. The specifications do not define Creative Component clone/fork semantics; WORK-0005 must not add them.

## Why the current specifications are insufficient

The example in Core §9 is explicitly conceptual. Treating its fields as a normative schema, or omitting some of them, would select an unapproved object model. The DAW Adapter's rename guidance (§27) says that a rename does not automatically create a new Component and SHOULD be treated as metadata evolution, but does not define where that metadata lives or whether it is historical. Interaction and Platform examples describe presentation and indexing, not the Core object schema. The existing invariants settle stable identity and separation from files, but not the field or metadata boundary.

Choosing a schema or assigning historical/mutable semantics would affect interoperable models and potentially canonical historical data; it cannot be inferred from examples.

## Affected work

- WORK-0005: Creative Component identity/model schema and its acceptance tests.
- WORK-0006 and WORK-0007: consume the identity/Project State membership boundary without expanding the generic Component object.
- WORK-0008: continues to identify a Project State; no direct Revision schema change is required.

## Can unaffected work continue?

Yes. Completed and independently verified WORK-0001 through WORK-0004 remain unaffected. The approved decision resolves the WORK-0005 blocker. The package may proceed under ADR-0010; downstream packages remain subject to their normal sequence and dependencies.

## Candidate directions

Before resolution, possible object-field and metadata placements were discussion material only and were not normative. The approved direction is recorded exclusively in ADR-0010 and the updated Specs.

## Required decision

Define the OMVCS 0.1 Creative Component object boundary: whether it has fields beyond its assigned `component_id`; how Project association is represented; and whether `kind`, `name`, `created_at`, or comparable descriptive values are historical Component fields, mutable descriptive/Platform/UI metadata, or outside the M1 Component object. Specify any resulting historical-change behavior. Do not infer clone/fork semantics unless that is separately and explicitly decided.

## Resolution

Resolved by the human-approved decision recorded in [ADR-0010](../decisions/ADR-0010-creative-component-identity-boundary.md) and implemented in the affected Specs:

- The generic OMVCS 0.1 Creative Component object contains only its required assigned `component_id`.
- Project association is represented through Project State membership/reference to Component and Component State identifiers; there is no Component `project_id`.
- `kind`, `name`, `created_at`, and comparable descriptive fields are excluded from the generic object and do not affect Component identity. Historical significance is determined only by an approved schema that places a value in Component State or Adapter State.
- Presentation renames do not change Component identity or automatically require a new Component State.
- This resolution defines no cross-Project reuse, copy, import, move, clone, fork, ownership, or identity-preservation semantics. These behaviors remain unspecified by design and must not be inferred.

Cross-Spec review found no unresolved contradiction requiring another Design Gap. Existing Project State membership and Adapter State mapping already express the relationships needed by the approved M1 model without selecting cross-Project lifecycle behavior.
