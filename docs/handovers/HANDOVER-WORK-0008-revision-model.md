# Handover WORK-0008

From agent: OMVCS Spec Guardian
To agent: Core Engineer
Date: 2026-10-09
Branch: `work/0008-revision-model`
HEAD: `ebeb5c066840af807485c99186794e941e6ed154` (implementation branch starts from the updated integration base)

## Completed

WORK-0008 has been revalidated against the approved Revision contract and
unblocked for its bounded M1 generic data-model scope. The Revision's exact
closed body, admission conditions, ancestry rules, timestamp, ActorId
authorship, message, set-like collection behavior, identity, and hash
preimage are specified by ADR-0014 and the cited Specs.

The open DG-0015 concerns mapping operation-specific provenance obligations
for integration, fork/import, and conversion to concrete schema-approved
entries. It blocks those provenance-dependent operations in M6; it does not
block generic WORK-0008 schema-directed Revision validation. Generic model
code MUST NOT invent a provenance vocabulary. Non-empty entries are admitted
only through the exact available versioned Revision schema authority.

## Specifications implemented

No production implementation has been performed yet. The work package is
bounded by:

- Glossary: Actor Identifier (ActorId), Revision, Revision Identifier,
  Parent Revision, Revision Graph, Provenance, and Provenance Link.
- Core Specification §§4–5, 14–15, 23, 42–43, 55–56, 59–60, 76–77.
- Core Invariants INV-HIST-001–009, INV-COL-003–004, INV-RES-002,
  INV-PROJ-001.
- ADR-0001, ADR-0002, ADR-0010, ADR-0011, ADR-0012, ADR-0014.
- Verified dependencies WORK-0001 through WORK-0003 and WORK-0007.

## Files changed

No implementation files changed at handoff. See
`docs/plans/WORK-0008-revision-model.md` for the allowed scope.

## Tests added or changed

None yet. Derive focused conformance tests from WORK-0008 and ADR-0014.
Test provenance schema authority using explicitly test-only schema fixtures;
do not treat fixture entries as normative provenance vocabulary.

## Commands run

- `git diff --check` — passed for the specification/handoff documentation
  changes.
- No Rust formatting, tests, or Clippy commands run; implementation has not
  started.

## Semantic decisions made beyond the specification

`None`. The only planning clarification is scope: DG-0015 blocks M6
operation-specific provenance mapping, not the generic schema-authority
contract already stated by ADR-0014 and Core §§14, 43, 76.

## Design Gaps discovered

- DG-0015 remains OPEN: operation-specific provenance requirements must be
  mapped to exact-schema-approved Revision entries. It blocks M6 provenance-
  dependent operations, not this package.

## Assumptions

- As with other schema-owned metadata, the model may accept a resolver/
  validator boundary for the exact versioned Revision schema. If that
  authority is missing, unavailable, or does not match `schema`, the candidate
  cannot become valid/admitted history.
- Test-only validators establish implementation behavior only, not OMVCS 0.1
  provenance entry meanings.

## Known limitations

- Operation-specific provenance facts and their concrete encodings remain
  undefined under DG-0015.
- The independent Verifier must review the implementation against the original
  Specs and ADR-0014 before integration.

## Remaining work

1. Implement only WORK-0008 in `crates/omvcs-model/` and focused tests.
2. Keep Lines, Releases, Working State, repository mutation, publication,
   storage, contribution integration, and branch/ref mechanics out of scope.
3. Run formatting, focused tests, applicable conformance tests, and Clippy
   with warnings denied for touched Rust crates.
4. Update the WORK-0008 plan and coverage; prepare an implementation handover.
5. Request independent Verifier review before any integration.
6. Do not start WORK-0009 or M6 work.

## Git state

Working tree: CLEAN before adding this handover; commit this handover before
implementation starts.
Remote push performed: NO for this work branch.
