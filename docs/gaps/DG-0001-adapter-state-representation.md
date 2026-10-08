# DG-0001 — Adapter State representation in Project State

Status: RESOLVED
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

The DAW Adapter unresolved-decision list left the representation open despite the canonical metadata requirements and typed Adapter State reference shown elsewhere.

## Affected work

- M1 Project State and Revision data models and their identity tests.
- Related canonical serialization/hash test vectors.

## Can unaffected work continue?

Yes. Assigned identifiers, raw Resource identity, Resource references, and Component State work that does not freeze the Adapter State representation may proceed independently. Project State and Revision schema/hash work may proceed under the representation fixed by ADR-0004. The separate canonical map-ordering question was tracked as DG-0005 and has since been resolved by ADR-0005.

## Candidate directions

The approved decision selects a canonical OMVCS metadata object for every Adapter State. It may reference one or more opaque Resource Objects containing native DAW state.

## Required decision

Every Adapter State MUST be a canonical OMVCS metadata object. It MAY reference one or more opaque Resource Objects containing native DAW state. Project State MUST reference the Adapter State object and MUST NOT reference a native Resource directly as the complete Adapter State.

## Resolution

Resolved by [ADR-0004](../decisions/ADR-0004-adapter-state-object.md). Every Adapter State is a canonical OMVCS metadata object with its own content-derived Adapter State Identifier. Project State references that metadata object; native DAW state may be held in opaque Resource Objects referenced by the Adapter State.
