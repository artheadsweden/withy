# Handover WORK-0006 — Component State model

From agent: Core Engineer
To agent: Verifier
Date: 2026-10-08
Branch: `work/0006-component-state`
Implementation HEAD: `298e5d0198e9b125d8f9b258800ed4f2620ec196`
Accepted specification base: `0ff182ab005fff7d58cebdea73dcb36e82c33b42`

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

- Independent Verifier acceptance after remediation is recorded below.
  W6-V001 is resolved; no WORK-0006 verification blocker remains.
- WORK-0007 and WORK-0008 remain unstarted.

## Git state

Working tree: CLEAN after the completion commit.
Remote push performed: NO.
Remote publishing enabled: YES in `docs/project-state.md`; no push or integration was requested or performed.

## Independent Verifier gate — 2026-10-08

From agent: Verifier
To agent: Core Engineer / OMVCS Lead
Branch: `work/0006-component-state`
Reviewed implementation: `298e5d0198e9b125d8f9b258800ed4f2620ec196`
Reviewed base: `0ff182ab005fff7d58cebdea73dcb36e82c33b42`
Result: **REJECTED — W6-V001 remains blocking**.

### Authority and scope inspected

Independently read AGENTS, this plan/handover, resolved DG-0011, ADR-0011 and
ADR-0001/0003/0005/0007/0008/0009/0010, Core §§5.1, 7, 10–11, 23, 56, 76–77,
Glossary Component State, INV-HIST-001–003/006, INV-PROJ-002, INV-PROV-003,
INV-RES-004/008, INV-DAW-004 and relevant DAW Adapter §§20, 27–28. Searched the
complete Spec set for cross-spec requirements and inspected WORK-0001–0005
public APIs, verified tests and handovers, and every WORK-0006 changed file plus
base-to-implementation history.

The implementation delta is one commit and seven scoped files. Only a
crate-private prevalidated JCS composition helper was added to the canonical
foundation; identifiers, hashing, Resource and Creative Component APIs were not
changed. No Project State, Revision, Adapter State, repository/storage,
clone/fork or later-package behavior was implemented.

### Blocking finding W6-V001 — positional Resource References enter history

Core §§7, 10 and ADR-0007/0011 require embedded Resource References with named
`resource_id` and `byte_length` fields. The following invalid input succeeds:

```json
{
  "schema": "test.verifier-state/1",
  "component_id": "019cc17d-1b22-7a41-9fe9-c345c468f82c",
  "resources": [
    ["omvcs:resource:sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad", 3]
  ],
  "metadata": {}
}
```

Both `serde_json::from_str::<ComponentStateCandidate>` and
`serde_json::from_value::<ComponentStateCandidate>` accept it.
`ComponentStateCandidate::admit` then returns an admitted object and identity,
silently converting the positional sequence into a named-field canonical object.
The property-bearing sequence `[resource_id, 3, "primary", "audio/wav", {}]`
also succeeds with one exact synthetic property authority.

Root cause: `src/resource.rs:264–304` checks raw duplicate names, then delegates
to derived `ResourceReferenceWire` decoding, which also accepts sequences.
`src/component_state.rs:183–199, 284–297` consumes those candidates without
requiring each raw reference to be an object. This is an inherited WORK-0004
decoder defect exposed through WORK-0006's admission boundary; prior green
Resource tests did not exercise positional decoding. The Component State
top-level object-only gate does not protect nested Resource References.

Retained failing tests in `tests/component_state_acceptance.rs`:

- `embedded_resource_references_reject_positional_arrays_before_history`;
- `property_bearing_resource_references_reject_positional_arrays_before_history`.

Both report `text_rejected=false`, `value_rejected=false`,
`historical_admission=true`. No production remediation was made and neither
regression is ignored or weakened.

### Other observations and API semantics

The remaining examined behavior conforms: exact closed outer body, required
empty collections, typed assigned/content ID separation, omitted versus empty
parents, preserved known lineage, canonical-byte set sorting and duplicate
rejection (including equivalent nested canonical elements), exact schema and
unique property authority, schema-directed nested collection rules, recursive
raw duplicate-name rejection, RFC 8785 UTF-16 map ordering, body-only SHA-256
with no prefix, and excluded operational/evidence/wrapper fields.

