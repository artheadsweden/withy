# WORK-0011 — Immutable Release references

Status: BLOCKED — DG-0017
Owner agent: Core Engineer
Milestone: M2
Branch: `work/0011-release-history-reference`

## Objective

Implement the approved immutable Release model and its association with an
admitted Revision, distinct from mutable Lines.

## Normative requirements

- Core Specification §§4.1–4.2, 18, 56, 74, and 82–83.
- Glossary: Project, Revision, Line, and Release.
- Core Invariants: INV-HIST-001–005, INV-HIST-009, INV-GC-001, and INV-REC-002.

## Dependencies

- WORK-0001 through WORK-0008 verified identity and Revision APIs.
- DG-0017 must be resolved by an approved ADR and corresponding Spec updates
  before implementation.
- DEC-CORE-002 remains open for Reference Render policy and must be resolved
  before any Release field or rule depends on that policy.

## Allowed scope

- `crates/omvcs-core/`
- Focused Release model, admission, target-validation, and immutability tests.

## Deliverables

- The exact Release identity, body, required/mutable members, and name
  uniqueness rules selected by the approved decision.
- Creation/admission checks for the approved Release-to-Project/Revision
  association.
- Enforcement that an admitted Release continues to identify the same
  Revision.

## Acceptance tests

- After DG-0017 resolution, tests cover the exact approved Release schema,
  identity, requiredness, metadata mutability, and name-namespace rules.
- Creation follows the approved preconditions and failure behavior.
- The target Revision is valid/admitted and Project association follows the
  approved contract.
- An admitted Release cannot be moved to a different Revision.
- Reference Render behavior is tested only if separately specified by
  resolution of DEC-CORE-002.
- No test or implementation equates Release with a Line, Git tag, or mutable
  reference.

## Explicit non-goals

- Line lifecycle, Working State, repository reachability, or garbage
  collection.
- Reference Render production or Adapter execution.
- Any behavior left undecided by DG-0017 or DEC-CORE-002.

## Known Design Gaps

- DG-0017 blocks the Release identity/schema and admission contract.
- DEC-CORE-002 remains open for Reference Render policy.

## Implementation plan

1. Revalidate this package against the resolved ADRs and updated Specs.
2. Implement the selected immutable Release object and admission rules.
3. Add closed-schema, naming, association, and immutability tests.

## Verification requirements

The independent Verifier must compare the implementation to the approved
Release contract, validate Project/Revision association, and confirm no
mutable-Line or Git-tag semantics were introduced.

## Completion criteria

Formatting, focused tests, coverage-map update, independent verification,
handover, and clean Git state.
