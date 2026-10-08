# Handover WORK-0004 — Resource and Resource Reference model

From agent: OMVCS Core Engineer
To agent: Verifier; follow-up Core Engineer after DG-0009 resolution
Date: 2026-10-08
Branch: `work/0004-resource-model`
HEAD at handover authoring: `086708c` (`docs(WORK-0004): record properties validation gap`)

> **Status: PARTIAL / BLOCKED.** The unambiguous model and byte-length work is implemented and tested, but WORK-0004 is not complete or approved. DG-0009 is open and classified `BLOCKS-FEATURE`. Do not treat the Resource Reference properties admission boundary as conforming, and do not start WORK-0005+.

## Completed

- Added a typed `ResourceByteLength` constrained to `0..=9007199254740991`; JSON numeric values are checked by exact decimal arithmetic, including integer-valued decimal/exponent forms.
- Added an immutable in-memory `ResourceObject` that holds its complete raw bytes, derives its `ResourceId` from those bytes alone, and can create a Resource Reference with the corresponding exact byte count.
- Added a typed `ResourceReference` with required `ResourceId` and bounded complete-resource `byte_length`, optional `role`, `media_type`, and object-map `properties`; fields are private and optional fields are omitted when absent.
- Added exact resource matching by both Resource Identifier and byte count.
- Reject unknown generic Resource Reference fields, including names and physical storage/reconstruction fields.
- Preserve RFC 8785 map canonicalization and reject duplicate object member names on both canonicalization and typed Resource Reference JSON deserialization paths.
- Added acceptance tests and updated coverage to `implemented` only for bounded model requirements that are implemented. The separate properties semantic-admission requirement is `blocked (DG-0009)`. Nothing is marked `verified`.
- The independent Verifier found an unresolved properties validation/admission boundary. Spec Guardian created DG-0009 and updated the decision register. The independently specified duplicate-name issue was corrected after that review; it has not received independent re-review.

## Specifications implemented

The following requirements have implementation/test coverage, subject to the explicit DG-0009 exception below:

- Glossary: Resource, Resource Object, Resource Identifier, Resource Manifest, Friendly Name.
- Core Specification §§5.1, 6, 7 (required/reference-field shape and byte-length contract), 8 (complete-byte identity boundary only), and 56 (model-level reference validation only).
- Core Invariants INV-RES-001–004 and INV-PROJ-003 at the Resource model boundary. No operational Replica or storage lifecycle behavior was implemented.
- ADR-0001 and ADR-0005 for map ordering and duplicate-member rejection.
- ADR-0007 and ADR-0008 for the Resource Reference field boundary and exact length domain.
- Storage Adapter Specification §§16–18, 74, and 162 were read for the identity/storage separation; no Storage Adapter behavior was implemented.

**Blocked normative requirement:** Core §7 / ADR-0007 require `properties` to be approved immutable interpretation metadata and forbid using it to reintroduce excluded names or physical storage/reconstruction information. DG-0009 records that the Specs do not define the generic validation authority, evidence, or admission boundary. The current generic `with_properties`/decode path does not enforce that semantic rule.

## Files changed

- `crates/omvcs-model/Cargo.toml` — enable serde derive and raw JSON value support for typed JSON models and exact token validation.
- `Cargo.lock` — lockfile update for serde derive.
- `crates/omvcs-model/src/lib.rs` — expose the Resource model module.
- `crates/omvcs-model/src/resource.rs` — Resource Object, bounded byte length, Resource Reference, JSON handling.
- `crates/omvcs-model/src/canonical.rs` — crate-private raw JSON duplicate-member validation reused by typed Resource Reference ingestion.
- `crates/omvcs-model/tests/resource_model.rs` — focused Resource model/conformance tests.
- `docs/spec-coverage.md` — mark implemented rows as `implemented`, and the unresolved properties boundary as blocked.
- `docs/gaps/DG-0009-resource-reference-properties-validation.md` — open Design Gap, including the separate duplicate-name issue now corrected.
- `docs/decision-register.md` — record DG-0009.
- `docs/handovers/HANDOVER-WORK-0004-resource-model.md` — this handover.

## Tests added or changed

