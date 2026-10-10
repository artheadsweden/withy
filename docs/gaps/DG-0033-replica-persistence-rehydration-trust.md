# DG-0033 — Replica persistence rehydration trust and provenance

Status: RESOLVED
Classification: BLOCKS-FEATURE
Discovered by: Spec Guardian
Discovered during: WORK-0016 Storage Engineer review
Date: 2026-10-10

## Relevant specifications

- `Specs/OMVCS Core Specification.md` §§32–34.
- `Specs/OMVCS Storage Adapter Specification.md` §§29–30, 51, 181, and 183.
- `Specs/OMVCS Glossary.md`, Resource Replica and Replica Addition.
- `Specs/OMVCS Core Invariants Specification.md`, INV-INT-003.
- `docs/plans/WORK-0016-replica-storage-map.md`.
- ADR-0032, Decisions 1 and 8.

## Problem

The normative text says an incomplete or unverified candidate MUST NOT be
registered as a Resource Replica in the Storage Map, and that registration
may occur only after applicable Content Verification succeeds. It also
requires Repository Home to durably persist or reconstruct the Storage Map
and its generation together.

The Specs do not say whether reconstruction from persisted Storage Map
records is a trusted continuation of a prior successful registration, what
provenance makes a persisted record authoritative, or whether rehydration
must re-establish any verification fact. Consequently, it is unclear whether
accepting a structurally valid Replica record during reconstruction is
outside the registration rule or is itself a way of admitting an unverified
candidate.

In the WORK-0016 implementation under review, `Replica` has public
deserialization that accepts an assigned `ReplicaId`, and
`StorageMap::from_persisted_records` accepts resulting `Replica` values.
An arbitrary caller can therefore serialize candidate-shaped data with an
assigned ID, deserialize it as a `Replica`, and construct a Storage Map.
The helper's intended use is reconstitution of already-persisted records,
but the API cannot distinguish that source from caller-synthesized data.
This records the implementation concern; it does not itself establish
whether the normative persistence boundary is trusted.

## Why the current specifications are insufficient

Core §32 and Storage §§29–30 define the registration prerequisite, while
Storage §51 requires persistence/reconstruction. Neither defines the trust
or provenance boundary between those operations. A rule that persisted
records are trusted, a rule that rehydration must re-verify content, and a
rule requiring some other provenance check have materially different
semantics. Selecting one would add a rule not presently stated.

The Specs also do not define whether public record deserialization is a
mere data-decoding operation or an authorized way to create a registered
Replica value. Consequently, the compliance status of the described
deserialize-then-reconstruct API path is not unambiguous.

## Affected work

- WORK-0016: `Replica` deserialization, Storage Map reconstruction, and
  `StorageMapPersistence::load` conformance to the candidate-registration
  boundary.
- Future Repository Home implementations and any import/restore path that
  reconstructs persisted Storage Map records.

## Can unaffected work continue?

Yes. The approved Replica identity/representation model and Storage Map
generation/CAS mechanics may proceed independently. This gap blocks
acceptance of the persistence rehydration boundary as preserving the
candidate-registration rule; it does not reopen ADR-0032 or ADR-0034 and
does not authorize verified promotion, which remains gated by WORK-0017 and
DEC-STORAGE-004/005.

## Resolution

Resolved by ADR-0035. An authoritative Repository Home's persisted Storage
Map represents prior successful registration. Reconstruction validates
structure and map/generation consistency, but does not re-verify Resource
bytes solely because the map is loaded. Decoding a Replica record alone is
not registration and does not establish Repository Home authority.
Repository Home implementations MUST uphold the persistence authority
contract; records from other sources remain subject to the applicable
new-registration requirements.

This decision does not define a verification-evidence format, verification
taxonomy, or import policy, and does not authorize verified promotion before
the approved WORK-0017 result contract.

See [ADR-0035](../decisions/ADR-0035-replica-persistence-rehydration-trust.md).
