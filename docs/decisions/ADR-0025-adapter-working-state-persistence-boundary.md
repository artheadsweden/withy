# ADR-0025 — Adapter Working State persistence boundary

Status: ACCEPTED
Date: 2026-10-09
Decision owner: Human
Related Design Gap: DG-0025

## Context

ADR-0018 requires persistent local Core Working State metadata and recovery
across process/DAW restart, but mutable DAW/project-level state is neither
immutable historical Adapter State nor merely volatile process memory. A
Core/Adapter boundary is required without inventing provider mechanics.

## Decision

### AdapterWorkingStateRef

1. Introduce the language-neutral conceptual type
   `AdapterWorkingStateRef`. Its concrete representation is an opaque
   Adapter-defined reference/token.
2. The reference is operational local/repository Working State metadata,
   opaque to generic Core except for equality and reference handling. It is
   not content-addressed OMVCS historical identity, provenance, ResourceId,
   AdapterStateId, Platform identity, or a credential container. The
   Core-visible form MUST contain no credentials.
3. Core Working State persists the current optional
   `AdapterWorkingStateRef`.

### Ownership and authority

4. The Adapter owns encoding of mutable DAW/project Working State, durable
   bytes/records behind its reference, validation that a reference belongs
   to the applicable Adapter/Project association, restoration of the
   referenced state, and production of a new durable reference when mutable
   state is prepared.
5. Core owns which reference is currently committed for the Project
   Working State, coordination with Base Revision/Line/component-source
   metadata, and selection of when a prepared reference becomes
   authoritative.
6. Physical provider and storage mechanics are outside WORK-0012.

### Prepare and commit

7. Updating Adapter Working State uses a logical prepare/commit boundary:
   - Adapter prepares a complete recoverable mutable state and returns a
     new `AdapterWorkingStateRef`;
   - the old Core Working State remains authoritative during preparation;
   - Core validates the prepared reference as required;
   - Core atomically commits the new Working State metadata, including the
     reference and any Base Revision, Line, or source-map changes required
     by the same operation;
   - only after Core commit may the previous reference become eligible for
     later operational cleanup.
8. Failed preparation MUST NOT change authoritative Core Working State.
   An uncommitted prepared reference is non-authoritative operational
   staging/orphan data, not history. Its cleanup policy is outside
   WORK-0012.

### Restart recovery

9. After restart, Core loads the persisted Working State and supplies its
   current reference to the applicable Adapter for restoration.
10. The persisted Core record remains the authoritative intended local
    operational state even if live restoration is not yet successful. If
    the reference is missing, invalid, unavailable, or unrestorable, Core
    MUST NOT fabricate another state, rewrite Base Revision, or silently
    fall back to historical Adapter State while claiming exact recovery.
    It reports the recovery condition specified by ADR-0026.

### Historical boundary

11. Capturing mutable Adapter Working State creates no AdapterStateId,
    Project State, or Revision. It does not enter immutable history merely
    because it is durable. Historical Adapter State remains the immutable
    content-addressed object defined by the existing Specs.

## Specification impact

- Core Specification §§3, 19, 21, 24, 62, and 82–83.
- Glossary: Adapter Working State Reference and Working State recovery
  condition.
- Core Invariants INV-PROJ-004 and new INV-WORK-006.
- DAW Adapter Specification §§9, 12, 13, 35–41, 118, and 159–163.
- DAW Adapter Specification §183.
- Ardour Reference Adapter Design §133.
- WORK-0012 and WORK-0013 planning/coverage.
- No provider-specific Storage Adapter or Platform behavior is added.

## Test impact

WORK-0012 tests operational/non-historical reference treatment, prepare
failure preserving the old authoritative record, restart restore through the
committed reference, invalid/unavailable reference reporting recovery rather
than silent fallback, and atomic Core commit of the reference with related
metadata.

## Compatibility

The conceptual reference is a language-neutral Adapter boundary, not a
concrete wire encoding or physical storage design. Adapter operation
serialization remains governed by its separate specification work.
