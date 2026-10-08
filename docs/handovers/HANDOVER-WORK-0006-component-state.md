# Handover WORK-0006 — Component State model

From agent: Core Engineer
To agent: Verifier
Date: 2026-10-08
Branch: `work/0006-component-state`
HEAD: the single WORK-0006 implementation-and-handover commit at the tip of this branch (full SHA is in the completion report)

## Completed

Implemented the immutable Component State model and its unchecked-candidate to admitted-history boundary in `omvcs-model`. Candidates require the closed OMVCS 0.1 body; admission requires one exact versioned Component State schema authority and, for every property-bearing Resource Reference, one exact contextual validator. Only the admitted type exposes a typed Component State ID and canonical body bytes.

The admitted body is built from exactly `schema`, `component_id`, `resources`, `metadata`, and `parents` only when present. Metadata and Resource Reference nested collections are normalized under their owning exact schemas before the final RFC 8785 body serialization. Validation context/evidence is not part of the body or identifier.

The WORK-0006 coverage row is `implemented`, not `verified`. No WORK-0007 or WORK-0008 work was started.

## Specifications implemented

- Component State Glossary entry.
- Core Specification §§5.1, 7, 10–11; exact schema admission and evolution rules in §§56, 76–77.
- Core Specification §23 was consulted for known-origin context; no Custom Working State or separate provenance model was added.
- Core Invariants: INV-HIST-001 and INV-HIST-006; INV-RES-004 and INV-RES-008; the Component identity boundary in INV-PROJ-002; known-lineage behavior in INV-PROV-003. Revision/Project State behavior from INV-HIST-002–003 is not implemented by this package.
- ADR-0011; existing rules from ADR-0001, ADR-0003, ADR-0005, ADR-0007, ADR-0008, ADR-0009, and ADR-0010.

## Files changed

- `crates/omvcs-model/src/component_state.rs` — schema-bound candidate admission, immutable admitted object, canonical body, typed ID and errors.
- `crates/omvcs-model/src/canonical.rs` — crate-private RFC 8785 composition function for already schema-validated and normalized body values.
- `crates/omvcs-model/src/lib.rs` — public module and admitted type export.
- `crates/omvcs-model/tests/component_state.rs` — direct Component State conformance tests and compile-fail API-boundary doctests.
- `docs/plans/WORK-0006-component-state-model.md` — status changed from `PLANNED` to `IMPLEMENTED`.
- `docs/spec-coverage.md` — WORK-0006 status changed to `implemented`; the row identifies the new test suite.
- `docs/handovers/HANDOVER-WORK-0006-component-state.md` — this handover.

## Tests added or changed

Added 21 Component State tests covering:

- required `schema`, typed `component_id`, `resources`, and `metadata`; empty required collections; omitted required fields; closed top-level field set and JSON-object-only decoding;
- preservation of a typed unknown-schema candidate without admission, unavailable/ambiguous schema authorities, schema-owned required metadata keys, key and shape rejection, nested array classifications, and unclassified-array rejection;
- omitted versus empty `parents`, known parents, parent/resource permutation invariance and duplicate rejection;
- exact contextual admission of property-bearing references, empty `properties`, unknown/unavailable/non-unique/mismatched property authority, semantic rejection, nested property arrays, and exact byte-length vectors;
- metadata and properties map insertion-order invariance and recursive duplicate JSON member rejection;
- identity equality for identical bodies, changed IDs for changed present Component State and Resource Reference fields (including `schema`), exact body-only canonical bytes, absence of operational/evidence/wrapper fields, and typed ID namespace separation;
- compile-fail guarantees that an unchecked candidate cannot yield an identity and an admitted state cannot be mutated through its public API.

Existing canonical serialization, hashing, Creative Component, and Resource model tests were retained unchanged.

## Commands run

- `cargo fmt --package omvcs-model` — passed.
- `cargo fmt --package omvcs-model -- --check` — passed.
- `cargo test --locked -p omvcs-model --test component_state` — final run passed, 21 tests.
- `cargo test --locked -p omvcs-model` — passed: 7 unit, 16 canonical serialization, 21 Component State, 8 content hashing, 6 Creative Component, 3 Creative Component acceptance, 18 Resource model, and 6 compile-fail doctests (85 total).
- `cargo clippy --locked -p omvcs-model --all-targets -- -D warnings` — final run passed.
- `git diff --check` and `git diff --cached --check` — passed.

The first targeted run exposed a test-fixture name-shadowing error, and an initial Clippy run reported test-only lint issues. Both were corrected; all final commands above passed. No production test was weakened.

## Semantic decisions made beyond the specification

`None`. The implementation follows the approved ADRs and Specs only.

## Design Gaps discovered

No new gap. DG-0011 is resolved by ADR-0011. No open Design Gap affecting WORK-0006 was found during preflight.

## Assumptions

- The caller supplies the available exact-version Component State schema authorities and Resource property validators. Matching is exact; unknown, unavailable, or non-unique authority prevents admission.
- The Specs intentionally do not define a universal metadata vocabulary for Component State. No built-in metadata key meanings or schema-version behavior were invented; concrete versioned schemas provide their own structure and semantic validator.

These are implementation boundaries, not additional historical semantics.

## Known limitations

- This model crate contains no built-in production Component State metadata vocabulary. It provides the exact-version authority boundary and exercises it with synthetic test schemas.
- Operation-specific mandatory parentage/provenance behavior remains outside this package.
- Project State, Revision, DAW Adapter, storage, FFI, and repository-operation models are not included.

## Remaining work

- Independent Verifier review is required before integration.
- WORK-0007 and WORK-0008 remain unstarted.

## Git state

Working tree: CLEAN after the completion commit.
Remote push performed: NO.
Remote publishing enabled: YES in `docs/project-state.md`; no push or integration was requested or performed.
