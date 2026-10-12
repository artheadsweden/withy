# DG-0038 — Resource replication resumability boundary

Status: OPEN
Classification: BLOCKS-FEATURE
Discovered by: OMVCS Lead
Discovered during: DG-0037 resolution impact review
Date: 2026-10-12

## Relevant specifications

- `Specs/OMVCS Core Specification.md` §§69, 83.
- `Specs/OMVCS Storage Adapter Specification.md` §§54–59, 82,
  112–115, 252–253.
- ADR-0043, Per-Resource replication outcomes.
- WORK-0020, Resource replication.

## Problem

Core §69 requires long-running operations to be resumable and says Core
MUST determine the last confirmed state after restart and continue or
safely roll back operational changes where possible. The list of
operations for which transaction state SHOULD be persisted includes bulk
replication. ADR-0043 resolves the per-Resource result and ordinary
retry/idempotency contract, but explicitly leaves §69 unchanged and
excludes M4 durable cross-step transaction/recovery machinery.

WORK-0020 covers a single Resource but places no size or duration bound on
the operation. The Specs do not determine whether a long-running
single-Resource replication is subject to §69's resumability requirement,
or how that requirement composes with ADR-0043's prohibition on automatic
Core retry after a returned non-success result.

## Why the current specifications are insufficient

Treating every WORK-0020 operation as outside §69 would narrow an existing
Core requirement without an approved decision. Treating all potentially
long-running Resource replication as §69-governed would add durable
resumability/recovery work excluded by the approved DG-0037 decision and
the current WORK-0020 package. Caller reinvocation is a new operation that
re-observes current state; it does not by itself satisfy §69's restart
continuation requirement.

## Affected work

- WORK-0020 readiness, scope, and acceptance tests.
- Any later bulk Resource replication work covered by Core §69.
- No impact on the accepted DG-0037 outcome contract or unrelated
  Storage Adapter stage-local retry contracts.

## Can unaffected work continue?

Yes. ADR-0043 resolves DG-0037 and can be used for normative outcome
reporting. WORK-0020 implementation remains blocked until the §69
applicability and resumability boundary is decided. No production
replication implementation is authorized by this gap record.

## Candidate directions

None recorded. No single-Resource duration threshold, §69 exemption,
resumability mechanism, or package expansion is selected here.

## Required decision

Determine whether and how Core §69 applies to single-Resource replication
within WORK-0020, including its interaction with the approved prohibition
on automatic Core retry after non-success and the exclusion of M4 durable
cross-step transaction/recovery semantics. Update the affected ADR,
Specifications, WORK-0020 acceptance tests, and coverage records under an
approved decision workflow.

## Resolution

UNRESOLVED
