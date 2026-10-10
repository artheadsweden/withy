# In-memory byte test adapter (WORK-0018)

`MockStorage` implements the existing `omvcs_storage::ByteStorage` boundary.
Wrap a clone in `omvcs_storage::Storage` for checked Resource/Chunk transfers;
retain the original handle for explicit fixture configuration and inspection.
Clones share state; newly constructed mocks are independent. Configure each
caller-assigned Endpoint ID with `EndpointConfiguration` before use.

## Test controls

- `enqueue(ScriptStep)` adds an exact FIFO expectation for Endpoint, operation,
  and caller-supplied logical key. `pending_script()` and `calls()` expose
  expectations and selected effects. Capability discovery and unsupported
  operations do not consume steps; a mismatch fails explicitly without consuming
  the expectation. Once matched, a step is consumed even if the operation fails.
- `Effect::Pass`, `Fail`, and transfer `FailAfter` support multi-step scenarios.
  Failure classes/recoverability are explicitly supplied by the test. A transfer
  interrupted at its full length still fails before EOF; a threshold beyond EOF
  is not reached. Put failure never commits its buffered candidate or rewrites
  an existing object. This is mock behavior, not a new generic transaction rule.
- `Absent` temporarily changes one get/stat result without removing bytes.
  Visibility is controlled by calls, not seconds. No eventual-consistency or
  slow-storage guarantee is implemented.
- Get-only `ReturnBytes`, `Truncate`, and `Corrupt` modify a download snapshot,
  never stored immutable bytes. Truncation caps output length; XOR specifies the
  exact byte change. An out-of-bounds corruption offset fails explicitly.
  These adversarial streams are not successful verified retrievals. With the
  checked facade, EOF mismatch prevents `Download::complete()`. Use the raw
  stream with WORK-0017 verification APIs to distinguish checked mismatches
  (`failed`) from inaccessible/interrupted bytes (`indeterminate`).
- `stored_bytes()` is an inspection facility, not evidence of verification.
  Stat returns only actual stored length, never a clock, ETag, or verification.
- Mock-only `checksum_report()` performs the scripted stat and hashes actual
  stored bytes with SHA-256. Scope is exactly complete Resource bytes or exactly
  one Chunk's bytes. Consume the report with the existing WORK-0017 checksum
  evaluators; the request identity is not used as the reported digest.

Objects are addressed within a mock by configured Endpoint and opaque caller
key; no key-layout or locator grammar is chosen. Identical writes establish
exact correspondence; conflicting bytes never overwrite existing objects.
Raw put validates identity, expected hash, and length as well as consuming EOF.
Downloads own their open-time byte snapshot. No network/filesystem/persistence,
randomness, clocks, threads, automatic retries, namespaces, deletion, retention,
or Repository Home implementation is present.

## Verification and Core boundary

Integration tests consume fixed chunking, manifest derivation, destination
verification, promotion eligibility, and Core map CAS unchanged. The private
`RepositoryFixture` in `tests/verification_and_core.rs` supplies coherent
snapshots and explicit CAS failures for Core tests only; it is not implemented
by `MockStorage` or exported as a Home adapter. Core's assigned Replica IDs are
not reproduced by the mock. Tests compare their relationships, not random or
clock-derived values.

The existing Storage and model Rust `ChunkId` types differ; fixtures bridge
them by the unchanged SHA-256 digest, without changing either production API.
History tests use external byte/identifier sentinels, not a history admission
implementation. Shared `omvcs-test-support` remains untouched because these
controls are mock-specific and no new implementation-independent suite was
needed.

This package makes no official reference-conformance designation
(DEC-STORAGE-013 OPEN), shared-namespace claim (DEC-STORAGE-011 OPEN), or
ProviderLocator lexical validation claim (DG-0034 OPEN).
