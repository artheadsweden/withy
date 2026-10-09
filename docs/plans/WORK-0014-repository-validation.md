# WORK-0014 — Repository validation

Status: BLOCKED — DG-0019
Owner agent: Core Engineer
Milestone: M2
Branch: `work/0014-repository-validation`

## Objective

Implement the approved read-only or explicitly side-effect-bounded repository
validation operation, preserving the distinction between metadata integrity,
history completeness, and Resource availability.

## Normative requirements

- Core Specification §§21–22, 29, 47–56, 62–67, 76–77, and 82–83.
- Glossary: OMVCS Repository, Repository Metadata, Historical Metadata,
  Operational Metadata, Resource Availability, Corrupt Replica, Reachability,
  and Repository Recovery.
- Core Invariants: INV-HIST-003, INV-HIST-009, INV-RES-007, INV-WORK-002–004,
  INV-INT-001–003, and INV-GC-001–003.

## Dependencies

- WORK-0009 through WORK-0013 for admitted graph, history objects, roots, and
  reachability.
- DG-0019 must be resolved by an approved ADR and corresponding Spec updates
  before implementation.
- DEC-CORE-004 remains open for local metadata-history completeness.
- DEC-CORE-009 remains open for shallow/incomplete history imports.
- Resource retention/deletion decisions DEC-CORE-005 and DEC-CORE-008 are not
  selected or implemented here.

## Allowed scope

- `crates/omvcs-core/`
- Focused repository validation models and tests.

## Deliverables

- Invocation scope and permitted effects exactly matching the approved
  `ValidateRepository` contract.
- Machine-readable results distinguishing the states selected by the approved
  contract without conflating invalid metadata, missing/incomplete metadata,
  and unavailable or corrupt Resource data.
- Reuse of the verified object-level admission and identity rules.

## Acceptance tests

- After DG-0019, DEC-CORE-004, and DEC-CORE-009 are resolved or explicitly
  bounded for the package, tests cover the approved invocation modes, effects,
  result categories, and completeness behavior.
- Canonical hashes, schemas, and required metadata references are checked
  according to their existing admission contracts.
- Missing Resource bytes alone do not invalidate otherwise valid historical
  metadata.
- Available Resource bytes are verified only at the depth selected by the
  approved validation contract; corrupt and unavailable data are distinct
  outcomes where specified.
- No implicit repair, mutation, Resource download, or history-completeness
  policy is invented.
- Validation does not treat a metadata-complete/resource-sparse repository as
  corrupt solely because Resource bytes are not locally materialised.

## Explicit non-goals

- Repository repair or recovery transactions.
- Selecting metadata-history completeness or shallow-import policy.
- Resource garbage collection, retention, or Line deletion pinning.
- Storage-provider-specific verification beyond the approved Core result
  contract.

## Known Design Gaps

- DG-0019 blocks the end-to-end validation operation contract.
- DEC-CORE-004 and DEC-CORE-009 affect completeness and incomplete-import
  results and must be resolved or explicitly scoped before implementation.

## Implementation plan

1. Revalidate this package against DG-0019's approved decision and the
   applicable completeness/import decisions.
2. Implement the specified operation scope and result model using existing
   admission and reachability primitives.
3. Add tests for valid, invalid, incomplete, unavailable, and corrupt cases
   according to the final normative contract.

## Verification requirements

The independent Verifier must inspect operation effects and all result
categories, distinguish metadata validity from Resource availability, and
attempt to expose any inferred repair or incompleteness policy.

## Completion criteria

Formatting, focused tests, coverage-map update, independent verification,
handover, and clean Git state.
