# Handover WORK-0003 — Content-derived object hashes

From agent: Core Engineer
To agent: Verifier
Date: 2026-10-08
Branch: `work/0003-hashes`
HEAD: `5f2334f94ac3d9f5cbfd7c2744a78d27a042e95a` (WORK-0003 implementation commit)

## Completed

Implemented SHA-256 Resource and metadata-object identifier construction in
`omvcs-model`. Resource IDs consume exactly the supplied complete raw Resource
bytes. Metadata IDs consume exactly the canonical object-body bytes supplied by
WORK-0002. The object type is represented by the returned typed ID; no type,
schema, or domain-separation prefix is added to either digest input.

The WORK-0003 implementation is committed, the independent Verifier signoff
is complete, and the package plan is marked complete. Its coverage row is
`verified`. The separate shared coverage row covering WORK-0003 and WORK-0004
chunking/location behaviour remains `planned` until that broader work is
complete.

## Specifications implemented

- Core Specification §§4–6 and 55.
- Core Invariants INV-HIST-002 and INV-RES-001–004.
- Canonical metadata input consumed under ADR-0001 and ADR-0005 through the
  WORK-0002 API.
- Resolved DG-0003 and DG-0005; no Design Gap was discovered.

## Files changed

- `crates/omvcs-model/Cargo.toml` — add the SHA-256 implementation dependency.
- `Cargo.lock` — record the dependency graph.
- `crates/omvcs-model/src/lib.rs` — expose `hashing`.
- `crates/omvcs-model/src/hashing.rs` — raw Resource and typed metadata hash
  functions.
- `crates/omvcs-model/tests/content_hashing.rs` — SHA-256, preimage,
  namespace, canonicalization, wrapper, infrastructure-independence, and
  deterministic byte-input tests.
- `docs/spec-coverage.md` — mark the WORK-0003 content-derived-identity row
  `verified` following independent Verifier signoff; leave the shared
  WORK-0003/WORK-0004 row planned.
- `docs/plans/WORK-0003-content-hashes.md` — mark the package complete after
  independent Verifier signoff.
- `docs/handovers/HANDOVER-WORK-0003-content-hashes.md` — this handover.

## Tests added or changed

`crates/omvcs-model/tests/content_hashing.rs` adds tests for:

- Standard SHA-256 vectors for empty input and `abc`.
- A fixed canonical-metadata byte vector whose expected digest is independent
  of the implementation under test.
- Identical digest bytes across typed object namespaces, while formatted
  identifiers remain distinct.
- Metadata map insertion-order and set-like array-order invariance using the
  WORK-0002 canonicalization API.
- Rejection of duplicate raw JSON member names before hashing.
- External wrapper/signature/timestamp changes with the same body bytes.
- Identical raw Resource bytes across changed filename, path, provider,
  Platform URL, credential, and replica-availability contexts.
- One-byte Resource mutation and deterministic hashing across byte inputs of
  lengths 0 through 1024.

No streaming API was added, so the conditional streaming/whole-buffer test
does not apply.

## Commands run

- `cargo fmt --all` — passed.
- `cargo fmt --all -- --check` — passed.
- `cargo test -p omvcs-model` — passed (7 unit tests, 16 canonicalization
  integration tests, 8 content-hashing integration tests; doc tests passed).
- `cargo clippy -p omvcs-model --all-targets -- -D warnings` — passed.
- `git diff --check` — passed.
- Independent Verifier review — no blocking findings. The non-blocking
  observation about unused location/replica fixture fields and the scope of
  the shared coverage row was addressed: the identity test now varies and
  asserts those fields, and the broader row remains `planned`.
- Verifier follow-up on that remediation — approved; the specific finding is
  fully resolved.

The independent Verifier also reported passing its locked focused test,
formatting, strict Clippy, and diff-whitespace checks.

## Semantic decisions made beyond the specification

None.

## Design Gaps discovered

None. DG-0003 and DG-0005 are resolved by ADR-0001 and ADR-0005.

## Assumptions

- Callers pass the canonical historical object-body bytes produced by
  `canonicalize_metadata_body`; hashing does not redo serialization or unwrap
  external storage/protocol envelopes.
- SHA-256 is provided by the `sha2` crate; this is an implementation choice,
  not OMVCS semantics.

## Known limitations

- No streaming hash interface, chunk hashing, storage layout, or chunk
  reconstruction behavior is implemented.
- Metadata schemas and historical-object validation remain outside WORK-0003.
- The coverage entry for the shared WORK-0003/WORK-0004 chunking/location
  requirement remains `planned`; no WORK-0004 behavior was started.

## Remaining work

- None within WORK-0003. WORK-0004 remains a separate planned package and was
  not started.
- Do not start WORK-0004 or any later M1 package as part of this handover.

## Git state

Working tree: CLEAN
Remote push performed: NO
