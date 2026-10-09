# ADR-0024 — Destructive Working State replacement authorization

Status: ACCEPTED
Date: 2026-10-09
Decision owner: Human
Related Design Gap: DG-0024

## Context

ADR-0018 defines Core Working State as mutable local operational state and
requires unpublished work not be silently discarded. It leaves authorization
for replacing a current `changed` or `unknown` Working State unspecified.
Existing Adapter acknowledgement and temporary-checkpoint guidance do not
define the Core operation contract for replacement.

## Decision

1. Core MUST default to preserving unpublished Working State.
2. Immediately before a replacement operation's destructive step, Core
   evaluates the current derived Core Working State comparison status:
   `unchanged`, `changed`, or `unknown`.
3. Replacement MAY proceed without destructive-replacement authorization
   when status is `unchanged`, subject to all other operation preconditions.
4. If status is `changed` or `unknown`, replacement MUST NOT proceed unless
   that invocation supplies explicit authorization equivalent to:
   `replacement_authorization = discard_current_working_state`.
   The default is `preserve`. `unknown` is conservative because Core cannot
   establish absence of unpublished work.
5. This authorization applies to exactly one invocation. It is not persisted,
   remembered, reusable, or a Project preference. It creates no provenance
   or history and does not imply that a safety checkpoint exists.
6. When `changed` or `unknown` work would be replaced under `preserve`, Core
   returns the distinct result `replacement_requires_authorization`. It
   performs no destructive Adapter replacement and changes none of the
   Base Revision, Line association, component-source mapping, or committed
   Adapter Working State reference. No Revision is created. This result is
   neither an integrity error nor a concurrency conflict.
7. Core authorization does not collapse or override DAW-native dirty state.
   Any separate Adapter requirement for acknowledging unsaved native state
   remains applicable. Adapter failure is not converted into success by the
   Core authorization.
8. A temporary safety checkpoint is not a prerequisite for this
   authorization. Any higher-level use of checkpoints remains subject to
   DEC-INTERACTION-004.

## Specification impact

- Core Specification §§21, 82–83.
- Core Invariants INV-WORK-004 and new INV-WORK-005.
- DAW Adapter Specification §§41, 64, 118–119, and 173.
- Interaction Specification §§13–14 and restore/destructive-operation
  guidance.
- WORK-0012 and WORK-0013 planning/coverage references.

## Test impact

WORK-0012 tests replacement of `unchanged` work without discard
authorization, rejection of `changed` and `unknown` work under `preserve`,
allowance with per-invocation discard authorization, no metadata/live-state
mutation on rejection, and no remembered authorization following failure.
Separate DAW-native dirty acknowledgement remains independently tested.

## Compatibility

This decision defines only the authorization and refusal boundary for
replacing Working State. It does not define checkpoint behavior or
provider-specific transaction mechanics.
