# ADR-0004 — Canonical Adapter State object

Status: ACCEPTED
Date: 2026-10-08
Decision owner: Human
Related Design Gap: DG-0001

## Context

The Core model represented `adapter_state` as an Adapter State reference and the DAW Adapter Specification required content-addressing through canonical metadata, but its unresolved-decision list left open whether an opaque Resource could instead be the complete Adapter State. Those alternatives produce different object types and identity graphs for Project State and Revision.

## Decision

1. Every Adapter State MUST be a canonical OMVCS metadata object with its own content-derived Adapter State Identifier.
2. An Adapter State MAY reference one or more opaque Resource Objects containing native DAW state.
3. Every Project State MUST reference exactly one Adapter State object by its Adapter State Identifier.
4. A Project State MUST NOT reference a native Resource Object directly as the complete Adapter State.

## Rationale

This makes Adapter State a consistently typed metadata object in immutable history while allowing DAW-native bytes to remain opaque Resource content. The Project State reference graph therefore distinguishes OMVCS metadata identity from native Resource identity.

## Alternatives considered

- Allow an opaque Resource Object to serve as the complete Adapter State: rejected. It would make the Project State field represent different object types and bypass the canonical Adapter State metadata identity.
- Permit both forms: rejected. It would preserve the same representation ambiguity and require extra schema/type discrimination not selected by this decision.

## Specification impact

- `Specs/OMVCS Core Specification.md` §§12–13: define the Adapter State object and require the Project State field to reference it.
- `Specs/OMVCS DAW Adapter Specification.md` §§20–21, 214 item 10: require the canonical metadata object and close the unresolved question.
- `Specs/OMVCS Glossary.md`, DAW concepts: define Adapter State and distinguish it from native Resource payloads.
- `Specs/OMVCS Core Invariants Specification.md`, INV-HIST-003: require the complete Project State to reference exactly one Adapter State metadata object.
- `Specs/Ardour Reference Adapter Design.md` §§10, 31, 182: describe native session snapshots as Resource payloads referenced by the canonical Adapter State object.

## Test impact

M1 conformance tests must verify:

- Adapter State is serialized and identified as a canonical OMVCS metadata object.
- A Project State references exactly one Adapter State Identifier.
- A native Resource Identifier is rejected as the complete Adapter State reference.
- Adapter State may reference opaque native-state Resource Objects without Core interpreting their bytes.
- Changing an Adapter State reference or its referenced Resource set changes the appropriate immutable object identity.

These cases are tracked in WORK-0007 and `docs/spec-coverage.md`. No code or tests are part of this documentation task.

## Implementation impact

- `crates/omvcs-model/`: typed Adapter State metadata model and Project State reference validation (WORK-0007).
- DAW Adapter capture: serialize canonical Adapter State metadata and reference native-state Resources.
- Revision identity consumes Project State identity through the existing WORK-0008 dependency.

## Compatibility / migration impact

No production objects or implementations are known in this bootstrap workspace. Any future Adapter State Resource bytes must be represented by a canonical Adapter State metadata object and referenced from Project State through its Adapter State Identifier.

## Notes

No additional semantic decisions were made beyond the approved human decision.
