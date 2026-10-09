# WORK-0014 — Repository validation

Status: VERIFIED for the original partial Line/Release validation scope at `3f37fa1a981fa06408616b080edb6c56355bc1b0`; ADR-0030/0031 root-coverage extension planned
Owner agent: Core Engineer
Milestone: M2
Branch: `work/0014-repository-validation`

The status and implementation details through the original completion
criteria below record the accepted Line/Release-only validation scope. The
ADR-0030 extension status at the end of this file is not implemented.

## Objective

Implement the approved strictly read-only repository-validation operation,
preserving the distinction between metadata integrity, history completeness,
Resource state, and requested-scope/provider coverage.

## Normative requirements

- Core Specification §§4, 10, 13–14, 18, 21–22, 29–30, 47–56, 62–67,
  76–77, and 82–83.
- Glossary: OMVCS Repository, Content-derived Identifier, Repository
  Metadata, Historical Metadata, Operational Metadata, Declared History
  Boundary, History Completeness, Metadata Integrity, Validation Coverage,
  Resource Verification Depth, Availability State, Corrupt Replica,
  Reachability, and Repository Recovery.
- Core Invariants: INV-HIST-003, INV-HIST-008–009, INV-RES-007,
  INV-WORK-002–004, INV-WORK-007, INV-INT-001–005, INV-REC-002 and INV-REC-006,
  INV-GC-001–003.
- ADR-0027, ADR-0028, ADR-0029, and ADR-0030.

## Dependencies

- WORK-0006 through WORK-0013 provide historical object models, admission
  rules, graph traversal, roots, and reachability; all are verified and
  integrated prerequisites.
- DG-0019, DEC-CORE-004, and DEC-CORE-009 are resolved by ADR-0027–0029 and
  the corresponding Spec updates.
- The WORK-0013 Line/Release reachability API is deliberately partial; its
  coverage MUST remain identified as partial.
- Resource retention/deletion decisions DEC-CORE-005 and DEC-CORE-008 are not
  selected or implemented here.
- DG-0015 remains open and Contributions remain outside this package. DG-0027
  is resolved by ADR-0030 and DG-0028 by ADR-0031; the original
  implementation predates both decisions.

## Allowed scope

- `crates/omvcs-core/`
- Targeted `crates/omvcs-model/` changes only to separate exact-schema,
  canonical-body Identifier calculation/verification from reference
  resolution and historical admission for metadata candidates.
- Focused repository validation models and tests.

## Deliverables

- A strictly read-only `ValidateRepository` operation with explicit
  Repository/Project scope and Resource-verification depth.
- Machine-readable result dimensions for metadata integrity, history
  completeness, Resource state, provider/root coverage, and typed findings.
- Provider-neutral object/root enumeration, exact declared-boundary lookup,
  and Resource-verification request boundaries, with explicit provider
  failures.
- Body-derived content Identifier verification independent of local
  reference resolution, while preserving existing strict historical
  admission requirements.
- Reuse of the verified object-level admission and identity rules.

## Acceptance tests

- Tests cover both scopes and all verification depths; strict read-only
  behavior; report state aggregation; typed findings; invocation errors
  versus completed reports; and explicit partial/unavailable provider
  coverage.
- Canonical hashes, schemas, and required metadata references are checked
  according to their existing admission contracts.
- Exact matching and non-matching declared-boundary tuples are tested.
  Declared boundaries do not resolve or admit absent targets; a later
  available target is validated normally.
- Canonically identical metadata bodies retain the same valid
  content-derived Identifier whether or not referenced targets resolve
  locally; absent targets still prevent admission. Unknown/unavailable
  schemas and invalid bodies do not produce valid Identifiers.
- Missing Resource bytes alone do not invalidate otherwise valid historical
  metadata.
- Available Resource bytes are verified only at the explicit requested
  depth; corrupt, unavailable, and not-checked are distinct outcomes.
- Unsupported Core §62 providers produce honest partial/unavailable
  coverage. No global unreachable result is exposed.
- Provider errors are never converted to empty success-shaped results.
- No repair, mutation, automatic Resource fetch/materialisation, root
  semantics, or persistence format is invented.
- Validation does not treat a metadata-complete/resource-sparse repository as
  corrupt solely because Resource bytes are not locally materialised.

## Original implementation non-goals

- Repository repair or recovery transactions.
- Implementing durable metadata-history completeness or boundary persistence
  and wire encoding.
- Changing historical object schemas, hash preimages, or existing
  reference/admission requirements.
- Resource garbage collection, retention, or Line deletion pinning.
- Storage-provider-specific verification beyond the approved Core result
  contract.
- Resolving Contribution semantics. The original implementation excluded
  Working State safety-reference roots; ADR-0030 defines them for the
  pending extension.

## Known Design Gaps

- DG-0015 leaves Contribution roots outside scope. ADR-0031 requires missing
  Working State safety-root targets to be `unresolved`, without boundary
  lookup. DEC-CORE-005/008 and DEC-INTERACTION-004 remain outside the
  original implementation scope.

## Implementation plan

1. Implement the provider-neutral Core interfaces and report model from
   ADR-0029 without defining persistence mechanics.
2. Expose exact-schema/canonical-body Identifier calculation before
   reference admission for metadata candidates; keep all existing
   admission checks intact.
3. Use the explicitly partial WORK-0013 Line/Release reachability result and
   surface unavailable root providers.
4. Add focused conformance tests for identity/admission separation, scope,
   read-only effects, all status dimensions, boundary lookup, Resource
   depths, provider failures, and honest partial coverage.

## Verification requirements

The independent Verifier must inspect operation effects and all result
categories, distinguish metadata integrity from history completeness and
Resource state, verify strict admission of missing-target objects, and
attempt to expose inferred repair, root, persistence, or incompleteness
policy.

## Completion criteria

Formatting, focused and workspace tests, coverage-map update, independent
verification, handover, integration, and clean Git state are complete. The
initial implementation retains explicitly partial coverage for unsupported
Core §62 root providers and does not claim global unreachability.

## ADR-0030 extension status — Working State root coverage

The verified integration recorded above covers the then-supported partial
Line/Release reachability result. ADR-0030 now defines Working State
safety-reference roots, and WORK-0014 must be extended after WORK-0013 adds
those roots:

- report coverage for Line, Release, and Working State safety-reference
  roots;
- continue to report Contributions, configured archival pins, and pending
  publication transactions as unsupported;
- preserve explicitly partial Core §62 reachability and do not infer global
  unreachability.

DG-0028 is resolved by ADR-0031. Missing Working State safety-root targets
are `unresolved`; do not infer a referring Identifier or edge kind, issue a
declared-boundary lookup, or report `declared_incomplete`. WORK-0014's
extension remains pending implementation and independent verification. Its
existing verified Line/Release validation remains accepted within that
original scope.

Required extension tests include:

- report Working State root-provider coverage as supported after the
  WORK-0013 extension;
- keep unsupported later root classes visibly partial;
- ensure missing Working State safety-reference targets remain unresolved
  and do not trigger declared-boundary lookups;
- ensure validation remains read-only and never repairs, fetches, or mutates
  Working State or historical objects.
