# WORK-0019 — Local filesystem Storage Adapter

Status: PLANNED — base opaque-key provider may proceed; logical layout and Repository Home features are gated
Owner agent: Storage Engineer
Milestone: M3
Branch: `work/0019-local-filesystem-storage`
Required review: Verifier

## Objective

Implement a local filesystem Storage Adapter for the approved generic
contract, with safe immutable object creation and the explicitly approved
logical-key and discovery/layout rules.

## Normative requirements

- Storage Adapter Specification §§10–21, 22–38, 49–53, 63–67, 73–76,
  85–86, 118–129, 176–189, and filesystem conformance sections.
- Core Specification §§29–36, 47–55, and 57–58.
- Core Invariants: INV-RES-001–007, INV-STOR-001–005, INV-INT-001–003,
  INV-SYNC-001–003.

## Dependencies

- WORK-0015 through WORK-0017.
- DEC-STORAGE-001: canonical logical-key layout. Without it, limit the
  Adapter to caller-supplied opaque keys and make no OMVCS logical-layout
  claim.
- DEC-STORAGE-003: Repository Home minimum, if this Adapter claims Home
  conformance.
- DEC-STORAGE-014: layout marker and repository discovery. Without it, do
  not implement Home discovery/bootstrap or layout migration.
- Chunking-dependent storage behavior waits for joint DEC-CORE-001 /
  DEC-STORAGE-002 resolution.
- DEC-STORAGE-013 is required only for official reference-conformance
  designation; implementation may not claim that designation while open.

## Allowed scope

- `crates/omvcs-storage-local/`
- Focused filesystem-specific integration tests and fixtures.

## Deliverables

- Local filesystem provider implementation for approved byte-I/O operations.
- Safe immutable creation and exact-byte retrieval.
- Filesystem byte-I/O over caller-supplied opaque keys before layout
  decisions; approved key/layout and discovery/version-marker behavior only
  after their decisions.
- Explicit handling of filesystem-specific case, path-length, link, and
  atomicity constraints within the normative contract.

## Acceptance tests

- Stored/retrieved Resource and Chunk bytes are exact and identity-checked
  under the approved verification contract.
- Atomic-create races do not silently replace existing immutable content.
- Path traversal, symlink, case-folding, and overlong-path cases cannot
  escape or alias the approved logical namespace.
- Interrupted writes do not appear as complete valid objects.
- Layout version/discovery behavior matches the resolved contract and
  rejects unsupported/ambiguous markers explicitly.
- Filesystem errors remain typed and do not become not-found or success.
- Moving a file or changing a locator does not change Resource or history
  identity.

## Explicit non-goals

- S3-compatible or other remote providers.
- Choosing logical keys, Repository Home minimum, layout marker, or chunking.
- Official reference-conformance designation before DEC-STORAGE-013.
- GC, delete, orphan cleanup, retention, encrypted storage, or public grants.
- M4 publication/recovery transaction semantics.

## Known Design Gaps

- Storage Map mutation integration must implement ADR-0034's complete-map
  conditional atomicity; a filesystem path/provider token is not the Core
  generation.
- DEC-STORAGE-001/003/014 and joint chunking decision gate their
  corresponding layout/Home/chunked features, not the restricted opaque-key
  filesystem backend.

## Implementation plan

1. Implement only isolated opaque-key filesystem operations against the
   approved contract.
2. Do not implement layout, discovery, or Home behavior until the listed
   decisions are recorded in updated Specs and this plan.
3. Test filesystem failure/aliasing/atomicity cases on supported platforms.
4. Obtain independent Verifier acceptance.

## Verification requirements

Verifier must test hostile paths, races, interrupted writes, and observable
filesystem-specific behavior, and confirm no provider metadata leaks into
historical identities.

## Completion criteria

Relevant platform tests, rustfmt, warnings-denied Clippy, coverage update,
handover, and clean diff checks pass. No conformance status is claimed beyond
what DEC-STORAGE-013 approves.
