# Handover WORK-0013

From agent: Core Engineer, coordinated by OMVCS Lead
To agent: OMVCS Lead (WORK-0013 closeout)
Date: 2026-10-09
Branch: `work/0013-repository-reachability`
Implementation base HEAD: `0b33ed2b99e8fbf9ab8871d2bda584af8807aa31`

## Completed

Implemented and independently verified the partial Line/Release historical
reachability subset. The Verifier's initial review found one missing Line-move
test; the test was added and independently confirmed to clear the sole finding.
The result remains partial and does not claim full Core §62 reachability.

Public APIs:

- `line::LineEnumerationBoundary::retained_lines`
- `release::ReleaseEnumerationBoundary::admitted_releases`
- `reachability::AdmittedAdapterStateResourceResolver::resolve_resource_ids`,
  extending `omvcs_model::project_state::AdmittedAdapterStateResolver`
- `reachability::partial_line_release_reachability`
- `PartialReachability`, `HistoricalId`, `ReachabilityDefect`, `ReachabilityError`

Root enumeration is complete for each included class/read across all repository
Projects. In-memory implementations return clones under existing locks. Existing
mutation traits and WORK-0009's fail-fast ancestry API remain unchanged.

The result returns sorted/deduplicated Line, Release, Revision, Project State,
Component State, Adapter State, and Resource IDs. Referenced IDs remain reached
even when unresolved or defective; missing metadata and identity/cycle defects
are separate. Independent branches continue. No resource-byte availability,
global unreachable, repository completeness, or deletion permission is reported.

## Specifications implemented

- Core §62: included Line/Release roots, Release -> Revision edge, Revision ->
  Project State -> Component/Adapter State -> Resource edges.
- Core §§10–15, 56: asserted Component parentage, Revision parentage, exact
  admitted identity/reference checks (consumes admission, not import validation).
- Core §12 and DAW Adapter §§20–22: exact admitted Adapter metadata Resource
  projection through a provider-neutral trusted resolver; Core does not parse
  adapter-owned semantics or define a new Adapter body/wire/admission format.
- Glossary Reachability and Repository Metadata.
- INV-GC-001–003 and INV-WORK-002–003: read-only historical references independent
  of Resource bytes; no deletion or availability inference.
- INV-WORK-004–006 / ADR-0018 and ADR-0025: operational Working State separation
  is preserved; no Working State or AdapterWorkingStateRef roots are invented.
- ADR-0016/0017 Line/Release root contracts and ADR-0021 Default Line distinction.

## Files changed

Implementation:

- `crates/omvcs-core/src/lib.rs`
- `crates/omvcs-core/src/line.rs`
- `crates/omvcs-core/src/release.rs`
- `crates/omvcs-core/src/reachability.rs` (new)
- `crates/omvcs-core/tests/repository_reachability.rs` (new)

Documentation:

- `docs/plans/WORK-0013-repository-reachability.md`
- `docs/spec-coverage.md`
- `docs/milestones.md`
- `docs/project-state.md`
- `docs/handovers/HANDOVER-WORK-0013-repository-reachability.md` (this file)

Preserved pre-existing intended preflight changes:

- `docs/decision-register.md`
- `docs/gaps/DG-0027-working-state-safety-reference-roots.md`
- Existing plan/coverage bounded-scope decisions remain intact.

No Specs, ADRs, other crates, manifests, or lockfile were changed.

## Tests added or changed

28 focused tests: 25 behavior-named integration tests in
`repository_reachability.rs`, 3 synthetic graph unit tests in `reachability.rs`.
Exact behavior names and requirement mapping are linked in `docs/spec-coverage.md`.

Coverage includes all in-scope roots across Projects, Line movement changing
the next calculation without mutating history, removed Lines, Default Line
non-root, convergence/deduplication, Revision ancestry and earlier Project States,
Component/parent/Resource edges, unknown-lineage non-inference, exact admitted
Adapter Resource projection, byte independence, unresolved references, separate
identity defects, continuation of independent branches, enumeration failure,
no mutation, no invented roots/global unreachability, Revision/Component cycle
termination, and a 10,000-node nonrecursive graph.

Existing tests were not changed.

## Commands run

- Editor `runTests` found no registered Rust tests; Cargo was used instead.
- `cargo test -p omvcs-core --locked --test repository_reachability`:
  25 passed, 0 failed (final run).
- `cargo test -p omvcs-core --locked --lib reachability::tests`:
  3 passed, 0 failed.
- `cargo test --workspace --locked`:
  246 passed, 0 failed (239 unit/integration + 7 doc tests).
- `cargo fmt --all`, then `cargo fmt --all -- --check`: passed.
- `cargo clippy -p omvcs-core --all-targets --locked -- -D warnings`: passed.
  Earlier too-many-lines/redundant-closure and test empty-assertion diagnostics
  were corrected; no lint suppression was added for them.
- `git diff --check`: passed.
- Editor Problems check for `crates/omvcs-core`: no errors.

The independent Verifier reviewed the full bounded implementation and found
one P2 test omission for Line movement. After the divergent-target,
history-preservation regression test was added, the Verifier independently
confirmed that the finding was cleared. No other actionable findings remain.
The review is accepted; no verifier changes were made.

## Semantic decisions made beyond the specification

None.

## Design Gaps discovered

None newly discovered. DG-0027 remains OPEN/BLOCKS-FEATURE for the exact Working
State safety-reference root set/edges. It does not block this authorized subset.
Other Core §62 minimum roots remain normative requirements, not erased by this
implementation or DG-0015.

## Assumptions

- Immutable model resolvers uphold their existing exact historical admission
  contracts. Returned identifier mismatches are defensively reported.
- Root enumeration providers uphold complete enumeration for each successful
  read, not page/Project-filtered views.
- The Adapter Resource projection provider upholds exact admitted metadata and
  complete Resource reference extraction by the unique applicable schema.
  Tests supply a trusted provider-neutral double; Core invents no Adapter parser.

## Known limitations

- Always partial: Working State safety references, Contributions, configured
  archival pins, and pending publication transactions are not root inputs.
- No completeness/unreachable classification, deletion, retention, GC, pins,
  archives, publication, storage/replica, Platform, FFI, or DAW-native behavior.
- Unresolved or defective metadata prevents following that object's unknown
  edges but does not remove its referenced ID from the result.
- Root enumerations are separate reads; no cross-boundary transaction snapshot
  or concurrent-publication safety guarantee is introduced.
- No production durable enumeration adapter or Adapter body/admission/projection
  implementation; these boundaries consume the independently specified contracts.
- Read-only traversal is not repository-wide import validation. Exact admitted
  same-Project/schema guarantees remain owned by existing admission APIs.
- Independent verification is complete and accepted. Closeout commit/push/
  integration details will be recorded after those operations.

## Remaining work

Closeout/integration is authorized and in progress. The implementation does
not complete all Core §62 root coverage; DG-0027 and the other excluded root
classes remain outstanding. Do not start WORK-0014.

## Git state

Working tree: DIRTY at implementation base — intended WORK-0013 code, tests,
DG-0027/preflight and closeout documentation only; no unrelated paths found.
Remote push performed: NO as of handover preparation.
Remote publishing: ENABLED in `docs/project-state.md`; configured remote is
`origin`.
Independent Verifier review: ACCEPT after the sole test finding was cleared.
WORK-0014 was not started.
