# WORK-0007 — Project State model

Status: VERIFIED — independent Verifier review accepted (2026-10-09)
Owner agent: Core Engineer
Milestone: M1
Branch: `work/0007-project-state`

## Objective

Represent one complete immutable logical Project State using stable Project identity and immutable Component State references, while preserving the DAW-independent boundary.

## Normative requirements

- Glossary: Project State.
- Core Specification, sections 5.1, 7, 12–13, 24, 56, and 76–77.
- Core Invariants: INV-HIST-003, INV-HIST-006, INV-HIST-008, INV-RES-004, INV-RES-008, INV-PROJ-004, INV-DAW-004.

## Dependencies

- WORK-0001 through WORK-0006. WORK-0001 supplies typed Project, Component, Component State, Adapter State, and Project State identifiers; WORK-0002/0003 supply canonical serialization and hashing; WORK-0004/0006 supply the admitted Resource Reference and Component State boundaries; WORK-0005 supplies the Creative Component identity boundary.
- ADR-0004 resolves the Adapter State representation: Project State references exactly one canonical Adapter State metadata object, which may reference opaque native-state Resources.
- ADR-0007 defines the Resource Reference fields used by Adapter State resource entries and excludes names and physical storage/reconstruction data from those references.
- ADR-0008 defines the exact accepted integer range for each Resource Reference `byte_length`.
- ADR-0009 defines the versioned schema/Adapter validation authority and historical admission boundary for Resource Reference `properties`.
- Hashed JSON object maps follow RFC 8785 member ordering only, and duplicate member names are rejected before hashing/canonical serialization (ADR-0005).
- ADR-0010 defines Project State membership/reference as the Project-to-Component association; do not add a `project_id` back-reference or infer cross-Project ownership/reuse behavior.
- ADR-0011 defines the closed Component State historical body and valid identity that Project State component references consume.
- ADR-0012 defines the closed Project State body, admission rules, and exact hash preimage.
- DEC-PLATFORM-016 remains open. Do not add licensing fields to the M1 Project State model unless its ownership/location is decided first.
- DG-0012 is resolved by ADR-0012; implement only its approved closed Project State contract.

## Allowed scope

- `crates/omvcs-model/`
- Focused Project State validation/identity tests and conformance vectors explicitly permitted by this package.

## Deliverables

- Complete Project State references, not a change list.
- Storage-independent Project and Component State references.
- Typed reference to exactly one valid/admitted canonical Adapter State metadata object through its `AdapterStateId`; Adapter State internals remain owned by Core §12 and the exact Adapter schema.
- A trusted `AdmittedAdapterStateResolver` consumption boundary for the already-established exact Core/Adapter admission result; this model neither parses nor defines Adapter State internals.

## Acceptance tests

- A Project State identifies a complete logical state, not merely a delta.
- The closed OMVCS 0.1 body contains exactly required `schema`, `project_id`, `components`, `adapter_state_id`, and `project_metadata`; missing required members and any unknown top-level member are rejected.
- Exact Project State schema is known and available before admission; unknown/unavailable schema candidates cannot produce a valid Project State or Project State Identifier.
- `project_id` is the typed assigned Project Identifier and participates in identity; otherwise identical bodies with different Project IDs have different canonical bytes.
- `components` is a required Creative-Component-ID-to-Component-State-ID map, accepts an empty map, rejects omission, validates canonical typed keys and typed values, rejects duplicate raw keys, and is invariant under member insertion-order permutations.
- Every referenced Component State is resolvable and valid/admitted, and its `component_id` matches the map key; reject mismatch. Missing Resource bytes under a valid Component State do not by themselves prevent Project State admission.
- `adapter_state_id` is a required typed Adapter State Identifier; the referenced canonical Adapter State is resolvable and valid/admitted. Native Resource IDs, unchecked Adapter State candidates, or unavailable references cannot satisfy it. Missing Resource bytes beneath otherwise valid Adapter State do not by themselves prevent Project State admission.
- `project_metadata` is required, accepts an empty map, rejects omission, and validates keys, values, nested schemas, and ordered/set-like nested arrays under the exact Project State schema; reject unknown/disallowed keys, invalid shapes, and unclassified arrays.
- Identical canonical five-member bodies produce identical Project State IDs. Changing `schema`, `project_id`, component membership or state ID, `adapter_state_id`, or any present canonical `project_metadata` value changes canonical identity.
- Component-map and metadata-map insertion order alone does not change identity; duplicate raw member names are rejected under ADR-0005.
- Canonical bytes contain only the exact five-member historical body; operational, presentation, storage, validation, wrapper, signature, credential, Platform, and Resource Replica fields are excluded.
- Project identity and Component State references are explicit and stable under the approved Project State schema.
- Project State membership determines which Creative Components and corresponding Component States participate in that historical Project State; the Creative Component object itself has no `project_id`.
- Storage locations, platform URLs, local paths, and availability do not enter Project State identity.
- Different creative-object references yield different Project State identities.
- Component-map and project-metadata map insertion-order permutations yield identical canonical bytes/identities; duplicate member names are rejected; no additional entry sorting is applied.
- The model consumes a valid/admitted Adapter State reference at the Core §12 / DAW Adapter §20 boundary; it does not define or implement a generic Adapter State schema, Adapter State hash body, or Adapter State `component_bindings` model.

## Explicit non-goals

- Working State, materialisation, DAW restoration, Resource retrieval, or storage maps.
- Inferring missing component states or choosing defaults not defined by the Specs.
- Adapter State schema/model implementation, Adapter-specific metadata or Resource Reference property validation, and Adapter State `component_bindings`.
- DAW-specific fields in the Core model.

## Known Design Gaps

- DG-0006, DG-0007, DG-0008, and DG-0009 are resolved by ADR-0006, ADR-0007, ADR-0008, and ADR-0009; apply the generic binding-map, Resource Reference, and property-admission contracts consistently.
- DG-0012 is resolved by ADR-0012. No additional generic Adapter State body or cross-Project ownership semantics were introduced.

## Implementation plan

1. Apply the resolved Component State and Adapter State reference contracts.
2. Implement the closed complete-state model and schema-owned metadata admission without adding adapter/provider fields.
3. Derive identity from exactly the five canonical historical members and test every acceptance criterion.

## Verification requirements

The Verifier must compare Project State references and invariants to the original sections and attempt invalid/partial-state cases without assuming an unspecified default.

## Completion criteria

Formatting, focused and full model tests, coverage-map update, independent Verifier review, and handover. Commit/integration/push are not part of this package execution.
