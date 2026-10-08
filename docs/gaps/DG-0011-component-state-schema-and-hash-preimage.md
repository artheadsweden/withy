# DG-0011 — Component State schema and hash preimage

Status: OPEN
Classification: BLOCKS-MILESTONE
Discovered by: Spec Guardian
Discovered during: WORK-0006 preflight
Date: 2026-10-08

## Relevant specifications

Complete search performed across all eight files in `Specs/`:

- `Specs/OMVCS Core Specification.md` §§5–5.1, 7, 10–11, 12–13, 23, 56, and 76–77.
- `Specs/OMVCS Core Invariants Specification.md`, INV-HIST-001, INV-HIST-006, INV-RES-004, INV-RES-008, INV-PROJ-002, INV-PROV-003, and INV-DAW-004.
- `Specs/OMVCS Glossary.md`, Creative Component, Component State, and Project State.
- `Specs/OMVCS DAW Adapter Specification.md` §§20, 22, and 25–27.
- `Specs/Ardour Reference Adapter Design.md` §§21, 24, and 31.
- `Specs/OMVCS Interaction Specification.md` §§51 and 56.
- `Specs/OMVCS Platform Protocol.md` §§7 and 31–32.
- `Specs/OMVCS Storage Adapter Specification.md` §§7 and 162.

The remaining three Specs (Interaction, Platform Protocol, and Storage Adapter) contain no independent generic Component State schema definition. The Ardour reference design describes adapter-specific behavior, not a normative generic Core Component State member schema.

Relevant accepted decisions:

- ADR-0001, set-like ordering for Component State `parents` and `resources`.
- ADR-0003, optional parentage and omitted-versus-empty semantics.
- ADR-0005, RFC 8785 map handling and duplicate member rejection.
- ADR-0007 and ADR-0008, Resource Reference fields and `byte_length` domain.
- ADR-0009, exact-context validation and historical admission for property-bearing Resource References.
- ADR-0010, required one-field generic Creative Component object and separation from Component State.

## Problem

The current text resolves several Component State rules, but does not formally define the complete Component State schema/member set, the required-versus-optional status of all its fields, or a complete schema contract for its metadata and Resource Reference property contexts.

The following points are already resolved and are not part of this gap:

- Core §76 requires every canonical metadata object to identify its schema version. Core §10 says a Component State MUST identify its Creative Component by its typed `component_id`. These are required members.
- Core §§5 and 10 establish that the Component State Identifier hashes the canonical representation; fields present in the containing historical object contribute to its canonical body. ADR-0010 confirms `component_id` identifies the referenced Component, not an expansion of the generic Creative Component object.
- `parents` is optional, and ADR-0003 resolves omitted versus explicitly empty semantics.
- `parents` and `resources` are set-like arrays (Core §5.1 and ADR-0001).
- `metadata` is classified as a JSON object map keyed by metadata property name (Core §5.1), whose canonical member ordering and duplicate-name rejection follow RFC 8785 and ADR-0005.
- Resource Reference shape, `byte_length`, forbidden generic fields, and the admission rule for `properties` are governed by Core §7 and ADR-0007/0008/0009.

What remains unresolved:

1. Core §10 calls `resources` a set-like field and describes Resource Objects required to represent the state, but does not expressly state whether the field is required or optional. The §10 “Canonical conceptual structure” shows `schema`, `component_id`, `parents`, `resources`, and `metadata`, but does not purport to define a normative complete member set or settle the presence of every shown field. Core §5.1 classifies collection/map forms, not the complete Component State schema.
2. The Specs do not say whether `metadata` must be present, what value shapes or semantic domain it permits, or whether it is merely a generic Core map versus a map whose values and interpretation are owned by the applicable versioned Component State schema. RFC 8785 settles map canonicalization, not these schema/value semantics.
3. The exact Component State canonical member set/hash preimage is therefore not fixed beyond the generic canonical-object rule and the resolved individual constraints above. In particular, the status of additional schema-defined creative fields and the treatment of fields outside a closed schema are not stated.
4. Core §10 requires Resource Reference `properties` to be validated against the versioned schema context governing the Component State. ADR-0009 requires each versioned containing schema that permits properties to bind them to one exact validation authority/version. However, no complete Component State schema contract states which exact authority/version governs such properties for a Component State, or how that authority is identified when the meanings are schema-owned versus Adapter-supplied. Core §76's requirement to identify a schema version and ADR-0009's general context rule do not themselves define that mapping.

## Why the current specifications are insufficient

The §10 structure is explicitly labelled “Canonical conceptual structure,” not a normative schema declaration. Core §76 makes schema-version identification mandatory, and §10 makes `component_id` mandatory, but neither supplies the remaining member allowlist/presence rules or the metadata value contract. The word “field” and the collection classification for `resources` do not unambiguously say whether absence is invalid. The example's `metadata` value cannot establish whether arbitrary map values are permitted or whether a versioned schema owns their type and meaning.

Likewise, ADR-0009 resolves how property-bearing references are admitted and forbids choosing an unrelated or merely latest validator. It does not select or define the exact Component State schema/Adapter authority binding. Guessing the field set, optionality, metadata value semantics, extension behavior, or property-validation authority would determine canonical historical object shape and identity and could make independently implemented Component States incompatible. Treating an illustrative example as normative would invent those semantics.

## Affected work

- WORK-0006: Component State schema, validation, Resource Reference admission context, and canonical identity are blocked.
- WORK-0007: depends on WORK-0006 and cannot complete Project State modeling against a stable Component State reference contract.
- WORK-0008: depends on WORK-0007 and is consequently blocked downstream.
- M1: the normative data-model milestone includes Component State, Project State, and Revision; the dependency chain prevents completion of the milestone while this contract remains unresolved.

Previously completed WORK-0001 through WORK-0005 are unaffected.

## Can unaffected work continue?

Yes, but no implementation or conformance claim may choose the missing Component State schema semantics. Independent work not depending on the WORK-0006 → WORK-0007 → WORK-0008 chain may continue. WORK-0006, WORK-0007, and WORK-0008 must remain blocked pending the decision and approved ADR/Spec update.

## Candidate directions

None recorded. No candidate schema or metadata policy is selected by this gap entry.

## Required decision

The human decision must define:

1. The complete OMVCS 0.1 Component State member set and each member's required/optional status, including the already-resolved mandatory `schema` and `component_id`, optional `parents`, and the unresolved presence rules for `resources` and `metadata`; state whether the schema is closed or permits a defined extension mechanism.
2. The `metadata` map's permitted value shapes and semantic authority: whether Core treats it only as a canonical generic map or the applicable versioned Component State schema owns its values/meaning, and how that context is identified.
3. The exact canonical Component State body/hash field set, including whether any schema-defined creative fields are admitted and how non-schema fields are handled.
4. The exact versioned validation authority for Resource Reference `properties` in Component State use and how it is determined from the containing Component State schema and, where applicable, Adapter contract, while preserving ADR-0009's rule against an independent Resource Reference property-schema identifier.

The decision must preserve the already-resolved `component_id`, parentage, collection, map-canonicalization, Resource Reference, and admission rules unless the human explicitly reopens them.

## Resolution

UNRESOLVED

When resolved, link the approved ADR and resulting specification changes here.
