# ADR-0039 — Canonical Resource and Chunk logical keys

Status: ACCEPTED
Date: 2026-10-10
Decision owner: Human
Related decision: DEC-STORAGE-001

## Context

Storage operations used logical keys but the Specs did not select a
canonical provider-neutral layout for immutable Resource and Chunk bytes.
Examples did not establish a normative path.

## Decision

1. The canonical Resource key is
   `resources/sha256/<p1>/<p2>/<digest>`.
2. The canonical Chunk key is
   `chunks/sha256/<p1>/<p2>/<digest>`.
3. `<digest>` is the complete 64-character lowercase hexadecimal SHA-256
   digest of the corresponding ResourceId or ChunkId. `<p1>` and `<p2>`
   are characters 1–2 and 3–4 of the digest.
4. Logical keys use `/` on every host OS, are relative, contain only the
   prescribed lowercase ASCII components, and contain no empty, dot,
   absolute, drive, platform-separator, or user-controlled filename
   components. They contain no metadata, Project/creator names,
   timestamps, Endpoint IDs, or credentials.
5. ResourceId and ChunkId remain authoritative; keys are operational
   addresses, never identity or historical data. A configured physical
   root prefix or deterministic internal mapping is permitted only if
   logical keys remain externally observable and distinct logical keys
   cannot alias.
6. Existing bytes satisfy immutable creation only if they verify as the
   requested identifier. Mismatch is an integrity/storage conflict and
   MUST NOT be silently overwritten.
7. This decision does not define canonical path layouts for metadata
   objects. Metadata continues to use its applicable logical operation
   contract.

## Rationale

A single logical key for each immutable byte identity supports portable
addressing without coupling identity to a provider, filesystem, endpoint,
or user-facing name.

## Alternatives considered

- Leave Resource and Chunk key spelling to each provider: rejected because
  callers require one provider-neutral logical address.
- Use physical paths or Endpoint identifiers as identity: rejected because
  location and provider mapping are mutable operational facts.
- Define a canonical metadata path layout here: not selected; it was not
  part of the approved decision, and metadata addressing remains governed
  by its operation contracts.

## Specification impact

- `Specs/OMVCS Core Specification.md` §8.3.
- `Specs/OMVCS Storage Adapter Specification.md` §§14–15, 19–22, 63–67,
  and decision list §270.
- `Specs/OMVCS Glossary.md`, Logical Storage Key.
- `Specs/OMVCS Core Invariants Specification.md`, existing INV-RES-001–004
  remain controlling; no new identity invariant is required.

## Test impact

Conformance tests MUST assert exact Resource and Chunk key bytes for
representative digest vectors, reject noncanonical/traversal/platform
forms before native-path resolution, and prove key/path/provider remapping
does not change content identity. Creation races and mismatching existing
bytes MUST not overwrite the canonical object.

## Implementation impact

WORK-0019 MUST use these logical keys for Resource and Chunk byte storage.
The Storage Adapter maps them safely to physical storage. WORK-0020/0021
continue to use existing content identifiers and do not change historical
identity.

## Compatibility / migration impact

The prior examples were illustrative, not persisted normative keys.
Provider physical layout may differ while preserving the canonical logical
key. No existing content or identity changes.

## Notes

DEC-STORAGE-013 remains OPEN. This decision does not designate an official
reference Adapter or mandate a provider's internal physical layout.
