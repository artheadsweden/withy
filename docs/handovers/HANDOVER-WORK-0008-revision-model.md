# Handover WORK-0008 — Immutable Revision model

From agent: OMVCS Core Engineer
To agent: Integration maintainer
Date: 2026-10-09
Branch: `work/0008-revision-model`
HEAD: `63b4aad38721a022d8e35810a84d420faf644ae0` (implementation is in the worktree; this commit contains the initial handoff)

## Completed

Implemented the bounded generic OMVCS 0.1 Revision model in
`crates/omvcs-model/`. `RevisionCandidate` holds unchecked typed input;
`Revision` can only be produced after the exact Revision schema authority,
the admitted Project State, and all admitted same-Project parents resolve.
The admitted Revision exposes only immutable fields, canonical bytes of the
exact seven-member body, and its typed SHA-256 `RevisionId`.

The exact schema authority owns provenance entry shape, requiredness, nested
schema/collection rules, and semantic validation. Core does not define or
infer an entry vocabulary. Test integration schemas use explicitly
test-only fixture fields and make no normative provenance claims.

The Verifier's remediation finding that second `60` was accepted on any
month-end date is fixed. Timestamp admission now accepts `:60` only at
`23:59:60Z` on an announced positive UTC leap-second date, using the known
announcement list through 2016-12-31; the demonstrated
`2026-10-31T23:59:60.000000000Z` is rejected.

## Specifications implemented

- Glossary: Actor Identifier (ActorId), Revision, Revision Identifier,
  Parent Revision, Revision Graph, Provenance, and Provenance Link.
- `Specs/OMVCS Core Specification.md` §§4–5, 14–15, 23, 42–43, 55–56,
  59–60, and 76–77.
- `Specs/OMVCS Core Invariants Specification.md`: INV-HIST-001–009,
  INV-COL-003–004, INV-RES-002, and INV-PROJ-001.
- ADR-0001, ADR-0002, ADR-0010, ADR-0011, ADR-0012, and ADR-0014.
- Dependency contracts from verified WORK-0001 through WORK-0003 and
  WORK-0007.

Operation-specific provenance mapping referenced by INV-COL-003–004 and
workflow examples was not implemented; DG-0015 remains OPEN and is scoped to
M6 provenance-dependent operations.

## Files changed

- `crates/omvcs-model/src/lib.rs` — exports the Revision model.
- `crates/omvcs-model/src/revision.rs` — candidate/admitted types, exact
  schema-authority trait, admitted Project State/Revision resolver
  boundaries, timestamp validation, same-Project ancestry checks, canonical
  body generation and Revision identity.
- `crates/omvcs-model/tests/revision.rs` — 18 focused specification-derived
  tests, including test-only schema fixtures.
- `docs/plans/WORK-0008-revision-model.md` — implementation status and
  delivery details.
- `docs/spec-coverage.md` — WORK-0008 rows marked `verified` after independent
  verifier acceptance.
- `docs/handovers/HANDOVER-WORK-0008-revision-model.md` — this handover,
  updated from the repository template.

No dependency, Cargo manifest, Spec, or Design Gap file was changed.

## Tests added or changed

`crates/omvcs-model/tests/revision.rs` covers:

- exact closed seven-member body, exact canonical-byte vector, and SHA-256
  preimage;
- required members, unknown top-level members, duplicate JSON member names,
  and typed identifier namespace validation;
- missing, mismatched, and duplicate exact-schema authority; schema-directed
  entry validation and semantic rejection;
- valid/admitted Project State resolution, unavailable parent rejection,
  same-Project parent enforcement, and metadata-only references without
  Resource-byte materialization;
- empty, one-parent, and multiple-parent history; duplicate parent rejection
  and parent-order identity invariance;
- direct ActorId, identity change for each canonical member, empty and
  verbatim Unicode/whitespace messages, and timestamp/ancestry separation;
