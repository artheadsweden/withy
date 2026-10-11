# Handover WORK-0019 — Local Filesystem Storage Adapter

From agent: OMVCS Lead (WORK-0019 implementation)
To: Project record and next M3 package owner
Date: 2026-10-11
Branch: `work/0019-local-filesystem-storage`
Feature commits: `0506261`, `e1c1573`, `54a2b9c`
Integration merge: `0f426322394467c35e721248b8a0a18ab9a970bf`

## Completed

Implemented the bounded WORK-0019 filesystem Storage Adapter: canonical
Resource/Chunk key handling, immutable byte storage, guarded operational
metadata, Project Storage Map persistence/CAS, and filesystem Repository
Home bootstrap/discovery under ADR-0039–0042. Bootstrap exposes no partial
Home as initialized, records generation zero exactly, recognizes
incomplete-initialization evidence, and permits safe same-Project retries.
Existing-object retries re-establish ancestor-directory durability barriers.
Caller-stream failures remain distinct from content mismatches, while
structured `StorageError` values from the checked stream are preserved.
Opened Homes declare the six independent minimum ADR-0040 §10 capability
groups. Bootstrap retry and matching immutable-metadata retry re-establish
file and ancestor-directory durability before success.

The implementation and validation are complete. Independent Verifier
review ACCEPTED with no findings. The feature branch was pushed and
integrated without conflicts or production-code changes. WORK-0019 is
VERIFIED / INTEGRATED at `0f426322394467c35e721248b8a0a18ab9a970bf`.

## Specifications implemented

- Storage Adapter Specification §§10–21, 22–38, 49–53, 63–67, 73–76,
  85–86, 118–129, 176–189, 240–243, and conformance/failure-injection
  sections §§252–253.
- Core Specification §§29–36, 47–55, and 57–58.
- Core Invariants INV-RES-001–007, INV-STOR-001–006, INV-INT-001–003,
  INV-SYNC-001–003.
- ADR-0034/0039/0040/0041/0042.
- DG-0035 is resolved by ADR-0042. DG-0034 and DG-0036 remain OPEN within
  their existing boundaries.

## Files changed

- `Cargo.lock`
- `crates/omvcs-storage-local/Cargo.toml`
- `crates/omvcs-storage-local/src/lib.rs`
- `crates/omvcs-storage-local/tests/byte_storage.rs`
- `crates/omvcs-storage-local/tests/home_initialization.rs`
- `crates/omvcs-storage-local/tests/home_metadata.rs`
- `crates/omvcs-storage-local/tests/storage_map_persistence.rs`
- `docs/milestones.md`
- `docs/plans/WORK-0019-local-filesystem-storage.md`
- `docs/project-state.md`
- `docs/spec-coverage.md`
- This handover.

No Specs, ADRs, gaps, public Core/model APIs, other Storage Adapters, or
WORK-0020/0021 files changed.

## Tests added or changed

Linux WORK-0019 tests: 37 passed (4 unit, 11 byte-storage, 10 Home
initialization, 5 Home metadata, 7 Storage Map persistence). These cover
canonical keys, hostile input, exact bytes and identities, immutable
no-replace races, invalid caller-stream errors, interruption, containment,
bootstrap commit-boundary interruption, retry/idempotency, incomplete
state, generation-zero initialization, metadata guards, and concurrent
CAS. Capability assertions check the independent Home conformance profile.
The interrupted-write assertion inspects the physical canonical destination,
not the unrelated logical-key path.

The Windows-targeted adapter run passed 8 platform-enabled tests. The full
workspace passed 378 Windows tests and 407 Linux tests.

## Commands run

- Post-merge `cargo test -p omvcs-storage-local -p omvcs-storage
  -p omvcs-model -p omvcs-core --locked` — 351 tests passed.
- Post-merge `cargo test --workspace --locked` — 378 passed on Windows.
- Post-merge `cargo fmt --all -- --check` — passed.
- Post-merge warnings-denied Clippy for affected crates/all targets —
  passed.
- Post-merge `git diff --check` — passed.
- `cargo test -p omvcs-storage-local --locked` — Windows-enabled adapter
  tests passed.
- `cargo test --workspace --locked` — 378 passed on Windows.
- Isolated Linux `cargo +stable test -p omvcs-storage-local --locked
  --manifest-path /work/Cargo.toml` — 37 passed.
- Isolated Linux `cargo +stable test --workspace --locked
  --manifest-path /work/Cargo.toml` — 407 passed. The runner emitted the
  existing warning that the `omvcs-c` `cdylib` crate type is unsupported
  for `x86_64-unknown-linux-musl`.
- `cargo fmt --all -- --check` — passed.
- Windows `cargo clippy -p omvcs-storage-local --all-targets --locked --
  -D warnings` — passed.
- Isolated Linux `cargo +stable clippy -p omvcs-storage-local --all-targets
  --locked --manifest-path /work/Cargo.toml -- -D warnings` — passed.
- `git diff --check` — passed.
- Independent Verifier review — ACCEPT, no findings, including bootstrap
  and discovery durability barriers, the ADR-0040 capability declaration,
  and corrected interrupted-write path assertion. The Verifier reviewed
  the implementation but did not independently rerun tests; Windows/Linux
  test runs above were performed by the implementation agent.

## Semantic decisions made beyond the specification

`None`.

## Design Gaps discovered

- DG-0034 remains OPEN only for generic ProviderLocator schema-identifier
  lexical validation; the approved opaque-envelope use is implemented
  without inventing a grammar.
- DG-0036 remains OPEN for non-filesystem Repository Home discovery and is
  explicitly outside WORK-0019.
- DG-0035 is resolved by ADR-0042.
- DEC-STORAGE-013 remains OPEN; no official reference-conformance status
  is claimed.
- DEC-STORAGE-011 remains OPEN; shared namespace, deletion, GC, retention,
  and orphan cleanup remain excluded.
- No new Design Gap was identified.

## Assumptions

- Linux operations requiring `openat2`, atomic no-replace publication, and
  filesystem durability barriers are enabled only where the platform
  provides the required primitives. Unsupported guarantees fail explicitly.
- No filesystem marker or staging semantics are generalized to non-
  filesystem Homes.

## Known limitations

- DG-0036 remains OPEN and outside WORK-0019; DG-0034 remains OPEN for
  generic ProviderLocator schema-identifier lexical validation.
- Official reference-conformance designation, non-filesystem Home
  discovery, shared namespace, deletion, GC, retention, orphan cleanup,
  M4 transaction/recovery semantics, and automatic layout migration remain
  excluded.

## Remaining work

1. WORK-0020 is next to inspect; WORK-0020–WORK-0021 remain unstarted.
2. Keep M3 IN PROGRESS.
   authorization.

## Git state

Working tree: DIRTY — WORK-0019 implementation, focused tests, status
documentation, and this handover are uncommitted on the feature branch.
Remote publishing: ENABLED in `docs/project-state.md`; approved remote is
`origin`. Remote operations performed: NO.
