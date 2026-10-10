# Handover WORK-0018 — Mock Storage Adapter

From agent: OMVCS Storage Engineer
To agent: Core Engineer reviewer, then independent Verifier
Date: 2026-10-10
Branch: `work/0018-mock-storage-adapter`
HEAD (validated implementation): `ec7551a9dcdc447ad42b84ca9a8cd8935d596c42`
Integration baseline: `565edf6836fc3971fa60ca7bd2e86339dc8d1c91`
Delivery HEAD: the documentation commit containing this handover; resolve
with `git log -1 --format=%H` on the branch. No code changes follow the
validated implementation commit.

## Completed

Implemented WORK-0018 only: deterministic, synchronous, entirely in-memory
`MockStorage` behind the existing `ByteStorage` production boundary. All six
Resource/Chunk put/get/stat operations preserve exact bytes in normal
operation; immutable repeated writes establish exact correspondence and
conflicting bytes do not overwrite. Raw put checks identity/hash/length and
consumes EOF, independently of the checked facade.

Mock-only API: `MockStorage::{new,configure_endpoint,enqueue,pending_script,
calls,stored_bytes,checksum_report}`, `EndpointConfiguration`, `ScriptStep`,
`Effect`, `Call`, and `ALL_BYTE_OPERATIONS`. Clones share explicit fixture
state. Configured caller-assigned Endpoint IDs and keys never define content
identity. Downloads own their open-time byte snapshots.

Explicit FIFO scripts support pass, typed failure, transfer failure after
N bytes, one-call absence, and wrong/truncated/XOR-corrupt returned bytes.
Discovery failures and unsupported operations remain distinct from absence.
Faults never rewrite immutable stored bytes. A failed put never commits its
buffered candidate; this test-backend behavior does not establish generic
transaction or retention semantics.

Checksum reports hash actual stored bytes with exact SHA-256 and exact
Resource/Chunk scope, after a successful scripted stat. The existing
WORK-0017 evaluators determine outcome/strength/evidence/promotion.
The mock does not promote/register Replicas or implement Storage Map CAS.
Tests consume existing WORK-0017/Core APIs unchanged. A private, coherent
map-snapshot fixture supplies explicit persistence failures/conflicts to
Core tests; it is not a production Home implementation.

## Specifications implemented

Read in preflight and implemented only at the authorized six-operation
boundary:

- Storage Adapter §§1–38, 85–86, 176–192, 199–205.
- Core §§8, 29–36, 47–55.
- INV-RES-001–007, INV-STOR-001–005, INV-INT-001–003.
- Approved ADR-0032/0033/0034/0036/0037/0038.
- Storage §253 consulted for fixture examples: byte interruptions and
  corruption are explicit; visibility is scripted per operation, not seconds.
  No wall-clock, eventual-consistency or slow-storage policy is implemented.

These references do not claim implementation of every operation in §176,
Repository Home conformance, physical migration/removal, or official
reference-adapter designation.

## Files changed

Implementation commit:

- `crates/omvcs-storage-mock/Cargo.toml`
- `crates/omvcs-storage-mock/src/lib.rs`
- `crates/omvcs-storage-mock/README.md`
- `crates/omvcs-storage-mock/tests/byte_contract.rs`
- `crates/omvcs-storage-mock/tests/verification_and_core.rs`
- `Cargo.lock`: dependency references only; no new package/version.

Documentation commit:

- `docs/plans/WORK-0018-mock-storage-adapter.md`
- `docs/spec-coverage.md`
- `docs/milestones.md`
- `docs/project-state.md`
- This handover.

No Specs, production Core/model/storage APIs, local Adapter, shared
test-support, gaps, decisions, or WORK-0019–0021 files changed.

## Tests added or changed

27 new tests; all passed:

- **21 byte-contract tests:** paired Resource/Chunk exact round-trip and stat,
  zero bytes, absent versus typed failures and unsupported, immutable
  repeats/conflicts, upload/download interruption at zero/middle/final byte,
  wrong/truncated/corrupt returns, scripted visibility, plus deterministic
  repeat traces/results, invalid/mismatched scripts, discovery failures,
  raw-boundary input checks, and Endpoint/key separation.
