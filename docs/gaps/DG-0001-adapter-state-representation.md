# DG-0001 — Adapter State representation in Project State

Status: OPEN
Classification: BLOCKS-MILESTONE
Discovered by: OMVCS Lead
Discovered during: M0
Date: 2026-10-08

## Relevant specifications

- `Specs/OMVCS Core Specification.md`, sections 12–13 and 89.
- `Specs/OMVCS DAW Adapter Specification.md`, sections 20–22 and 214, question 10.
- `Specs/OMVCS Core Invariants Specification.md`, INV-HIST-003.

## Problem

The Project State model needs to identify its Adapter State. The Core example presents `adapter_state` as a content-derived Adapter State identifier, and the DAW Adapter Specification requires Adapter State to be immutable and content-addressed through the canonical metadata mechanism. However, DAW Adapter Specification section 214, question 10, leaves open whether Adapter State must always be a canonical OMVCS metadata object or whether an Adapter may use an opaque Resource as the complete state representation.

These alternatives change the type and interpretation of the Project State reference and the identity graph hashed into Project State and Revision identifiers.

## Why the current specifications are insufficient

The Core and DAW Adapter examples do not select a representation, while the DAW Adapter unresolved-decision list explicitly says the choice remains open. Selecting one representation in the M1 model would therefore settle an unapproved semantic/schema decision.

## Affected work

- M1 Project State and Revision data models and their identity tests.
- Related canonical serialization/hash test vectors.

## Can unaffected work continue?

Yes. Assigned identifiers, raw Resource identity, Resource references, and Component State work that does not freeze the Adapter State representation may proceed independently. Project State and Revision schema/hash work must remain blocked until resolved.

## Candidate directions

Non-normative options include requiring a canonical metadata envelope for every Adapter State, or permitting an opaque Resource representation behind an explicit typed reference. Neither option is selected here.

## Required decision

Must every Adapter State be represented by a canonical OMVCS metadata object, or may an Adapter designate an opaque Resource as its complete Adapter State representation? If both are allowed, what distinguishes and identifies the two forms in Project State?

## Resolution

UNRESOLVED
