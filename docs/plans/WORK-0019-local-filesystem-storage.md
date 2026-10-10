# WORK-0019 — Local filesystem Storage Adapter

Status: PLANNED — PARTIALLY BLOCKED; byte storage may proceed, but Home
bootstrap/conformance is blocked by DG-0035
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
- ADR-0039 resolves DEC-STORAGE-001 for canonical Resource and Chunk
  logical keys. No canonical metadata path layout is defined; use the
  existing logical metadata operation contracts without adding a layout
  claim.
- ADR-0040 resolves DEC-STORAGE-003 and defines the minimum Home profile;
  advertise Home and Resource Storage capabilities independently.
- ADR-0041 resolves DEC-STORAGE-014 for filesystem Home marker,
  explicit-root bootstrap, and discovery. Automatic layout migration
  remains excluded.
- Chunking-dependent behavior follows the approved OMVCS 0.1 policy in
  ADR-0036; it does not choose alternate chunk boundaries.
- DEC-STORAGE-013 is required only for official reference-conformance
  designation; implementation may not claim that designation while open.

## Allowed scope

- `crates/omvcs-storage-local/`
- Focused filesystem-specific integration tests and fixtures.

## Deliverables

- Local filesystem provider implementation for approved byte-I/O operations.
- Safe immutable creation and exact-byte retrieval.
- Canonical Resource/Chunk logical-key mapping, independent of physical
  storage-root prefix and content identity.
- After DG-0035 is resolved: filesystem Repository Home metadata
  operations and ADR-0034-compatible complete-map CAS, plus
  marker/bootstrap/discovery under ADR-0041.
- Explicit handling of filesystem-specific case, path-length, link, and
  atomicity constraints within the normative contract.

## Acceptance tests

- Stored/retrieved Resource and Chunk bytes are exact and identity-checked
  under the approved verification contract.
- Atomic-create races do not silently replace existing immutable content.
- Path traversal, symlink, case-folding, and overlong-path cases cannot
  escape or alias the approved logical namespace.
- Interrupted writes do not appear as complete valid objects.
- Canonical Resource/Chunk keys match ADR-0039 exactly; hostile,
  alternate-case, absolute, traversal, and platform-separator forms are
  rejected before native path resolution.
- Symlink/reparse-point escape is prevented; unsupported safe containment
  or atomic no-replace publication fails explicitly.
- The following Home acceptance tests are gated by DG-0035; do not encode
  partial-bootstrap recovery behavior before its resolution:
- Layout marker/bootstrap/discovery matches ADR-0041, including missing,
  malformed, unsupported, and Project-mismatch cases; discovery does not
  search ancestors.
- Home capability claims meet ADR-0040, independently of Resource/Chunk
  byte-storage capability; Storage Map CAS preserves ADR-0034 atomicity.
- Filesystem errors remain typed and do not become not-found or success.
- Moving a file or changing a locator does not change Resource or history
  identity.

## Explicit non-goals

- S3-compatible or other remote providers.
- Changing the approved logical keys, Repository Home minimum, or layout
  marker.
- Defining alternate chunking behavior; OMVCS 0.1 uses ADR-0036.
- Official reference-conformance designation before DEC-STORAGE-013.
- GC, delete, orphan cleanup, retention, encrypted storage, or public grants.
- M4 publication/recovery transaction semantics.

## Known Design Gaps

- Storage Map mutation integration must implement ADR-0034's complete-map
  conditional atomicity; a filesystem path/provider token is not the Core
  generation.
- DG-0034 remains OPEN and limits only generic lexical validation of
  ProviderLocator schema identifiers; WORK-0019 uses the approved opaque
  envelope and MUST NOT invent a generic grammar.
- DG-0035 remains OPEN and blocks filesystem Repository Home
  bootstrap/conformance only. Resource/Chunk byte-storage implementation
  may proceed; WORK-0019 MUST NOT invent partial-initialization recovery
  semantics or claim complete Home conformance.
- DEC-STORAGE-011 and DEC-STORAGE-013 remain OPEN and are excluded:
  no namespace-sharing claims or official reference designation.

## Implementation plan

1. Implement canonical Resource/Chunk logical keys and safe byte storage.
2. After DG-0035 resolution, implement the independently declared
   Repository Home capability profile, marker/bootstrap/discovery, and
   ADR-0034-compatible CAS.
3. Test filesystem byte-storage failure, aliasing, containment, durability,
   and atomicity on supported platforms; fail explicitly when guarantees
   are unavailable. Add Home bootstrap tests only after the gap is resolved.
4. Obtain independent Verifier acceptance.

## Verification requirements

Verifier must test hostile paths, races, interrupted writes, and observable
filesystem-specific behavior, and confirm no provider metadata leaks into
historical identities.

## Completion criteria

Relevant platform tests, rustfmt, warnings-denied Clippy, coverage update,
handover, and clean diff checks pass. No conformance status is claimed beyond
what DEC-STORAGE-013 approves.
