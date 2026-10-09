# WORK-0013 — Repository history reachability

Status: BLOCKED — WORK-0010, WORK-0011, and WORK-0012
Owner agent: Core Engineer
Milestone: M2
Branch: `work/0013-repository-reachability`

## Objective

Compute historical object reachability from the OMVCS roots defined by the
specifications, without treating reachability as permission to delete data.

## Normative requirements

- Core Specification §§56 and 62.
- Glossary: Reachability and Repository Metadata.
- Core Invariants: INV-GC-001–003 and INV-WORK-002–004.

## Dependencies

- WORK-0009 complete Revision ancestry traversal.
- WORK-0010 Line root contract.
- WORK-0011 Release root contract.
- WORK-0012 Working State root/persistence contract.
- DG-0016–DG-0018 must be resolved as applicable to the root contracts before
  this package begins.
- ADR-0016, ADR-0020, and ADR-0022 resolve the Line record/root contract.
- ADR-0021 resolves the Default Line preference; it designates an existing
  Line and is not a separate historical root.
- DEC-CORE-005 and DEC-CORE-008 govern retention/deletion safety, not the
  reachability calculation; this package MUST NOT implement deletion or
  garbage collection.
- Contributions are excluded; DG-0015 remains M6 and no Contribution roots
  are added by this package.

## Allowed scope

- `crates/omvcs-core/`
- Focused reachability and reference-traversal tests.

## Deliverables

- A read-only reachability calculation from the approved Line, Release, and
  Working State roots.
- Traversal through the historical object references and Revision ancestry
  required by Core §62.
- Separation of metadata reachability from Resource-byte availability.

## Acceptance tests

- Every approved root class is included according to its resolved object
  contract.
- Reachability follows the specified metadata references and Revision parent
  graph.
- A reachable historical object remains reachable even when its Resource
  bytes are not locally materialised.
- Resource availability does not change historical identity or reachability.
- Unreachable objects are reported as unreachable only; no deletion,
  retention-period, archival, or pin behavior is performed.
- No Contribution roots or unspecified references are inferred.

## Explicit non-goals

- Garbage collection, physical deletion, retention policy, or automatic
  Line-deletion pinning.
- Contribution reachability, import completeness, or storage Replica
  availability.
- Any root or edge not specified by the resolved OMVCS contracts.

## Known Design Gaps

- DG-0017–DG-0018 block the complete Release and Working State root
  contracts.
- DG-0016 and DG-0021 are resolved by ADR-0016 and ADR-0021 respectively.
- DEC-CORE-005 and DEC-CORE-008 remain open for later deletion/retention
  behavior and are explicit non-goals here.
- DG-0015 blocks M6 Contribution provenance; Contributions are excluded.

## Implementation plan

1. Revalidate the exact roots and edges against the resolved object contracts.
2. Implement read-only traversal using WORK-0009 and the approved root
   resolvers.
3. Add conformance tests for each root class, metadata edges, and
   resource-sparse history.

## Verification requirements

The independent Verifier must trace every included root and edge to the
specifications, attempt to expose omissions or invented roots, and verify
reachability never authorizes deletion or depends on local Resource bytes.

## Completion criteria

Formatting, focused tests, coverage-map update, independent verification,
handover, and clean Git state.
