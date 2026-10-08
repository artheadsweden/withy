# ADR-0011 — Component State schema and canonical hash preimage

Status: ACCEPTED
Date: 2026-10-08
Decision owner: Human
Related Design Gap: DG-0011

## Context

Core Specification §10 required a Component State to identify its Creative Component, classified `parents` and `resources`, and showed a conceptual body containing `schema`, `component_id`, `parents`, `resources`, and `metadata`. It did not define the full 0.1 member allowlist, field requiredness, metadata value authority, or a complete hash preimage. ADR-0009 also requires an exact validation authority for property-bearing Resource References but does not itself define the Component State schema's binding.

Those omissions could cause implementations to admit different Component State bodies or apply different property validators while claiming the same schema version.

## Decision

1. The closed OMVCS 0.1 Component State historical body is a JSON object containing exactly these permitted top-level members:
   - `schema` — REQUIRED;
   - `component_id` — REQUIRED;
   - `parents` — OPTIONAL;
   - `resources` — REQUIRED;
   - `metadata` — REQUIRED.
   No additional top-level members are permitted.
2. Existing Core schema-version representation rules remain in force. This ADR does not define a second schema identifier.
3. `component_id` is the required typed Creative Component Identifier whose state the object represents. It participates in the canonical body and Component State identity, and refers to the identity anchor defined by ADR-0010 without expanding that object.
4. `parents` remains optional under ADR-0003. Omission means unknown or unasserted lineage; an explicitly empty array means known zero-parent initial state. When present, `parents` is set-like under ADR-0001.
5. `resources` is a required set-like JSON array of historically admitted Resource References. It MAY be empty; omission is invalid. Each entry MUST satisfy ADR-0007, ADR-0008, and ADR-0009 before the Component State is admitted as valid history. Unchecked Resource Reference candidates MUST NOT enter a valid Component State.
6. `metadata` is a required JSON object map and MAY be empty. Its permitted keys, required/optional keys, value shapes and types, meanings, nested object schemas, and nested array classifications are owned by the exact versioned Component State schema identified by `schema`. Core enforces generic structural and canonical rules but MUST NOT infer or invent metadata semantics. Unpermitted keys, invalid shapes, or unclassified nested arrays prevent admission unless the applicable versioned schema explicitly permits them.
7. A Component State candidate MUST NOT be admitted as valid historical state unless its exact `schema` version is known, available, and validates the body. Unknown or unavailable schema candidates MAY be preserved outside valid history, but preservation does not establish validity or a valid Component State Identifier.
8. Every versioned Component State schema that permits Resource References with `properties` MUST deterministically bind each permitted property-bearing context to exactly one validation authority and version, as required by ADR-0009. The authority MAY be the Component State schema itself or an explicitly bound versioned Adapter/schema authority. The binding MUST be determinable from the Component State schema/Adapter contract and existing context. No independent property-schema field is added to Resource Reference. Authority selection MUST NOT use a latest-version preference, installed-implementation preference, property-key heuristics, or ambiguous validator selection. Without one unique applicable authority, the reference remains unchecked and the Component State MUST NOT be admitted as valid history. A Resource Reference without `properties` continues to require only generic Core validation under ADR-0009.
9. The Component State Identifier is derived using the existing WORK-0002/WORK-0003 rules: SHA-256 over the canonical serialized Component State historical-body bytes, with no type/domain prefix in the digest input.
10. The canonical historical body contains exactly `schema`, `component_id`, `resources`, `metadata`, and `parents` only when `parents` is present. Every present member participates in identity; `schema` participates in identity.
11. Validation evidence, validator identity, callbacks, timestamps, presentation metadata, storage information, transport wrappers, signatures, credentials, Platform metadata, and unknown extension fields MUST NOT enter the hash preimage.
12. OMVCS 0.1 Component State has no arbitrary top-level extension mechanism. Schema-specific creative semantics MUST be expressed through the schema-owned `metadata` map and the already-defined `resources` Resource References. A future addition of a top-level member requires an explicit versioned specification and compatibility decision.

## Rationale

A closed historical envelope fixes the canonical preimage while leaving creative metadata meanings to the exact versioned Component State schema. Requiring `resources` and `metadata` with empty values permitted gives each body one representation for an empty collection/map. Binding property validation to one exact authority preserves ADR-0009's unchecked-to-admitted boundary without adding schema identity to Resource Reference.

## Alternatives considered

- Treat the §10 conceptual example as an incomplete normative schema: rejected because that would leave field presence, allowed members, and hash identity implementation-dependent.
- Permit arbitrary top-level extensions in 0.1: rejected because unrecognized fields would alter or be omitted from canonical historical identity inconsistently.
- Allow optional `resources` or `metadata`: rejected; their absence would create additional representations whose semantics were not approved.
- Permit validator selection by installed/latest version or property names: rejected because it would violate exact-context admission and could make validity implementation-dependent.

## Specification impact

- `Specs/OMVCS Core Specification.md` §§5.1, 10–11, 56, 76–77: define the closed envelope, requiredness, schema-owned metadata, exact hash body, validation admission, and evolution rules.
- `Specs/OMVCS Glossary.md`, Component State: summarize the 0.1 historical body and identity boundary.
- `Specs/OMVCS Core Invariants Specification.md`, INV-HIST-006 and INV-RES-008: require closed deterministic Component State identity and exact property-validator binding.
- `Specs/OMVCS DAW Adapter Specification.md` §§20 and 27: align Adapter-supplied property authority and historical Component State metadata boundaries.
- WORK-0006 defines direct schema conformance; WORK-0007 and WORK-0008 consume the resulting Component State identity and reference contract.
- Ardour Reference Adapter Design has no conflicting generic Component State body example; its existing adapter-specific mapping/name guidance remains subject to the generic Core contract.

## Test impact

WORK-0006 MUST cover required `schema`, typed `component_id`, `resources`, and `metadata`; optional parent omission versus explicit empty; valid empty `resources` and `metadata`; required-field and unknown-top-level rejection; set-like parent/resource permutation invariance and duplicate rejection; schema-directed metadata validation, nested collection normalization, and unclassified-array rejection; exact-schema availability and property-authority admission; canonical hash stability and identity changes for changed present body fields including `schema`; and exclusion of operational, evidence, presentation, storage, transport, signature, credential, Platform, and extension data.

## Implementation impact

- WORK-0006 implements and tests the closed Component State model and admission boundary.
- WORK-0007 may reference only admitted Component State identifiers under this contract.
- WORK-0008 consumes Project State identity and does not alter Component State identity.

No production code is part of this ADR.

## Compatibility / migration impact

The decision fixes OMVCS 0.1 Component State bodies to one closed envelope. Objects with omitted required fields or unknown top-level members are invalid under this schema. Future top-level changes require an explicit schema/version and compatibility decision. No production Component State objects are known in this workspace.

## Notes

This ADR resolves DG-0011 only. Previously approved identifier, canonicalization, Resource Reference, parentage, and Creative Component identity rules remain in force. No semantic decisions were made beyond the human-approved decision recorded here.