No unapproved concrete metadata vocabulary was introduced. Metadata structure
and semantic validation are supplied by the exact versioned authority through
`ComponentStateSchemaValidator`; generic Core does not infer names, meanings or
types beyond declared shape. Synthetic test authorities are not production
schema definitions. Their implementations must honor the exact-version,
deterministic binding and semantic-exclusion contract; Core does not attempt to
guess or reproduce schema/Adapter meanings. This matches ADR-0009/0011.

Pure immutable model admission has no transactions, storage, interruption/retry
or concurrent mutation API to exercise. Operation-specific mandatory parentage
remains outside this work package; none was invented.

### Verification-only changes and commands

Changed only:

- `crates/omvcs-model/tests/component_state_acceptance.rs` — 12 independent
  acceptance tests, including the two retained blockers and a bounded exhaustive
  property over 24 parent permutations × 24 resource permutations.
- `docs/spec-coverage.md` — WORK-0006 evidence note; status stays `implemented`.
- `docs/plans/WORK-0006-component-state-model.md` — rejected gate and follow-up.
- This handover — independent evidence and accurate remaining work.

Commands actually run:

- Editor test discovery for `tests/component_state.rs`: no tests found; Cargo
  used instead.
- Before adding regressions, `cargo test --locked -p omvcs-model --test component_state`
  and `cargo test --locked -p omvcs-model`: passed, 21 targeted and 85 full tests.
- Final `cargo test --locked -p omvcs-model --test component_state --test component_state_acceptance --no-fail-fast`:
  21 original tests and 10 reviewer tests passed; two reviewer regressions failed.
- Final `cargo test --locked -p omvcs-model --no-fail-fast`: **95 passed, 2 failed**
  across all targets, including six passing compile-fail doctests.
- `cargo fmt --package omvcs-model -- --check`: passed.
- `cargo clippy --locked -p omvcs-model --all-targets -- -D warnings`: passed.
- `git diff --check` and `git diff --check 0ff182a..298e5d0`: passed.
- Editor Problems for the added test file: no errors.

Intermediate verifier-test syntax/format/lint issues were corrected in test
support only. The final two failures are normative regressions, not tooling
errors. Existing implementation tests and production code are unchanged.

### Decisions, gaps, limitations and next action

Semantic decisions beyond Specs: **None**. New Design Gaps: **None**.
W6-V001 is a specified-behavior defect, not grounds to reopen DG-0011.
Coverage remains `implemented`, and the passing implementer handover commands
do not establish independent acceptance. Older global coverage prose claiming
WORK-0005 has not started is stale; the package rows correctly show WORK-0005
verified and WORK-0006 implemented. That unrelated prose was not changed.

Next action: Core Engineer remediates object-only embedded Resource decoding
under authorized scope, preserves these regressions, and requests independent
reverification. Do not integrate or start WORK-0007/0008 from this handover.

Verification changes are recorded in a local verification-only successor of
`298e5d0198e9b125d8f9b258800ed4f2620ec196` with the required Co-authored-by
trailer; the implementation commit is preserved unchanged. Working tree: CLEAN
after that commit. Remote push: NO. Integration: NO. Remote publishing remains
ENABLED but was not used.

## Independent Verifier acceptance after remediation — 2026-10-08

From agent: Verifier
To agent: OMVCS Lead / Core Engineer
Branch: `work/0006-component-state`
Original implementation: `298e5d0198e9b125d8f9b258800ed4f2620ec196`
Remediation: `ff3a531a7b3b106cdbd5918f0e4804058a788022`
Specification base: `0ff182ab005fff7d58cebdea73dcb36e82c33b42`
Original verification evidence: `b3f20468012a0baf04e378b4d8e5eec46ed68880`
Result: **VERIFIED — W6-V001 resolved, no remaining blocker**.

This acceptance supersedes the initial rejection above; that record describes
the original implementation and remains historical evidence, not current status.
The original 12-test `component_state_acceptance.rs` is byte-for-byte unchanged.
Both original W6-V001 regressions now pass without weakening or ignoring them.

### Remediation and complete contract review

The remediation is exactly two files: a map-only visitor in `src/resource.rs`
and three additional Resource model tests. Raw duplicate-name checking still
precedes map decoding; only `visit_map` can reach the derived required/optional
wire fields. Sequence/scalar forms cannot reach those fields, and unknown
wrapper/tag members are rejected by the closed wire contract. Valid object input
is not rejected merely because it uses owned or borrowed value deserialization.

