# WORK-0013 — Repository history reachability

Status: VERIFIED — independently accepted and integrated for partial Line/Release scope at `a67e1811ce0e7e15001f8517d379d7da607d3bdf`
Owner agent: Core Engineer
Milestone: M2
Branch: `work/0013-repository-reachability`

## Objective

Compute historical object reachability from retained Lines and admitted
Releases, without treating reachability as permission to delete data. This
package produces a partial root-set result, not complete repository
reachability or global unreachable-object classification.

## Normative requirements

- Core Specification §§56 and 62.
- Glossary: Reachability and Repository Metadata.
- Core Invariants: INV-GC-001–003 and INV-WORK-002–004, INV-WORK-006.
- Core §62's minimum-root list bounds the incompleteness of this package;
  WORK-0013 implements only the independently specified Line and Release
  root classes.

## Dependencies

- WORK-0009 complete Revision ancestry traversal.
- WORK-0010 Line root contract (verified).
- WORK-0011 Release root contract (verified and integrated).
- WORK-0012 is verified. ADR-0018 says Working State existence alone is not a
  root; ADR-0025 says `AdapterWorkingStateRef` is not a root. Core §62 also
  names explicit Working State safety references, but their exact set and
  edges are unresolved by DG-0027 and are excluded here.
- DG-0016–DG-0018 and DG-0024–DG-0026 are resolved as applicable to the
  root/persistence contracts before this package begins.
- ADR-0016, ADR-0020, and ADR-0022 resolve the Line record/root contract.
- ADR-0021 resolves the Default Line preference; it designates an existing
  Line and is not a separate historical root.
- DEC-CORE-005 and DEC-CORE-008 govern retention/deletion safety, not the
  reachability calculation; this package MUST NOT implement deletion or
  garbage collection.
- Contributions, configured archival pins, and pending publication
  transactions are required root classes under Core §62 but are outside this
  bounded package. Their omission means this package MUST NOT claim complete
  reachability or classify an object as globally unreachable.

## Allowed scope

- `crates/omvcs-core/`
- Focused reachability and reference-traversal tests.
- `docs/plans/WORK-0013-repository-reachability.md`
- `docs/spec-coverage.md`
- `docs/decision-register.md`
- `docs/gaps/DG-0027-working-state-safety-reference-roots.md`
- `docs/handovers/` for required package handover.
- `docs/milestones.md` and `docs/project-state.md` for milestone/status reconciliation.

## Deliverables

- A read-only calculation of objects reached from retained Line and admitted
  Release roots only.
- Traversal through the historical object references and Revision ancestry
  required by Core §62.
- Separation of metadata reachability from Resource-byte availability.
- An explicit partial-result boundary: objects not reached from these roots
  are not reported as globally unreachable.

## Acceptance tests

- Every retained Line and admitted Release is included according to its
  resolved object contract; the Default Line preference adds no duplicate
  root.
- Reachability follows the specified metadata references and Revision parent
  graph.
- A reachable historical object remains reachable even when its Resource
  bytes are not locally materialised.
- Resource availability does not change historical identity or reachability.
- Objects not reached by the included Line/Release roots are not reported as
  globally unreachable.
- No deletion, retention-period, archival, pin, or mutation behavior is
  performed.
- No Contribution, Working State, archival-pin, or pending-publication roots
  are inferred.

## Explicit non-goals

- Garbage collection, physical deletion, retention policy, or automatic
  Line-deletion pinning.
- Contribution, Working State safety-reference, configured archival-pin, and
  pending-publication-transaction roots.
- Complete repository reachability, global unreachable-object
  classification, import completeness, or storage Replica availability.
- Any root or edge not specified by the resolved OMVCS contracts.

## Known Design Gaps

- DG-0027 blocks Working State safety-reference root coverage. Only the
  independently resolved Line and Release roots are in scope.
- ADR-0017 resolves the Release root contract. ADR-0018 establishes that
  Working State existence alone is not a root; ADR-0025 confirms that
  `AdapterWorkingStateRef` itself is not a root.
- DG-0016 and DG-0021 are resolved by ADR-0016 and ADR-0021 respectively.
- DEC-CORE-005 and DEC-CORE-008 remain open for later deletion/retention
  behavior and are explicit non-goals here.
