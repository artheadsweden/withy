# ADR-0012 — Project State schema and hash preimage

Status: ACCEPTED
Date: 2026-10-09
Decision owner: Human
Related Design Gap: DG-0012

## Context

Core Specification §13 described a Project State using a conceptual example but did not define the complete OMVCS 0.1 top-level field set, requiredness, extension policy, Project identity participation, metadata authority, Component State/map-key consistency, or exact hash preimage. Those choices determine historical admission and content-derived identity.

## Decision

1. The generic OMVCS 0.1 Project State historical body is a closed JSON object containing exactly these REQUIRED members: `schema`, `project_id`, `components`, `adapter_state_id`, and `project_metadata`. No additional top-level members are permitted.
2. `schema` uses the canonical metadata schema-version representation defined by Core §76. Its exact versioned Project State schema MUST be known and available and MUST validate the candidate before historical admission. Unknown or unavailable schema candidates MAY be preserved as unchecked data where supported, but MUST NOT produce a valid Project State or Project State Identifier. `schema` participates in identity.
3. `project_id` MUST be the typed assigned Project Identifier of the Project represented. It participates in identity. Project State identity is Project-specific; this identifier is not derived from component membership, Adapter State, Resource content, storage, Platform identity, or location.
4. `components` is a REQUIRED JSON object map from canonical typed Creative Component Identifier text to typed Component State Identifier. It MAY be empty; omission is invalid. Map ordering and duplicate-member rejection follow ADR-0005; no set-like array sorting applies. Every referenced Component State MUST be valid/admitted, resolvable as metadata, and have its required `component_id` equal to the map key. A mismatch prevents Project State admission. The referenced Component State's Resource bytes need not be locally materialised. This rule does not define clone, copy, import, fork, move, ownership, or cross-Project identity-preservation semantics.
5. `adapter_state_id` is REQUIRED and MUST be a typed Adapter State Identifier referencing exactly one resolvable, valid/admitted canonical Adapter State metadata object under Core §12 and the exact applicable Adapter State schema. A native Resource Identifier is not a valid complete Adapter State reference. Resource bytes beneath an otherwise valid Adapter State need not be locally materialised. Adapter runtime state, validation evidence, installation state, local DAW state, and operational Adapter metadata are not separately embedded in Project State.
6. `project_metadata` is a REQUIRED JSON object map and MAY be empty; omission is invalid. The exact versioned Project State schema owns its permitted keys, required and optional keys, value shapes/types, semantic meanings, nested object schemas, and nested array classifications. Core enforces generic structural/canonical rules and MUST NOT infer metadata semantics. Unknown keys, invalid shapes, or unclassified nested arrays prevent admission unless that exact schema explicitly permits them. Project metadata is for schema-approved historical Project-level state, not a generic operational/presentation container. Presentation/UI metadata, local paths, storage/Replica information, credentials, Platform indexing/account information, validator evidence, timestamps, and other operational data MUST NOT enter merely as generic metadata. A value may enter only where an approved Project State schema explicitly defines it as historical Project state without conflicting with another Core invariant or ADR.
7. A Project State may be admitted only after its exact schema validates; `project_id` is valid; component keys and values are correctly typed; each referenced Component State is admitted and has a matching `component_id`; the referenced Adapter State is valid/admitted; `project_metadata` validates under the exact schema; and no unknown top-level member is present. Failure prevents valid historical admission and a valid Project State Identifier.
8. The Project State Identifier is SHA-256 over the canonical Project State historical-body bytes under WORK-0002/WORK-0003. No type/domain prefix is included in the digest input. The canonical body contains exactly `schema`, `project_id`, `components`, `adapter_state_id`, and `project_metadata`; all five members participate in identity. Map insertion order alone does not affect identity.
9. Validation evidence, validator implementations, timestamps, presentation/UI metadata, local DAW state, storage information, Resource Replica state, transport wrappers, signatures, credentials, Platform metadata, and unknown extension fields MUST NOT enter the hash preimage.
10. OMVCS 0.1 has no arbitrary top-level Project State extension mechanism. New top-level members require an explicit future schema/version and compatibility decision. Versioned Project-specific historical semantics belong in schema-approved `project_metadata`, Component membership/state, or the referenced Adapter State as appropriate.

## Rationale

The fixed body gives every implementation the same Project-specific identity and complete-state reference envelope while retaining schema-owned Project metadata semantics. Requiring resolvable, admitted historical metadata references prevents invalid or mismatched Component State and Adapter State objects from entering a valid Project State, without confusing metadata completeness with local Resource-byte availability.

## Alternatives considered

- Treat the §13 example as an incomplete or extensible schema: rejected; it would leave requiredness, identity, and compatibility behavior implementation-dependent.
- Permit Project State fields outside the declared body: rejected for OMVCS 0.1; an extension changes the canonical body and requires a future versioned decision.
- Accept a Component State under a map key different from its own `component_id`: rejected; it would make membership internally inconsistent.
- Require referenced Resource bytes to be locally materialised: rejected; metadata validity and Resource availability are separate.
- Define a new generic Adapter State body or duplicate Adapter validation evidence in Project State: not selected. Adapter State validity remains governed by Core §12 and its exact Adapter schema under the existing Adapter-owned validation boundary.

## Specification impact

- `Specs/OMVCS Core Specification.md` §§5.1, 13, 56, 76–77.
- `Specs/OMVCS Core Specification.md` §53 clarifies that Reference Render publication does not add a Project State member; detailed render policy remains DEC-CORE-002.
- `Specs/OMVCS Glossary.md`, Project State.
- `Specs/OMVCS Core Invariants Specification.md`, INV-HIST-003, INV-HIST-006, and new INV-HIST-008.
- `Specs/OMVCS DAW Adapter Specification.md` §20 for required `adapter_state_id` resolution/admission in a Project State.
- `Specs/Ardour Reference Adapter Design.md` §190 clarifies that the publication illustration does not add Reference Render or other direct fields to the closed Project State body.
- No change is required in the Interaction, Platform Protocol, or Storage Adapter examples: review found no example that defines or conflicts with the Project State historical-body schema. Adapter operation payloads using an `adapter_state` member are not Project State bodies.

## Test impact

WORK-0007 MUST test all five required members, valid empty `components` and `project_metadata`, omitted-field and unknown-top-level rejection, typed identities, exact schema availability, Component State admission and key/state identity consistency, Adapter State admission, metadata schema-directed validation and nested collection normalization, map insertion-order invariance and duplicate raw-key rejection, exact canonical bytes/hash, identity changes for every canonical member, and exclusion of operational/wrapper fields. Project State tests MUST confirm metadata-object resolvability without requiring local Resource-byte materialisation.

WORK-0008 MUST consume exactly one valid/admitted Project State reference and retain its own Revision identity and ancestry conformance requirements.

## Implementation impact

- WORK-0007 implements the Project State model, validation, identity, and conformance tests against this closed contract.
- WORK-0008 depends on the admitted Project State type and identity contract.
- No production implementation is included in this ADR change.

## Compatibility / migration impact

No Project State implementation or production history is known in the bootstrap workspace. The previous §13 example was conceptual and used `adapter_state`; the normative OMVCS 0.1 member is `adapter_state_id`. Any existing drafts using other field sets are not valid OMVCS 0.1 Project State bodies.

## Notes

No semantic decisions were made beyond the human-approved decision recorded in this ADR. No additional Adapter State schema or cross-Project ownership semantics were selected.
