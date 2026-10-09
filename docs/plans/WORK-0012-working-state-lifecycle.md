# WORK-0012 — Working State lifecycle

Status: PLANNED
Owner agent: Core Engineer
Milestone: M2
Branch: `work/0012-working-state-lifecycle`

## Objective

Implement the approved Core Working State representation and lifecycle while
keeping mutable local work distinct from immutable Project State and Revision
history.

## Normative requirements

- Core Specification §§3, 19–25, 56, 62, 65, and 82–83.
- Glossary: Working State, Base Revision, Materialisation, Selective
  Materialisation, Custom Working State, Local Modification,
  AdapterWorkingStateRef, and Working State recovery condition.
- Core Invariants: INV-PROJ-004–005 and INV-WORK-001–006.
- DAW Adapter Specification §§9, 11–13, 30, 35–44, 65–69, 119, 153,
  159–163, and 183.
- Interaction Specification §§13–14 and 35–36.
- ADR-0018, ADR-0024, ADR-0025, and ADR-0026.

## Dependencies

- WORK-0001 through WORK-0008 verified historical object APIs.
- WORK-0010 Line contract, for optional Working State Line association.
- DG-0018 is resolved by human-approved ADR-0018 and corresponding Spec
  updates.
- DG-0024 is resolved by ADR-0024: operation-scoped destructive replacement
  authorization.
- DG-0025 is resolved by ADR-0025: AdapterWorkingStateRef, ownership,
  prepare/commit, and restart-recovery boundary.
- DG-0026 is resolved by ADR-0026: operation failure, retry, idempotency,
  partial-failure, and recovery-status contracts.
- DEC-INTERACTION-004 remains separate for temporary local safety
  checkpoints; it does not block this Core package and its UX/lifecycle is
  excluded.

## Allowed scope

- `crates/omvcs-core/`
- Focused Working State model, lifecycle, and Core tests.

## Deliverables

- Persistent local Core Working State operational metadata, distinct from
  historical Project State and Revision identity.
- Optional Base Revision, pre-first-Revision state, and optional Line
  association with no automatic Line-following.
- Component source mapping and derived Core `unchanged`/`changed`/`unknown`
  comparison status, distinct from DAW-native dirty state.
- Opaque AdapterWorkingStateRef and Core/Adapter ownership, prepare/commit,
  atomic metadata update, and restart-recovery behavior.
- Explicit per-operation results, preconditions, failure, retry, and
  idempotency behavior.
- Scoped replacement authorization and explicit partial-failure recovery
  status; no implicit destructive retry.

## Acceptance tests

- Pre-first-Revision state has no Base Revision or synthetic initial
  Revision; successful full materialisation sets Base Revision and source
  mappings without modifying history.
- Working State mutation/materialisation does not create or mutate
  historical objects. AdapterWorkingStateRef and its backing mutable state
  are operational, not historical objects, identifiers, or provenance.
- Optional Line association does not follow Line movement or change Base
  Revision. Repeating the existing association succeeds as a no-op.
- Derived Core `unchanged`/`changed`/`unknown` status is distinct from
  DAW-native dirty state and from recovery condition.
- Replacement of `unchanged` state may proceed without discard
  authorization; replacement of `changed` or `unknown` state requires
  per-invocation authorization. Default preserve refusal leaves live and
  persisted state unchanged and invokes no destructive Adapter operation.
- Adapter prepare failure leaves the old Core record authoritative.
  Reference, Base Revision, Line association, and component-source mapping
  commit atomically where one operation changes them together.
- Restart restore uses the committed reference; invalid/unavailable/
  unrestorable references are explicitly reported without fabricated state
  or silent historical Adapter State fallback.
- Adapter partial failure reports `recovery_required`, never normal success;
  no automatic destructive retry occurs. A caller retry re-reads and
  revalidates state and supplies fresh authorization if needed.
- Initial-state creation against an existing Working State returns
  `working_state_already_exists` without mutation.
- Custom/selective materialisation validates sources before updating their
  mapping and does not change Base Revision unless it is explicitly a full
  rematerialisation.
- Temporary checkpoint behavior remains excluded under
  DEC-INTERACTION-004.

## Explicit non-goals

- DAW-native dirty-state implementation beyond the existing Adapter
  contract.
- Revision creation/publication, Line lifecycle beyond Working State
  association metadata, Releases, or temporary checkpoint UX/lifecycle.
- Temporary checkpoint UX/lifecycle under DEC-INTERACTION-004.
- Provider-specific storage/transaction/cleanup mechanics beyond the
  bounded Core/reference-store and Adapter contracts.
- M3 Storage Adapters, M4 publication transactions, Working State
  reachability traversal, Platform transport, FFI, or DAW-specific behavior.

## Known Design Gaps

- No open Design Gap blocks WORK-0012.
- DEC-INTERACTION-004 remains open for temporary local safety checkpoints
  and does not block WORK-0012.

## Implementation plan

1. Revalidate this package against ADR-0018 and ADR-0024–ADR-0026.
2. Implement only the approved Core Working State representation and
   lifecycle.
3. Add focused conformance and failure-injection tests for the acceptance
   criteria.

## Verification requirements

The independent Verifier must check the full approved lifecycle,
persistence/recovery boundary, Base Revision behavior, failure/retry
contracts, and separation from immutable history and Adapter-native dirty
state.

## Completion criteria

Formatting, focused tests, coverage-map update, independent verification,
handover, and clean Git state.