- Core §62 also requires Contributions, configured archival pins, and
  pending publication transactions as minimum roots. Those root classes are
  excluded by this bounded package, so its results are partial and cannot
  support global unreachable classification. DG-0015 does not remove the
  normative Contribution-root requirement.

## Implementation plan

1. Implement read-only enumeration/traversal for retained Lines and admitted
   Releases using WORK-0009 and the approved root resolvers.
2. Return reached and unresolved metadata identifiers without assigning
   global unreachable status.
3. Add conformance tests for included roots, metadata edges, and
   resource-sparse history.

## Implemented boundary

- `line::LineEnumerationBoundary::retained_lines` and
  `release::ReleaseEnumerationBoundary::admitted_releases` enumerate every
  included root across repository Projects. Their in-memory implementations
  clone the complete current set under their existing locks. They are separate
  read-only traits so existing mutation boundaries/implementations are unchanged.
  Enumeration failure returns an error, never an empty/truncated success.
- `reachability::partial_line_release_reachability` returns
  `PartialReachability`: sorted, deduplicated Line, Release, Revision,
  Project State, Component State, Adapter State, and Resource IDs, plus typed
  unresolved metadata references and separate identity/cycle defects.
- A referenced ID remains reached when its metadata is unresolved or its
  resolver returns the wrong identity. Unknown outgoing edges are not guessed;
  independent branches still continue. Resource IDs are terminal and never
  classified as unresolved because bytes are absent.
- Traversal follows Revision parents and Project States, Component mappings,
  asserted Component State parents and Resource references, and Adapter State
  Resource references. Unknown Component lineage is not inferred.
- `AdmittedAdapterStateResourceResolver` extends the existing admitted Adapter
  identity boundary with a trusted complete projection of Resource IDs for
  that exact admitted metadata object. Core §12 and DAW Adapter §§20–22
  specify these references and exact-schema admission; this API consumes those
  guarantees without parsing adapter-owned semantics. Missing projection is
  unresolved, not proof of zero edges; a wrong identity is a defect. No new
  Adapter body, admission implementation, wire format, or native behavior is
  defined here. Production projection implementations belong to later work.
- The traversal uses WORK-0009's iterative active-path ancestry strategy rather
  than invoking its fail-fast `revision_ancestors` API, which cannot retain
  independent branches after a missing parent. WORK-0009 remains unchanged.
- Root enumerations are individual reads; no cross-boundary atomic snapshot
  or publication transaction is introduced. The result is always partial,
  even with no unresolved IDs or defects. No globally-unreachable or
  repository-completeness classification exists.

## Implementation validation (2026-10-09)

- 28 focused tests pass: 25 behavior-named integration tests in
  `crates/omvcs-core/tests/repository_reachability.rs` and 3 unit tests in
  `reachability.rs` for synthetic Revision/Component cycles and a 10,000-node
  ancestry chain. Coverage includes Line movement changing subsequent
  reachability without mutating history, roots across Projects, convergence,
  historical Project State edges, omitted lineage, byte independence,
  unresolved metadata, exact identity defects, enumeration failure, stable
  ordering, no mutation, excluded roots, and the partial result shape.
- `cargo test --workspace --locked`: 246 passed (239 unit/integration tests
  and 7 doc tests), no failures.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy -p omvcs-core --all-targets --locked -- -D warnings`: passed.
- `git diff --check`: passed.
- Independent Verifier ACCEPT. Its initial review found one P2 omission: a
  Line-movement regression test. That test was added with divergent
  same-Project targets and immutable-history assertions; the Verifier
  confirmed the sole finding was cleared. No actionable findings remain.
- DG-0027 remains OPEN/BLOCKS-FEATURE for Working State safety-reference
  roots. Contributions, configured archival pins, and pending publication
  transactions also remain required Core §62 roots outside this package.
- The handover is
  `docs/handovers/HANDOVER-WORK-0013-repository-reachability.md`.
  WORK-0014 was not started.

## Verification requirements

The independent Verifier must trace every included root and edge to the
specifications, verify the partial-result boundary and explicit root
omissions, attempt to expose invented roots, and verify reachability never
authorizes deletion or depends on local Resource bytes.

## Completion criteria

Implementation formatting, focused/workspace tests, strict Clippy,
coverage-map reconciliation, independent verification, handover, and
integration are complete for this partial Line/Release scope. This does not
claim complete Core §62 reachability. WORK-0014 remains blocked and unstarted.
