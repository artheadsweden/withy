# ADR-0002 — Actor Identifier representation

Status: ACCEPTED
Date: 2026-10-08
Decision owner: Human
Related Design Gap: DG-0004

## Context

Revisions carry author identity in their immutable, content-addressed metadata. The Core Specification required stable authorship but left the representation open, and the Platform Protocol combined the identifier format with the separate question of proving an account association. Examples also used noncanonical values such as `actor:anna`.

## Decision

1. In OMVCS 0.1, an Actor Identifier (`ActorId`) is an assigned UUID version 7 serialized in lowercase canonical textual form.
2. ActorId is independent of display name, email address, username, Platform account, and signing keys.
3. Changing or removing a Platform/account linkage, changing presentation data, or rotating signing keys MUST NOT change ActorId or rewrite historical authorship.
4. Platform/account linkage and proof that an account controls or represents an ActorId are separate concerns. This ADR does not select an association or proof-of-control mechanism.
5. A Revision's author field and any other historical actor reference use ActorId, not a display/account/signing identifier.

## Rationale

Using the existing assigned-identifier UUIDv7 format gives historical authorship one stable, platform-independent representation. Separating the ActorId from mutable profiles, accounts, and keys allows account or cryptographic changes without rewriting history or conflating authorship with authorization.

## Alternatives considered

- Platform account identifiers as ActorId: rejected because accounts are platform-scoped and mutable/removable.
- Display name, email, or username as ActorId: rejected because these are mutable presentation/contact data.
- Signing key as ActorId: rejected because key rotation must not change authorship and signing identity is a separate concern.
- Platform-specific association proof in Core: rejected because the approved decision keeps this as a separate Platform concern and does not choose a mechanism.

## Specification impact

- `Specs/OMVCS Core Specification.md` §§4.1, 14, 31, 38, 57, 59, 89: classify Actors as assigned identifiers, specify Revision author serialization and stable identity, update ActorId examples, and remove the resolved item from open Core questions.
- `Specs/OMVCS Glossary.md`, new Actor Identifier (ActorId) term: define the normative identity and independence requirements.
- `Specs/OMVCS Core Invariants Specification.md`, INV-HIST-007: require canonical ActorId representation and stability across account/profile/key changes.
- `Specs/OMVCS Platform Protocol.md` §§15–19, 245: separate ActorId from Platform accounts and proof mechanisms; retain proof-of-control as an unresolved question.
- `Specs/OMVCS Storage Adapter Specification.md` §6 and Core examples: update illustrative legacy `actor:` values to canonical UUIDv7 ActorIds.

## Test impact

M1 tests and conformance vectors must cover:

- ActorId parses and serializes as a lowercase canonical UUIDv7.
- Non-v7 and noncanonical textual forms are rejected where an ActorId is required.
- Revision author serialization uses ActorId and contributes the ActorId value to the immutable Revision object.
- Changing display name, email, username, Platform account association, or signing key does not change ActorId or historical authorship.
- Key rotation does not rewrite a Revision or alter its ActorId.
- Platform account association/proof is not inferred from ActorId; no particular proof mechanism is required by this ADR.

These cases are tracked in `docs/spec-coverage.md` and WORK-0001/WORK-0008. No tests or implementation are part of this documentation task.

## Implementation impact

- `crates/omvcs-model/`: assigned identifier model (WORK-0001) and Revision author field/identity (WORK-0008).
- M6 Contribution/provenance models use the same ActorId type; account association remains outside Core identity.

## Compatibility / migration impact

Examples using values such as `actor:anna` or `actor:joakim` are not canonical ActorIds and have been replaced in the Specs. Any external Draft 0.1 data or implementations using such values would need an agreed migration/compatibility treatment before interoperability; none is known in this bootstrap workspace.

## Notes

DEC-PLATFORM-001 remains open only for the mechanism by which a Platform account proves control of or association with an ActorId. The separate signing/authorization decision remains open and is not resolved here.
