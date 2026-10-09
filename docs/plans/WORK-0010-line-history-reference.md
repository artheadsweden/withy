# WORK-0010 — Line history references

Status: BLOCKED — DG-0016
Owner agent: Core Engineer
Milestone: M2
Branch: `work/0010-line-history-reference`

## Objective

Implement the approved OMVCS Line record and lifecycle for mutable references
to Revisions, without importing Git branch or ref semantics.

## Normative requirements

- Core Specification §§4.1–4.2, 16–17, 34, 56–58, 74, and 82–83.
- Glossary: Line and Default Line.
- Core Invariants: INV-HIST-005, INV-GC-001, and INV-UX-002.

## Dependencies

- WORK-0001 through WORK-0008 verified identifier and Revision APIs.
- DG-0016 must be resolved by an approved ADR and corresponding Spec updates
  before implementation.
- DEC-CORE-008 separately blocks Line deletion archival/pinning behavior.
  Deletion must remain outside the package until that decision is resolved.
- DEC-INTERACTION-001 and DEC-INTERACTION-003 concern presentation terminology
  and numbering; this Core package does not define those presentation choices.

## Allowed scope

- `crates/omvcs-core/`
- Focused Line model, reference-validation, and lifecycle tests.

## Deliverables

- The exact Line record, identity, and lifecycle selected by the approved
  decision and updated specifications.
- Create/move behavior and expected-current-target compare-and-swap behavior
  under the normative contract.
- Project-consistent validation of Line Revision targets.
- Deletion only if DEC-CORE-008 and the resulting approved contract permit it.

## Acceptance tests

- After DG-0016 resolution, tests cover the exact approved Line member set,
  identifier, naming/uniqueness, default-Line, and mutability rules.
- A stale expected target produces the specified conflict and does not move
  the Line; a matching expected target follows the specified movement rule.
- Target Revision validity and Project association follow the approved
  contract.
- Any generation/CAS behavior is tested exactly as selected by the approved
  decision; no generation rule is inferred before then.
- Deletion and any archival/pin interval are tested only after DEC-CORE-008 is
  resolved.
- No test or implementation assumes Git branch, force-push, reflog, or
  detached-HEAD behavior.

## Explicit non-goals

- Release semantics, Working State lifecycle, repository reachability, or
  garbage collection.
- Musician-facing terminology and display numbering.
- Any behavior left undecided by DG-0016 or DEC-CORE-008.

## Known Design Gaps

- DG-0016 blocks the Line object and lifecycle contract.
- DEC-CORE-008 blocks Line deletion archival/pinning behavior.
- DEC-INTERACTION-001 and DEC-INTERACTION-003 remain presentation decisions;
  they are not prerequisites for the Core Line model.

## Implementation plan

1. Revalidate this package against the resolved ADR and updated Specs.
2. Implement only the selected Line record and reference lifecycle.
3. Add direct contract and CAS tests; add deletion tests only if unblocked.

## Verification requirements

The independent Verifier must check every approved member and operation rule,
CAS conflict behavior, target Project consistency, and absence of inferred
Git semantics.

## Completion criteria

Formatting, focused tests, coverage-map update, independent verification,
handover, and clean Git state.
