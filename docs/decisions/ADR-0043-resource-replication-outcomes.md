# ADR-0043 — Per-Resource replication outcomes

Status: ACCEPTED
Date: 2026-10-12
Decision owner: Human
Related gap: DG-0037

## Context

Core and Storage Adapter contracts define retrieval, transfer, verification,
Replica registration, and Storage Map CAS separately, but did not specify
how Core reports their composed result for one Resource replication
operation. DG-0037 records this gap. The user approved the outcome,
destination-effect, registration, source-preservation, and retry semantics
below.

ADR-0034 defines atomic Storage Map compare-and-swap, but does not specify
that every provider error proves non-commit, nor does it rule out an
all-or-nothing commit whose acknowledgement is lost. This decision reports
registration uncertainty only when the applicable persistence outcome
cannot establish whether the CAS committed; it does not weaken or redefine
the CAS contract.

## Decision

1. A Core per-Resource replication result has one top-level outcome:
   `success`, `incomplete`, or `failed`.
2. Every non-success result identifies the furthest relevant stage:
   `source_retrieval`, `destination_transfer`,
   `destination_verification`, or `storage_map_registration`.
3. Every non-success result classifies destination effects as exactly one of:
   - `none`: Core establishes that no persistent destination replication
     effect exists;
   - `may_exist`: provider or interruption uncertainty prevents Core from
     establishing whether destination effects remain;
   - `complete_unverified`: destination transfer is known complete, but
     destination-applicable `resource_identity` verification has not
     succeeded;
   - `verified_unregistered`: destination-applicable
     `resource_identity` verification succeeded, but successful
     registration is not confirmed. This classification does not prove the
     Storage Map lacks the Replica when registration status is `unknown`.
4. Registration status is exactly one of `not_attempted`, `not_committed`,
   `unknown`, or `committed`. `unknown` is reported only when the applicable
   CAS/persistence outcome does not establish whether an atomic commit
   occurred. A typed conflict, validation failure, unsupported result, or
   generation exhaustion that guarantees no mutation is `not_committed`.
   `committed` is valid only with overall `success`.
5. `success` requires sufficient destination transfer for verification,
   successful destination-applicable `resource_identity` verification,
   successful and known-committed Storage Map CAS registration, and the
   returned destination `ReplicaId`. A successful result MUST NOT be
   returned without all of these conditions.
6. `incomplete` is required for every non-success result when destination
   effects may or definitely do exist. It MUST NOT be represented as a
   weaker form of success. An uncertain provider outcome is `incomplete`
   with `may_exist`; it does not establish corruption, verification, or
   registration and does not authorize registration or cleanup.
7. A verification result of `failed` or `indeterminate` prevents
   registration. `indeterminate` is not corruption. A verified destination
   followed by a CAS conflict or known registration failure is
   `incomplete` with `verified_unregistered` and `not_committed`. When the
   CAS outcome cannot establish commit status, registration is `unknown`;
   the overall result remains `incomplete`.
8. `failed` is permitted only when Core establishes that no persistent
   destination replication effect exists. `failed` therefore uses
   destination effect `none` and MUST NOT claim registration committed.
9. Replication MUST NOT remove, invalidate, or demote the source Replica,
   including on success. It MUST NOT alter ResourceId, ChunkId, historical
   bytes, Component State, Project State, Revision, or creative history.
   This is distinct from Storage Migration.
10. Adapter transport retries remain permitted only where existing
    operation contracts deem them safe and idempotent. Core MUST NOT
    automatically retry the composed replication operation after it has
    returned a non-success result. A caller reinvocation is a new operation
    that re-observes the authoritative Storage Map and current evidence.
    No global replay idempotency, physical deduplication, or automatic
    operation-level deduplication is promised.
11. This decision adds no automatic cleanup, deletion, garbage collection,
    retention, source removal, durable cross-step transaction/recovery
    mechanism, or change to the separate Core §69 resumability requirement.
    Whether a WORK-0020 Resource operation falls within Core §69's
    long-running-operation requirement is tracked separately in DG-0038.

## Rationale

The result distinguishes what Core can establish from what a provider may
have persisted. It prevents provider uncertainty, partial destination
effects, verification results, and Storage Map registration from being
collapsed into success or a misleading definitive failure. Re-observation
on caller reinvocation respects the Project-wide CAS boundary without
promising replay deduplication.

## Alternatives considered

- Treat every non-success as `failed`: rejected because a failed or
  interrupted provider operation may leave destination effects.
- Treat provider write success as verification: rejected because
  destination-applicable `resource_identity` verification is required
  before registration.
- Automatically retry the entire composed operation: rejected because a
  retry could act on destination or map state the previous invocation did
  not confirm.
- Claim every provider CAS error is definitely uncommitted: rejected
  because ADR-0034 does not establish that guarantee for every uncertain
  persistence outcome.
- Add automatic cleanup or source removal: excluded from the approved
  decision and WORK-0020.

## Specification impact

- `Specs/OMVCS Core Specification.md` §§32–35, 47–51, 55, 83.
- `Specs/OMVCS Storage Adapter Specification.md` §§45, 59, 82, 252–253.
- `Specs/OMVCS Core Invariants Specification.md`, INV-RES-005 and
  INV-INT-003–004.
- `Specs/OMVCS Glossary.md`, Resource Replica, Replica Addition, and new
  Resource Replication Outcome terms.
- `Specs/OMVCS Interaction Specification.md` §§158 and 244.
- WORK-0020 acceptance tests and spec coverage.
- DG-0037, decision register, M3 preflight, milestone, project state, and
  handover.

## Test impact

WORK-0020 acceptance tests MUST cover each outcome, stage, destination
effect, and registration-state combination required by this contract,
including:

- known no-effect failures during source retrieval or destination transfer
  only when Core establishes that no persistent destination effect exists;
- verification failures after a complete destination transfer reported as
  `incomplete`/`complete_unverified`, never `failed`;
- uncertain transfer results mapping to `incomplete`/`may_exist`, without
  verification, registration, corruption, or cleanup claims;
- completed but unverified destination bytes, including failed and
  indeterminate verification, never registered;
- verified destination followed by known CAS non-commit and unknown commit
  status, both reported as `incomplete`/`verified_unregistered` without
  success, with registration `not_committed` or `unknown` respectively;
- success only after destination `resource_identity` verification and
  known-committed CAS registration with a returned `ReplicaId`;
- a non-success after any persistent or uncertain destination effect never
  reported as `failed`;
- source Replica remains registered, retrievable, and untouched on success
  and every failure path; all creative/historical identities remain
  unchanged;
- Adapter stage-local retries only where existing contracts permit them;
- Adapter failure injection proves non-idempotent provider actions are not
  blindly repeated when duplicate effects are possible;
- Core does not retry a returned non-success; caller reinvocation
  re-observes current map/evidence and receives no global deduplication
  guarantee.

Storage Adapter failure-injection tests remain stage-local and MUST NOT
claim to verify the composed Core result. WORK-0020's Core tests own that
composition. DG-0038 remains a separate readiness gate for §69 resumability.

## Implementation impact

WORK-0020 may implement the bounded per-Resource outcome contract after
DG-0038 is resolved. The Core owns outcome composition and Storage Map
registration semantics; the Storage Adapter reports only its own
operation-local facts and provider outcomes.

## Compatibility / migration impact

No persisted Resource, Replica, Storage Map, or historical representation
is changed. The contract constrains Core operation results and does not
change Resource/Chunk identity or existing Adapter operation semantics.

## Notes

DG-0037 is resolved. DG-0038 remains open and blocks WORK-0020 readiness.
No production implementation or tests are added by this ADR update.
