# Handover WORK-0020 — ADR-0044 resumability boundary

From agent: OMVCS Lead
To agent: Core Engineer, Storage Engineer, independent Verifier
Date: 2026-10-12
Branch: `work/0020-resource-replication`
HEAD at start: `841633b0a82e682863027ba6a90c0c1399c69cb8`

## Completed

Recorded the human-approved DG-0038 decision in ADR-0044 and reconciled the
affected Specs and planning records. Durable resumability now follows an
explicit workflow designation, never Resource size, Chunk count, elapsed
time, background execution, progress reporting, or operation identifiers.
WORK-0020 is one invocation-scoped Resource replication operation; future
bulk/multi-Resource replication remains durable workflow scope.

The update preserves ADR-0043 without modification. A fresh independent
Verifier initially returned NEEDS CHANGES only for premature handover Git
claims. After correction, re-review returned ACCEPT for both the amended
ADR-0043 test plan and ADR-0044/DG-0038 reconciliation. WORK-0020 is
READY / AUTHORIZED. No production code or executable test source changed.

## Specifications implemented

- Core Specification §34, per-Resource restart/re-observation boundary.
- Core Specification §69, explicit durable resumable workflow designation.
- Core Specification §83, mandatory resumability designation.
- Storage Adapter Specification §§27–28 and 57, transport resume versus
  Core durable continuation.
- Interaction Specification §§160 and 241–243, background/transfer-resume
  presentation boundary.
- Glossary: Invocation-Scoped Operation, Durable Resumable Workflow, and
  Transport Resume.

## Files changed

- `docs/decisions/ADR-0044-durable-resumable-workflow-boundary.md`
- `docs/gaps/DG-0037-replication-outcome-contract.md`
- `docs/gaps/DG-0038-resource-replication-resumability-boundary.md`
- `docs/gaps/DG-0039-storage-migration-resumability-designation.md`
- `docs/decision-register.md`
- `docs/plans/WORK-0020-resource-replication.md`
- `docs/plans/WORK-0021-storage-migration.md`
- `docs/plans/M3-STORAGE-PREFLIGHT.md`
- `docs/spec-coverage.md`
- `docs/milestones.md`
- `docs/project-state.md`
- `docs/handovers/HANDOVER-WORK-0020-ADR-0044.md`
- `Specs/OMVCS Core Specification.md`
- `Specs/OMVCS Storage Adapter Specification.md`
- `Specs/OMVCS Interaction Specification.md`
- `Specs/OMVCS Glossary.md`

ADR-0043 was reviewed but not modified.

## Tests added or changed

No executable tests were added or changed. WORK-0020's acceptance plan now
requires:

- confirmed success with returned ReplicaId;
- source preservation on success and every failure path;
- known CAS non-commit versus permitted unknown status;
- safe Adapter stage-local retry/transport resume;
- no automatic composed Core retry;
- interruption with destination residue;
- fresh authoritative re-observation after an incomplete/prior interrupted
  invocation;
- no result fabricated after process loss; and
- no false durable-resume claim based on size, duration, background
  execution, progress, operation identifiers, or transfer resumption.

## Commands run

- `git status --short --branch`, `git rev-parse HEAD`, `git log -2
  --oneline`, `git remote -v`, and `git ls-remote` — confirmed the clean
  feature branch and remote at
  `841633b0a82e682863027ba6a90c0c1399c69cb8`.
- Cross-Spec searches covered all eight Specs for long-running, resumable,
  resume, restart, retry, bulk replication, replication, migration,
  operation log, transaction, checkpoint, interruption, and recovery.
- `git diff --check` — passed after final edits.
- No executable tests/build/Clippy run because code and test source were
  unchanged.

## Semantic decisions made beyond the specification

`None`.

The human-approved semantic decision is recorded in ADR-0044. Migration
workflow designation, bulk migration details, durable checkpoint schema,
and bulk continuation policy were not inferred or decided.

## Design Gaps discovered

- DG-0038 — RESOLVED by ADR-0044.
- DG-0039 — OPEN, BLOCKS-FEATURE for WORK-0021 only: Storage Migration
  resumability designation.
- No new Design Gap affects WORK-0020.
- DG-0034 and DG-0036 remain OPEN and out of scope.
- DEC-STORAGE-011 and DEC-STORAGE-013 remain OPEN and out of scope.

## Assumptions

None.

## Known limitations

WORK-0020 authorizes only provider-neutral single-Resource replication:
complete-object or Chunk-preserving transfer, destination verification,
Storage Map registration, ADR-0043 outcomes, and source-preserving failure
semantics. It does not include bulk replication, durable continuation,
global replay idempotency, migration/source removal, deletion, GC,
retention, cleanup, or M4 recovery machinery.

## Remaining work

1. Implement WORK-0020 only within its authorized scope.
2. Add the specified direct and failure-injection tests.
3. Obtain Storage Engineer review and independent implementation Verifier
   acceptance.
4. Keep WORK-0021 blocked until DG-0039 is resolved and WORK-0020
   implementation is verified.
5. Define bulk/multi-Resource replication only through a future bounded
   work package.

## Git state

State at independent review:

- Working tree: DIRTY — ADR-0044/specification documentation is uncommitted.
- Remote push performed for this update: NO.
- Local and remote feature tips before this update:
  `841633b0a82e682863027ba6a90c0c1399c69cb8`.

Final commit, push, synchronized-tip verification, and clean-tree
verification remain required after independent Verifier ACCEPT.
Integration occurred: NO.
