# ADR-0001 — Canonical ordering of hashed array collections

Status: ACCEPTED
Date: 2026-10-08
Decision owner: Human
Related Design Gap: DG-0003

## Context

RFC 8785 canonicalizes JSON object members but preserves array order. OMVCS content-addressed metadata includes both semantically ordered sequences and collections whose order has no semantic meaning. Without a declared field classification and normalization rule, semantically equivalent objects could receive different canonical bytes and identifiers.

## Decision

1. Every array-valued collection field included in a hashed OMVCS metadata object MUST explicitly declare whether it is an ordered sequence or a set-like collection. This ADR's normalization rules apply only to array-valued collections.
2. Ordered sequences preserve their semantic order and MUST NOT be reordered by canonicalization.
3. For a set-like collection, each element is serialized according to the applicable canonical JSON schema, then elements are sorted in ascending lexicographic order of those canonical serialized bytes before the containing object is serialized.
4. Duplicate elements in a set-like collection are invalid. Equality for this check is equality of canonical serialized element bytes.
5. The rule applies recursively to nested collection fields. A collection lacking an explicit classification is invalid for hashing.
6. The OMVCS 0.1 conceptual model classifies Chunk Manifest `chunks` as an ordered sequence in reconstruction order; Component State `parents` and `resources`, Adapter State `resources`, and Revision `parents` and `provenance` as set-like arrays.

## Rationale

This preserves order where it is semantically meaningful, including the reconstruction order of Chunks, while giving implementations a deterministic representation for unordered arrays. Rejecting duplicates prevents a set from acquiring list-like multiplicity. Using canonical serialized element bytes provides one ordering rule independent of implementation-specific object layout.

JSON object maps are not array-valued collections and are outside this ADR's scope. They are governed solely by RFC 8785 object-member ordering; no additional element-byte sorting transformation applies to map entries. Duplicate member names are invalid and MUST be rejected before hashing/canonical serialization. This map rule is resolved by ADR-0005.

## Alternatives considered

- Treat all arrays as ordered: rejected because it gives semantically unordered collections accidental identity differences based on input order.
- Treat all arrays as sets: rejected because it would destroy meaningful order, including Chunk reconstruction order.
- Sort set-like collections by a selected identifier/property: rejected because it introduces field-specific sort rules instead of the approved general rule.
- Permit duplicate values in set-like collections: rejected because multiplicity contradicts set semantics and can create ambiguity.

## Specification impact

- `Specs/OMVCS Core Specification.md` §5.1, §8.2, §10, §12–14: define the array normalization rule and classify M1 array fields; JSON object map handling is defined separately by ADR-0005.
- `Specs/OMVCS Core Invariants Specification.md` §1, INV-HIST-006: require explicit and deterministic hashed collection semantics.
- `Specs/OMVCS Glossary.md`, Chunk Manifest: clarify that reconstruction order is semantically significant.
- `Specs/OMVCS DAW Adapter Specification.md` §§20–22: require hashed Adapter State schemas to classify generic and opaque collections.
- `Specs/OMVCS Storage Adapter Specification.md` §162: classify hashed Chunk Manifest order.
- `Specs/Ardour Reference Adapter Design.md` §§23, 31: classify the conceptual binding/resource/dependency collections.

## Test impact

Conformance tests and vectors must cover:

- ordered arrays preserve element order and yield different canonical bytes when order differs;
- set-like arrays yield the same canonical bytes and object identifier for every permutation;
- set-like arrays reject duplicate elements according to their canonical serialized bytes;
- nested set-like collections normalize recursively before their parent element is serialized;
- JSON object maps are serialized using RFC 8785 object-member ordering regardless of insertion order, with no additional entry sorting;
- JSON object maps with duplicate member names are rejected before canonical serialization or hashing;
- object-map values are normalized recursively according to their own schemas, including any set-like array fields;
- Chunk Manifest order remains reconstruction order and is not sorted as a set;
- hashed schemas with array-valued collections lacking declared semantics are invalid, never guessed; map schemas identify their keys and use RFC 8785 semantics.

These cases must be linked to the M1 serialization and model rows in `docs/spec-coverage.md` before implementation.

## Implementation impact

- `crates/omvcs-model/`: canonical serialization and hashed object schemas (WORK-0002).
- Component State and Project State models (WORK-0006 and WORK-0007).
- Revision model (WORK-0008).
- Relevant conformance/property test suites under `tests/conformance/` and model unit tests.

No implementation is part of this ADR task.

## Compatibility / migration impact

This decision changes Draft 0.1 canonical bytes for set-like arrays whose existing element order is not already canonical. Any pre-existing metadata identifiers produced without this normalization cannot be assumed to equal identifiers produced under the resolved rule. Before such data is treated as interoperable, its schema/version and migration or compatibility treatment must be established. No production objects or code are known to exist in this bootstrap workspace.

## Notes

This ADR approves array collection classification and canonicalization only. At the time of approval it did not resolve DG-0001 (Adapter State representation), DG-0002 (Component State parentage), DG-0004 (Actor Identifier representation), or DG-0005 (JSON object-map normalization). DG-0001, DG-0002, DG-0004, and DG-0005 were subsequently resolved by ADR-0004, ADR-0003, ADR-0002, and ADR-0005 respectively.
