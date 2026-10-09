# DG-0026 — Working State operation failure and retry contracts

Status: RESOLVED
Resolved by: ADR-0026
Classification: BLOCKS-FEATURE
Discovered by: Spec Guardian
Discovered during: DG-0018 resolution
Date: 2026-10-09

## Relevant specifications

- `Specs/OMVCS Core Specification.md` §§19–24 and 82–83.
- `Specs/OMVCS DAW Adapter Specification.md` §§12, 35–44, 118, and 159–163.
- `docs/decisions/ADR-0018-working-state-lifecycle-and-base-contract.md`,
  Decision 16.

## Problem

ADR-0018 defines the accepted Working State record concepts, state effects,
and atomicity boundary. Core §83 additionally requires each operation to
specify inputs, preconditions, authorization, effects, failure modes, retry
behavior, idempotency behavior, and result.

The existing operation descriptions do not define, for each initial-state,
full/custom materialisation, Line-association, and inspection/change-
detection operation, the outcomes for repeated requests, retry after partial
Adapter work, or partially successful Adapter restore/capture results.

## Why the current specifications are insufficient

These behaviors determine whether a retry replaces current local work,
returns an existing state, or reports partial/failure status. Choosing
operation results or idempotency rules would add observable Core semantics
not stated by ADR-0018 or the existing operation contracts.

## Affected work

- WORK-0012 operation contracts and conformance tests.
- Core/DAW Adapter lifecycle boundary.

## Can unaffected work continue?

Yes. ADR-0018's classification, Base Revision, Line association, source-map,
and derived change-status rules are settled. WORK-0012's executable
operation-level implementation remains blocked until these contracts are
specified.

## Candidate directions

Non-normative only:

- define same-target rematerialisation as idempotent only when no conflicting
  current Working State exists;
- define explicit partial results for Adapter restore and a retry/resume
  boundary.

Neither direction is selected.

## Required decision

Specify per-operation inputs, preconditions, authorization, Core/Adapter
effects, failure modes, retry behavior, idempotency behavior, and result for
the WORK-0012 Working State operation set, including how partial Adapter
work is represented without violating DG-0024's replacement safeguards.

## Resolution

RESOLVED by human-approved ADR-0026. Working State operations have explicit
results, preconditions, retry and idempotency behavior. Initial-state
creation refuses an existing state; inspection is read-only; same-value
Line association is a no-op. Preparation failure preserves the old
authoritative record. Partial destructive Adapter failure reports
`recovery_required`, is not automatically retried, and requires a new
explicit recovery invocation. Retries re-read and revalidate state and do
not inherit discard authorization. Recovery condition is distinct from
Core comparison status.
