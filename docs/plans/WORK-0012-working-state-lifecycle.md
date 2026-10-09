# WORK-0012 — Working State lifecycle

Status: BLOCKED — DG-0018
Owner agent: Core Engineer
Milestone: M2
Branch: `work/0012-working-state-lifecycle`

## Objective

Implement the approved Core Working State representation and lifecycle while
keeping mutable local work distinct from immutable Project State and Revision
history.

## Normative requirements

- Core Specification §§3, 19–23, 25, 56, 62, 65, and 82–83.
- Glossary: Working State, Base Revision, Materialisation, Selective
  Materialisation, Custom Working State, and Local Modification.
- Core Invariants: INV-PROJ-004–005 and INV-WORK-001–004.
- DAW Adapter Specification §§9, 11–13, 30, 35–44, 65–69, 119, 153, and
  159–162.

## Dependencies

- WORK-0001 through WORK-0008 verified historical object APIs.
- WORK-0010 Line contract, if the approved Working State design associates
  Working State with a Line.
- DG-0018 must be resolved by an approved ADR and corresponding Spec updates
  before implementation.
- DEC-INTERACTION-004 separately governs temporary local safety checkpoints;
  this package does not define their UX or lifecycle.

## Allowed scope

- `crates/omvcs-core/`
- Focused Working State model, lifecycle, and Core tests.

## Deliverables

- The approved persistent-versus-ephemeral Working State representation.
- Approved Base Revision, pre-first-Revision, and Line-association behavior.
- Approved representation of custom state sources, component changes, and
  Core change status, distinct from DAW-native dirty state.

## Acceptance tests

- After DG-0018 resolution, tests cover the exact Working State lifecycle and
  persistence boundary.
- Pre-first-Revision behavior and Base Revision presence/update rules match
  the approved contract.
- Working State materialisation or modification does not itself create or
  mutate immutable historical objects.
- Custom Component State/resource sources and change-status behavior match
  the approved contract.
- Adapter-reported `changed`, `unchanged`, or `unknown` is not conflated with
  Core's Base Revision comparison unless the Specs explicitly define that
  relationship.
- Temporary checkpoint behavior is tested only under its separate approved
  contract.

## Explicit non-goals

- DAW-native dirty-state implementation beyond the existing Adapter contract.
- Revision creation/publication, Lines, Releases, or temporary checkpoint UX.
- Any behavior left undecided by DG-0018 or DEC-INTERACTION-004.

## Known Design Gaps

- DG-0018 blocks the Core Working State lifecycle and Base Revision contract.
- DEC-INTERACTION-004 remains open for temporary local safety checkpoints.

## Implementation plan

1. Revalidate this package against the resolved ADR and updated Specs.
2. Implement only the approved Core Working State representation and lifecycle.
3. Add tests for historical separation, Base Revision rules, and change
   tracking.

## Verification requirements

The independent Verifier must check the full approved lifecycle, persistence
boundary, Base Revision behavior, and separation from immutable history and
Adapter-native dirty state.

## Completion criteria

Formatting, focused tests, coverage-map update, independent verification,
handover, and clean Git state.
