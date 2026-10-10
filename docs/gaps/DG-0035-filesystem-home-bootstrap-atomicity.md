# DG-0035 — Filesystem Repository Home bootstrap atomicity

Status: RESOLVED
Classification: BLOCKS-FEATURE
Discovered by: Spec Guardian
Discovered during: WORK-0019 pre-implementation audit
Date: 2026-10-10

## Relevant specifications

- `Specs/OMVCS Core Specification.md` §34, Storage Map initialization.
- `Specs/OMVCS Storage Adapter Specification.md` §§46, 51, 124–126.
- ADR-0034, Storage Map generation and CAS.
- ADR-0041, filesystem Repository Home marker and discovery.
- `docs/plans/WORK-0019-local-filesystem-storage.md`.

## Problem

Filesystem Repository Home bootstrap MUST create the marker and initialize
the empty Storage Map with generation `0`. The marker is explicitly a
separate record, and bootstrap MUST fail if it cannot initialize the
required operational state. The specifications do not define whether the
marker and map/generation initialization form one atomic operation, nor the
required state and recovery behavior if a failure or interruption occurs
between those writes.

For example, if the marker is created but map initialization fails, the
root contains a valid marker while lacking required operational state.
Discovery validates the marker, but the specifications do not define
whether that root is recoverable by retry, invalid until repaired, or
handled another way. The new-or-empty-root bootstrap precondition also does
not specify how a partially initialized root may be retried.

## Why the current specifications are insufficient

Storage Adapter §§46 and 51 require guarded/logically atomic persistence
for mutable operational metadata and the Storage Map plus generation, but
do not extend that atomicity to the separate marker or the complete
bootstrap operation. ADR-0041 requires atomic marker creation and
initialization of the empty map at the existing operational-metadata
boundary, but does not define cross-record failure atomicity or recovery.
Choosing an ordering, rollback, retry, or repair behavior would add
normative bootstrap semantics not currently specified.

## Affected work

- WORK-0019: filesystem Repository Home bootstrap and Home-conformance
  claims.
- Future filesystem Repository Home bootstrap implementations and their
  conformance tests.

## Can unaffected work continue?

Yes. WORK-0019 Resource/Chunk byte-storage operations and tests may
proceed. WORK-0019 MUST NOT claim complete Repository Home bootstrap
conformance or define retry/repair behavior for a partially initialized
Home until this gap is resolved.

## Candidate directions

None recorded. No bootstrap failure/recovery behavior is selected here.

## Required decision

Specify whether filesystem Repository Home marker creation and initial
Storage Map/generation creation must be atomic as one logical bootstrap
operation. Define the observable and recoverable state after interruption
or failure at each initialization boundary, including whether and how
bootstrap may be retried.

## Resolution

Resolved by human-approved ADR-0042. Filesystem Repository Home bootstrap
is one logical atomic initialization operation. The marker and complete
required initial operational state, including the empty Storage Map at
generation `0`, become authoritative together at a durable logical commit
point. Discovery classifies roots as uninitialized, initialized, or
`incomplete_initialization` and preserves the explicit invalid-marker,
unsupported-version, and Project-mismatch outcomes. Same-Project retry is
idempotent; safe resume/restart is allowed only from validated bootstrap
state, and automatic repair is not defined. See ADR-0042 and Storage
Adapter §§46, 51, 64, 124–126; Core §34; INV-STOR-006.
