# Handover WORK-0013/0014 — Working State safety-reference roots

From agent: Core Engineer
To agent: Integration / M2 closeout
Date: 2026-10-09
Branch: `work/0013-working-state-roots`
Implementation base: `566679f40e8d1d1646d64a0ee5d4cf13c1885065`

## Completed

Implemented the approved ADR-0030/0031 extension in the current worktree.
Read-only partial reachability now enumerates currently persisted Working
State records across Projects and includes a present Base Revision and each
present component-source Component State as roots. The result preserves
unresolved target identifiers and root-field context, identity/cycle defects,
deduplication, and the lack of any global-unreachable classification.

Renamed the public operation to
`partial_repository_reachability`; the result remains `PartialReachability`
and explicitly partial. Validation reports Line/Release and Working State
root-provider coverage on successful enumeration, while Contributions,
configured archival pins, and pending publication transactions remain
partial. Missing Working State safety-root targets are reported as
`unresolved` without declared-boundary lookup or invented historical
referrer/edge-kind context. Validation remains read-only.

Updated both work plans, `docs/spec-coverage.md`, and milestone/project-state
text to distinguish the original verified/integrated Line/Release
implementation from this extension.

## Specifications implemented

- Core Specification §§19, 56, 62, and 64.
- Glossary: Working State, Base Revision, Declared History Boundary, and
  Reachability.
- Core Invariants: INV-WORK-007 and INV-GC-001–003.
- ADR-0030 and ADR-0031.

The normative text and ADRs were committed in `566679f40e8d1d1646d64a0ee5d4cf13c1885065`.
The pre-existing Line/Release, traversal, and validation behavior remains
within its original previously verified scope.

## Files changed

- `crates/omvcs-core/src/working_state.rs`
- `crates/omvcs-core/src/reachability.rs`
- `crates/omvcs-core/src/repository_validation.rs`
- `crates/omvcs-core/tests/repository_reachability.rs`
- `crates/omvcs-core/tests/repository_validation.rs`
- `docs/plans/WORK-0013-repository-reachability.md`
- `docs/plans/WORK-0014-repository-validation.md`
- `docs/spec-coverage.md`
- `docs/milestones.md`
- `docs/project-state.md`
- `docs/handovers/HANDOVER-WORK-0013-0014-working-state-roots.md`

## Tests added or changed

- `repository_reachability.rs`: 30 tests pass. New cases cover a Base-only
  root; Base protection independent of Line movement; absent Base/source and
  pre-first-Revision zero roots; custom source outside Base ancestry; current
  persisted record updates; convergent/deduplicated roots; operational-field
  exclusions; unresolved root target reachability/context without fabricated
  metadata identity or edge kind; no mutation; and Working State
  enumeration/provider failure.
- `repository_validation.rs`: 18 tests pass. Added coverage verifies typed
  unresolved Base/source findings, unresolved history completeness, zero
  declared-boundary lookups for Working State roots, supported-provider
  coverage, unsupported partial later root classes, and read-only/provider
  behavior.

## Commands run

- `cargo test --locked -p omvcs-core --test repository_reachability` —
  30 passed, 0 failed.
- `cargo test --locked -p omvcs-core --test repository_validation` —
  18 passed, 0 failed.
- `cargo test --workspace --locked -q` — 274 passed, 0 failed.
- `cargo fmt --all -- --check` — passed.
- `cargo clippy -p omvcs-core --all-targets --locked -- -D warnings` —
  passed.
- `git diff --check` — passed.

Independent Verifier result: **ACCEPT**, no blocking or non-blocking
findings. The Verifier confirmed root membership/exclusions, missing-target
classification and no-boundary-lookup behavior, partial coverage semantics,
read-only behavior, and test coverage.

## Semantic decisions made beyond the specification

`None`. Implementation follows ADR-0030 and ADR-0031 only.

## Design Gaps discovered

`None` newly discovered. DG-0027 and DG-0028 are resolved by ADR-0030 and
ADR-0031 respectively. DG-0015 remains open for Contribution semantics and
outside this package; DEC-INTERACTION-004 remains separate for temporary
checkpoints.

## Assumptions

- The Working State enumeration boundary returns the complete current
  Core-owned Working State record set for that read; it does not return
  recovery-only records as fabricated Working States.
- Each root's Project and field context is diagnostic context only, not a
  historical Identifier or metadata edge kind.
- Existing admitted-object resolver contracts and historical graph traversal
  are reused unchanged.

## Known limitations

- The result remains explicitly partial. Contributions, configured archival
  pins, and pending publication transactions are unsupported; no global
  unreachable result or deletion/retention authority is exposed.
- Enumerations remain separate reads and do not create an atomic
  cross-provider snapshot or publication transaction.
- A missing Working State safety-root target has unresolved status and no
  declared-boundary lookup, per ADR-0031.
- No physical Working State persistence adapter, storage-provider mechanics,
  FFI, repair, garbage collection, deletion, or temporary-checkpoint policy
  was added.
- M2 closeout is recorded complete at the integration commit below.

## Remaining work

Verify the integration branch has been pushed and its local/remote tips
match. Do not add Contributions, archival pins, pending publication
transactions, temporary checkpoints, repair, GC, storage-adapter persistence,
FFI, or M3 implementation in this extension.

## Git state

Implementation and focused tests are committed at `b96ed7a`
(`core(WORK-0013/0014): add Working State safety roots`); verification
documentation is committed at `1ac35dc`. The feature branch was pushed to
`origin/work/0013-working-state-roots`. The integration merge commit is
`3eb1c3e01fa01904f8925f9bcdf3171059594c50` on
`spec/0003-canonical-collection-order`. Remote publishing is ENABLED in
`docs/project-state.md`.
