# WORK-0009 — Complete Revision graph traversal

Status: PLANNED
Owner agent: Core Engineer
Milestone: M2
Branch: `work/0009-revision-graph-traversal`

## Objective

Provide read-only ancestry traversal and graph-integrity checks over a complete,
resolvable set of admitted Revisions within one Project.

## Normative requirements

- Core Specification §§14–15 and 56.
- Glossary: Revision, Parent Revision, and Revision Graph.
- Core Invariants: INV-HIST-003, INV-HIST-004, and INV-HIST-009.

## Dependencies

- WORK-0001 through WORK-0008 verified APIs, especially the admitted Revision
  model from WORK-0008.
- No open Design Gap blocks traversal over a complete, resolvable graph.
- DEC-CORE-009 remains open for shallow/incomplete history import policy. This
  package does not choose or implement that policy.
- DG-0015 remains open for M6 operation-specific provenance. It does not block
  generic Revision ancestry traversal.

## Allowed scope

- `crates/omvcs-core/`
- Focused Core tests for Revision graph traversal and integrity.

## Deliverables

- Read-only traversal of direct and transitive Revision ancestry through
  admitted parent references.
- Graph-integrity validation that enforces the specified directed acyclic,
  same-Project Revision graph over the supplied complete graph.
- Explicit rejection of cyclic or cross-Project graph fixtures without
  modifying Revision history.

## Acceptance tests

- An initial Revision with no parents has no ancestors.
- A single-parent chain returns the direct and transitive ancestors.
- A multi-parent Revision traverses the ancestry of every parent; parent
  ordering does not change the graph result.
- Repeated ancestors reached through multiple paths are represented once in
  the traversal result.
- Cyclic graph fixtures are rejected; traversal does not loop indefinitely.
- A parent or node from another Project is rejected from the graph.
- Ancestry is determined only by parent relationships, never by timestamps.
- Missing, unadmitted, or unavailable parent data is not silently interpreted
  as a complete graph. This package's graph API operates on complete,
  resolvable input and does not decide shallow/import semantics.
- Traversal and validation do not mutate Revisions or assign Line, Release,
  Working State, Contribution, storage, or publication meanings to graph nodes.

## Explicit non-goals

- Lines, Releases, Working State, repository-wide reachability, or garbage
  collection.
- Shallow/incomplete import policy or treatment of unavailable ancestors beyond
  requiring complete, resolvable input for this package.
- Contribution/integration provenance or other M6 behavior.
- Repository-wide `ValidateRepository` invocation/results.
- Persistent graph indexes, mutable refs, timestamps as ordering, or any Git
  branch/tag/reflog/merge-base semantics.

## Known Design Gaps

- DG-0015 is open for M6 operation-specific provenance and does not block this
  package.
- DEC-CORE-009 is open for shallow/incomplete history imports and is explicitly
  outside this package's complete-graph precondition.
- DG-0016–DG-0019 block other M2 features but do not affect this bounded graph
  traversal.

## Implementation plan

1. Inspect the existing admitted Revision and repository/Core APIs.
2. Implement traversal and graph-integrity checks only for complete,
   resolvable admitted Revision inputs.
3. Add conformance tests for roots, single and multiple parents, shared
   ancestors, cycles, same-Project enforcement, and timestamp independence.

## Verification requirements

The independent Verifier must compare traversal and graph validation against
the cited normative rules, attempt cyclic and cross-Project fixtures, verify
multi-parent behavior and non-semantic parent ordering, and confirm the
implementation does not add incomplete-history or Git-derived semantics.

## Completion criteria

Formatting, focused tests, coverage-map update, independent verification,
handover, and clean Git state.
