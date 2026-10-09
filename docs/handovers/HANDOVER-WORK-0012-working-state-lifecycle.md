# Handover WORK-0012

From agent: OMVCS Core Engineer
To agent: OMVCS Lead (WORK-0012 closeout)
Date: 2026-10-09
Branch: `work/0012-working-state-lifecycle`
Implementation/test commit: `6e1f6e8`

## Completed

Implemented the Core-side Working State lifecycle in `omvcs-core`. The
representation is mutable Project operational metadata, with optional Base
Revision and Line, a component-source map, and an optional opaque
`AdapterWorkingStateRef`. It has no WorkingStateId, persisted modified
Boolean, historical serialization, or history operation.

Operations cover initial creation, inspection, full materialisation,
selective/custom source updates, Line association, and committed-reference
restore. Adapter work is expressed as prepare, reference validation, restore,
and one mutex-protected Core metadata commit. The Core exposes a
provider-neutral `WorkingStateRecordStore` boundary to atomically commit and
reload complete operational records plus their recovery conditions. Failure
results are typed and
distinguish preserve authorization, missing/unadmitted/cross-Project
sources,
prepare/capture, restore/reference, partial failure, and no-op/success
outcomes. Partial destructive failure leaves the prior committed record
authoritative, sets recovery-required, and is not retried automatically.

Verifier findings addressed: restart marking preserves `recovery_required`;
repository reconstruction loads persisted records and retains that
condition; full materialisation supports a missing current record, validates
the target before Adapter work, and creates no record on prepare failure.
Success commits Base Revision, source map, Line association, reference, and
recovery condition together.

Latest REJECT finding addressed within Core §§21/83 and ADR-0025/0026:
`PersistedWorkingState` now identifies its Project separately and contains
optional Working State metadata. A recovery-only record stores
`recovery_required` without inventing a Working State, Base Revision, Line,
component map, or Adapter reference. `working_state()` now returns
`Option<&WorkingState>` and `project_id()` identifies the operational record.
Reload preserves status even when Working State metadata is absent.
`recovery_condition(Project)` exposes it without requiring Working State.
`persist_recovery_condition(Project)` is an explicit idempotent retry of
operational-record persistence after a store outage; it does not restore live
state, clear the condition, or invoke the Adapter. Explicit full
rematerialisation can recover a Project with no committed Working State.

Partial restore and post-restore new-record commit failure now attempt marker
persistence even without prior Working State metadata. Failed new-state
commits preserve all prior authoritative metadata, including absence, and
only commit that metadata plus recovery status if the store accepts it.
Ongoing store failure returns `RepositoryUnavailable`; the runtime marker
remains queryable and available for explicit later persistence. No persistence
success is claimed while the store is unavailable.

## Specifications implemented

- Core §§3, 19–25, 56, 62, 65, 82–83.
- Glossary: Working State, Base Revision, Materialisation, Selective
  Materialisation, Custom Working State, Local Modification,
  AdapterWorkingStateRef, and Working State recovery condition.
- INV-PROJ-004–005 and INV-WORK-001–006.
- DAW Adapter §§9, 11–13, 30, 35–44, 65–69, 119, 153, 159–163, 183.
- Interaction §§13–14 and 35–36.
- ADR-0018, ADR-0024, ADR-0025, ADR-0026.

## Files changed

- `crates/omvcs-core/src/lib.rs` — exports the new Core module.
- `crates/omvcs-core/src/working_state.rs` — lifecycle, adapter boundary,
  metadata/status types, typed results, in-memory repository, and the
  provider-neutral Core/reference-store load/commit/export boundary.
- `crates/omvcs-core/tests/working_state_lifecycle.rs` — 30 observable
  lifecycle and failure-injection tests.
- `docs/plans/WORK-0012-working-state-lifecycle.md` — verified package state
  and acceptance outcomes.
- `docs/spec-coverage.md` — verified status and focused coverage.
- `docs/milestones.md` and `docs/project-state.md` — WORK-0012/M2 status.
- `docs/handovers/HANDOVER-WORK-0012-working-state-lifecycle.md` — this
  closeout handover.

No model APIs, manifests, lockfiles, dependencies, schemas, FFI, DAW/provider
implementation, or historical object behavior changed.

## Tests added or changed

The 30 focused integration tests cover pre-first-Revision state/no synthetic
history, existing-state refusal, initial full materialisation with no record
(including failure leaving no partial record), successful full
Base/source/ref/Line commit, full rematerialisation, selective/custom Base
preservation, custom sources from another same-Project Revision and local
components, rejection of unadmitted/cross-Project sources before Adapter
work, Line association idempotency and Line movement, derived tri-state vs
native dirty state, operation-scoped replacement authorization/fresh retry
authorization, status recheck after preparation, prepare and restore failure
atomicity, invalid/unavailable/unrestorable references, reference-store
round-trip into a distinct repository followed by restoration through the
committed ref, `recovery_required` preservation across restart and reload,
partial failure/no auto-retry, and normative status/failure tokens.

