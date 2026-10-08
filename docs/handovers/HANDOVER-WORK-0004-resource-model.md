# Handover WORK-0004 — Resource and Resource Reference model

From agent: Core Engineer
To agent: Verifier
Date: 2026-10-08
Branch: `work/0004-resource-model`
HEAD: completion commit updating this handover; resolve with `git log -1 --format=%H -- docs/handovers/HANDOVER-WORK-0004-resource-model.md`.
Preparation HEAD: `0bb31e2` (normal integration merge; both prior WORK-0004 and the accepted M1 base are ancestors).
Accepted base: `983150eadfae597015e6c25ee9ae6023fcfe395d`, `spec/0003-canonical-collection-order`.

## Completed

WORK-0004 is implemented, **not independently verified**. This supersedes the prior partial/blocked handover now that ADR-0009 is accepted.

- Normally merged the accepted base, without rebase/history rewrite. Conflicts were limited to `docs/decision-register.md`, DG-0009, WORK-0004 planning, and coverage. Superseded OPEN/blocker text was resolved to accepted ADR-0009; prior implementation coverage was retained. Specs and ADR-0009 match the base exactly. WORK-0006/0007 plan changes are inherited from that merge only.
- Preserved immutable raw-byte Resource Object identity, bounded complete-resource byte length, and the prior recursive duplicate-member rejection fix.
- Unchecked `ResourceReferenceCandidate` supports JSON preservation; parsing checks duplicate names before any property-map decoding. Present null cannot masquerade as omission for properties, role, or media type.
- `ResourceReference` is validated-for-admission. No properties needs only generic Core validation; any property object, even `{}`, requires one available exact-context authority, schema-directed shape/collection normalization, and successful semantic validation.
- Unknown/ambiguous binding (`None`), unavailable/unrelated/latest-version authority, and non-unique matching authorities cannot produce an admitted reference. Rejection reasons propagate. Failure retains the unchecked candidate.
- Historical output is explicitly fallible and context-checked. Admitted references have no unconditional `Serialize` or direct `Deserialize`; candidates have no historical-output API. Replacing properties revokes admission.
- Context/evidence is operational and excluded from canonical historical fields. Normalized properties are immutable through shared access, affect containing identity, and use RFC 8785 map ordering.

## Specifications implemented

- Glossary: Resource, Resource Object, Resource Identifier, Resource Reference/candidate, Resource Manifest, Friendly Name.
- Core §§5–5.1: canonical maps, unique names, schema-directed recursive shapes and classified ordered/set-like arrays.
- Core §§6–8: complete raw-byte identity, embedded Resource Reference fields, ADR-0008 range, contextual admission, and physical chunk/storage exclusion.
- Core §§10, 12–13, 56, 76–77: Resource Reference boundary for containing historical use and exact versioned schema/Adapter context. No containing-object models or repository-validation operations implemented in this package.
- INV-RES-001–008, INV-PROJ-003, INV-DAW-004 at the model boundary: immutable/content-derived identity, operational independence, checked admission, and separation of generic Core mechanics from Adapter meanings. No Replica lifecycle behavior implemented.
- ADR-0001, ADR-0005, ADR-0007, ADR-0008, ADR-0009.

## Files changed

Focused completion changes after the merge:

- `crates/omvcs-model/src/resource.rs`
- `crates/omvcs-model/tests/resource_model.rs`
- `docs/plans/WORK-0004-resource-model.md`
- `docs/spec-coverage.md`
- `docs/handovers/HANDOVER-WORK-0004-resource-model.md`

The integration merge also brought the accepted Specs/ADR/gap/register and dependent plan updates from the published base. No Specs were edited during implementation.

Earlier WORK-0004 changes preserved: serde derive/raw-value configuration in `crates/omvcs-model/Cargo.toml` and `Cargo.lock`, Resource module exposure in `src/lib.rs`, and the crate-private raw duplicate-name check in `src/canonical.rs`. Hashing/identifier implementation is unchanged. No dependencies were added by this completion.

## Tests added or changed

