# Handover WORK-0011

From agent: OMVCS Core Engineer
To agent: Project owner / integration
Date: 2026-10-09
Branch: `work/0011-releases`
HEAD at closeout documentation preparation: `3854c3b3699bbb7658bceb0bcfa6f26667633281`

## Completed

Implemented the Release model, content-derived identity, same-Project Revision
admission, and atomic in-memory Core creation boundary for WORK-0011.
Implementation and required validation are complete. Independent Verifier
acceptance is ACCEPT with no remaining actionable conformance findings.
WORK-0011 is VERIFIED but not yet integrated; coverage is `verified`.

Accepted, human-approved ADR-0023 records that Core MUST return the integrity
violation when one `CreateRelease` request simultaneously encounters an
existing same-ReleaseId/different-canonical-body integrity violation and a
Project/name binding conflict. The name conflict MUST NOT mask it. Failure
remains atomic with all stored objects/bindings unchanged. This decision is
limited to this exact overlap; it defines no general failure precedence.

## Specifications implemented

- Core Specification §§4.1–4.2, 18, 56, 62, 74, 82–83.
- Glossary: Release and Release Identifier.
- Core Invariants: INV-HIST-001–005, INV-HIST-009, INV-HIST-012,
  INV-GC-001, INV-REC-002.
- ADR-0017 Release contract; ADR-0023 bounded failure precedence.
- DG-0023 is RESOLVED by accepted, human-approved ADR-0023 and the Core §18
  wording.

## Files changed

WORK-0011 implementation:

- `crates/omvcs-model/src/lib.rs` — exports Release model and typed ReleaseId.
- `crates/omvcs-model/src/hashing.rs` — hashes canonical Release bytes.
- `crates/omvcs-model/src/release.rs` — ReleaseCandidate, Release,
  ReleaseSchemaValidator, strict body decoding/admission, identity, getters,
  and ReleaseAdmissionError.
- `crates/omvcs-model/src/timestamp.rs` — shared Core §15 timestamp
  validation; Revision now reuses the same validation logic.
- `crates/omvcs-model/src/revision.rs` — shares timestamp validation without
  changing Revision timestamp behavior.
- `crates/omvcs-core/src/lib.rs` — exports Release operation module.
- `crates/omvcs-core/src/release.rs` — ReleaseOperationBoundary,
  InMemoryReleaseRepository, atomic CreateRelease, lookup, and distinct
  errors.
- `crates/omvcs-model/tests/release.rs` — model admission, closed-body,
  identity and golden-vector tests.
- `crates/omvcs-core/tests/release_operations.rs` — Core duplicate,
  uniqueness, atomicity, cross-Project and concurrent creation tests.

WORK-0011 documentation and directly required decisions:

- `Specs/OMVCS Core Specification.md` — §18 Release contract and
  overlapping-failure precedence.
- `Specs/OMVCS Core Invariants Specification.md` — INV-HIST-012.
- `Specs/OMVCS Glossary.md` — Release and Release Identifier.
- `Specs/OMVCS Interaction Specification.md` — Release workflow wording.
- `Specs/OMVCS Platform Protocol.md` — Release mirror/deletion boundary.
- `docs/decisions/ADR-0017-release-object-and-admission-contract.md`.
- `docs/decisions/ADR-0023-create-release-failure-precedence.md`.
- `docs/gaps/DG-0023-create-release-overlapping-failure-precedence.md`.
- `docs/gaps/DG-0017-release-object-and-admission-contract.md`.
- `docs/decision-register.md`.
- `docs/plans/WORK-0011-release-history-reference.md`.
- `docs/plans/WORK-0013-repository-reachability.md` — Release root dependency.
- `docs/spec-coverage.md` — WORK-0011 verified after independent acceptance.
- `docs/milestones.md` and `docs/project-state.md` — M2 remains IN PROGRESS;
  WORK-0011 verified but not integrated; WORK-0012–0014 unstarted.
- `docs/handovers/HANDOVER-WORK-0011-release-representation.md`.

The working tree at the start of WORK-0011 contained approved, uncommitted
ADR-0017 and cross-Spec changes on the Release-contract branch. Those were
preserved and included in the Release specification decision commit; no
unrelated changed paths were found during the worktree audit.

## Tests added or changed

- `omvcs-model/tests/release.rs`: exact seven-member body, unknown/null/
  missing/duplicate rejection for each required member, typed identifier
  namespaces, exact unique schema authority, invalid/valid timestamp forms
  including announced leap second, ActorId and description handling, target
  admission and Project matching without resource resolution, distinct
  mismatching Revision/Project State resolver identity errors, immutable body
  getters, ReleaseId format, all-seven-field identity sensitivity, and
  literal canonical JCS/hash vector.
