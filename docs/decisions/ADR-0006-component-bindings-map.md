# ADR-0006 — Adapter State component bindings map

Status: ACCEPTED
Date: 2026-10-08
Decision owner: Human
Related Design Gap: DG-0006

## Context

The generic OMVCS Core and DAW Adapter specifications define Adapter State `component_bindings` as a JSON object map keyed by Creative Component Identifier. The Ardour Reference Adapter Design instead illustrated an array of binding records, each containing a `component_id`. The conflicting shapes produce different canonical Adapter State schemas and identities.

## Decision

1. `Adapter State.component_bindings` uses the generic OMVCS representation: a JSON object map keyed by `CreativeComponentId` (Creative Component Identifier).
2. Each map value contains the adapter-specific binding record, such as `binding_kind`, `native_ids`, and permitted adapter-specific metadata.
3. The Creative Component Identifier MUST NOT be duplicated inside the value merely to repeat the map key unless a future schema has a separate justified need.
4. The array-of-records representation is not an exception and MUST NOT be used for `component_bindings`.
5. Map insertion order has no semantic significance. Canonical ordering is governed solely by RFC 8785 object-member ordering, according to ADR-0005. Duplicate object member names MUST be rejected before hashing or canonical serialization.
6. Array-valued collections inside binding records follow their own declared ordering semantics under Core Specification §5.1. In the Ardour example, `native_ids` is set-like.

## Rationale

Using the already specified generic map representation keeps the adapter binding shape consistent across OMVCS and Ardour. Putting the Component Identifier in the key avoids redundant identity data in the value. The existing map canonicalization decision provides deterministic hashing without an extra entry-sorting rule.

## Alternatives considered

- Keep an array of records with a `component_id` field: rejected because it conflicts with the generic contract and duplicates the map key identity if projected into that contract.
- Treat the Ardour example as an exception: rejected; no adapter-specific exception was approved, and it would produce a separate binding model for the reference adapter.

## Specification impact

- `Specs/OMVCS Core Specification.md` §5.1 and §12: specify map values as adapter-specific binding records and prohibit redundant repetition of the key.
- `Specs/OMVCS DAW Adapter Specification.md` §20: specify map value shape, identifier non-duplication, and retained RFC 8785 map behavior.
- `Specs/Ardour Reference Adapter Design.md` §§23 and 31: replace arrays of binding records with a Component-Identifier-keyed map and classify nested array fields separately.

## Test impact

Documentation/conformance vectors must cover:

- map keys are Creative Component Identifiers and values are adapter-specific binding records;
- the key is not repeated inside the value merely to duplicate it;
- changing map insertion order leaves canonical bytes and Adapter State identity unchanged;
- RFC 8785 object-member ordering alone governs the map, with duplicate member names rejected before hashing/canonicalization;
- nested `native_ids` ordering follows its declared set-like semantics, including permutation invariance and duplicate rejection.

These cases are added to WORK-0007 and `docs/spec-coverage.md`. No executable tests are part of this documentation task.

## Implementation impact

- Generic Adapter State model and Project State reference validation: WORK-0007.
- Ardour Adapter State component-binding schema and conformance tests: M8 implementation/reconnaissance.

## Compatibility / migration impact

The prior Ardour array-of-records example is superseded and is not a conforming `component_bindings` representation. No production objects or implementations are known in the bootstrap workspace. Any external Draft 0.1 data using that example would require an explicit schema migration before interoperability; no migration behavior is specified here.

## Notes

No additional semantic decisions were made beyond the approved human decision. The ADR-0005 map canonicalization rule is unchanged.
