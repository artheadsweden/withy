# DG-0024 — Working State destructive replacement authorization

Status: RESOLVED
Resolved by: ADR-0024
Classification: BLOCKS-FEATURE
Discovered by: Spec Guardian
Discovered during: DG-0018 resolution
Date: 2026-10-09

## Relevant specifications

- `Specs/OMVCS Core Specification.md` §§21–23, 82–83.
- `Specs/OMVCS Core Invariants Specification.md`, INV-WORK-004.
- `Specs/OMVCS DAW Adapter Specification.md` §§41, 64, 118–119.
- `Specs/OMVCS Interaction Specification.md` §§13–14, 43, 47, and 121–124.
- `docs/decisions/ADR-0018-working-state-lifecycle-and-base-contract.md`,
  Decisions 10, 12, and 17.

## Problem

The accepted Working State contract permits explicit materialisation from a
Revision to establish a new Base Revision and requires that unpublished work
not be silently discarded. Existing Adapter requirements distinguish
replacement from merge, identify destructive integration as requiring
explicit caller acknowledgement, and recommend temporary safety checkpoints
before some destructive operations. They do not specify the Core operation
preconditions, acknowledgement input/result, or failure behavior when
replacement would overwrite a current Working State reported `changed` or
`unknown`.

## Why the current specifications are insufficient

Whether to reject replacement, require a particular explicit authorization,
or expose another safe operation result changes observable Core behavior.
The integration-only acknowledgement rule does not define a materialisation
or Base-change contract, and a recommended checkpoint does not establish
permission to discard work. Selecting one would invent operation semantics.

## Affected work

- WORK-0012 destructive full replacement and any reset-like operation that
  replaces a `changed` or `unknown` Working State.
- Core materialisation operation preconditions, results, and tests.
- DAW Adapter replacement/restore contract as consumed by Core.

## Can unaffected work continue?

Yes. The Working State record, initial pre-first-Revision state, non-
destructive Line association, derived status model, and materialisation into
a new/otherwise safe state can be designed independently. WORK-0012's full
replacement behavior remains blocked until this gap is resolved.

## Candidate directions

Non-normative only:

- reject replacement when current Core status is `changed` or `unknown`;
- require an explicit caller authorization with a specified operation
  result and recovery behavior.

Neither direction is selected.

## Required decision

Define how Core authorizes or refuses materialisation/replacement when the
current Working State is `changed` or `unknown`, including the operation's
inputs, preconditions, result, and atomic failure behavior. Keep temporary
checkpoint behavior separate under DEC-INTERACTION-004.

## Resolution

RESOLVED by human-approved ADR-0024. Replacing Working State whose Core
comparison status is `changed` or `unknown` requires explicit
operation-scoped `discard_current_working_state` authorization. The default
is `preserve`; authorization is not remembered. Refusal occurs before
destructive Adapter work and leaves Working State unchanged. Temporary
checkpoint semantics remain separate under DEC-INTERACTION-004.
