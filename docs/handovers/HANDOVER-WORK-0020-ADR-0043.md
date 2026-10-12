# Handover WORK-0020 — ADR-0043 replication outcome resolution

From agent: OMVCS Lead
To agent: Core Engineer, Storage Engineer, independent Verifier
Date: 2026-10-12
Branch: `work/0020-resource-replication`
HEAD at start of decision-resolution work: `b28c322374c65731a556774b6e23d52e805a1f10`

## Completed

Recorded the human-approved DG-0037 decision in ADR-0043 and updated the
affected Specs, WORK-0020 acceptance plan, gap/decision registers, M3
status, spec coverage, and project state. Core review found no Core-spec
semantic inconsistency and requested explicit acceptance assertions;
Storage review found no storage-boundary issue. The independent Verifier
returned NEEDS CHANGES for missing success/retry acceptance coverage.
The success result/ReplicaId/commit assertion, source preservation on
success, known non-commit status, and safe Adapter retry coverage were
added to the WORK-0020/ADR-0043 test plans; no second Verifier review was
requested.

Cross-Spec review found that Core §69 resumability may apply to a
potentially long-running single-Resource operation, while the approved
decision excludes M4 durable cross-step recovery. This was not resolved by
assumption: DG-0038 was recorded and WORK-0020 remains BLOCKED.

No production implementation or executable test changes were made.

## Specifications implemented

- Core Specification §34, Resource replication outcome; §83 cross-reference.
- Storage Adapter Specification §§59, 252–253.
- Core Invariants INV-RES-005 and INV-INT-003–004.
- Glossary, Resource Replication Outcome.
- Interaction Specification §§158 and 244.

## Files changed

- `docs/decisions/ADR-0043-resource-replication-outcomes.md`
- `docs/gaps/DG-0037-replication-outcome-contract.md`
- `docs/gaps/DG-0038-resource-replication-resumability-boundary.md`
- `docs/decision-register.md`
- `docs/plans/WORK-0020-resource-replication.md`
- `docs/plans/WORK-0021-storage-migration.md`
- `docs/plans/M3-STORAGE-PREFLIGHT.md`
- `docs/spec-coverage.md`
- `docs/milestones.md`
- `docs/project-state.md`
- `docs/handovers/HANDOVER-WORK-0020-ADR-0043.md`
- `Specs/OMVCS Core Specification.md`
- `Specs/OMVCS Storage Adapter Specification.md`
- `Specs/OMVCS Core Invariants Specification.md`
- `Specs/OMVCS Glossary.md`
- `Specs/OMVCS Interaction Specification.md`

## Tests added or changed

No executable tests were added or changed. WORK-0020 and ADR-0043
acceptance/test plans now require coverage for all result/stage/effect/
registration states, success with returned ReplicaId and known CAS commit,
unknown versus known non-commit, source preservation on all paths,
verification failures, safe Adapter stage-local retry, no Core retry after
non-success, and caller re-observation.

## Commands run

- `git status --short --branch`, `git log -1 --oneline`, `git remote -v` —
  confirmed clean `work/0020-resource-replication` at the recorded start
  HEAD and configured `origin`.
- `git diff --check` — passed after the final documentation/spec edits.
- No build or executable tests run because no production code or test source
  changed.

## Semantic decisions made beyond the specification

The human-approved semantic decision is recorded in ADR-0043. No additional
semantic decision was made. The unresolved §69 applicability was recorded
as DG-0038, not decided here.

## Design Gaps discovered

- DG-0037 — RESOLVED by ADR-0043.
- DG-0038 — OPEN, BLOCKS-FEATURE: Core §69 resumability boundary for
  potentially long-running single-Resource replication.
- DG-0034 remains OPEN for generic ProviderLocator schema-identifier
  lexical validation only.
- DG-0036 remains OPEN for non-filesystem Repository Home discovery only.
- DEC-STORAGE-011 and DEC-STORAGE-013 remain OPEN and outside this work.

## Assumptions

None.

## Known limitations

WORK-0020 is not READY / AUTHORIZED. Its implementation remains blocked by
DG-0038. ADR-0043 does not waive Core §69, define a single-Resource duration
threshold, or add durable recovery semantics. No production replication
flow has been implemented.

## Remaining work

1. Obtain a human decision resolving DG-0038 and update ADR/Specs/tests as
   authorized.
2. Reassess WORK-0020 readiness and revise its bounded scope if required.
3. Implement and test WORK-0020, then obtain Storage Engineer review and
   independent Verifier acceptance.
4. Keep WORK-0021 unstarted until its prerequisites are verified.
5. Integrate only after acceptance; this task does not integrate.

## Git state

Working tree: CLEAN after the documentation commit.
Remote push performed: YES — `work/0020-resource-replication` pushed to
`origin`, with the remote feature tip verified to match; no integration
occurred.
