# Handover WORK-0015 — Storage primitives and provider-neutral contract

From agent: Storage Engineer, coordinated by OMVCS Lead
To agent: Future OMVCS Lead
Date: 2026-10-10
Feature branch: `work/0015-storage-primitives-contract`
Integration branch: `spec/0003-canonical-collection-order`
Implementation commit: `1be170e4cc3afd8e825580d41129847870f359f7`
Documentation/status commit: `74474db9d3007ffa0ee54623ba9e1d4d4d719f5d`
Integration commit: `aade8af1031a42fd9c7d093f35f6c4ae3f2818a0`

## Completed

Implemented and independently accepted the restricted provider-neutral
Resource/Chunk byte-I/O seam. Added Endpoint-specific capabilities,
caller-supplied identities and opaque logical keys/operation tokens,
streaming transfers, SHA-256 correspondence checking, structured errors,
operational-only stat metadata, and focused contract tests.

The implementation intentionally does not provide a concrete Storage
Adapter, persistent Replica or Storage Map model, verification-strength
taxonomy, Repository Home conformance, physical key layout, or chunking
policy.

## Specifications implemented

- Core Specification §§4.1–4.2, 6–8.1, 29–31.
- Storage Adapter Specification §§1–21, 22–29, 35–36, 85–86,
  176–182, 185, and 190.
- Glossary: Resource Object, Resource Identifier, Chunk, Chunk Identifier,
  Storage Adapter, Storage Endpoint, Operational Metadata.
- Core Invariants: INV-RES-001–007, INV-STOR-001–003, INV-INT-001–003.

## Files changed

- `crates/omvcs-storage/Cargo.toml`
- `crates/omvcs-storage/src/lib.rs`
- `crates/omvcs-storage/src/identity.rs`
- `crates/omvcs-storage/src/stream.rs`
- `crates/omvcs-storage/tests/contract.rs`
- `Cargo.lock` (direct `sha2` dependency bookkeeping; existing locked version)
- `docs/plans/WORK-0015-storage-primitives-contract.md`
- `docs/spec-coverage.md`
- `docs/handovers/HANDOVER-WORK-0015-storage-primitives.md`

## Tests added or changed

`crates/omvcs-storage/tests/contract.rs` has 24 tests covering Resource and
Chunk exact-byte round trips; missing/unsupported/provider failure
distinctions; capability declarations; wrong input/output bytes; stream
completion and failure; identity preservation; stat non-verification;
existing-key correspondence/conflict behavior; bounded-memory streaming;
and absence of implicit history, Replica, or chunking effects.

## Commands run

- `cargo test -p omvcs-storage --locked` — passed, 24 tests.
- `cargo test --workspace --locked` — passed, 298 tests.
- `cargo fmt --all -- --check` — passed.
- `cargo clippy -p omvcs-storage --all-targets --locked -- -D warnings` —
  passed.
- `git diff --check` — passed.

## Semantic decisions made beyond the specification

`None`.

## Design Gaps discovered

`None` for WORK-0015. DG-0029 remains OPEN and continues to block WORK-0016
Replica/Storage Map semantics; it is not changed by this work.

## Assumptions

- The model crate's existing `StorageEndpointId` assigned UUIDv7 type is
  reused for operational Endpoint identity, as specified by Core §4.1.
- `ChunkId` is defined narrowly in the Storage crate because the bootstrap
  model crate does not yet expose one; it implements Core §8.1's
  `omvcs:chunk:sha256:` namespace only and selects no chunking policy.
- Providers implement the documented durability and existing-object
  correspondence obligations. No concrete provider is included here.
- Download bytes are provisional until `complete()` reaches EOF and verifies
  the requested digest; dropping/abandoning the stream is not successful
  retrieval.

## Known limitations

- No concrete provider exists in this package; provider durability,
  capability truthfulness, atomic immutable writes, and provider-context
  sanitization remain obligations of future provider implementations.
- No persisted Replica or verification evidence exists; successful transfer
  completion is not Replica registration or verification-strength evidence.
- No logical-key layout or Repository Home conformance is claimed.

## Remaining work

- WORK-0015 Core Engineer review: ACCEPT, no findings.
- WORK-0015 independent Verifier review: ACCEPT, no findings.
- No remaining WORK-0015 implementation or verification tasks.
- WORK-0016–WORK-0021 were not started by this closeout.
- Feature branch was pushed and merged no-fast-forward into the integration
  branch; the integration branch was pushed after status reconciliation.

## Git state

Working tree: CLEAN on `spec/0003-canonical-collection-order` after status
reconciliation and integration push.
Remote push performed: YES — feature and integration branches.
