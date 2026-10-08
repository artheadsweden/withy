# DG-0009 — Resource Reference properties validation boundary

Status: OPEN
Classification: BLOCKS-FEATURE
Discovered by: Spec Guardian, assessing independent Verifier findings
Discovered during: WORK-0004 independent verification
Date: 2026-10-08

## Relevant specifications

- `Specs/OMVCS Glossary.md`, Resource, Resource Object, Resource Identifier, Resource Manifest, Friendly Name, and DAW Adapter entries.
- `Specs/OMVCS Core Specification.md` §§5–5.1, 6–7, 8.2, 10, 12–13, 56, and 76–77.
- `Specs/OMVCS Core Invariants Specification.md`, INV-HIST-005–006, INV-RES-001–007, INV-PROJ-003, and INV-DAW-004.
- `Specs/OMVCS DAW Adapter Specification.md` §§16 and 20–22.
- `Specs/Ardour Reference Adapter Design.md` §§15 and 31.
- `Specs/OMVCS Storage Adapter Specification.md` §§7 and 162.
- ADR-0001 and ADR-0005: recursive schema-directed collection normalization and duplicate object member rejection.
- ADR-0007: generic Resource Reference fields, interpretation-property ownership, exclusions, and historical identity participation.
- ADR-0008: the separately resolved byte-length domain.
- WORK-0004, including its schema/Adapter-supplied properties deliverable, exclusion acceptance tests, and prohibition on inventing format-specific metadata.

## Problem

Core §7 and ADR-0007 permit an OPTIONAL canonical JSON object map of immutable interpretation metadata explicitly supplied by the relevant schema or Adapter. They also forbid using `properties` to reintroduce logical/Friendly names, filenames, Chunk/Chunk Manifest information, Storage Endpoint/Location/Replica information, credentials, provider metadata, or other physical reconstruction/storage values.

The WORK-0004 model exposes `Option<BTreeMap<String, serde_json::Value>>` and an unrestricted `with_properties` constructor. Its exclusion comment and top-level unknown-field rejection do not validate property contents or establish that a relevant schema/Adapter supplied them. A map such as `{"filename":"mix.wav"}` can therefore enter the model under `properties`; serializing it into a historical body would make the excluded filename affect that body's identity.

The invalidity of that filename example is already specified. The new question is how the generic Core model establishes and enforces the schema/Adapter-approved property boundary while remaining ignorant of Adapter-specific interpretation semantics.

## Why the current specifications are insufficient

Assessment: **C — genuinely unspecified validation/admission boundary**, not a contradiction in the exclusion rule.

The complete eight-Spec search establishes what admissible properties must mean, but does not establish an executable property-admission contract for WORK-0004:

- No generic property vocabulary, key/value admissibility schema, or recursive semantic classification rule is supplied. Core §7's `sample_rate` and `channels` object is conceptual, not an exhaustive whitelist or property-validation schema.
- No rule identifies what evidence of applicable schema/Adapter approval the generic model accepts, which authority validates exclusions, or when a decoded map becomes a valid historical Resource Reference rather than unchecked input.
- Core §5.1 requires map key meanings and value normalization to come from their schemas. It also requires explicit ordering declarations for nested arrays; Core cannot infer them from JSON values.
- Core §§12 and 76, DAW Adapter §§20–22, and INV-DAW-004 require preservation/version-aware handling of Adapter-owned state without Core interpreting its internal meaning. They do not define a Resource Reference property-validation handoff. Adapter State's permission to contain native naming information does not extend to Resource Reference properties.
- Core §56 lists Resource Reference validation but supplies no additional property-validation procedure.

A `BTreeMap` establishes a map representation, not schema/Adapter provenance or semantic admissibility. The existing `MetadataSchema` mechanism can enforce declared JSON shape and collection normalization; a shape-compatible string called `filename` can still violate Core §7. Neither mechanism alone proves the required semantic boundary.

Rejecting a few reserved spellings could catch the reported filename but cannot establish the general prohibition: excluded information can occur under different keys or inside nested values. Selecting a complete denylist, recursive interpretation rules, a default whitelist, or the conceptual example as an exhaustive schema would invent semantics. Conversely, merely documenting a caller precondition would choose an unspecified validation/trust boundary; it does not make this unrestricted model an enforced valid-reference contract.

Omitting `properties` is valid for individual references because the field is optional. Rejecting or removing all non-empty properties is not, without approval, a complete implementation of WORK-0004's deliverable to represent schema/Adapter-supplied interpretation metadata. An unchecked transport representation could be useful, but cannot silently be treated as a validated historical reference. Multiple implementation mechanisms could eventually satisfy an approved boundary; this gap does not prescribe a Rust API.

DG-0007 remains resolved: ADR-0007 settled field inclusion and identity participation, not this validation handoff. DG-0008 remains resolved and unaffected.

### Specification and conformance impact