Re-read ADR-0011, resolved DG-0011, the plan and original findings, applicable
Core §§5.1, 7, 10–11, 56, 76–77, Glossary and relevant invariants. Re-reviewed
the complete unchanged Component State admission/body/ID/immutable API and
original tests against those requirements, including existing
ADR-0001/0003/0005/0007/0008/0009/0010 boundaries. Findings:

- Exact closed outer object, required members, empty resources/metadata and
  typed Component ID are preserved; unknown members and lossy optional-field
  coercions remain rejected.
- The canonical preimage contains all and only the specified body members,
  including schema and parents only when present; SHA-256 receives no type
  prefix, evidence, operational data or wrappers.
- Parent omission/empty/known-set distinctions, no fabricated lineage,
  canonical-byte sorting and duplicate rejection remain correct. The retained
  576 combined permutation property and equivalent-canonical-resource
  rejection pass.
- Resources are admitted under generic structure and exact unique contextual
  property authority. Empty properties still require validation. Unavailable,
  ambiguous, unrelated/latest-version and mismatched bindings remain blocked.
  Byte-length types, exact safe-integer endpoints and over-precision rejection
  remain enforced.
- Schema-owned metadata shapes/requirements, recursive array declarations,
  duplicate raw names, RFC 8785 UTF-16 member order and body-only identity
  remain covered. No concrete metadata vocabulary or key-based semantic
  heuristic was introduced into Core.
- Candidates still have no valid-history output; admitted objects remain
  immutable. Existing typed-ID and compile-fail boundary tests pass.
- No Project State, Revision, Adapter State, storage/repository, clone/fork or
  WORK-0007/0008 implementation was added. Operation-specific mandatory lineage
  and actual Adapter semantic vocabularies remain outside this bounded package.

### New independent parser attack matrix

Added only `tests/component_state_remediation_acceptance.rs` (four tests):

1. Positional array prefixes of lengths 0–8, all eight partial/full optional
   subsequences, and all eight null-hole combinations.
2. Object/array payloads inside body/reference/ResourceReference/Some/Ok
   wrappers, externally/adjacently tagged representations, nested arrays,
   scalar/null inputs, and valid fields mixed with unknown wrapper fields.
3. Positive named-object controls for all eight optional-member combinations:
   exact field preservation and identical Component State bodies/IDs.
4. Raw escaped-equivalent duplicates and trailing JSON documents remain invalid.

The first two attacks exercise four direct Resource candidate ingress paths
and four embedded Component State paths: JSON text, `from_value`, owned
`Deserialize` and borrowed `Deserialize`. Every invalid shape fails parsing
before historical admission; diagnostics also attempt public-constructor
admission if a direct decode unexpectedly succeeds. Positive controls use an
exact synthetic authority, not a production Core metadata vocabulary.

### Commands and results

- Editor targeted test discovery: no tests found; used Cargo.
- `cargo test --locked -p omvcs-model --test component_state --test component_state_acceptance --test component_state_remediation_acceptance --test resource_model --no-fail-fast`
  — **58 passed**, zero failures.
- `cargo test --locked -p omvcs-model --no-fail-fast` — **104 passed**, zero
  failures: 7 unit, 16 canonical, 21 Component State, 12 original verifier,
  4 remediation verifier, 8 hashing, 6 Component, 3 Component acceptance,
  21 Resource and 6 compile-fail doctests.
- `cargo fmt --package omvcs-model -- --check` — passed.
- `cargo clippy --locked -p omvcs-model --all-targets -- -D warnings` — passed.
- `git diff --check` and base-to-HEAD diff check — passed.
- Editor Problems for the new test file — no errors.

An initial new-test Clippy idiom warning was corrected using `is_ok_and` in test
diagnostics only. No production changes or assertion weakening occurred.

### Verification changes, limitations and Git state

This gate changes only the new verifier test file, the WORK-0006 coverage status
and acceptance evidence, this plan and handover. The earlier rejected-gate docs
and tests remain in history and their evidence sections are retained. Unrelated
stale global coverage prose was not changed.

Semantic decisions beyond Specs: **None**. New Design Gaps: **None**.
No remaining WORK-0006 verification work; integration requires a separate
instruction. No authorization to start WORK-0007/0008 is implied.

The verification-only local successor of `ff3a531` carries the required
Co-authored-by trailer. Original implementation, rejected-gate evidence and
remediation commits are preserved as ancestors. Working tree: CLEAN after the
verification commit. Push: NO. Integration: NO. Remote publishing: ENABLED,
unused. WORK-0007/0008 remain unstarted and their plans unchanged.
