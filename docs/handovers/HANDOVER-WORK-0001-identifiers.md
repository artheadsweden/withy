# Handover WORK-0001 — Normative identifier types

From agent: Core Engineer
To agent: Verifier
Date: 2026-10-08
Branch: `work/0001-identifiers`
HEAD: `e2bc52b908666ad9a2bc8ad86b9202523e95f95e` (implementation/coverage commit; this handover and completion-status update are committed in a documentation-only successor)

## Completed

- Added distinct assigned identifier types for Project, Creative Component, Storage Endpoint, Contribution, and ActorId. Generation uses UUIDv7; parsing requires canonical lowercase hyphenated UUID text, UUIDv7, and the RFC UUID variant.
- Added typed SHA-256 identifiers for Resource, Component State, Adapter State, Project State, and Revision objects. Formatting and parsing enforce each object-type namespace and exactly 64 lowercase hexadecimal digest characters.
- Added focused model tests for generation, canonical formatting/parsing, malformed UUIDs (including wrong version and variant), object-type namespace separation, digest boundaries/encodings, and assigned-ID independence from profile/location values.
- Preserved the Lead's WORK-0001 and coverage-map clarifications that WORK-0001 owns identifier type/format/namespace tests while WORK-0003 owns raw Resource byte-to-ID equality and mutation behavior.
- Updated the coverage map to distinguish WORK-0001 identifier behavior from WORK-0003 content hashing. Coverage is marked `implemented`, not `verified`.
- Independent Verifier review found three issues: UUID variant validation, weak independence-test assertions, and a stale coverage summary. All three were addressed before handover. The focused checks below pass after those changes.

## Specifications implemented

- Core Specification §§4, 4.1, 4.2, 6, 9–14, and 59.
- Core Specification §5.1 was used only as a boundary reference: no canonical metadata digest bytes or serialization behavior were implemented.
- Glossary: Project Identifier, Creative Component, Resource Identifier, Revision Identifier, Actor Identifier (ActorId).
- Core Invariants: INV-HIST-002, INV-HIST-007, INV-RES-002, INV-RES-003, INV-PROJ-001, INV-PROJ-002, INV-PROJ-003.
- ADR-0002 — Actor Identifier representation.
- DG-0001 through DG-0006 were confirmed resolved; none affects this work.

## Files changed

- `crates/omvcs-model/Cargo.toml` — added `uuid` with the `v7` feature for standards-based UUIDv7 generation.
- `crates/omvcs-model/src/lib.rs` — assigned and typed content identifier APIs, parsing/formatting, and focused unit tests.
- `Cargo.lock` — resolved dependency lock entries for `uuid`. This is the workspace lockfile consequence of the crate dependency.
- `docs/plans/WORK-0001-identifiers.md` — retained the planning clarifications and marked the package complete.
- `docs/spec-coverage.md` — assigned test ownership and implementation status updated consistently.
- `docs/handovers/HANDOVER-WORK-0001-identifiers.md` — this handover, based on `docs/handovers/HANDOVER-TEMPLATE.md`.

## Tests added or changed

Seven focused unit tests in `crates/omvcs-model/src/lib.rs` cover:

- UUIDv7 generation and canonical lowercase formatting for all assigned ID types.
- Assigned-ID round trip and rejection of uppercase/compact text, wrong UUID version, invalid UUID text, and a UUID with the wrong variant.
- ActorId stability while test-fixture display name, email, username, Platform account, signing key, and rotated signing key values change.
- Project/Component ID stability while test-fixture filename, path, storage endpoint, Platform URL, and credential-reference values change.
- Typed content-ID round trips, exact prefixes, and namespace separation for the supported object classes.
- Invalid digest length boundaries (62, 63, 65, and 66 characters), uppercase hex, invalid hex, and unsupported format/prefix.
- Preservation of the full 32-byte digest value.

No raw Resource-byte hashing, equality, or mutation test was added; those remain explicitly assigned to WORK-0003.

## Commands run

- `cargo fmt --package omvcs-model` — passed.
- `cargo test --locked -p omvcs-model` — passed; 7 unit tests, 0 failures; no doc tests.
- `cargo clippy --locked -p omvcs-model --all-targets -- -D warnings` — passed.
- `cargo fmt --package omvcs-model -- --check` — passed.
- `git diff --check` and `git diff --cached --check` — passed.

Initial test/Clippy attempts exposed a malformed assertion and documentation/test lints; those were corrected. Final results above are from the passing validation runs.

## Semantic decisions made beyond the specification

`None`.

## Design Gaps discovered

None affecting WORK-0001. DG-0001 through DG-0006 are resolved. The Platform account proof-of-control question remains separate and does not define or block ActorId representation.

## Assumptions

- `uuid` is used only as an implementation dependency for UUIDv7 generation and UUID parsing/formatting. It does not introduce OMVCS identity semantics.
- Content identifier constructors receive an already computed 32-byte digest. Computing hashes is not part of this package.

## Known limitations

- No SHA-256 content computation was implemented.
- No canonical serialization or metadata-object hashing was implemented.
- No Revision author/history model was implemented; that remains WORK-0008 scope.
- No Resource byte-to-ID equality/mutation behavior, storage, repository, filename, endpoint, or Platform behavior was implemented.
- Tests are focused crate unit tests; this package did not add repository-level conformance-suite cases.
- `Cargo.lock` changed to record the required crate dependency; no other cross-cutting implementation path was changed.

## Remaining work

- No WORK-0001 implementation work remains. The independent review findings were addressed and the package was validated and committed.
- Verifier may use this handover for final gate confirmation. Do not begin WORK-0002 through WORK-0008 until the requested WORK-0001 review/handover/clean-Git gate is accepted.

## Git state

Working tree: CLEAN after the handover/completion-status commit.
Remote push performed: NO.
Remote publishing: ENABLED in `docs/project-state.md`; no push was performed, as instructed.
