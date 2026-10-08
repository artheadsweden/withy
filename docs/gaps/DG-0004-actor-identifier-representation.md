# DG-0004 — Actor Identifier representation in Revision identity

Status: OPEN
Classification: BLOCKS-MILESTONE
Discovered by: OMVCS Lead
Discovered during: M0
Date: 2026-10-08

## Relevant specifications

- `Specs/OMVCS Core Specification.md`, sections 14 and 59 and section 89, question 7.
- `Specs/OMVCS Platform Protocol.md`, sections 15–19 and 245, question 1.
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

Yes. Identifier types, raw Resource identity, and model work independent of Revision author representation may proceed. Revision schema/hash conformance must remain blocked until the Actor Identifier representation is resolved or a normative opaque representation is approved.

## Candidate directions

Non-normative options include a specified textual identifier format or a typed opaque identifier with separately specified canonical serialization. No option is selected here.

## Required decision

What stable Actor Identifier representation must an OMVCS Revision serialize and hash, independent of any Platform account? How should a conforming implementation validate that representation?

## Resolution

UNRESOLVED
