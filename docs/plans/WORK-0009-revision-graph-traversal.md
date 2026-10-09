# WORK-0009 — Complete Revision graph traversal

Status: VERIFIED — independently accepted
Owner agent: Core Engineer
Milestone: M2
Branch: `work/0009-revision-graph`

## Objective

Provide read-only ancestry traversal and graph-integrity checks over admitted
Revisions within one Project. Keep graph-structure validity, traversal
resolvability, and Resource availability distinct.

## Normative requirements

- Core Specification §§14–15, 55–56.
- Glossary: Revision, Parent Revision, and Revision Graph.
- Core Invariants: INV-HIST-003, INV-HIST-004, and INV-HIST-009.

## Dependencies

- WORK-0001 through WORK-0008 verified APIs, especially the admitted Revision
  model from WORK-0008.
- No open Design Gap blocks traversal over a complete, resolvable graph.
- DEC-CORE-009 was open during this package's planning; this package did not
  choose or implement shallow/incomplete history import policy. It was later
  resolved by human-approved ADR-0028.
- DG-0015 remains open for M6 operation-specific provenance. It does not block
  generic Revision ancestry traversal.

## Allowed scope

- `crates/omvcs-core/`
- Focused Core tests for Revision graph traversal and integrity.

## Deliverables

- Read-only traversal of direct and transitive Revision ancestry through
  admitted parent references, starting from a typed Revision Identifier and
  confirming that each resolver result's Revision Identifier matches the
  requested identifier.
- Graph-integrity validation that enforces the specified directed acyclic,
  same-Project Revision graph over the requested ancestry closure when its
  required Revision metadata is resolvable.
- Graph-local outcomes that distinguish invalid graph structure from
  unresolved parent metadata, without defining repository-wide validation
  results.
- Ancestry results determined from Revision metadata only; Resource-byte
  availability is not consulted.

## Acceptance tests

- Resolution begins with a typed Revision Identifier; each resolved object
  must report the same Revision Identifier. Parent identifier typing,
  duplicate-parent rejection, and same-Project parent admission remain
  guaranteed by WORK-0008 and are not redefined here.
- An initial Revision with no parents has no ancestors.
- A single-parent chain returns the direct and transitive ancestors.
- A multi-parent Revision traverses the ancestry of every parent; parent
  ordering does not change the graph result.
- Repeated ancestors reached through multiple paths are represented once in
  the traversal result.
- A stable result for a given graph is independent of resolver/map iteration
  order and set-like parent input order; result ordering does not imply
  ancestry semantics.
- Cyclic graph structure is rejected and is not treated as an incomplete
  traversal; any test-only synthetic graph fixture does not imply that
  cyclic Revisions can be admitted.
- A parent or node from another Project is rejected from the graph. WORK-0008
  admission already rejects cross-Project parent references.
- Ancestry is determined only by parent relationships, never by timestamps.
- A missing, unadmitted, or unavailable parent is reported as unresolved
  metadata, distinct from a detected graph-structure defect; it is never
  silently skipped or treated as proof that the graph is complete. This does
  not decide shallow/import policy.
- Resource-byte availability is not part of graph structure or traversal
  completeness; traversal succeeds from admitted metadata without requiring
  Resource-byte materialisation.
- The operation keeps three concerns separate: detected graph-structure
  validity, completeness of resolving the requested ancestry, and Resource
  availability. Resource availability is not evaluated by this operation.
- Traversal and validation do not mutate Revisions or assign Line, Release,
  Working State, Contribution, storage, or publication meanings to graph nodes.

## Explicit non-goals

- Lines, Releases, Working State, repository-wide reachability, or garbage
  collection.
- Shallow/incomplete import policy (DEC-CORE-009), or repository-wide
  classification of missing history (DEC-CORE-004).
- Contribution/integration provenance or other M6 behavior.
- Repository-wide `ValidateRepository` invocation/results (DG-0019).
- Persistent graph indexes, mutable refs, timestamps as ordering, or any Git
  branch/tag/reflog/merge-base semantics.

## Known Design Gaps

- DG-0015 is open for M6 operation-specific provenance and does not block this
  package.
- At package planning, DEC-CORE-009 and DG-0016–DG-0019 tracked separate M2
  questions/features and did not affect this bounded graph traversal.
- DEC-CORE-009 was later resolved by ADR-0028; DG-0019 was later resolved by
  ADR-0029.

## Implementation plan

1. Inspect the existing admitted Revision and repository/Core APIs.
2. Implement traversal and graph-integrity checks only for complete,
   resolvable admitted Revision inputs.
3. Add conformance tests for roots, single and multiple parents, shared
   ancestors, graph-local structure failures, unresolved parent metadata,
   same-Project enforcement, deterministic results, and timestamp
   independence. Confirm Resource-byte availability does not affect
   traversal.

## Verification requirements

The independent Verifier must compare traversal and graph validation against
the cited normative rules, attempt cyclic and cross-Project fixtures, verify
multi-parent behavior and non-semantic parent ordering, and confirm that graph
structure, traversal resolvability, and Resource-byte availability remain
distinct. The implementation must not add incomplete-history or Git-derived
semantics.

## Completion criteria

Formatting, focused and workspace tests, warnings-denied Clippy, coverage-map
update, independent verification, handover, and clean Git state.

## Implementation status

- Added `omvcs_core::revision_graph::revision_ancestors`, a read-only
  operation that starts from a typed `RevisionId` and resolves only admitted
  Revision metadata through the WORK-0008 resolver boundary.
- The result contains each direct and transitive ancestor once, excluding the
  requested Revision. Identifiers are sorted for stable representation; this
  ordering has no ancestry meaning.
- The operation distinguishes unresolved Revision metadata from detected
  invalid graph structure (cycles and resolver identifier mismatches).
  Same-Project parent validity relies on the WORK-0008 admission guarantee.
- The operation does not inspect timestamps or Resource bytes and has no
  Resource resolver/materialization dependency.
- Added Core integration coverage for root and transitive ancestry,
  multiparent/shared-ancestor traversal, parent/map order independence,
  timestamps, metadata-only traversal, unresolved parents, resolver ID
  mismatch, and cross-Project admission rejection. A private synthetic graph
  fixture covers cycle detection without implying cyclic Revisions can be
  admitted.
- Independent Verifier accepted the implementation with no findings.
- `docs/spec-coverage.md` marks WORK-0009 `verified`.
- No Specs, Design Gaps, ADRs, or semantic decisions were added or changed.