| Area | Current rule / impact of a decision |
|---|---|
| Glossary: Resource, Resource Object/Identifier, Resource Manifest, Friendly Name | Preserve the distinction between raw-byte identity, interpretation metadata, descriptive manifests, and presentation/native naming; no terminology change is presently justified. |
| Core §§5.1 and 7; INV-HIST-006, INV-RES-004 | Direct clarification needed for property admission/validation and schema-directed values, including nested collections. Existing exclusions and identity participation remain binding. |
| Core §§10, 12–13, 56, 76–77 | Embedded-reference validation, canonical historical bodies, validation timing, and applicable/unknown schema handling must remain consistent. |
| INV-HIST-005, INV-RES-003/005/006, INV-PROJ-003, INV-DAW-004 | Do not turn operational changes or renames into history changes, or require Core to interpret DAW-native semantics. |
| DAW Adapter §§16, 20–22 | Any selected schema/Adapter validation handoff must preserve the separation between capture/native naming and generic historical references. |
| Ardour §§15 and 31 | A reference implementation may need to supply the approved handoff; it must not define generic property semantics by precedent. |
| Storage Adapter §§7 and 162 | Secrets and physical reconstruction remain excluded; no Storage Adapter property authority is implied. |
| Interaction Specification and Platform Protocol | Searched for Resource Reference/Manifest, property, schema, interpretation, and validation rules; neither supplies the missing property-admission contract. No direct revision identified at this stage. |
| Tests and coverage | Property exclusion tests must cover construction and raw JSON ingestion, nested/aliased excluded content as governed by the decision, applicable/unknown schema handling, positive approved-property cases, and schema-directed array normalization. Preserve existing raw-byte identity and map-order tests. |

This is not solved by a local field-type/comment change alone. The approved decision needs at least a Core contract clarification; a schema/Adapter handoff may require coordinated DAW Adapter and reference-design updates. Exact revisions and executable test cases follow the human decision and ADR, not this assessment.

### Separate duplicate-map-name finding: implementation issue, not this gap

Assessment: **A — already specified** by Core §5.1, INV-HIST-006, and ADR-0005. Duplicate object member names MUST be rejected before hashing or canonical serialization, including objects nested within property values. No property interpretation semantics are needed to implement that rule.

An in-memory map's unique keys do not prove its raw JSON input was unique: deserializing directly into `BTreeMap`/`serde_json::Value` can discard earlier repeated members. This implementation issue was corrected within WORK-0004 by validating unique member names on raw `ResourceReference` JSON before decoding, including nested objects and escaped-equivalent names. `tests/resource_model.rs::resource_reference_deserialization_rejects_duplicate_member_names_recursively` exercises that path. ADR-0005's duplicate rejection remains a separate, implemented requirement and is not part of this open gap.

## Affected work

- WORK-0004: property-bearing Resource Reference construction/deserialization, validation status, historical serialization, exclusion conformance tests, and completion/verification claims in `docs/spec-coverage.md`.
- `crates/omvcs-model/src/resource.rs` and focused Resource model tests; integration with schema-directed canonicalization where properties enter containing historical bodies.
- WORK-0006: Component State embeds Resource References and hashes their present fields.
- WORK-0007: Adapter State Resource References and their contribution to Project State identity.
- WORK-0005 depends on completion of WORK-0004; it must not proceed on an assumption that this contract is verified. WORK-0008 transitively depends on the affected state models.
- M1 is the earliest affected milestone. Classification is BLOCKS-FEATURE because the missing rule is localized to property admission, not to already specified identifiers, raw-byte hashing, or canonical map rules.

## Can unaffected work continue?

Yes: previously verified WORK-0001–0003 remain unaffected. Required Resource fields, ADR-0008 byte-length enforcement, raw-byte Resource identity, references without properties, and the independently specified duplicate-name correction can be implemented without deciding this gap.

Stop implementing or declaring verified the disputed property-admission/validation contract. Do not approve or integrate WORK-0004 as complete while that deliverable is unresolved; do not begin WORK-0005 or later dependent packages on that basis. Existing `implemented` coverage wording is not independent verification and does not override this stop.

For this assessment task, stop after gap documentation: no production/test changes, no coverage/plan rewrites, and no WORK-0005+ work.

## Candidate directions

The following are discussion material only and are NOT approved decisions:

- Define a schema/Adapter validation handoff, with explicit authority and validation status, that Core requires before properties enter a valid historical reference.
- Define a versioned generic property-admission schema and extension rules with mechanically enforceable exclusions.
- Define the boundary between unchecked preservation and validated historical use, including behavior when the relevant schema/Adapter cannot validate the input.

No direction is selected. A validator callback or wrapper is an implementation technique, not by itself a definition of which authority and result satisfy the normative contract.

## Required decision

For OMVCS 0.1 Resource Reference properties:

1. Which authority establishes that a property map is applicable-schema/Adapter-supplied immutable interpretation metadata and contains no excluded values, including within nested values?
2. What schema context or validation evidence must the generic Core model receive, and at which boundary must it require validation: construction, decoding, admission to historical state, or another explicitly defined boundary?
3. What behavior is required when that context/authority is unavailable or unknown? May input be preserved as unchecked data, and what prevents its use as a valid historical reference?
4. How are property key meanings and nested collection classifications supplied without Core inventing or interpreting Adapter-specific semantics?
5. What positive/negative conformance cases establish that contract, and what bounded WORK-0004 implementation is authorized?

The decision must preserve ADR-0007's exclusions and field identity participation, ADR-0005's duplicate rejection, and ADR-0008's byte-length domain. No new property meanings are approved by this gap.

## Resolution

UNRESOLVED

Requires an explicit human decision, ADR, approved affected Spec updates, and corresponding test/work-package updates before the disputed feature resumes.