- 18 Resource model tests: immutable/equal-byte identity; presentation/physical-context exclusion; exact required fields and ID namespace; integer endpoints, host-u64 values outside the normative domain, over-precision fractional and alternate encodings; present optional fields cannot silently disappear; contextual admission including empty properties; exact authority rejection; unknown/unavailable/non-unique authority and unchecked preservation; containing-schema/Adapter identity/version mismatches; property replacement revokes validity; nested ordered/set-like arrays, duplicate sets, shape/unclassified rejection; normalized callback input; successful map-order invariance; present field/value identity changes; evidence absent from bytes.
- Raw duplicate-name cases cover top-level names, properties names, nested maps, escaped-equivalent names, and objects inside nested arrays. Rejection precedes map overwriting. Existing canonicalization regression tests were not weakened or removed.
- Four compile-fail doctests enforce the API boundary: candidate lacks historical bytes, candidate cannot be passed as admitted, admitted reference cannot serialize unconditionally, and arbitrary JSON cannot decode directly as admitted.
- Semantic validators are synthetic test authorities only. Rejection fixtures exercise nested/alternate-key inputs without introducing real Adapter property rules or a Core deny list.

## Commands run

- `git switch work/0004-resource-model`
- `git merge --no-ff --no-commit 983150eadfae597015e6c25ee9ae6023fcfe395d`: four documentation conflicts, resolved as above; committed normal merge `0bb31e2`.
- VS Code focused test discovery returned no tests; used Cargo as the test runner.
- `cargo fmt --all`: passed.
- `cargo test -p omvcs-model --test resource_model --test canonical_serialization`: final run, 18 Resource + 16 canonical tests passed.
- `cargo fmt --all -- --check`: passed.
- `cargo test -p omvcs-model`: final run, 7 unit/identifier + 16 canonical + 8 hashing + 18 Resource tests, and 4 compile-fail doctests passed (53 total, no failures).
- `cargo clippy -p omvcs-model --all-targets -- -D warnings`: passed with no warnings.
- Editor Problems check for touched Rust files: no diagnostics.
- `git diff --check` and focused diff review: no whitespace errors or unrelated completion changes.
- `git diff 983150e -- Specs docs/decisions/ADR-0009-resource-reference-properties-admission.md`: no differences.

An intermediate run caught an accidentally nested test declaration via warnings-denied Clippy. It was moved to module scope and all final commands above were rerun successfully.

## Semantic decisions made beyond the specification

None. Distinct Rust candidate/admitted types, borrowed authority callbacks, and operational context storage are implementation mechanisms explicitly permitted by ADR-0009, not normative additions.

## Design Gaps discovered

None in this completion. Previously discovered DG-0009 is now resolved by accepted ADR-0009; DG-0007/0008 remain resolved by ADR-0007/0008.

## Assumptions

- The containing versioned schema/Adapter contract supplies its exact binding and available authorities. A missing or ambiguous binding is represented as `None`; multiple matches fail explicitly. Core never guesses a binding, chooses a latest version, or interprets keys.
- A schema/Adapter authority implements its own semantic contract faithfully, including ADR-0007 exclusions. Core enforces structural/canonical mechanics and propagates rejection; it cannot independently determine semantic meaning.
- Mathematically integral JSON-number spellings remain supported by the prior exact-decimal parser, as required by ADR-0008's mathematical-value contract; strings/tagged values are not alternate accepted encodings.
- Raw hashing/canonicalization utilities from prior packages remain low-level primitives, not assertions of historical validity. Future containing-object admission APIs must consume validated references through context-checked output, not treat candidate transport serialization as validated history.

## Known limitations

- No real Adapter vocabularies, validator registry/discovery, containing Component/Adapter/Project/Revision models, history commits, storage, chunking, or materialization APIs are part of WORK-0004.
- Resource Objects own an in-memory byte buffer; this does not prescribe storage/streaming mechanics.
- Deep byte verification is optional/separate from reference decoding; `matches_resource` checks exact identity and complete byte count when bytes are available. A missing Resource does not erase the historical reference.
- Context evidence is checked operationally during historical output but never becomes a Resource Reference schema field or independent property-schema identifier.

## Remaining work

Independent Verifier must attempt to disprove the admission boundary, exact-context/version checks, byte-length domain, duplicate-member rejection, recursive normalization, identity invariance/exclusions, and absence of evidence in historical bytes. In particular attempt unchecked serialization, wrong-context use, alternate/nested semantic rejection, and ambiguous authority.

Only the Verifier may mark WORK-0004 verified. Real versioned schemas and Adapter conformance suites must enforce their own semantic exclusions when assigned. WORK-0005 and later were not begun and must wait for independent acceptance as required by the integrated planning state.

## Git state

Working tree: CLEAN after committing this focused completion.
Remote push performed: NO.
Remote publishing: ENABLED in project-state, but this task requests clean local delivery; no push needed or performed.
