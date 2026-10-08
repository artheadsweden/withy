# DG-0004 — Actor Identifier representation in Revision identity

Status: RESOLVED
Classification: BLOCKS-MILESTONE
Discovered by: OMVCS Lead
Discovered during: M0
Date: 2026-10-08

## Relevant specifications

- `Specs/OMVCS Core Specification.md`, sections 14 and 59; originally also listed in section 89, question 7.
- `Specs/OMVCS Platform Protocol.md`, sections 15–19 and 245, question 1; the remaining account-association proof question is tracked as DEC-PLATFORM-001.
- `Specs/OMVCS Core Invariants Specification.md`, INV-HIST-002 and INV-SEC-002.

## Problem

A Revision includes an author Actor Identifier, and that author field is part of the content-addressed Revision object. The Core Specification requires stable actor identifiers but leaves their representation outside a particular Platform unresolved. The Platform Protocol separately leaves the exact Actor Identifier format unresolved.

Without a common representation, implementations cannot validate or consistently serialize the author field that contributes to Revision identity. The account-to-Actor association mechanism is a distinct, later Platform concern.

## Why the current specifications are insufficient

The examples use an unspecified string value, while the Core and Platform decision lists explicitly leave the identity representation/format open. Treating an arbitrary implementation-specific string format as normative would select a semantic and serialization rule not established by the Specs.

## Affected work

- M1 Revision author field, validation, canonical serialization, and identity vectors.
- Later Platform account association, authentication, and provenance interoperability.

## Can unaffected work continue?

At discovery, identifier types, raw Resource identity, and model work independent of Revision author representation could proceed, while Revision schema/hash conformance was blocked on this gap. The representation is now resolved by ADR-0002; remaining M1 blockers are tracked separately, and Platform account-association proof remains DEC-PLATFORM-001.

## Candidate directions

Non-normative options include a specified textual identifier format or a typed opaque identifier with separately specified canonical serialization. No option is selected here.

## Required decision

What stable Actor Identifier representation must an OMVCS Revision serialize and hash, independent of any Platform account? How should a conforming implementation validate that representation?

## Resolution

Resolved by [ADR-0002](../decisions/ADR-0002-actor-identifier.md), approved by the human decision owner on 2026-10-08.

In OMVCS 0.1, ActorId is an assigned UUIDv7 serialized in lowercase canonical textual form. It is independent of display name, email, username, Platform account, and signing keys; key rotation and Platform/account linkage changes do not alter ActorId or historical authorship.

Platform/account linkage and proof of control remain separate. The association/proof mechanism was not selected and remains tracked by DEC-PLATFORM-001; it does not block M1 ActorId or Revision author representation.
