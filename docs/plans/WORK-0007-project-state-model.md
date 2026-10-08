# WORK-0007 — Project State model

Status: BLOCKED ON DG-0001 AND DG-0003
Owner agent: Core Engineer
Milestone: M1
Branch: `work/0007-project-state-model`

## Objective

Represent one complete immutable logical Project State using stable Project identity and immutable Component State references, while preserving the DAW-independent boundary.

## Normative requirements

- Glossary: Project State.
- Core Specification, sections 13, 24, and 55.
- Core Invariants: INV-HIST-003, INV-RES-004, INV-PROJ-004.

## Dependencies

- WORK-0001, WORK-0002, WORK-0003, WORK-0005, and WORK-0006.
- DG-0001 must resolve the Adapter State representation/reference.
- DG-0003 must resolve any collection-order behavior incorporated into the hashed object.
- DEC-PLATFORM-016 remains open. Do not add licensing fields to the M1 Project State model unless its ownership/location is decided first.

## Allowed scope

- `crates/omvcs-model/`
- Focused Project State validation/identity tests and conformance vectors explicitly permitted by this package.

## Deliverables

- Complete Project State references, not a change list.
- Storage-independent Project and Component State references.
- DAW Adapter State represented only after DG-0001 is resolved.

## Acceptance tests

- A Project State identifies a complete logical state, not merely a delta.
- Project identity and Component State references are explicit and stable.
- Storage locations, platform URLs, local paths, and availability do not enter Project State identity.
- Different creative-object references yield different Project State identities.
- Adapter State encoding and hashed-collection cases remain blocked until DG-0001/DG-0003 are resolved.

## Explicit non-goals

- Working State, materialisation, DAW restoration, Resource retrieval, or storage maps.
- Inferring missing component states or choosing defaults not defined by the Specs.
- DAW-specific fields in the Core model.

## Known Design Gaps

- DG-0001 — Adapter State representation in Project State.
- DG-0003 — canonical order of hashed collection fields.

## Implementation plan

1. Confirm resolved Component State and Adapter State references.
2. Implement the complete-state model without adding adapter/provider fields.
3. Verify identity independence and complete-state—not-delta—semantics.

## Verification requirements

The Verifier must compare Project State references and invariants to the original sections and attempt invalid/partial-state cases without assuming an unspecified default.

## Completion criteria

Formatting, focused tests, coverage-map update, independent verification, handover, and clean Git state.