- `omvcs-core/tests/release_operations.rs`: idempotent exact duplicate,
  exact Project-scoped name equality, case and Unicode distinction, isolated
  name conflict atomicity, reuse across Projects, missing Project and
  cross-Project target, exposed Release-to-Revision root edge, and concurrent
  name uniqueness.
- Core module tests: failed target resolution leaves no name claim; corrupted
  ID/body binding returns integrity failure; overlapping integrity/name
  condition returns integrity failure first and preserves both stored records
  unchanged.
- Verifier's two test-only findings were addressed: missing-member decoding is
  tested directly for all seven fields, and deliberately mismatching
  `AdmittedRevisionResolver` / `AdmittedProjectStateResolver` implementations
  prove the separate resolver-ID-mismatch errors. No implementation behavior
  was changed for those findings.
- No WORK-0011 Rust implementation or test contains forbidden unrelated
  reference-render behavior.

## Commands run

- `cargo fmt --all` — passed.
- `cargo fmt --all -- --check` — passed.
- `cargo test -p omvcs-model --test release --locked` — 11 passed.
- `cargo test -p omvcs-model --test revision --locked` — 18 passed.
- `cargo test -p omvcs-core release --locked` — 3 Core Release
  unit tests passed; other tests were filtered by the `release` name filter.
- `cargo test -p omvcs-core --test release_operations --locked` — 6 passed.
- `cargo test -p omvcs-core release --locked` also ran three matching
  integration tests; those are a subset of the complete six-test operation
  suite above. Distinct focused Release/Core/Revision tests total 38 passed
  (11 model, 18 Revision regression, 3 Core unit, 6 Core integration).
- `cargo test --workspace --locked` — passed during final closeout validation.
- `cargo test -p omvcs-model --test release --locked` — 11 passed.
- `cargo test -p omvcs-model --test revision --locked` — 18 passed.
- `cargo test -p omvcs-core --test release_operations --locked` — 6 passed.
- `cargo test -p omvcs-core release --locked` — 3 Release unit tests and
  3 matching integration tests passed, including the integrity-precedence
  overlap/atomicity test.
- `cargo fmt --all -- --check` — passed during final closeout validation.
- `cargo clippy --locked -p omvcs-model -p omvcs-core --all-targets -- -D warnings`
  — passed during final closeout validation.
- `git diff --check` — passed during final closeout validation.

## Semantic decisions made beyond the specification

None beyond the approved Specs. ADR-0023's exact `CreateRelease` overlap
decision is explicitly human-approved and reflected in Core §18. This
approval normalization does not represent a new implementation audit or
test run.

## Design Gaps discovered

- DG-0023 — resolved by accepted, human-approved ADR-0023 and the §18 update,
  limited to the exact simultaneous integrity/name-conflict conditions.
- Existing DG-0015, DG-0018, and DG-0019 remain OPEN and outside this package.
- DEC-CORE-002 remains OPEN and outside Release body, identity, preconditions,
  implementation, and tests.

## Assumptions

- `InMemoryReleaseRepository` is a mutex-backed reference boundary, not a
  durable persistence or transaction implementation.
- Resolver interfaces are trusted admission boundaries as in WORK-0008.
- The Core unit test injects an inconsistent in-memory identifier/body
  binding to test integrity handling; it does not claim that a SHA-256
  collision was produced.

## Known limitations

- No durable storage adapter, FFI, DAW integration, Release mutation/delete,
  reachability traversal, garbage collection, or Working State behavior was
  implemented.
- Independent Verifier acceptance is ACCEPT with no remaining actionable
  conformance findings.
- The Release coverage row is marked verified after acceptance.
- The closeout audit classified all changed paths as approved Release
  specification/decision work, WORK-0011 implementation/tests, or
  WORK-0011 verification/closeout; no unrelated or ambiguous-provenance
  changes were found.

## Remaining work

1. Integrate only after the project owner approves the integration step;
   no commit or push was made as part of this work.
2. Keep WORK-0012–0014 unstarted; WORK-0013 remains dependent on integration
   of WORK-0011 and WORK-0012.

## Git state

Working tree: CLEAN after the closeout commit sequence.
Remote push performed: NO as of this handover preparation.
Remote publishing: ENABLED per `docs/project-state.md`.

The closeout commits are grouped as Release specification decisions,
WORK-0011 implementation/tests, and WORK-0011 verification/package closeout.
No existing history was rewritten.
