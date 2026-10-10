# Handover WORK-0017 — Resource and Chunk verification model

From agent: OMVCS Core Engineer
To agent: Integration owner
Date: 2026-10-10
Branch: `work/0017-resource-chunk-verification`
HEAD: `4e2e13313625c1e7409d4138100cc324a3bd60f1`

## Completed

Implemented the WORK-0017 verification and destination-promotion model.
Deterministic chunking and streaming verification produce typed, operational
results that distinguish strength, outcome, method, and evidence. Direct
Resource and Chunk checks, destination-applicable checksum results, and
verified deterministic reconstruction are available. Chunk Manifest
construction and deserialization enforce the fixed-size OMVCS 0.1 policy,
including exactly one zero-length Chunk for a chunked empty Resource.

Verified candidates can be staged as Storage Map additions only after
destination-bound `resource_identity` assurance. Promotion is committed
through the existing Project `StorageMapGeneration` CAS operation; failed,
indeterminate, stale, source-only, or mismatched evidence cannot register
the candidate. WORK-0016 Replica identity, representation, locator,
generation, persistence, and CAS contracts remain unchanged.

## Specifications implemented

- Core Specification §§8–8.2, 32, 47–55.
- Storage Adapter Specification §§30–36, 73–76, 109–112, 183, 192,
  204–205, and 240–243.
- Glossary: Resource Object, Chunk, Chunk Manifest, Resource Replica,
  Availability State, Corrupt Replica, Content Verification, Verification
  Strength, Verification Method, Verification Evidence, Verification Result,
  and Chunking Policy.
- Core Invariants: INV-RES-001–007, INV-STOR-003–005, INV-INT-001–003.
- Human-approved ADR-0036, ADR-0037, and ADR-0038.
- Existing WORK-0016/ADR-0034 StorageMapGeneration CAS used without semantic
  change.

## Files changed

- `crates/omvcs-model/src/lib.rs`
- `crates/omvcs-model/src/replica.rs`
- `crates/omvcs-model/src/verification.rs`
- `crates/omvcs-model/tests/replica_model.rs`
- `crates/omvcs-model/tests/verification.rs`
- `crates/omvcs-core/src/storage_map.rs`
- `crates/omvcs-core/tests/storage_map_operations.rs`
- `crates/omvcs-storage/Cargo.toml`
- `crates/omvcs-storage/src/lib.rs`
- `crates/omvcs-storage/src/verification.rs`
- `crates/omvcs-storage/tests/verification.rs`
- `Cargo.lock`
- `docs/plans/WORK-0016-replica-storage-map.md`
- `docs/plans/WORK-0017-resource-chunk-verification.md`
- `docs/project-state.md`
- `docs/spec-coverage.md`
- `docs/handovers/HANDOVER-WORK-0017-resource-chunk-verification.md`

`serde_json` was added only as a dev-dependency for Storage boundary tests;
it is already present in the workspace lockfile. No new package/dependency
version was introduced.

## Tests added or changed

- `crates/omvcs-model/tests/verification.rs` (9 tests): zero, one-byte,
  below/equal/plus-one/multi-target deterministic boundaries; stable
  ChunkIds/manifests; complete-object Resource identity; empty Chunk rule;
  rejection of empty/oversized/nonconforming manifests at construction and
  deserialization; verified/failed/indeterminate Resource and Chunk checks;
  stream interruption/unavailability; ordered complete reconstruction;
  destination/source/stale/wrong-subject/missing/reordered evidence; provider
  checksum algorithm and scope; historical identity independence.
- `crates/omvcs-storage/tests/verification.rs` (1 test): equivalent exact
  SHA-256 byte scopes accepted, non-equivalent algorithm/scope indeterminate,
  and a checked false digest failed.
- `crates/omvcs-core/tests/storage_map_operations.rs`: two promotion/CAS
  scenarios added for successful destination assurance, source-only
  rejection, stale-generation conflict, invalid-batch atomicity, and unchanged
  persisted map on failures.
- `crates/omvcs-model/tests/replica_model.rs`: existing two-Chunk fixture now
  follows the exact fixed-size policy.

Final focused command ran **43 passing tests**: 22 Storage Map tests,
11 Replica model tests, 9 verification model tests, and 1 Storage checksum
test. Workspace validation ran **343 passed, 0 failed** across 40 test
suites, including doctests.

## Commands run

- `cargo test --locked -p omvcs-model --test replica_model --test verification -p omvcs-core --test storage_map_operations -p omvcs-storage --test verification`
  — passed (43 tests).
- `cargo test --workspace --locked` — passed (343 passed, 0 failed).
- `cargo fmt --all -- --check` — passed.
- `cargo clippy --locked -p omvcs-model -p omvcs-core -p omvcs-storage --all-targets -- -D warnings`
  — passed.
- `git diff --check` — passed before the handover file was added; rerun after
  any handover edits.
- `cargo check --locked -p omvcs-model -p omvcs-core -p omvcs-storage`
  — compiled; an initial unused-function warning was resolved by exporting
  the generic checksum verification functions.

## Semantic decisions made beyond the specification

`None`.

## Design Gaps discovered

- DG-0034 remains OPEN, unchanged; its ProviderLocator schema-identifier
  lexical grammar is not used or validated by this work.
- DG-0032 is RESOLVED by ADR-0034 and was not changed.
- DG-0033 is RESOLVED by ADR-0035 and was not changed.
- DEC-STORAGE-011 remains OPEN and outside scope.
- No new Design Gap was found.

## Assumptions

- `ProviderChecksumAlgorithm::OmvcsSha256` is an Adapter assertion that
  documented and enforced provider semantics cover exact SHA-256 over the
  named complete Resource or exact Chunk bytes. Concrete provider
  conformance is not implemented here.

## Known limitations

- The Storage Engineer reviewed and ACCEPTED the generic Adapter
  checksum/verifier boundary; this was a scoped review, not acceptance of the
  entire work package.
- The Verifier initially REJECTED one high-severity case: malformed Chunk
  manifests could pass the general manifest constructor. The policy
  enforcement and constructor/deserialization tests were added and the
  specific remediation was accepted. The final independent review of the
  complete package returned ACCEPT with no findings.
- No provider implementation, replication/migration orchestration, automatic
  repair, garbage collection, deletion, retention, shared namespace,
  encryption, grants, publication/M4, Platform, or DAW behavior is included.

## Remaining work

1. WORK-0017 has the required Storage Engineer and independent Verifier
   acceptance and is integrated. WORK-0018 is the next M3 package to inspect
   and authorize.
2. WORK-0018–WORK-0021 remain unstarted.

## Git state

Implementation commit: `7b2ffb8222b87c14080cc8db51966d695c583a5f`.
Implementation/handover HEAD reviewed:
`4e2e13313625c1e7409d4138100cc324a3bd60f1`. The Verifier independently
accepted the complete package at this HEAD with no findings. No-fast-forward
integration commit: `60f71c9c624eb7b08bb8fd18ac7abb7a7a7ce250`. Feature and
integration branches were pushed and verified synchronized. Post-merge
validation passed. Documentation-only integration reconciliation followed;
no production code changed after verification.
