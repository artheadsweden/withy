# DG-0025 — Adapter Working State persistence boundary

Status: RESOLVED
Resolved by: ADR-0025
Classification: BLOCKS-FEATURE
Discovered by: Spec Guardian
Discovered during: DG-0018 resolution
Date: 2026-10-09

## Relevant specifications

- `Specs/OMVCS Core Specification.md` §§3, 19–21, and 82–83.
- `Specs/OMVCS DAW Adapter Specification.md` §§9, 12, 35–44, 65–69, and
  159–162.
- `Specs/OMVCS Core Invariants Specification.md`, INV-PROJ-004 and
  INV-DAW-006.
- `docs/decisions/ADR-0018-working-state-lifecycle-and-base-contract.md`,
  Decisions 5–7.

## Problem

ADR-0018 requires persistent local Core Working State operational metadata
and identifies the Project, optional Base Revision, optional Line,
component-source map, and references needed to identify current mutable
working content. It also requires preservation of mutable Adapter/project-
level state needed to reconstruct a local session without inventing a
historical Adapter State identity.

The generic Adapter contract describes capture and restore of native DAW
state, DAW-native dirty state, and a durable Project association, but does
not define what Core persists or references for mutable Adapter/project-
level Working State across restart, nor the ownership/lifecycle contract
between that data and the Core Working State record.

## Why the current specifications are insufficient

Treating mutable Adapter data as a historical Adapter State would conflict
with its immutable identity and historical schema. Treating it as opaque
local state without defining how Core can identify and recover it would not
fulfill the approved persistence/reconstruction requirement. The ownership,
reference, and restart behavior therefore require a normative boundary, not
an implementation guess.

## Affected work

- WORK-0012 persistence and restart recovery of adapter/project-level
  mutable Working State.
- Core/DAW Adapter boundary and corresponding persistence/recovery tests.

## Can unaffected work continue?

Yes. The Core Working State Base Revision, Line association, component-source
map, and change-status rules can be specified independently. WORK-0012's
complete persistence/reconstruction acceptance remains blocked until this
gap is resolved.

## Candidate directions

Non-normative only:

- Adapter-owned durable mutable state referenced by Core operational
  metadata;
- Adapter-provided recoverable state handles with Core-managed lifecycle.

Neither direction is selected. Immutable historical Adapter State identity
must not be repurposed as the mutable-state identifier.

## Required decision

Define what Core persists or references for mutable Adapter/project-level
Working State, which boundary owns the data, how it is recovered after
restart, and how updates coordinate atomically with the Core Working State
record. Keep provider-specific durable storage mechanics outside WORK-0012.

## Resolution

RESOLVED by human-approved ADR-0025. Core persists an opaque
`AdapterWorkingStateRef` and owns which reference is committed; the Adapter
owns the durable mutable state behind it. Adapter preparation precedes
atomic Core commit with related Working State metadata. Restart recovery
must explicitly report missing, invalid, unavailable, or unrestorable
references and must not silently fabricate state or misrepresent historical
Adapter State as mutable state. Physical storage and transaction mechanics
remain outside WORK-0012.
