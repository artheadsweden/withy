# ADR-0036 — OMVCS 0.1 reference chunking policy

Status: ACCEPTED
Date: 2026-10-10
Decision owner: Human
Related Design Gap: N/A
Related decisions: DEC-CORE-001, DEC-STORAGE-002

## Context

Core and Storage Adapter specifications described fixed 8 MiB chunking as a
recommendation and left open whether to use it or another deterministic
policy. WORK-0017 and downstream chunk-aware storage work require one
interoperable 0.1 policy. The decision must not change Resource or Chunk
identity rules.

## Decision

1. OMVCS 0.1 defines fixed-size sequential chunking with an exact target size
   of `8 * 1024 * 1024 = 8,388,608` bytes.
2. A conforming chunker processes the complete Resource byte stream from byte
   zero and emits contiguous, non-overlapping chunks in byte-stream order.
   Every non-final Chunk has exactly 8,388,608 bytes; the final Chunk contains
   all remaining bytes and may be smaller.
3. A Resource of at most 8,388,608 bytes produces one Chunk when represented
   using this chunking policy. Existing 0.1 Resource rules accept a
   zero-byte Resource and do not prohibit a zero-length Chunk; therefore the
   chunked reference representation of an empty Resource contains one final
   zero-length Chunk. A complete-object representation remains an independent
   alternative and need not be chunked.
4. Identical Resource bytes chunked under this OMVCS 0.1 policy MUST yield the
   same ordered Chunk sequence. Chunk IDs remain derived from each Chunk's
   bytes; ResourceId remains SHA-256 of the complete Resource bytes.
5. Chunking affects representation boundaries and resulting ChunkIds, but
   not ResourceId, historical bytes, creative history, or provenance.
6. A complete-object representation is independent of the Chunking policy.
   Provider-internal segmentation below the OMVCS representation boundary
   MUST NOT be exposed as different OMVCS Chunk identities.
7. An Endpoint preference MUST NOT automatically cause rechunking of an
   existing representation. The representation's policy is retained.
8. OMVCS 0.1 does not negotiate multiple OMVCS chunking algorithms. A future
   protocol version may add other deterministic policies only by explicitly
   identifying/versioning them; an existing representation retains its
   creating policy.

## Rationale

One exact, deterministic reference policy gives implementations the same
manifest boundaries for the same Resource while leaving complete Resource
identity independent of physical representation. It also prevents provider
preferences from silently changing OMVCS Chunk identity.

## Alternatives considered

- Leave 8 MiB as a recommendation: rejected because it does not establish
  deterministic interoperable 0.1 Chunk boundaries.
- Adopt content-defined or adaptive chunking in 0.1: rejected by the
  approved decision for one fixed-size policy.
- Rechunk automatically to match an Endpoint's preferred size: rejected
  because provider preference does not replace the policy of an existing
  representation.

## Specification impact

- `Specs/OMVCS Core Specification.md` §§8–8.2 and 89.
- `Specs/OMVCS Storage Adapter Specification.md` §§160, 162, 263, and 270.
- `Specs/OMVCS Glossary.md`, Chunk and Chunk Manifest.
- ADR-0007, Decision 6 and its physical-layout test wording.
- Core Invariants remain unchanged in meaning; their Resource identity and
  location rules are cross-checked against this decision.
- Decision register and M3 work packages WORK-0017, WORK-0019, and WORK-0020.

## Test impact

WORK-0017 MUST test deterministic boundaries for below-target, exact-target,
target-plus-one-byte, multi-target, and zero-byte Resources; contiguous
offsets, no gaps or overlap; consistent ordered Chunk IDs; same ResourceId
across complete-object and chunked representation; no automatic rechunking
due solely to Endpoint preference; and provider-internal segmentation not
being exposed as OMVCS Chunk identity.

## Implementation impact

The reference chunking and reconstruction logic is in the Core/model scope
of WORK-0017. Storage Adapters preserve and transfer the approved logical
representation; provider-internal segmentation remains below that boundary.
Chunk-aware filesystem and replication work consumes this policy.

## Compatibility / migration impact

This defines the OMVCS 0.1 reference chunking policy and makes no change to
existing ResourceId or ChunkId algorithms. Previously created complete-object
representations remain valid. OMVCS chunked representations claiming 0.1
conformance must use this policy; provider-internal storage segmentation is
not an OMVCS representation.

## Notes

No separate Design Gap was discovered for zero-length Chunks: the existing
Specs accept zero-byte Resources and do not forbid zero-length Chunks. The
approved at-most-target rule therefore yields one zero-length Chunk for the
empty Resource's chunked reference representation.