- exact UTC nanosecond timestamp shape, calendar/time rejection cases,
  rejection of non-leap `:60` (including 2026-10-31), and acceptance tests
  for every positive UTC leap-second date announced through 2016-12-31;
- nested provenance schema validation, set-like sorting and duplicate
  rejection, ordered nested-array preservation, and unclassified-array
  rejection using test-only fixture schemas.

The test fixtures assert only plumbing and canonical behavior. Their
`fixture_*` fields do not define production provenance semantics.

## Commands run

- `cargo fmt --package omvcs-model` — passed.
- `cargo fmt --package omvcs-model -- --check` — passed.
- `cargo test --locked -p omvcs-model --test revision` — passed, 18 tests.
- `cargo test --locked -p omvcs-model` — passed, all 134 tests across unit,
  integration, and documentation test binaries; no failures.
- `cargo clippy --locked -p omvcs-model --all-targets -- -D warnings` —
  passed.
- `git diff --check` — passed.

An initial strict Clippy run during remediation flagged the leap-date pattern
format; the pattern was regrouped and the final strict run passed.

## Semantic decisions made beyond the specification

`None`.

## Design Gaps discovered

- DG-0015 remains OPEN. It blocks operation-specific provenance mapping for
  M6 integration/fork/import/conversion behavior, not the generic
  schema-directed Revision model.
- No additional Design Gap was identified.

## Assumptions

- As with the Project State model, implementations of
  `AdmittedProjectStateResolver` and `AdmittedRevisionResolver` honor their
  documented contract and return only the requested valid/admitted immutable
  object.
- An exact `RevisionSchemaValidator` is trusted to implement the selected
  version's schema-owned provenance requiredness and semantics. The generic
  model independently applies structural validation, nested collection
  normalization, canonical ordering, and duplicate rejection.
- A Revision graph assembled only from previously admitted parents remains a
  DAG; candidate admission does not allow unchecked parent objects to enter
  history.
- Leap-second acceptance uses a code-owned list of all announced positive UTC
  leap-second dates through 2016-12-31. No later positive UTC leap second has
  been announced; adding future announced leap seconds requires updating that
  list and its test vector. This rejects second `60` on unannounced dates.

## Known limitations

- No production OMVCS 0.1 Revision schema authority is included in
  `omvcs-model`; the model exposes the exact-schema boundary for a concrete
  schema provider to implement.
- Operation-specific provenance obligations remain unresolved under
  DG-0015; do not infer them from Core §42 illustrations or test fixtures.
- Future UTC leap seconds are not automatically discovered; the supported
  announced-date list must be updated if a later positive leap second is
  announced.
- This is model-only work. No Lines, Releases, Working State, repository
  mutation, publication, storage, Contribution integration, signatures,
  authorization, FFI, or branch/ref mechanics were implemented.
- The Verifier accepted the implementation after the leap-second remediation
  with no blocking findings. Initial review found that invalid `:60` values
  were accepted on any month-end; the implementation now accepts only the 27
  announced positive UTC leap-second dates through 2016-12-31. Regression tests
  reject `2026-10-31T23:59:60.000000000Z` and other invalid dates/times and
  accept each listed announced date. A nonblocking note observed there is no
  separate pre-1972 `:60` test, but the whitelist rejects it.

  The verifier also confirmed the previously identified stale M1 planning
  status was corrected. Coverage is now marked `verified` only after this
  independent acceptance.

## Remaining work

1. Commit and push the verified WORK-0008 branch, then integrate it into
   `spec/0003-canonical-collection-order` using a no-fast-forward merge.
2. Confirm the integration base is clean and retains verified WORK-0001
   through WORK-0008 coverage.
3. Do not implement WORK-0009 or M6 provenance mapping as part of this
   handover.

## Git state

Working tree: CLEAN after closeout commit.
Remote push performed: YES for `work/0008-revision-model`.
Remote publishing: ENABLED in `docs/project-state.md`.
