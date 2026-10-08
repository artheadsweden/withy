# ADR-0008 — Resource Reference byte-length range

Status: ACCEPTED
Date: 2026-10-08
Decision owner: Human
Related Design Gap: DG-0008

## Context

ADR-0007 defines `ResourceReference.byte_length` as the required byte count of the complete Resource but does not specify the numeric range. Because the field participates in the containing historical object's canonical JSON, all conforming implementations need the same exact domain and validation rules.

## Decision

1. In OMVCS 0.1, `ResourceReference.byte_length` is a JSON number whose mathematical value MUST be an integer in the inclusive range `0 ..= 9007199254740991` (`2^53 - 1`), the maximum interoperable JSON safe integer used by the RFC 8785/JCS numeric model.
2. The value is the exact number of bytes in the complete Resource.
3. Conforming implementations MUST reject negative, fractional/non-integral, and greater-than-`9007199254740991` values. They MUST also reject string or other alternate representations.
4. The upper bound is normative even where a host-language numeric type can exactly represent larger values. Implementations MUST NOT expand the accepted domain based on host-language capacity.
5. Canonical serialization remains governed by RFC 8785/JCS. OMVCS 0.1 defines no decimal-string, tagged-big-integer, or other alternate representation for this field.
6. This limit applies only to the maximum size of one individual Resource represented by a Resource Reference. It does not define repository, Project, Storage Endpoint, or aggregate Project size limits.
7. A future OMVCS version MAY define a larger representation only through an explicit schema/version decision that defines compatibility and canonical-identity consequences. The OMVCS 0.1 domain MUST NOT be silently widened.

## Rationale

The explicit interoperable safe-integer range ensures the Resource byte count has the same exact mathematical value and canonical JSON representation across implementations. A normative domain independent of host integer capacity prevents implementations from assigning different validity or historical identities to the same field.

## Alternatives considered

- Permit each implementation's full native integer range: rejected because host-language capacities differ and do not define interoperable canonical JSON semantics.
- Encode larger values as strings or tagged big integers: rejected for OMVCS 0.1; these would change the field representation and canonical identity.
- Leave the accepted upper bound implicit in individual canonicalization libraries: rejected because it produces implementation-dependent validation.

## Specification impact

- `Specs/OMVCS Core Specification.md` §7: define the exact integer domain, rejected forms, and conformance vectors.
- `Specs/OMVCS Core Invariants Specification.md`, INV-RES-004: require the same domain for historical Resource References.
- `Specs/OMVCS Storage Adapter Specification.md` §§19, 35, and 218: distinguish operational byte-length measurements from the historical Resource Reference field and cross-reference the historical field's range when applicable.
- Glossary Resource/Resource Object definitions need no independent schema change; the exact field contract belongs to Core §7.

## Test impact

WORK-0004 and every downstream model test that validates an embedded Resource Reference MUST accept:

- `0`;
- `1`;
- `9007199254740991`.

They MUST reject:

- `-1`;
- a fractional value such as `1.5`;
- `9007199254740992`;
- a string such as `"1"`.

Tests MUST also confirm that the bound is enforced regardless of the host integer type and that no alternate encoding is accepted.

## Implementation impact

- WORK-0004 validates the Resource Reference field and its exact accepted domain.
- WORK-0006 and WORK-0007 validate embedded Resource References using the same contract.
- The canonical serializer remains the RFC 8785/JCS implementation from WORK-0002; no alternate numeric encoding or widening is added.
- Storage Adapter operational measurements remain operational values, not alternate historical-field encodings; a Resource represented historically must satisfy the Core §7 field constraint.

No production code is part of this ADR task.

## Compatibility / migration impact

OMVCS 0.1 Resource References cannot represent individual Resources larger than `9007199254740991` bytes. Repository and aggregate Project size are not limited by this field decision. A future larger per-Resource range requires an explicit versioned schema and compatibility decision.

## Notes

This ADR resolves DG-0008 only. It does not define chunk lengths, aggregate storage limits, or any other numeric field's range.
