# DG-0009 — Resource Reference properties validation boundary

Status: RESOLVED
Classification: BLOCKS-FEATURE
Discovered by: Spec Guardian, assessing independent Verifier findings
Discovered during: WORK-0004 independent verification
Date: 2026-10-08

## Relevant specifications

- `Specs/OMVCS Glossary.md`, Resource, Resource Object, Resource Identifier, Resource Manifest, and Friendly Name.
- `Specs/OMVCS Core Specification.md` §§5–5.1, 6–7, 8.2, 10, 12–13, 56, and 76–77.
- `Specs/OMVCS Core Invariants Specification.md`, INV-HIST-005–006, INV-RES-001–008, INV-PROJ-003, and INV-DAW-004.
- `Specs/OMVCS DAW Adapter Specification.md` §§16, 20–22, and 102.
- `Specs/Ardour Reference Adapter Design.md` §§15 and 31.
- ADR-0001 and ADR-0005 for collection normalization and duplicate member rejection; ADR-0007 for Resource Reference fields and exclusions; ADR-0008 for `byte_length`.

## Problem

The specifications defined Resource Reference `properties` as immutable interpretation metadata supplied by a relevant schema or Adapter and prohibited using them to encode excluded presentation, naming, storage, credential, provider, or physical reconstruction information. They did not define how a decoded property-bearing candidate becomes valid for historical use, which authority validates its semantics, or how an implementation must behave if that authority or version is unknown or unavailable.

## Why the original specifications were insufficient

Generic Core parsing and canonicalization can enforce JSON shape, duplicate member rejection, schema-declared structure, and collection normalization, but cannot infer whether arbitrary nested values represent permitted interpretation metadata or prohibited operational/presentation data. A map type, unrestricted constructor, key-name deny list, or heuristic would not establish the required semantic authority.

## Affected work

- WORK-0004: Resource Reference validation and historical admission.
- WORK-0006 and WORK-0007: embedded Resource References.
- DAW Adapter schemas and their conformance suites, including the Ardour reference design.

## Resolution

Resolved by the human-approved decision recorded in [ADR-0009](../decisions/ADR-0009-resource-reference-properties-admission.md) and implemented normatively in the affected Specs:

- Distinguish decoded/preserved candidates from Resource References validated for historical admission.
- A reference without `properties` may use generic Core validation alone. A reference with `properties` must pass Core's generic structural/canonical validation and semantic validation under the exact applicable versioned schema/Adapter context before entering valid historical state.
- Core owns structural/canonical mechanics. The applicable schema/Adapter authority owns property meanings and semantic exclusion of ADR-0007-forbidden information; Core must not infer semantics from property names or heuristics.
- The containing schema/Adapter contract must identify the exact applicable context. Existing versioned historical schema context and Adapter identity/schema version are used; no independent Resource Reference property-schema identifier is introduced.
- Unknown, unavailable, or non-unique context may permit preservation as explicitly unchecked data, but such data cannot be admitted, used to create a valid historical identity, or committed as valid OMVCS history.
- Validation status/evidence remains operational and does not affect canonical historical bytes. Present property values continue to participate in identity after successful validation.
- Applicable schemas supply value shapes and nested array classifications; duplicate-member rejection remains independently required under ADR-0005.

The existing context is sufficient as specified: canonical metadata objects identify their schema version (Core §76), while Adapter State identifies `adapter_id` and `adapter_state_schema` (Core §12 and DAW Adapter §20). A future concrete schema that cannot determine its validator from its containing schema/Adapter contract requires a new Design Gap and approved decision; it must not silently add a Resource Reference field.
