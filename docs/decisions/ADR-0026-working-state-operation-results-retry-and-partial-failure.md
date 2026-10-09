# ADR-0026 — Working State operation results, retry, and partial Adapter work

Status: ACCEPTED
Date: 2026-10-09
Decision owner: Human
Related Design Gap: DG-0026

## Context

ADR-0018 defines the Working State semantics. ADR-0024 defines
replacement authorization, and ADR-0025 defines the Adapter-owned durable
operational-state reference and prepare/commit boundary. Core §83 requires
implementation-facing operation contracts to state inputs, preconditions,
authorization, effects, failure modes, retry, idempotency, and result.

## Decision

### Operation contracts

1. `InspectWorkingState` is read-only. It returns persisted Core Working
   State metadata, derived Core comparison status
   (`unchanged`/`changed`/`unknown`), and applicable recovery/Adapter
   availability status. It MUST NOT mutate Working State. Repeated reads
   are idempotent, though derived status can differ if mutable content
   changes between calls.
2. `CreateInitialWorkingState` requires valid Project context and absence of
   a current Working State. It creates the initial persistent local record
   under ADR-0018. If any Working State already exists, it returns
   `working_state_already_exists`; it does not replace it. No operation ID
   is introduced solely for retry recognition.
3. `MaterialiseWorkingState` receives Project, target admitted Revision,
   optional Line association, and ADR-0024 replacement authorization. Before
   destructive work it validates Project/Revision association, derives
   current Working State status when present, enforces replacement
   authorization, and validates metadata required for the attempt.
4. Custom/selective materialisation applies the same preservation and
   failure principles. Selected Component State sources are validated
   before committing their source-map changes. It does not change Base
   Revision except when the operation explicitly performs full
   rematerialisation under ADR-0018.
5. `AssociateWorkingStateLine` changes only optional Core operational
   association metadata. Repeating the already-current association is
   idempotent success/no-op. It does not change Base Revision, move the
   Line, modify Adapter content, or create history.

### Prepare failures, restore failures, and recovery

6. If Adapter preparation/capture fails before Core commits a new
   `AdapterWorkingStateRef`, the operation fails and the previous Core
   Working State remains authoritative and unchanged. Any staged Adapter
   artifact is non-authoritative. A retry starts from the still-current
   persisted Working State and re-runs validation and authorization;
   destructive authorization is never remembered from a failed invocation.
7. If Adapter restore fails before changing live mutable state, the
   operation fails cleanly, leaves persisted Working State unchanged,
   reports Adapter restore failure, and permits an explicit caller retry.
8. If destructive Adapter work has partially changed live DAW/session state
   and restoration/materialisation cannot complete, Core returns an explicit
   result equivalent to `adapter_partial_failure_recovery_required`.
   The intended new Working State is not committed as authoritative;
   previous persisted Core metadata remains authoritative unless the
   defined commit point was reached, but the live Adapter/DAW session may
   no longer represent it. Core MUST NOT report normal success or
   `unchanged` solely because Core metadata was not committed. Automatic
   retry is prohibited; the caller/client must initiate explicit recovery
   or rematerialisation.
9. Recovery condition is separate operational status with at least:
   - `confirmed`: live Adapter state is confirmed to correspond to the
     committed `AdapterWorkingStateRef`;
   - `unconfirmed`: a committed reference exists but live restoration or
     verification has not established correspondence;
   - `recovery_required`: an Adapter operation partially changed or failed
     such that live state MUST NOT be assumed to match the committed state.
   It is not historical metadata and is distinct from Core comparison
   status.
10. Every mutating operation has one logical Core commit point. Before it,
    old persisted Working State is authoritative; after it, the new
    persisted Working State is authoritative. Base Revision, Line
    association, source map, and Adapter reference change atomically when
    an operation requires them to change together. This does not prescribe
    M3/M4 storage transaction machinery.

### Retry and idempotency

11. Core MUST NOT automatically retry destructive Adapter operations after
    authorization failure, partial Adapter failure, or a
    `recovery_required` condition.
12. A retry is a new invocation. It re-reads current Working State,
    re-evaluates derived comparison status, revalidates targets/source
    references, and supplies replacement authorization again if required.
13. Destructive materialisation has no general idempotency guarantee. It is
    a no-op/idempotent success only when Core can prove the requested
    committed state is already current and no destructive Adapter work is
    required. Otherwise repetition is a new operation and re-evaluates the
    current state.

### Failure taxonomy

14. Results MUST be machine-distinguishable for at least:
    missing/invalid Project; missing, unadmitted, or cross-Project Revision
    or Component State source; replacement requires authorization; Adapter
    preparation/capture failure; Adapter restore failure; Adapter partial
    failure/recovery required; invalid/unrecoverable
    `AdapterWorkingStateRef`; Working State already exists where creation
    requires absence; successful operation; and explicitly defined
    successful no-op.
15. DEC-INTERACTION-004 remains wholly separate. A checkpoint is not
    replacement authorization, is not required here, and does not change
    discard authorization. These ADRs do not define checkpoint lifecycle.

## Specification impact

- Core Specification §§19, 21–24, 82–83.
- Core Invariants INV-WORK-004–006.
- Glossary: Working State recovery condition and Adapter Working State
  Reference.
- DAW Adapter Specification §§12, 35–41, 64–65, 118–119, 159–163, and 183.
- Interaction Specification §§13–14 and destructive-operation/recovery
  presentation.
- WORK-0012 and WORK-0013 planning/coverage.

## Test impact

WORK-0012 tests operation-specific results and retries, Adapter prepare and
restore failures, partial destructive failure, recovery condition vs Core
comparison status, one logical commit point, atomic metadata changes,
no automatic retries, fresh authorization on retry, limited idempotency,
and the full failure taxonomy.

## Compatibility

This defines logical Core/Adapter operation behavior. It does not define
temporary checkpoint semantics or provider-specific durability mechanisms.
