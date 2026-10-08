# Handover WORK-0002 — Canonical metadata serialization

From agent: Core Engineer
To agent: Verifier / OMVCS Lead
Date: 2026-10-08
Branch: `work/0002-canonical-serialization`
HEAD: `4fa378a9ac314a674c3e7ccd0f5845d6aacc6a78` (implementation commit; handover recorded in a follow-up documentation-only commit)

## Completed

Implemented a schema-directed canonical metadata-body serializer in `omvcs-model`. It rejects duplicate object member names while parsing raw JSON, validates every schema branch for explicit array classification (including absent and empty values), recursively normalizes ordered and set-like arrays and map values, then emits RFC 8785 canonical bytes. The public entry point accepts only the historical object body and does not hash it.

The package passed independent Verifier review. The first review identified that an unclassified array could evade validation when its enclosing value was absent or empty. The implementation now validates the complete schema tree before value normalization, with regression tests for empty map values, absent struct fields, and empty arrays.

## Specifications implemented

- `Specs/OMVCS Core Specification.md` §§5 and 5.1: UTF-8 JSON, RFC 8785-compatible bytes, canonical object-body boundary, explicit recursive array classifications and ordering, set-like canonical-byte sorting and duplicate rejection, RFC 8785 map ordering only, unique member names, and schema-directed map values.
- `Specs/OMVCS Core Invariants Specification.md`: INV-HIST-002, INV-HIST-006, INV-RES-002. This package provides canonical serialization support; it does **not** implement content hashing or derive identifiers.
- `docs/decisions/ADR-0001-hashed-collection-ordering.md`: array-only scope, ordered preservation, recursive set-like normalization, and invalid unclassified arrays.
- `docs/decisions/ADR-0005-json-object-map-canonicalization.md`: duplicate-name rejection, RFC 8785 map ordering without extra entry sorting, and recursive map-value normalization.
- Cross-spec M1 rules checked: DAW Adapter Specification §§20–22; Storage Adapter Specification §162; Glossary Chunk Manifest and Adapter State; Ardour Reference Adapter Design §§23 and 31.

## Files changed

- `crates/omvcs-model/src/canonical.rs` — schema types, duplicate-aware JSON parser, complete schema validation, normalization, and RFC 8785 output.
- `crates/omvcs-model/src/lib.rs` — exports the canonical module; WORK-0001 behavior is unchanged.
- `crates/omvcs-model/Cargo.toml` and `Cargo.lock` — add Serde parsing and the exact-pinned `serde_jcs = 0.2.0` canonicalizer.
- `crates/omvcs-model/tests/canonical_serialization.rs` — focused behavior-named conformance tests and RFC-derived vectors.
- `docs/spec-coverage.md` — WORK-0002 coverage marked `verified` after Verifier signoff.
- `docs/plans/WORK-0002-canonical-serialization.md` — preserved the Lead's planning update and marked the package complete after verification.
- `docs/handovers/HANDOVER-WORK-0002-canonical-serialization.md` — this handover.

## Tests added or changed

`crates/omvcs-model/tests/canonical_serialization.rs` covers:

- Map insertion-order invariance and RFC 8785 UTF-16 member-name ordering, with no extra entry sorting.
- Duplicate raw member names at multiple nesting levels, including escaped aliases.
- Recursive schema-directed map-value normalization.
- Ordered sequence preservation and set-like permutation invariance.
- Recursive set-like duplicate rejection based on canonical element bytes.
- Rejection of unclassified arrays, including schemas nested under empty maps/arrays and absent struct fields.
- UTF-8 preservation without Unicode normalization; string escaping and RFC-derived number formatting/rejection vectors.
- JSON object-body-only API boundary and deterministic repeated serialization.

## Commands run

- `cargo fmt --all` — passed.
- `cargo fmt --all -- --check` — passed.
- `cargo test -p omvcs-model` — passed: 7 unit tests and 16 canonical-serialization integration tests.
- `cargo clippy -p omvcs-model --all-targets -- -D warnings` — passed.
- `git diff --check` — passed before implementation commit.
- Verifier independent review — PASS / signoff. The initial finding on schema validation behind absent/empty values was fixed and independently confirmed.

## Semantic decisions made beyond the specification

`None`.

## Design Gaps discovered

`None`. DG-0003 and DG-0005 are resolved; no open gap affecting WORK-0002 was found.

## Assumptions

`None` beyond the specified schema-directed contract. No object-field defaults or required-field semantics were added.

## Known limitations

- Hashing, SHA-256 content identifiers, and byte-to-Resource identity are not implemented; those belong to WORK-0003.
- This package supplies a generic schema-directed serialization API, not concrete Resource, Component, Project, or Revision schemas.
- The caller must supply only the historical object body. Extracting a body from storage/protocol wrappers and defining field-presence semantics remain outside this package.

## Remaining work

WORK-0002 has passed its implementation and independent verification gate. No WORK-0003 or later package was started. Proceed with later work only under its own approved package and sequencing.

## Git state

Working tree: CLEAN after the handover-only documentation commit.
Remote push performed: NO.
