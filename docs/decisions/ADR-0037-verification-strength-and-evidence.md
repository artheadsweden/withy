# ADR-0037 — Verification strength and evidence

Status: ACCEPTED
Date: 2026-10-10
Decision owner: Human
Related Design Gap: N/A
Related decision: DEC-STORAGE-004

## Context

The Storage Adapter Specification contained illustrative verification
strengths that mixed the proposition proved with the method used. A
normative, method-based contract is required so Core can distinguish
individual Chunk verification from complete Resource verification without
trusting provider-specific success indicators.

## Decision

1. Verification outcome, strength, and method/evidence are separate
   dimensions. A result identifies the subject, outcome, strength when
   established, method/evidence, and only operational timing/source
   information permitted by the applicable contract.
2. OMVCS 0.1 defines these verification strengths:
   - `chunk_identity`: the exact bytes of one Chunk were obtained and
     cryptographically verified against that ChunkId. This proves only that
     Chunk's identity.
   - `resource_identity`: the complete Resource byte sequence was
     cryptographically verified against its ResourceId, either by verifying
     one complete-object representation or by verifying the ordered Chunk
     representation and its complete reconstructed Resource identity under
     ADR-0038.
3. Verification methods are distinct from strengths. They include direct
   byte-read hashing, deterministic Chunk reconstruction, and
   `provider_equivalent_checksum`. A provider checksum is equivalent only
   when its documented and enforced semantics prove the exact OMVCS SHA-256
   identity over the exact same bytes: one Chunk's bytes for
   `chunk_identity`, or the complete Resource byte sequence for
   `resource_identity`. Otherwise it is operational evidence only.
4. Generic provider success, object existence, matching length, ETag, object
   version, CRC, multipart ETag, or any checksum whose algorithm/scope is not
   proven equivalent MUST NOT produce OMVCS content verification.
5. Verification outcomes are `verified`, `failed`, or `indeterminate`.
   `failed` means bytes actually checked against the required identity did
   not match. Missing or unavailable bytes yield an indeterminate/unavailable
   result and MUST NOT be classified as corrupt.
6. A Resource Replica may be treated as corrupt only after checked bytes
   fail the required identity. Corruption is per Replica and MUST NOT change
   ResourceId, invalidate another Replica, or make the Resource Object
   itself corrupt.
7. Content-verification evidence is operational metadata. It MUST NOT enter
   Resource hash preimages, Component State, Project State, Revision
   identity, or creative provenance. It MUST NOT contain credentials or
   transient signed URLs.
8. `chunk_identity` alone is insufficient to register a Resource Replica.
   New registration requires `resource_identity` assurance under ADR-0038.

## Rationale

Strength names the proposition actually established. Keeping method and
evidence orthogonal permits a provider checksum to be evaluated against its
true byte scope without elevating generic provider statuses into integrity
claims.

## Alternatives considered

- Treat provider checksum, copy success, existence, or length as independent
  verification strengths: rejected because none necessarily proves the
  required OMVCS SHA-256 identity.
- Make provider-equivalent evidence a separate strength: rejected because it
  describes how a proposition was proved, not what was proved.
- Conflate unavailable bytes with failed verification/corruption: rejected
  because no incorrect bytes were established.

## Specification impact

- `Specs/OMVCS Core Specification.md` §§32, 48–51, and 55.
- `Specs/OMVCS Storage Adapter Specification.md` §§30–36, 73, 76, 109–112,
  183, 192, 204–205, and 240–243.
- `Specs/OMVCS Glossary.md`, Resource Replica, Availability State, Content
  Verification, Corrupt Replica, and the new Verification Strength and
  Verification Method terms.
- `Specs/OMVCS Core Invariants Specification.md`, INV-RES-005 and
  INV-INT-001–003.
- Decision register and WORK-0017.

## Test impact

WORK-0017 MUST test both strengths and their exact byte scopes; verified,
failed, and indeterminate outcomes; direct hash and provider-equivalent
methods; rejection of non-equivalent provider checksums, ETags, generic
success, existence, and length; unavailable-versus-corrupt behavior; one
corrupt Replica not affecting another; and evidence independence from
historical identity.

## Implementation impact

WORK-0017 defines typed verification result, strength, method, and evidence
contracts in the Core/model and generic Storage Adapter boundaries.

## Compatibility / migration impact

The former illustrative values `full_content`,
`trusted_provider_checksum`, `chunk_verified_plus_manifest`,
`metadata_only`, and `unknown` are not OMVCS 0.1 verification strengths.
Existing Resource and Chunk identity algorithms are unchanged.

## Notes

DEC-STORAGE-005 separately defines when verified Chunk evidence may establish
Resource-level assurance for a destination representation.
