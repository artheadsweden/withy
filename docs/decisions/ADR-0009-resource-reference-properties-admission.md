# ADR-0009 — Resource Reference properties validation and historical admission

Status: ACCEPTED
Date: 2026-10-08
Decision owner: Human
Related Design Gap: DG-0009

## Context

ADR-0007 defines Resource Reference `properties` as optional immutable interpretation metadata supplied by a relevant schema or Adapter, requires present fields to participate in the containing historical object's identity, and prohibits properties from reintroducing presentation, naming, storage, credential, provider, or physical reconstruction information. The Specs did not define how Core distinguishes an unchecked decoded candidate from a Resource Reference valid for historical use, which authority validates property meanings, or what to do when the applicable context is unknown.

## Decision

1. OMVCS 0.1 distinguishes decoding or preservation of a Resource Reference candidate from admission of a valid Resource Reference into historical state.
2. A Resource Reference with no `properties` MAY be validated using the generic Core Resource Reference rules alone.
3. A Resource Reference with `properties` present MUST NOT be admitted into a valid historical object until its property map has passed validation under the exact applicable versioned schema or Adapter context governing that containing use. Only successfully validated Resource References may participate as valid references in Component State, Adapter State, Project State, Revision history, or another historical object.
4. Core owns generic structure and canonical validation, including generic Resource Reference field rules, duplicate JSON object-member rejection, canonical JSON requirements, schema-directed value shape, and explicit nested array classification and normalization.
5. The applicable versioned schema or Adapter authority owns semantic admissibility. It MUST define valid property meanings and structures for its context and MUST NOT admit logical/Friendly names, filenames, Chunk or Chunk Manifest information, Storage Endpoint/Location/Replica information, credentials, provider metadata, or other physical reconstruction/storage information prohibited by ADR-0007.
6. Core MUST NOT infer semantic admissibility from property key spelling, deny lists, heuristics, or DAW-specific knowledge. The applicable schema/Adapter contract MUST supply all value-shape and nested collection classification information required by the Core canonicalization contract.
7. Validation MUST use the exact applicable schema/Adapter context and version governing the containing historical use. Every versioned containing schema that permits `properties` MUST bind them to one exact validation authority/version, determinable from that schema/Adapter contract. If the authority cannot be uniquely determined, the candidate remains unchecked and cannot be admitted as valid history. A Resource Reference MUST NOT introduce an independent property-schema identifier unless a future approved decision determines that the existing containing context cannot identify the validator unambiguously. Validation against an unrelated, unknown, or merely latest schema/Adapter version is insufficient.
8. Implementations MAY preserve or transport a candidate whose applicable validator or context is unknown or unavailable, but it MUST remain explicitly unchecked. It MUST NOT be admitted into valid historical state, used to produce a valid containing historical object or valid historical object identity, or committed into valid OMVCS history until validation succeeds under the applicable context. Preservation does not imply semantic validity.
9. Validation status, validator implementation details, callbacks, timestamps, signatures, and other validation evidence are operational concerns. They MUST NOT become Resource Reference historical fields or affect canonical historical identity. After successful validation, the property values themselves remain in the containing object's canonical body and contribute to its identity as required by ADR-0007.
10. The exact Rust API/type mechanism remains an implementation decision. WORK-0004 MAY implement distinct unchecked-candidate and validated-for-admission representations, or an equivalent boundary, provided that accidental admission of an unchecked property-bearing candidate is impossible or explicitly fallible. An unrestricted property-map constructor alone MUST NOT establish a valid historical Resource Reference.

The exact applicable context is identified through the versioned schema/Adapter contract of the containing use: canonical metadata objects identify their schema version under Core §76; Adapter State additionally identifies its `adapter_id` and `adapter_state_schema`. Where an Adapter-defined property context is used outside Adapter State, the containing schema/Adapter contract MUST bind it unambiguously without adding a field to Resource Reference. An ambiguous or missing binding is an unchecked candidate, not permission to choose a validator or invent an identifier.

## Rationale

This separates generic historical validity mechanics from interpretation semantics. Core can enforce the declared JSON shape, duplicate handling, canonicalization, and collection rules without claiming knowledge of what Adapter-owned values mean. The schema/Adapter authority that owns those meanings is responsible for approving only immutable interpretation metadata within the ADR-0007 exclusions. Preserving unchecked data supports lossless transport without treating unknown semantics as valid history.

