# DG-0012 — Project State schema and hash preimage

Status: RESOLVED
Classification: BLOCKS-FEATURE
Discovered by: OMVCS Lead
Discovered during: WORK-0007 revalidation
Date: 2026-10-08

## Relevant specifications

- `Specs/OMVCS Core Specification.md`, sections 5.1, 12–13, 56, and 76–77.
- `Specs/OMVCS Glossary.md`, Project State.
- `Specs/OMVCS Core Invariants Specification.md`, INV-HIST-003, INV-HIST-006, and INV-PROJ-004.
- ADR-0004, ADR-0005, ADR-0010, and ADR-0011.

## Problem

The specifications establish that a Project State is complete, references exactly one canonical Adapter State by its Adapter State Identifier, and represents Component membership as a JSON object map keyed by Creative Component Identifier with values referencing corresponding Component State Identifiers. They do not define the complete normative OMVCS 0.1 Project State body or its exact hash preimage.

The following details needed by WORK-0007 remain unspecified:

- the complete top-level field set, each field's requiredness, and whether unknown top-level fields are rejected;
- whether `project_id` is a required Project State field and whether it participates in the Project State hash;
- whether `project_metadata` is a normative field, whether it is required, and which versioned schema owns its value semantics and nested collection classifications;
- whether the Component State referenced by each `components` map value MUST identify the same Creative Component as the map key, and any related membership consistency constraints;
- the exact set of fields included in the canonical Project State hash preimage.

Section 13's JSON is explicitly a conceptual structure. It does not resolve these points. Section 5.1 classifies `components` and `project_metadata` collection shapes but does not define their complete object schema. The general canonicalization rules and section 76 schema-version requirement do not define a Project State member set or hash-field set. INV-HIST-003 requires one Adapter State reference but does not settle the rest of the envelope.

## Why the current specifications are insufficient

Choosing whether fields are absent, optional, or required; deciding whether Project identity or descriptive metadata changes Project State identity; accepting or rejecting extensions; and enforcing map-key/value identity consistency all affect historical validity and content-derived identity. These choices cannot be safely inferred from a conceptual example or from Component State rules, which are explicitly specific to that object.

## Affected work

- WORK-0007 Project State model and its conformance tests.
- WORK-0008 Revision model and tests that consume a valid Project State.
- Project State / Revision historical admission and identity validation in `crates/omvcs-model/`.

## Can unaffected work continue?

Yes. WORK-0001 through WORK-0006 remain unaffected and verified. WORK-0007 schema-dependent implementation and WORK-0008 integration against a valid Project State must not begin until this gap is resolved. Independent, non-semantic repository maintenance may continue.

## Candidate directions

None recorded. No schema direction is approved by this gap entry.

## Required decision

Approve the complete OMVCS 0.1 Project State schema: exact top-level fields and requiredness, extension policy, Project identity participation, `project_metadata` semantics/authority, Component State/map-key consistency rules, and the exact canonical hash preimage. Specify typed member constraints and historical admission behavior for each field.

## Resolution

Resolved by the human-approved decision recorded in [ADR-0012](../decisions/ADR-0012-project-state-schema-and-hash-preimage.md) and specified in Core §§5.1, 13, 56, and 76–77; the Project State Glossary entry; INV-HIST-003, INV-HIST-006, and INV-HIST-008; and the DAW Adapter Specification §20.

The OMVCS 0.1 Project State body is closed and contains exactly required `schema`, `project_id`, `components`, `adapter_state_id`, and `project_metadata`. All five fields participate in the canonical hash preimage. Project State schema availability and validation, typed Project identity, admitted Component State references with matching map-key identity, an admitted Adapter State reference, and exact-schema `project_metadata` validation are required for historical admission. Referenced metadata objects must be resolvable; underlying Resource bytes need not be materialised. No new generic Adapter State schema or cross-Project ownership semantics are introduced.
