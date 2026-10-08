# DG-0008 — Canonical representation range for Resource byte length

Status: RESOLVED
Classification: BLOCKS-FEATURE
Discovered by: OMVCS Lead
Discovered during: WORK-0004 revalidation after DG-0007 resolution
Date: 2026-10-08

## Relevant specifications

- `Specs/OMVCS Core Specification.md` §§5 and 7.
- `Specs/OMVCS Glossary.md`, Resource and Resource Object.
- `Specs/OMVCS Core Invariants Specification.md`, INV-RES-001–002.
- ADR-0007, Resource Reference `byte_length` decision.
- WORK-0004, Resource Reference validation and identity tests.

## Problem

ADR-0007 and Core §7 require `byte_length` to represent the non-negative byte count of the complete Resource. It is included in a historical Resource Reference and therefore in the containing historical object's canonical JSON identity. Core §5 requires JSON Canonicalization Scheme semantics compatible with RFC 8785.

The specifications do not define the valid representable range or encoding for byte counts that exceed the interoperable safe-integer range of canonical JSON numbers. Selecting a bounded machine integer, a maximum accepted Resource size, or a different JSON representation affects validation and canonical bytes across implementations.

## Why the current specifications are insufficient

The required byte count must remain exact for the complete Resource, while canonical JSON number handling has finite numeric precision and does not provide interoperable safe-integer semantics for every arbitrarily large integer. The current rules say the count is non-negative but do not set an upper bound or define another representation for larger counts. An implementation choice could either reject otherwise valid large Resources or serialize a count without preserving its exact value, changing interoperable historical identity.

The approved DG-0007 resolution determines field presence, meaning, and identity participation, but does not decide this numeric representation/range question.

## Affected work

- WORK-0004 Resource Reference model and validation.
- Any downstream model embedding a Resource Reference, including WORK-0006 and WORK-0007.
- Canonical serialization/model conformance vectors for Resource Reference lengths.

## Can unaffected work continue?

The already verified WORK-0001 through WORK-0003 work remains unaffected. WORK-0004 MUST NOT select a maximum byte count, silently narrow the accepted range, or define an alternate encoding before this gap is resolved. WORK-0005 and later dependent model work must not assume a completed Resource Reference contract where the byte-length field is involved.

## Candidate directions

The following are discussion material only and are NOT approved decisions:

- Set an explicit maximum byte count that is exactly representable by the selected canonical JSON number model.
- Specify an exact representation for larger byte counts while retaining interoperability with canonical JSON.
- Define another versioned field representation with a migration/compatibility rule.

## Required decision

What exact non-negative byte-length values are valid in OMVCS 0.1, and how MUST values beyond the canonical JSON number safe-integer range be represented and validated so every conforming implementation preserves the exact byte count and canonical identity?

## Resolution

Resolved by human decision recorded in [ADR-0008 — Resource Reference byte-length range](../decisions/ADR-0008-resource-byte-length-range.md).

In OMVCS 0.1, `ResourceReference.byte_length` is a JSON number whose mathematical value MUST be an integer in `0 ..= 9007199254740991` (`2^53 - 1`). It is the exact byte count of one complete Resource. Negative, fractional/non-integral, above-maximum, string, and other alternate representations are rejected regardless of host-language numeric capacity. RFC 8785/JCS remains the canonical serialization rule. The bound is per Resource and does not constrain repository, Project, Storage Endpoint, or aggregate Project size. Storage Adapter byte-length inputs/results are operational measurements and not alternate historical encodings; a Resource represented by a historical Resource Reference must satisfy Core §7. A future wider representation requires an explicit schema/version decision defining compatibility and canonical-identity consequences. Core §7, INV-RES-004, Storage Adapter §§19, 35, 218, affected plans, decision register, and spec coverage have been updated with this decision and boundary test vectors.
