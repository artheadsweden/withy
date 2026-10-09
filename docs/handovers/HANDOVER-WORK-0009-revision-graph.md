# Handover WORK-0009 — Complete Revision graph traversal

From agent: OMVCS Core Engineer
To agent: Verifier
Date: 2026-10-09
Branch: `work/0009-revision-graph`
Base commit: `80a488aa56ad878d0acaab11714977ce0f07f77b`
WORK-0009 implementation commit: `bd64495`
Verification: Independently ACCEPTED by Verifier; no findings.

## Completed

- Implemented `omvcs_core::revision_graph::revision_ancestors`, a read-only
  traversal beginning with a typed `RevisionId` and following the complete
  direct/transitive parent closure through the WORK-0008 admitted-revision
  resolver.
- Returns each ancestor once and excludes the requested Revision. IDs are
  sorted for deterministic representation; their order has no ancestry
  significance.
- Returns unresolved metadata separately from invalid structure. Invalid
  structure currently covers cycles and resolver identifier mismatches.
- Same-Project parent validity relies on WORK-0008 admission. Resource bytes
  are not inspected, resolved, or required.
- Added focused integration and synthetic-helper tests; updated WORK-0009
  status/implementation notes and specification coverage.

## Specifications implemented

- Core Specification §§14–15, 55–56.
- Glossary: Revision, Parent Revision, Revision Graph.
- Core Invariants: INV-HIST-003, INV-HIST-004, INV-HIST-009.
- Related dependencies reviewed: ADR-0001, ADR-0014, and WORK-0008 admitted
  Revision APIs and tests.

## Files changed

- `Cargo.lock` — records the Core test-only `serde_json` dependency.
- `crates/omvcs-core/Cargo.toml` — test-only fixture dependency.
- `crates/omvcs-core/src/lib.rs` — exports the graph module.
- `crates/omvcs-core/src/revision_graph.rs` — graph traversal and graph-local
  outcome types.
- `crates/omvcs-core/tests/revision_graph.rs` — focused conformance scenarios.
- `docs/plans/WORK-0009-revision-graph-traversal.md` — implementation/status
  notes; pending independent acceptance.
- `docs/spec-coverage.md` — WORK-0009 status is `implemented`.
- `docs/handovers/HANDOVER-WORK-0009-revision-graph.md` — this handover.
- `docs/milestones.md` and `docs/project-state.md` were already modified in
  the preflight worktree when implementation began; those pre-existing edits
  were preserved and not further changed during implementation.

No Specs, ADRs, or Design Gap records were changed.

## Tests added or changed

`crates/omvcs-core/tests/revision_graph.rs` covers:

- initial Revision root returns no ancestors;
- direct and transitive ancestry independent of timestamp ordering;
- multiparent traversal and shared-ancestor deduplication;
- deterministic IDs across parent-array permutations and BTreeMap/HashMap
  resolver storage;
- unresolved parent metadata is returned as an error, not skipped;
- resolver ID mismatch is invalid structure, not unresolved metadata;
- cross-Project parent rejection at the WORK-0008 admission boundary;
- metadata-only traversal requires no Resource-byte resolver.

Private synthetic graph helper tests cover cycle detection and resolver ID
mismatch. The synthetic cycle fixture is expressly not an admitted Revision
and does not imply cyclic Revisions can be admitted.

## Commands run

- `cargo fmt --all` — passed.
- `cargo test -p omvcs-core --offline` — passed (2 unit tests, 7 integration
  tests; used once to update `Cargo.lock` for the test-only dependency).
- `cargo fmt --all -- --check` — passed.
- `cargo test --workspace --locked` — passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — passed.
- `git diff --check` — passed (Git emitted only Windows line-ending warnings).

No pre-existing test, formatting, or lint failures were observed.

The independent Verifier reran all four validation commands above and accepted
WORK-0009 without findings on 2026-10-09. Its review confirmed the complete
parent closure traversal, deterministic multi-parent results, separate
unresolved-metadata and invalid-structure outcomes, timestamp independence,
and no Resource-byte dependency.

## Semantic decisions made beyond the specification

`None`.

## Design Gaps discovered

`None` newly discovered.

DG-0015 remains OPEN and M6-only. DG-0016 through DG-0019 and
DEC-CORE-002/004/005/008/009 were not resolved or pre-empted. Missing parent
metadata is reported only as unresolved for this operation; this work does not
choose shallow/import policy or define repository-wide validation outcomes.

## Assumptions

- The supplied `AdmittedRevisionResolver` returns only WORK-0008 admitted,
  immutable Revision metadata. Core defensively verifies every returned
  Revision ID matches the requested typed ID.
- WORK-0008 admission already guarantees typed/unique parent references and
  same-Project parent validity; this traversal consumes those guarantees.
- Returning sorted IDs is only a stable representation choice, not a semantic
  parent/ancestry order.
- Resource availability is outside this operation and is not inferred from
  success or failure.

## Known limitations

- A missing/unadmitted parent is not classified as shallow history; it is
  unresolved, with policy left to the applicable future decision.
- The operation does not provide a repository-wide validation result model,
  Resource verification, graph persistence/indexing, mutation, or root-based
  reachability.
- Cross-Project links cannot be present in an admitted WORK-0008 Revision;
  the conformance test verifies rejection at admission rather than fabricating
  such a Revision for traversal.

## Remaining work

- Integrate through the normal no-fast-forward workflow; the independent gate
  has passed.
- Do not start WORK-0010 through WORK-0014 as part of this package.

## Git state

Working tree: CLEAN after WORK-0009 closeout commit. The preflight
`docs/milestones.md` and `docs/project-state.md` changes were committed in
M2 preflight commit `80a488aa56ad878d0acaab11714977ce0f07f77b`.

Remote push performed: NO for WORK-0009. Remote publishing is `ENABLED` in
project state.
