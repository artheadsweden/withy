# ADR-0016 — Line object and update contract

Status: ACCEPTED
Date: 2026-10-09
Decision owner: Human
Related Design Gap: DG-0016

## Context

Core §§16–17 described a mutable Line reference and expected-current-target
compare-and-swap, but did not define its complete record, identity, naming,
target validation, generation behaviour, or operation contracts. The
operation list also omitted rename. Glossary and examples did not fully
separate Line metadata from immutable Revision history or distinguish a
Default Line preference from a Line record.

## Decision

1. An OMVCS 0.1 Line is mutable operational repository metadata, not a
   content-addressed historical object. Its closed record contains exactly
   `line_id`, `project_id`, `name`, `target_revision`, and `generation`.
   Unknown additional members are invalid.
2. `line_id` is a globally unique, assigned Line Identifier using the
   assigned UUIDv7 profile and lowercase canonical textual representation.
   It is immutable for the Line's lifetime and independent of its name and
   target. `project_id` is the typed assigned Project Identifier; it is
   required and immutable. A Line cannot move between Projects. OMVCS 0.1
   defines no cross-Project Line transfer or copy.
3. `name` is required and mutable. Names are unique within one Project and
   compared by exact string/code-point equality. Core performs no case
   folding, locale-sensitive comparison, filesystem normalization, or
   Git-style normalization. Different Projects may use the same name.
   A conflicting rename fails atomically.
4. `target_revision` is one required typed Revision Identifier. Its target
   must be valid/admitted metadata in the same Project as `project_id`;
   Resource bytes need not be available. Different Lines may share a target.
   A Line never has a null/unborn target, so `CreateLine` requires an
   already-admitted target Revision.
5. `generation` is a required unsigned monotonic concurrency/version token.
   It starts at zero and increments by exactly one on every successful
   mutation of the Line record, including a successful move or rename.
   Failed mutations do not change any field or generation. Generation must
   not wrap; if its numeric domain has no next value, the mutation is
   rejected. ADR-0020 selects its exact JSON-number domain and serialization.
6. `CreateLine` takes a Project ID, name, and admitted target Revision ID.
   It requires the target to be valid/admitted in the specified Project and
   the name to be unused in that Project. Success atomically creates a fresh
   Line ID and a record at generation zero. Failure leaves no partial Line.
7. `MoveLine` atomically compares both the expected current target and
   expected generation, and takes a new target Revision ID. The Line must
   exist, both expected values must match, and the new target must be
   valid/admitted in the Line's Project. Success changes only the target and
   increments generation once. No fast-forward restriction applies; any
   admitted Revision in the same Project is an allowed target. Stale
   expectations are conflicts; invalid or cross-Project targets are
   validation failures. Failure is atomic. A successful compare-and-swap to
   the already-current target still increments generation once.
8. `RenameLine` is an explicit Core operation. It takes a Line ID, expected
   generation, and requested name. The Line must exist, generation must
   match, and no other Line in the Project may use the requested name.
   Success changes only the name and increments generation once. Failure is
   atomic and leaves the Line unchanged. Requesting the current name is valid
   and increments generation once.
9. `DeleteLine` removes the mutable Line record and does not, by itself,
   delete a Revision, Project State, Component State, or Resource. ADR-0022
   requires an atomic existence and expected-generation check, distinguishes
   missing from stale, and defines caller-directed retry behavior.
   Automatic pin/retention after deletion remains governed separately by
   DEC-CORE-008. This ADR defines no Line-mutation history or reflog.
10. A Default Line is not a distinct Line type and adds no field to a Line.
    A Project/repository may maintain an operational preference/reference
    naming at most one existing Line as its Default Line. Changing it does
    not mutate either Line, increment a Line generation, or alter Revision
    history. Its persistence location and update contract are defined
    separately by ADR-0021.
11. This decision does not introduce Git branch, tag, detached-HEAD,
    reflog, fast-forward, force-push, or merge-base semantics; implicit
    Revision creation; content-derived Line identity; or automatic
    deletion-retention/pinning behaviour.

## Rationale

A stable assigned identity allows a Line to be renamed or retargeted without
changing its identity. Keeping the mutable reference separate from immutable
Revision ancestry allows safe operational movement without rewriting history.
Project-scoped exact name uniqueness and same-Project target validation make
Line lookup and reference integrity deterministic without importing
filesystem or Git naming rules.

## Alternatives considered

- Deriving Line identity from name or target was rejected because either may
  change while the same Line persists.
- Treating Line movement as Revision creation or changing Revision ancestry
  was rejected because Lines are operational references and Revisions are
  immutable history.
- Requiring ancestry/fast-forward movement was rejected; the approved CAS
  contract permits any admitted same-Project target.
- Adding a Default Line flag to each Line was rejected; default designation
  is a separate operational preference.
- Choosing a host integer width was rejected; the numeric domain is resolved
  separately by ADR-0020. ADR-0021 separately defines the Default Line
  preference without changing the closed Line record.

## Specification impact

- `Specs/OMVCS Core Specification.md` §§4.1, 16–17, 56–58, 74, 82–83.
- `Specs/OMVCS Glossary.md`, Line and Default Line.
- `Specs/OMVCS Core Invariants Specification.md`, INV-HIST-005 and new
  INV-HIST-010.
- Line-generation and DeleteLine details are recorded in ADR-0020 and
  ADR-0022 respectively.
- `Specs/OMVCS Interaction Specification.md` §§126, 173–174 and examples
  that implied fast-forward or force-push semantics.
- `Specs/OMVCS Platform Protocol.md` §§10, 26–28, 208, clarifying Core Line
  generation versus Platform mirror generation.

## Test impact

WORK-0010 must test the exact record shape; UUIDv7 identity; project-scoped
exact name uniqueness; same-name cross-Project allowance; multiple Lines
sharing a target; same-Project admitted target validation without Resource
bytes; generation-zero creation; successful move and rename increments;
stale target and generation conflicts; atomic failures; arbitrary same-Project
target movement; field preservation; generation increments on successful
same-target moves and same-name renames; generation overflow under its
approved domain per ADR-0020; Line deletion and compare-and-delete behavior
per ADR-0022; and the separate Default Line preference contract in ADR-0021.
Automatic pin/retention tests remain governed separately by DEC-CORE-008.

## Implementation impact

- WORK-0010 implements the resolved Line record and operations.
- Default Line persistence/design is defined separately by ADR-0021.
- Generation numeric domain and exhaustion behavior are defined by ADR-0020.
- DeleteLine concurrency is defined by ADR-0022; automatic pin/retention is
  separately governed by DEC-CORE-008.
- WORK-0013 consumes Line roots only after WORK-0010's applicable Line
  contract is implemented.

## Compatibility / migration impact

No production implementation or persisted Line records are changed by this
documentation decision. Draft 0.1 implementations that admit extra Line
members, use non-canonical identifiers, permit duplicate names within a
Project, or apply Git movement rules will not conform to the closed contract.

## Notes

No semantic decisions were made beyond the human-approved contract. DG-0020,
DG-0021, and DG-0022 are resolved separately by ADR-0020, ADR-0021, and
ADR-0022. ADR-0021 does not change ADR-0016's closed Line record or
Line-mutation semantics.
