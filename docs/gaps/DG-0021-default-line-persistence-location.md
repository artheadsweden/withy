# DG-0021 — Default Line persistence location

Status: RESOLVED
Classification: BLOCKS-FEATURE
Discovered by: Spec Guardian
Discovered during: DG-0016 resolution
Date: 2026-10-09

## Relevant specifications

- `Specs/OMVCS Core Specification.md` §§3.2, 16–17, 26, 56–58, 82–83.
- `Specs/OMVCS Glossary.md`, Default Line and Operational Metadata.
- `Specs/OMVCS Platform Protocol.md` §§10–12, 26.
- `docs/decisions/ADR-0016-line-object-and-update-contract.md`.

## Problem

ADR-0016 defines Default Line as a mutable operational preference/reference
that designates at most one Line and is not part of a Line record. The
initial Specs did not identify where that preference is persisted, what its
owning record is, or the operation contract for changing it.

## Why the current specifications are insufficient

The general operational-metadata boundary alone did not define a concrete
record or operation for this preference. Putting a `default` flag in the
Line record would contradict ADR-0016; a human decision was required to
specify repository ownership, persistence, and updates.

## Affected work

- WORK-0010 Default Line persistence, designation, update, and conformance
  tests.
- Platform mirroring of the Default Line preference.

## Can unaffected work continue?

Yes. The Line record and CreateLine, MoveLine, RenameLine, and DeleteLine
contracts remain separate from the Default Line preference contract.

## Candidate directions considered when the gap was filed

At that time, the following were discussion material only and NOT approved:

- persist a Project-scoped Line Identifier in repository operational
  metadata;
- leave Default Line entirely as a client-local preference.

## Required decision

Specify whether and where a Default Line preference is persisted, its
Project/repository ownership and update operation, and how it is represented
and synchronized without adding members to the Line record.

## Resolution

Resolved by [ADR-0021](../decisions/ADR-0021-default-line-persistence-and-update-contract.md)
and corresponding updates to Core, Glossary, Core Invariants, Platform
Protocol, Interaction, WORK-0010, WORK-0013, the decision register, and
specification coverage.

The approved contract makes the Default Line an optional Project-scoped
`ProjectId -> optional LineId` preference in repository-owned operational
metadata persisted in Repository Home. Repository Home is authoritative;
Platforms may mirror but may not redefine it, and client-local selected/
current-Line preferences are separate. `SetDefaultLine` uses atomic
expected-current-value compare-and-swap and may clear the designation.
Creation does not set it implicitly. Setting or clearing it does not mutate
Lines or history. Deleting the currently designated Line fails atomically
until the preference is changed or cleared, with the default-reference check
coordinated with expected-generation deletion.

This decision does not change ADR-0016's closed five-member Line record or
resolve DEC-CORE-008 automatic deletion retention/pinning.
