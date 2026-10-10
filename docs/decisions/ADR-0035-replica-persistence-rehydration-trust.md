# ADR-0035 — Replica persistence rehydration trust

Status: ACCEPTED
Date: 2026-10-10
Decision owner: Human
Related Design Gap: DG-0033

## Context

Core and Storage Adapter specifications require verified registration of a
new Resource Replica and durable persistence or reconstruction of the
Storage Map, but did not define how reconstruction relates to registration.
Without that distinction, deserializing a Replica record could be confused
with admitting a candidate, or every map load could be incorrectly treated
as a new registration requiring byte verification.

## Decision

1. A Storage Map persisted by an authoritative Repository Home represents
   prior successful registration of its Replica records. Loading or
   reconstructing that map is not a new Replica Addition.
2. A conforming Repository Home MUST persist only Replica records that
   entered the Storage Map through the applicable registration requirements.
   Its persistence and reconstruction path MUST uphold that authority
   contract.
3. Reconstructing an authoritative persisted Storage Map MUST validate its
   structure and consistency, including the map and generation as one
   guarded state. It MUST NOT re-verify Resource bytes solely because the
   map is being reconstructed.
4. Deserializing or decoding a Replica record alone is data decoding, not
   registration and not evidence that the record came from an authoritative
   Repository Home. A record supplied from another source MUST NOT be
   admitted as a persisted registered record unless it satisfies the
   authoritative Repository Home contract or independently satisfies the
   applicable new-registration requirements.
5. Retrieved Resource bytes remain subject to the applicable Content
   Verification requirements. This decision does not change what constitutes
   successful verification, define verification evidence or strength, or
   authorize candidate promotion.

## Rationale

An authoritative Repository Home is the durable boundary for its previously
accepted operational map. Treating reconstruction as continuation of that
persisted state preserves the registration gate without requiring a costly
and semantically unrelated byte re-verification on every load. Keeping
deserialization separate from registration prevents a wire representation
alone from claiming authority.

## Alternatives considered

- Re-verify every Resource representation during map reconstruction:
  rejected because loading persisted operational metadata is not a new
  content-registration operation.
- Require separate persisted verification evidence or provenance records:
  rejected for this decision because it would add a distinct evidence format
  and validation contract beyond the approved authority boundary.
- Treat any structurally valid deserialized Replica as registered:
  rejected because decoding alone does not establish authoritative prior
  registration.

## Specification impact

- `Specs/OMVCS Core Specification.md` §§29, 32, and 34.
- `Specs/OMVCS Storage Adapter Specification.md` §§30 and 51.
- `Specs/OMVCS Glossary.md`, Resource Replica, Storage Map, and Replica
  Addition.
- ADR-0032, Decision 8 and its test/implementation impact.
- `docs/gaps/DG-0033-replica-persistence-rehydration-trust.md`.

## Test impact

WORK-0016 must distinguish record decoding, authoritative map
reconstruction, and new registration. Tests must show that decoding alone
does not mutate or populate a Storage Map; reconstruction validates and
restores an authoritative persisted map and its generation without a
content-byte verification operation; malformed or inconsistent persisted
maps fail reconstruction; candidate data cannot enter through a new
registration path without the applicable verification result; and
Repository Home implementations conform to the authoritative persistence
contract.

## Implementation impact

WORK-0016 may reconstruct records from authoritative Repository Home
records returned through the Storage Map persistence boundary after
structural validation, without re-verifying bytes. Core owns conversion of
that returned persistence data into a Storage Map; arbitrary deserialized
records do not have a public direct-construction path into a map.
Deserialization is not a registration operation. Verified
promotion/registration of new candidates remains gated by WORK-0017 and
DEC-STORAGE-004/005. External Repository Home implementations must honor
the same persistence authority contract.

## Compatibility / migration impact

No additional persisted fields or verification-evidence format are
required. A structurally valid map reconstructed from an authoritative
Repository Home retains its existing Replica records and Storage Map
generation. Non-authoritative imported records do not gain registration
authority merely by being deserialized.

## Notes

No verification taxonomy, provenance-record format, import policy, shared
namespace rule, or byte-verification algorithm is selected here.
