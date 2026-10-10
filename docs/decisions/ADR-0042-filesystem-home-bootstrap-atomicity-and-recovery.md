# ADR-0042 — Filesystem Repository Home bootstrap atomicity and recovery

Status: ACCEPTED
Date: 2026-10-10
Decision owner: Human
Related gap: DG-0035

## Context

ADR-0041 defines the filesystem Repository Home marker and initial
generation-zero Storage Map, but does not define cross-record bootstrap
atomicity, interruption classification, or retry behavior.

## Decision

1. Filesystem Repository Home bootstrap is one logical atomic initialization
   operation. It is successful only when both the valid ADR-0041 marker and
   all required initial ADR-0040 operational state exist, are durable under
   the declared filesystem/storage durability semantics, and are mutually
   consistent. This includes the empty Storage Map at
   `StorageMapGeneration` 0. Bootstrap MUST NOT report success before the
   same root can be rediscovered as that initialized Home.
2. The externally observable bootstrap states are:
   - `uninitialized`: no valid OMVCS Home has been established at the
     selected root;
   - `initialized`: the valid supported marker and complete required
     initial operational state are present, durable, and mutually
     consistent;
   - `incomplete_initialization`: artifacts of bootstrap exist but the
     complete required initialization set is absent, invalid, or
     inconsistent. This is not a usable Repository Home and MUST be reported
     explicitly, not as `not_repository`, initialized Home, or ordinary
     missing metadata.
3. The logical commit point is the durable transition at which the marker
   and complete initial operational state become authoritative together.
   The filesystem implementation MAY use private staging files/directories
   and multiple writes. Staging paths, private file names, and the physical
   commit mechanism are implementation details and MUST NOT become a
   normative layout unless independently required for interoperability.
   The marker alone is never the logical commit point while required
   operational state is absent.
4. Before logical commit, bootstrap failure MUST leave the root
   non-initialized; staging artifacts are non-authoritative. Automatic
   cleanup of abandoned staging artifacts is not required. After logical
   commit, discovery or a retry MUST recognize the complete Home as
   initialized even if the caller did not receive the original success
   response.
5. Discovery and retry at the explicit root MUST classify deterministically:
   - no marker and no evidence of bootstrap artifacts: `not_repository`
     / `uninitialized`;
   - complete valid supported marker and complete consistent required
     operational state: initialized;
   - malformed marker without conflicting/partial operational-state
     evidence: invalid repository metadata;
   - valid but unsupported marker schema/layout: unsupported version;
   - requested Project differs from a valid marker: identity mismatch;
   - partial bootstrap artifacts, valid marker with missing/invalid
     required initial operational state, or initial operational state
     without a valid marker: `incomplete_initialization`.
   A result with a more specific invalid/unsupported/identity-mismatch
   cause MUST NOT be converted into success.
6. Retrying at the same root is idempotent for the same Project and
   supported layout. If no authoritative initialized state exists, retry
   MAY resume or restart only when the existing marker, if any, matches the
   requested Project and supported schema/layout, and the implementation
   can establish that completion is safe from its own validated bootstrap
   state. It MUST NOT trust incomplete artifacts merely because they
   exist. A fully valid Home for the same Project/layout returns a typed
   `already_initialized` or equivalent idempotent-success result and MUST
   NOT rewrite generation-zero metadata unnecessarily. Conflicting or
   inconsistent marker/operational state fails explicitly as
   `incomplete_initialization` or an equivalent typed inconsistency; no
   automatic repair is performed. A Home initialized for a different
   Project fails with identity mismatch and MUST NOT be overwritten.
7. Initial successful Home creation establishes exactly
   `StorageMapGeneration` 0. This is initialization, not a state-changing
   CAS mutation; bootstrap MUST NOT advance generation to 1 because it
   wrote multiple files. Subsequent Storage Map mutations continue to obey
   ADR-0034.
8. Crash/interruption consistency requires deterministic rediscovery as
   `uninitialized`, `initialized`, or `incomplete_initialization`, based on
   the persisted marker, required initial operational state, and validated
   bootstrap evidence. This ADR requires no perfect cleanup and makes no
   filesystem guarantee beyond the supported platform's declared
   durability and publish primitives. If a platform cannot provide the
   required logical atomicity, safe containment, and durability, it MUST
   report filesystem Home capability as unsupported rather than weaken the
   contract.
9. Any staging and commit machinery MUST remain beneath the explicitly
   selected repository root, obey WORK-0019 containment/no-follow rules,
   and avoid caller-controlled arbitrary paths or symlink/reparse escape.
10. This decision applies only to filesystem Repository Home bootstrap
    under ADR-0041. It does not define marker mapping, staging, or discovery
    semantics for non-filesystem Homes; DG-0036 remains OPEN. DEC-STORAGE-013
    remains OPEN and no official reference designation is made.

## Rationale

The logical initialization boundary prevents a valid-looking marker or
partial operational files from being mistaken for a usable Home, while
permitting platform-appropriate staging and durable publication.

## Alternatives considered

- Treat marker creation alone as successful bootstrap: rejected because
  required initial operational state may still be absent.
- Require one physical multi-file filesystem transaction: not required;
  platforms may use staging and a durable logical commit protocol.
- Automatically repair arbitrary marker/operational conflicts: rejected;
  repair semantics are outside this decision.
- Generalize the filesystem protocol to non-filesystem Homes: excluded;
  DG-0036 remains open.

## Specification impact

- `Specs/OMVCS Storage Adapter Specification.md` §§46, 51, 64, 124–126.
- `Specs/OMVCS Core Specification.md` §34.
- `Specs/OMVCS Glossary.md`, Incomplete Initialization.
- `Specs/OMVCS Core Invariants Specification.md`, new INV-STOR-006.
- WORK-0019, decision register, spec coverage, M3 milestone, and project
  state.

## Test impact

Conformance tests MUST cover each discovery classification; failure or
interruption before and after logical commit; marker-only and
operational-state-only partial cases; idempotent same-Project retry;
different-Project rejection; generation remaining 0 after initialization;
non-authoritative staging; no auto-repair; root containment; and explicit
unsupported behavior when required durability/publication primitives are
unavailable.

## Implementation impact

WORK-0019 may implement filesystem bootstrap staging, deterministic
classification, and safe idempotent retry under this contract. It does not
authorize generic repair, destructive cleanup, orphan deletion, M4 recovery
transactions, or non-filesystem Home semantics.

## Compatibility / migration impact

This clarifies the existing filesystem Home initialization boundary. It
does not alter Resource/Chunk keys, Home minimum capabilities, marker
schema, Storage Map CAS, or non-filesystem provider semantics.

## Notes

DG-0036 remains OPEN. DEC-STORAGE-013 remains OPEN.
