# Handover WORK-0014 — Repository validation

From agent: Core Engineer; closeout by OMVCS Lead
To agent: M2 integrator / next Core maintainer
Date: 2026-10-09
Branch: `spec/0003-canonical-collection-order`
HEAD: `3f37fa1a981fa06408616b080edb6c56355bc1b0` (WORK-0014 integration
merge; closeout documentation commit follows)

## Completed

Implemented the strictly read-only `ValidateRepository` operation with
Repository and Project scopes, explicit Resource verification depths,
machine-readable findings and coverage, body-Identifier verification
separate from strict historical admission, exact declared-boundary queries,
and honest partial Line/Release root coverage.

WORK-0014 was independently accepted and integrated. The verifier's two
blocking findings about unresolved reachability results and incomplete root
coverage were corrected and independently re-verified. The verifier also
requested additional report-level tests for unresolved Release roots and
unattributed reachability metadata; both were added without production-code
changes. No actionable findings remain.

## Specifications implemented

- Core Specification §§4, 10, 13–14, 18, 21–22, 29–30, 47–56, 62–67,
  76–77, and 82–83.
- Core Invariants: INV-HIST-003, INV-HIST-008–009, INV-RES-007,
  INV-WORK-002–004, INV-INT-001–005, INV-REC-002, INV-REC-006, and
  INV-GC-001–003.
- Glossary: OMVCS Repository, Content-derived Identifier, Repository
  Metadata, Historical Metadata, Operational Metadata, Declared History
  Boundary, History Completeness, Metadata Integrity, Validation Coverage,
  Resource Verification Depth, Availability State, Corrupt Replica,
  Reachability, and Repository Recovery.
- ADR-0027, ADR-0028, and ADR-0029.

No normative Specs were modified during implementation.

## Files changed

- `crates/omvcs-core/src/lib.rs`
- `crates/omvcs-core/src/reachability.rs`
- `crates/omvcs-core/src/repository_validation.rs`
- `crates/omvcs-core/tests/repository_reachability.rs`
- `crates/omvcs-core/tests/repository_validation.rs`
- `crates/omvcs-model/src/component_state.rs`
- `crates/omvcs-model/src/project_state.rs`
- `crates/omvcs-model/src/release.rs`
- `crates/omvcs-model/src/revision.rs`
- `crates/omvcs-model/tests/repository_validation_identity.rs`
- `docs/plans/WORK-0014-repository-validation.md`
- `docs/spec-coverage.md`
- `docs/milestones.md`
- `docs/project-state.md`
- `docs/handovers/HANDOVER-WORK-0014-repository-validation.md`

## Tests added or changed

- `omvcs-core/tests/repository_validation.rs`: 17 contract tests covering
  scope/depth echoing, aggregate status states, exact boundary lookup and
  failures, missing and later-resolved targets, invalid identity/schema,
  same-Project constraints, cycles, Resource verification behavior,
  read-only guarantees, partial root coverage, unresolved Line/Release
  roots, generic unresolved reachability results, and invocation failures
  versus completed reports with findings.
- `omvcs-model/tests/repository_validation_identity.rs`: 5 tests for
  body-Identifier verification independent of missing referenced targets,
  while preserving strict admission and rejecting invalid/unavailable
  schemas or bodies.
- Existing `omvcs-core/tests/repository_reachability.rs` updated for
  direct unresolved Line/Release root context; 25 tests pass.

## Commands run

- `cargo test -p omvcs-core --test repository_validation --locked` — 17
  passed.
- `cargo test -p omvcs-core --test repository_reachability --locked` — 25
  passed.
- `cargo test --workspace --locked` — 268 passed, 0 failed.
- `cargo fmt --all -- --check` — passed.
- `cargo clippy -p omvcs-core --all-targets --locked -- -D warnings` —
  passed.
- `git diff --check` — passed.

Independent Verifier: ACCEPT WITH FINDINGS on re-review. Both blocking
findings were cleared. Its remaining non-blocking test-coverage note was
addressed with two additional tests and did not require further production
re-verification.

## Semantic decisions made beyond the specification

`None`.

## Design Gaps discovered

- DG-0019 is RESOLVED by ADR-0029.
- DG-0027 remains OPEN/BLOCKS-FEATURE for Working State safety-reference
  root semantics and is explicitly outside this partial validation scope.
- DG-0015 remains open for operation-specific provenance outside this
  package.
- DEC-CORE-005, DEC-CORE-008, and DEC-INTERACTION-004 remain separate.

No new Design Gap was required.

## Assumptions

None. Provider contracts remain provider-neutral and use existing model
identity/schema and admission APIs.

## Known limitations

- Core §62 root coverage remains partial: Working State safety references,
  Contributions, configured archival pins, and pending publication
  transactions are reported as partial or unavailable.
- Validation MUST NOT claim complete reachability or global unreachability
  from WORK-0013's partial Line/Release result.
- Provider-specific persistence and Resource storage mechanics are not
  implemented. Resource validation does not fetch or materialise bytes.
- Declared boundaries classify omissions but do not resolve or admit
  missing targets.

## Remaining work

No further WORK-0014 implementation or verification is pending. Do not widen
this package to implement unresolved root classes, storage-provider
persistence, repair/recovery, retention, garbage collection, or M3 work.
M2 remains IN PROGRESS; no M3 work has started.

## Git state

Working tree: CLEAN after the WORK-0014 documentation closeout commit.
Remote push performed: YES — feature branch
`origin/work/0014-repository-validation` and integration branch
`origin/spec/0003-canonical-collection-order`; publishing was ENABLED.