Five regression tests added for the latest REJECT:

- `initial_partial_restore_persists_recovery_without_fabricating_working_state`
  — status-only query/export/commit/reload, no fabricated metadata, no
  automatic retry, and explicit caller full rematerialisation.
- `initial_post_restore_commit_failure_keeps_absent_metadata_and_reloads_marker`
  — one-shot new-record commit failure followed by status-only commit/reload.
- `initial_commit_outage_reports_unavailability_and_allows_explicit_marker_persistence`
  — ongoing store outage preserves absent authoritative record, explicitly
  reports unavailability, then supports idempotent marker persistence/reload
  with no Adapter invocation.
- `initial_partial_restore_with_unavailable_store_reports_repository_unavailable`
  — partial Adapter failure cannot claim marker persistence during outage;
  explicit later marker persistence/reload is supported.
- `existing_post_restore_commit_failure_preserves_all_old_metadata_and_reloads_recovery`
  — attempted Base/Line/source/ref replacement fails atomically; old metadata
  plus recovery marker reloads, and comparison remains unknown.

The test store injects one-shot commit failure and ongoing load/commit outage
deterministically, without modifying its records on a failed commit.

## Commands run

- `cargo test -p omvcs-core --test working_state_lifecycle --locked` —
  passed, 30 tests.
- `cargo test --workspace --locked` — passed, 218 tests across 32 test
  result groups.
- `cargo fmt --all -- --check` — passed.
- `cargo clippy -p omvcs-core --all-targets --locked -- -D warnings` —
  passed.
- `git diff --check` — passed.

The editor test runner found no Rust tests, so Cargo ran the focused suite.
Initial warnings-denied Clippy runs caught lock-lifetime tightening warnings
in export and the test store; explicit guard drops fixed them before the final
checks.

## Semantic decisions made beyond the specification

None.

## Design Gaps discovered

None. DG-0018 and DG-0024–0026 are resolved by their accepted ADRs.
DEC-INTERACTION-004 remains separate and was not implemented.

## Assumptions

- The optional Line argument on materialisation is the resulting
  association; callers retaining one pass its ID.
- `confirmed` and `unconfirmed` require a committed Adapter reference.
  `recovery_required` does not: failed initial Adapter work can require
  recovery even when no Working State or reference was committed.
- The APIs take typed identifiers, so malformed textual Project identifiers
  cannot be supplied; an absent Project is reported as `ProjectNotFound`.

## Known limitations

- `WorkingStateRecordStore` receives a complete typed Core record and
  recovery condition, with Working State metadata optional; its `commit`
  contract is atomic per Project and a
  failed commit leaves the prior record authoritative. `load_from_record_store`
  reconstructs Core state, treats ordinary referenced records as
  `unconfirmed`, and preserves persisted `recovery_required`.
- The reference-store round-trip uses an in-memory test store; it validates
  Core record export/commit/load and repository reconstruction, not physical
  crash durability. No wire encoding or provider mechanics are included;
  durable storage implementations must implement the store contract.
- The Adapter boundary is a generic Core contract only. It does not implement
  DAW restoration, resource retrieval, native dirty-state reporting, or
  provider behavior.
- If the reference store is unavailable while persisting a recovery
  transition, Core reports repository unavailability. A later reload
  conservatively treats ordinary committed references as `unconfirmed`;
  retaining `recovery_required` across reload depends on the store accepting
  the recovery-condition commit. With no prior record and an unavailable
  store, the runtime marker cannot survive process loss until a store commit
  succeeds; `persist_recovery_condition` enables an explicit retry while the
  repository instance still holds the marker.
- No checkpoint behavior, history/reachability traversal, or Working State
  publication is implemented.

## Remaining work

- Independent Verifier re-review: ACCEPT; no further actionable findings.
- Implementation/tests are committed at `6e1f6e8`; closeout documentation
  remains to be committed, then the feature branch pushed and integrated.
- WORK-0013 and WORK-0014 have not started.

## Git state

Working tree: closeout documentation changes pending commit.
Remote push performed: NO.
Remote publishing enabled: YES (`docs/project-state.md`); no publishing
performed or authorized by this request.

## Independent verification

The Verifier accepted the final recovery/persistence revision after reviewing
recovery-only records, restart/reload, failed post-restore commits, and the
added failure-injection tests. The Verifier confirmed all 30 focused lifecycle
tests and the Core test suite passed; the OMVCS Lead also reran the 30 focused
tests, locked workspace tests (218 passed), rustfmt check, warnings-denied
Core Clippy, and `git diff --check`. Acceptance covers the provider-neutral
Core store contract and its test double only, not physical durable storage.
