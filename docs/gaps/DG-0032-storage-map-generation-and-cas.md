# DG-0032 — Storage Map generation domain and CAS contract

Status: RESOLVED
Classification: BLOCKS-FEATURE
Discovered by: Spec Guardian
Discovered during: Resolving DG-0029 and DEC-STORAGE-007
Date: 2026-10-10

## Relevant specifications

- `Specs/OMVCS Core Specification.md` §§33–34 and 58.
- `Specs/OMVCS Storage Adapter Specification.md` §§44–45, 50–51, 124,
  126, and 261.
- `Specs/OMVCS Glossary.md`, Storage Map Generation.
- `Specs/OMVCS Core Invariants Specification.md`, INV-INT-003.
- ADR-0032, Decision 7; resolved by ADR-0034.

## Problem

Core §34 requires operational versioning and describes previous/new Storage
Map generations, and Storage §§44–45 defines a generic conditional-write
abstraction with an expected version/generation/token and a conflict
outcome. The Specs do not define the interoperable Storage Map generation
value domain, initial value, successful advance rule, or exact Core
compare-and-swap operation/result contract.

The human-approved ADR-0032 contract requires a stale update to fail
atomically without changing any part of the Storage Map. That requirement
does not determine the generation representation or advancement semantics.
Line generation rules are a separate record-specific contract and cannot be
copied to Storage Map without an explicit decision.

## Why the current specifications are insufficient

An implementation needs a comparable, persistable expected version and a
defined success transition to determine whether a mutation is stale and to
return the next state. Choosing an integer domain, opaque token, initial
value, or increment/replace rule would add semantics not established by the
general ConditionalPut contract. Multi-entry mutations also need a defined
logical atomicity boundary.

## Affected work

- WORK-0016 guarded/versioned Storage Map updates.
- WORK-0017–WORK-0021 wherever they mutate the Storage Map.
- Core and Storage Adapter conformance tests for concurrent map mutation.

## Can unaffected work continue?

At discovery, WORK-0016 could model Replica identity, representation,
locator, and Storage Map contents, while guarded mutation remained blocked.
This gap is resolved by ADR-0034; the authorized scope is recorded below.
WORK-0015 was and remains unaffected.

## Candidate directions

No candidate is selected. Numeric and opaque version-token models are both
discussion possibilities only.

## Required decision

Define the Storage Map generation representation and domain, initial value,
successful advancement rule, and Core CAS request/result semantics,
including atomicity for mutations affecting multiple map entries. Preserve
the already approved requirement that a stale update fails atomically
without partial map mutation.

## Resolution

Resolved by human-approved
[ADR-0034](../decisions/ADR-0034-storage-map-generation-and-cas.md) and
specified in Core §§33–34 and 58, Storage Adapter §§44–45, 50–51, 124,
126, and 261, INV-INT-003, and the Glossary.

Each Project has one `StorageMapGeneration` guarding the complete logical
Storage Map. It is an exact unsigned JSON integer in `0 ..= 2^53 - 1`;
initialization establishes an empty map at generation 0, while absence
outside initialization remains missing/incomplete metadata. A state-
changing valid CAS commits all changes atomically and increments once.
No-op does not increment. Stale, invalid, unsupported/provider-failed, and
exhausted requests do not mutate. Provider conditional tokens are distinct
from the portable Core generation. Core §58 no longer allows same-
generation Storage Map mutations to commute automatically.

WORK-0016 guarded map mechanics are unblocked. Verified promotion remains
gated by WORK-0017 and DEC-STORAGE-004/005; DEC-STORAGE-011 shared-namespace
semantics and deletion/GC/retention remain excluded.