`crates/omvcs-model/tests/resource_model.rs` adds 10 tests covering:

- SHA-256 identity from complete immutable bytes and changed-byte identity.
- Required typed reference fields and byte count generated from a Resource Object.
- Exact accepted vectors `0`, `1`, `9007199254740991`; rejected vectors `-1`, `1.5`, `9007199254740992`, `"1"`; plus exact fractional/rounding cases and mathematically integral JSON number forms.
- Exact matching of Resource Object identity and byte count to a Resource Reference.
- Per-field containing-object identity changes for `resource_id`, `byte_length`, `role`, `media_type`, and `properties`, while descriptive changes leave the same-byte Resource Identifier unchanged.
- RFC 8785 `properties` map insertion-order invariance and duplicate-name rejection, including duplicate/escaped-equivalent names through direct typed deserialization.
- Rejection of generic name, filename, physical storage, Chunk, Replica, provider, credential, and Resource Manifest identifier fields.
- Separation of Resource References from external filename/location/provider/Replica/chunk-layout context.

No existing tests were weakened or removed.

## Commands run

- `cargo fmt --all` — passed.
- `cargo fmt --all -- --check` — passed after formatting.
- `cargo test -p omvcs-model --test resource_model` — passed before the final duplicate-ingestion test was added (9 tests).
- `cargo test -p omvcs-model` — final run passed: 7 unit tests, 16 canonical serialization tests, 8 content hashing tests, 10 Resource model tests; 41 total, 0 failures.
- `cargo clippy -p omvcs-model --all-targets -- -D warnings` — final run passed.
- `git diff --check` — passed for both focused commits.

Initial compile/Clippy attempts reported missing derive support and lint findings; these were corrected before the final successful runs.

## Semantic decisions made beyond the specification

`None`.

The implementation accepts JSON number tokens such as `1.0` and `1e0` when their exact mathematical value is an integer in range; Core §7 and ADR-0008 define validity by mathematical value, and the canonical output is governed by RFC 8785.

## Design Gaps discovered

- **DG-0009 — Resource Reference properties validation boundary** — OPEN, `BLOCKS-FEATURE`. The exclusion rule is specified, but the generic Core validation/admission authority and evidence for schema/Adapter-approved properties are not. See `docs/gaps/DG-0009-resource-reference-properties-validation.md`.
- DG-0007 and DG-0008 remain resolved by ADR-0007 and ADR-0008 respectively.

## Assumptions

- Raw JSON token access is an implementation mechanism for checking mathematical integer values and duplicate member names; it does not define additional OMVCS semantics.
- The model can only verify a reference's claimed byte count against bytes when the Resource Object is available. Historical references remain representable when the object is unavailable, consistent with historical identity/storage independence.

## Known limitations

- Do not claim or integrate WORK-0004 as complete while DG-0009 is open. In particular, the model currently accepts arbitrary property contents; it does not establish schema/Adapter provenance or enforce excluded-content semantics inside `properties`.
- The latest duplicate-map correction is tested but has not received independent Verifier re-review.
- `ResourceObject` is an in-memory value model. Streaming, transfer, chunking, storage, and physical reconstruction remain outside this package.
- No Resource Manifest historical object or identifier, Chunk/Chunk Manifest model, provider behavior, filename policy, or Adapter-specific naming rule was added.

## Remaining work

1. Obtain an explicit human decision and approved ADR/Spec update for DG-0009; do not choose a validator boundary or property vocabulary in code.
2. Update the bounded WORK-0004 plan/tests/coverage for that decision and implement only the approved property admission/validation contract.
3. Request independent Verifier review after resolving the remaining finding; only then may WORK-0004 be declared complete.
4. Do not begin WORK-0005 or any later package before WORK-0004 is resolved and independently verified.

No WORK-0005+ work was started.

## Git state

Working tree: CLEAN after the handover commit (verify with `git status --short --branch`).
Commits:
- `a073763` — `feat(model): add WORK-0004 resource model`
- `086708c` — `docs(WORK-0004): record properties validation gap`
- A separate handover-only commit follows this file.

Remote publishing: ENABLED in `docs/project-state.md`.
Remote push performed: NO.
