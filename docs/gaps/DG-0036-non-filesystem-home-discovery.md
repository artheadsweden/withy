# DG-0036 — Non-filesystem Repository Home discovery contract

Status: OPEN
Classification: BLOCKS-FEATURE
Discovered by: Spec Guardian
Discovered during: WORK-0019 cross-Spec impact audit
Date: 2026-10-10

## Relevant specifications

- `Specs/OMVCS Storage Adapter Specification.md` §§10, 13, 124–126.
- ADR-0040, Repository Home minimum capabilities.
- ADR-0041, filesystem Repository Home marker and discovery.

## Problem

Storage Adapter §10 requires a conforming Repository Home to provide
marker and discovery information defined in §§124–126. Those sections
define an explicit root marker and discovery operation only for a
filesystem-backed Repository Home. The specifications do not say whether
non-filesystem Repository Homes must implement the same logical marker
through provider-specific storage, use another discovery mechanism, or
declare the filesystem-only requirement inapplicable.

## Why the current specifications are insufficient

ADR-0040 establishes a minimum conformance profile for a Repository Home
generally and requires marker/discovery information defined by ADR-0041.
ADR-0041 defines a filesystem path and explicit filesystem-root behavior.
Applying that marker literally to non-filesystem Homes is not portable;
omitting marker/discovery may fail the unconditional Home profile in
Storage Adapter §10. Selecting a provider-neutral mapping or changing the
scope of the Home requirement would add semantics not stated in the
Specs.

## Affected work

- Future non-filesystem Repository Home implementations and conformance
  tests.
- WORK-0019 is not blocked: it implements a filesystem Home and can use
  the exact marker and discovery contract in ADR-0041.

## Can unaffected work continue?

Yes. WORK-0019 filesystem Home work and Resource/Chunk byte storage may
proceed, subject to DG-0035 for bootstrap failure atomicity. No
non-filesystem adapter may claim complete Repository Home conformance
based on an inferred marker mapping or discovery mechanism.

## Candidate directions

None recorded. No non-filesystem marker mapping or discovery behavior is
selected here.

## Required decision

Clarify whether the Home marker/discovery requirement in the minimum
conformance profile applies only to filesystem-backed Homes or define the
provider-neutral marker/discovery contract for non-filesystem Homes,
including how it maps to provider operations.

## Resolution

UNRESOLVED