## Alternatives considered

- Treat decoding or an unrestricted constructor as validation: rejected because it cannot establish schema/Adapter provenance or semantic admissibility.
- Have Core infer exclusions from key spellings, deny lists, or heuristics: rejected because forbidden information can be expressed under other names or nested values, and such checks would invent generic interpretation semantics.
- Define a generic Core property vocabulary: not selected; the approved rule assigns semantic authority to the applicable versioned schema or Adapter.
- Add validation status, evidence, or an independent property-schema identifier to each Resource Reference: rejected for OMVCS 0.1. Status/evidence is operational; the existing containing schema/Adapter context identifies the applicable validator.
- Preserve unknown property-bearing candidates as valid history: rejected because unknown or unavailable validation cannot establish semantic admissibility.

## Specification impact

- `Specs/OMVCS Glossary.md`: distinguish a preserved candidate from a Resource Reference validated for historical admission.
- `Specs/OMVCS Core Specification.md` §§5.1, 7, 10, 12–13, 56, and 76–77: establish the Core/schema validation split, required historical admission boundary, unknown-context handling, and schema-context association.
- `Specs/OMVCS Core Invariants Specification.md` INV-RES-004, INV-RES-008, and INV-DAW-004: require applicable-context validation and preserve the Core/Adapter semantic boundary.
- `Specs/OMVCS DAW Adapter Specification.md` §§20 and 22: require versioned Adapter property contracts and validation before valid historical admission.
- `Specs/Ardour Reference Adapter Design.md` §31: state the reference Adapter's responsibility without defining generic property meanings.

No changes are required to the Interaction, Platform Protocol, or Storage Adapter Specifications: the full eight-Spec search found no additional Resource Reference properties admission rule or contradiction in those documents. ADR-0005 duplicate-member rejection, ADR-0007 exclusions, and ADR-0008 byte-length range remain unchanged.

## Test impact

WORK-0004 and dependent conformance suites MUST cover:

- a Resource Reference without `properties` can pass generic Core validation;
- presence of `properties`, including an empty map, requires successful validation under its applicable context;
- a property-bearing candidate is admitted when the exact applicable context accepts it;
- semantic rejection by the applicable schema/Adapter prevents historical admission and propagates as admission failure;
- unknown or unavailable context cannot produce a valid historical Resource Reference or containing historical object;
- a non-unique applicable authority also leaves the candidate unchecked and cannot produce a valid historical object;
- unchecked candidate preservation, if supported, cannot enter valid historical state or be committed;
- Core does not infer semantic validity from property names;
- all applicable schema value shapes and nested ordered/set-like classifications are applied; unclassified arrays are rejected;
- duplicate object names, including nested and escaped-equivalent names, are rejected before decoding/canonicalization;
- property-map insertion-order changes do not affect identity after successful validation;
- validation status/evidence is absent from canonical historical bytes;
- Adapter-specific conformance tests reject ADR-0007-excluded semantics, including when represented under alternate keys or nested values.

Existing raw-byte Resource identity, byte-length, and generic map canonicalization vectors remain required.

## Implementation impact

- WORK-0004 may resume implementing the explicit unchecked-to-validated historical admission boundary and must update focused tests and handover; the implementation-specific Rust arrangement is not prescribed.
- WORK-0006 and WORK-0007 MUST apply the same admission rule to embedded Resource References in Component State and Adapter State.
- DAW Adapter conformance suites and the Ardour reference implementation are responsible for their own versioned property semantics.
- No production code is part of this specification/ADR task.

## Compatibility / migration impact

No production objects or implementations are known in the integration branch. An imported or preserved property-bearing candidate with unknown validation context is not thereby invalid as transport data, but it cannot be admitted as valid OMVCS 0.1 history until checked under the applicable context. Validation evidence is not added to historical bytes, so successful validation does not independently alter existing Resource Reference identity.

## Notes

The containing context is sufficient under the existing model: canonical metadata objects identify their schema version (Core §76), and Adapter State identifies `adapter_id` and `adapter_state_schema` (Core §12 and DAW Adapter §20). This decision does not add an independent schema/version field to Resource Reference. If a future concrete schema cannot identify its applicable validator through its containing schema/Adapter contract, that specific issue requires a new Design Gap and approved decision.
