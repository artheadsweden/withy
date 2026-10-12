# DG-0039 — Storage Migration resumability designation

Status: OPEN
Classification: BLOCKS-FEATURE
Discovered by: Spec Guardian
Discovered during: ADR-0044 cross-Spec review
Date: 2026-10-12

## Relevant specifications

- `Specs/OMVCS Core Specification.md` §§35, 69, and 83.
- `Specs/OMVCS Storage Adapter Specification.md` §§56–58 and 113–115.
- `Specs/OMVCS Interaction Specification.md` §§95–98 and 241–243.
- `Specs/OMVCS Glossary.md`, Storage Migration, Invocation-Scoped
  Operation, Durable Resumable Workflow, and Transport Resume.
- ADR-0044, Durable resumable workflow boundary.
- WORK-0021, Storage migration.

## Problem

Core §83 requires every implementation-facing operation contract to
designate the operation as invocation-scoped or as a durable resumable
workflow. ADR-0044 designates WORK-0020 per-Resource replication but
explicitly does not decide the designation for Storage Migration.

Core §35 describes one-Resource migration, Storage Adapter §57 gives a
migration phase model without making it durable, and Storage Adapter §113
requires Core-orchestrated retry/resume for bulk migration without stating
whether continuation survives Core process restart. WORK-0021 excludes
durable M4 operation state but currently lacks the §83 designation needed
before implementation.

## Why the current specifications are insufficient

Choosing invocation scope for all migration would infer that the §57 phase
model and §113 resume direction require no durable Core continuation.
Choosing durable resumability would add persisted workflow identity,
continuation state, restart, and rollback requirements that the Specs and
WORK-0021 do not define. Conflating Adapter Transport Resume with Core
workflow continuation would contradict ADR-0044.

The existing text therefore does not unambiguously establish the required
§83 designation or whether single-Resource and bulk migration have
different contracts.

## Affected work

- WORK-0021 Storage migration is blocked before implementation or
  authorization.
- Any future bulk migration work package must state its own designation and
  continuation semantics.
- WORK-0020 is unaffected and may proceed after its independent
  specification review.

## Can unaffected work continue?

Yes. WORK-0020 per-Resource replication is explicitly invocation-scoped by
ADR-0044. Existing Adapter transfer/resume capabilities and migration
examples remain valid within their current boundaries. No migration
implementation may infer the missing Core designation.

## Candidate directions

None selected. Whether single-Resource migration and bulk migration share
one designation or use distinct operation contracts requires human
approval.

## Required decision

Define the Core §83 resumability designation for single-Resource Storage
Migration and for any bulk migration workflow. If either is a durable
resumable workflow, define or assign a later work package for its durable
identity, persisted continuation state, restart behavior, and safe
continuation/rollback boundary. Preserve the distinction from Adapter
Transport Resume.

## Resolution

UNRESOLVED