- **6 verification/Core scenario tests:** corrupt/interrupted destination
  rejection and retained valid source; exact checksum scope and actual-byte
  digest versus expected identity; both representations and same-Endpoint
  Replicas, consuming the approved chunking APIs for zero/one/below/exact/
  plus-one/multi-target sizes; final-Chunk multi-step upload/read failures,
  missing/reordered evidence and missing manifest proof; Core multi-entry
  CAS write failure/conflict, stale/no-retry/no-op/exhaustion/invalid/load
  failure atomicity; external history-byte and identity sentinels.

Existing prerequisite tests ran **unchanged**: Storage contract 24, Storage
checksum 1, model Replica 11, model verification 9, Core Storage Map 22:
**67 passed**. Combined focused run: **94 passed, 0 failed**.

Full workspace: **370 passed, 0 failed**, across **42 suites including
doctests**. `tests/conformance/` contains only `.gitkeep`; there is no separate
shared executable suite to run. No shared fixtures were added.

## Commands run

Final successful validation:

```text
cargo test --locked -p omvcs-storage-mock -p omvcs-storage --test byte_contract --test verification_and_core --test contract --test verification -p omvcs-model --test replica_model --test verification -p omvcs-core --test storage_map_operations
cargo test --workspace --locked
cargo fmt --all -- --check
cargo clippy --locked -p omvcs-storage-mock -p omvcs-storage -p omvcs-model -p omvcs-core --all-targets -- -D warnings
git diff --check
git diff --cached --check
```

All passed. VS Code's test tool reported no discovered Rust tests; Cargo was
used. Editor diagnostics reported no errors. Initial compilation exposed the
existing distinct model/Storage ChunkId Rust types; a digest-only fixture
bridge resolved it without touching either production API. Initial Clippy
findings and a must-use warning were fixed before the final successful runs.
Final diff inspection confirmed bounded scope.

## Semantic decisions made beyond the specification

`None`.

## Design Gaps discovered

`None` for this bounded implementation.

- DG-0029 and DG-0032 remain RESOLVED.
- DG-0034 remains OPEN; no generic locator schema lexical grammar selected.
- DEC-STORAGE-013 remains OPEN; no official reference designation.
- DEC-STORAGE-011 remains OPEN; no stable/shared namespace, cross-Project
  physical-object or deduplication guarantees.

## Assumptions

- Tests configure stable IDs and opaque keys explicitly. Script matching,
  snapshot behavior and call-count visibility are mock mechanics only.
- An in-memory Adapter retains full stored object bytes; this is not a
  bounded-memory provider or a durable provider. I/O accepts `Read` streams.
- Model and Storage ChunkId fixtures bridge the identical normative digest.
- Core's assigned Replica IDs remain Core-owned; deterministic mock tests do
  not compare clock/random-derived Core IDs across runs.
- History sentinels demonstrate non-mutation, not historical-object admission.

## Known limitations

- Core Engineer review: **PENDING / not performed**.
- Independent Verifier acceptance: **PENDING / not performed**.
- No persistence, network/filesystem/cloud, Home layout/keys, retries,
  eventual-consistency/time delays, slow-storage guarantees, races,
  transactions, physical migration orchestration, Replica deletion,
  GC/retention/shared namespace, grants/encryption, Platform, or M4 behavior.
- Adversarial get effects change returned snapshots, not stored objects.
  They are explicit test fixtures, never successful verification.
- `omvcs-test-support` remains empty/untouched because no implementation-
  independent shared fixture was needed. Mock controls do not leak into
  generic production interfaces.

## Remaining work

1. Core Engineer review of this branch's bounded API/integration usage.
2. Independent Verifier acceptance against original Specs/invariants and
   adversarial failure cases; resolve findings on this branch if authorized.
3. Integration/publishing only under a separate instruction after acceptance.

WORK-0018 is IMPLEMENTED / LOCAL VALIDATION PASS, not VERIFIED/integrated.
M3 remains IN PROGRESS. WORK-0019–0021 remain unstarted.

## Git state

Working tree: CLEAN at delivery after the accompanying documentation commit
and final status/diff check.
Remote publishing: ENABLED in project state.
Remote push performed: NO.
Integration performed: NO.
Branch base remains the authorized integration baseline above.
