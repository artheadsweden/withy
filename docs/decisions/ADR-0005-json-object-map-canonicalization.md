# ADR-0005 — Canonicalization of hashed JSON object maps

Status: ACCEPTED
Date: 2026-10-08
Decision owner: Human
Related Design Gap: DG-0005

## Context

ADR-0001 defines canonical element-byte sorting for set-like JSON array fields. The Core model also uses JSON object maps in hashed metadata. RFC 8785 already defines object-member ordering by member name, but the relationship between that rule and ADR-0001's set-like element sorting was not explicit. Applying an additional sort to map entries could conflict with RFC 8785 and produce ambiguous hash inputs.

## Decision

1. Hashed JSON object maps are governed solely by RFC 8785 JSON Canonicalization Scheme object-member ordering.
2. Map insertion order has no semantic significance.
3. ADR-0001's canonical element-byte sorting rule applies only to fields represented as set-like JSON arrays. It MUST NOT be applied to JSON object/map entries.
4. Duplicate object member names are invalid and MUST be rejected before hashing or canonical serialization.
5. Map values are normalized according to their own schemas. Array-valued fields within a map value remain subject to their declared ordered or set-like rules.

## Rationale

This preserves RFC 8785 as the single canonical object-member ordering rule while keeping ADR-0001's element-byte ordering limited to set-like arrays. Explicit duplicate rejection prevents ambiguous JSON object inputs from reaching canonical serialization or hashing.

## Alternatives considered

- Apply ADR-0001's set-like byte sorting to map entries: rejected because maps are not array-valued collections, and the additional sort can conflict with RFC 8785's member-name ordering.
- Leave duplicate member handling to parser-specific behavior: rejected because the same ambiguous input could be interpreted differently before canonicalization.

## Specification impact

- `Specs/OMVCS Core Specification.md` §§5–5.1, 12–13: explicitly separate array normalization from map canonicalization; define map uniqueness and prohibit additional entry sorting.
- `Specs/OMVCS Core Invariants Specification.md`, INV-HIST-006: scope ordered/set-like semantics to arrays and state the map rules.
- `Specs/OMVCS DAW Adapter Specification.md` §20: apply the same map rule to Adapter State maps and reject duplicate names.
- `docs/decisions/ADR-0001-hashed-collection-ordering.md`: clarify its array-only scope.

## Test impact

M1 conformance vectors and tests must verify:

- Two semantically identical maps supplied in different insertion orders produce identical RFC 8785 canonical bytes and metadata identifiers.
- Canonical map output follows RFC 8785 object-member ordering by member name, including a vector where member-name order differs from value order.
- A map with duplicate member names in raw JSON input is rejected before canonical serialization or hashing.
- Map values containing set-like arrays receive the array normalization rule, without sorting the containing map entries.
- Existing set-like array permutation invariance, duplicate rejection, and ordered-array preservation remain unchanged.

These cases are linked to WORK-0002, WORK-0003, WORK-0006, and WORK-0007 in `docs/spec-coverage.md`. No code or executable tests are part of this documentation task.

## Implementation impact

- `crates/omvcs-model/`: RFC 8785-compatible map canonicalization and duplicate-name rejection at JSON input parsing, plus schema-directed value normalization (WORK-0002).
- Metadata object hashing and map-bearing model identity (WORK-0003, WORK-0006, WORK-0007).
- WORK-0008 consumes the resolved Project State identity from WORK-0007.

## Compatibility / migration impact

No production objects or implementations are known in this bootstrap workspace. Implementations that previously applied additional map-entry sorting or accepted duplicate member names are not conformant to this resolution.

## Notes

This decision resolves DG-0005 only. It does not change ADR-0001's rules for array-valued collections.
