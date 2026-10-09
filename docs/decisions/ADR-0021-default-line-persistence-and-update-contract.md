# ADR-0021 — Default Line persistence and update contract

- Status: ACCEPTED
- Date: 2026-10-09
- Decision owner: Human
- Related Design Gap: DG-0021
- Scope: OMVCS 0.1 shared Default Line operational metadata

## Context

ADR-0016 defines the closed five-member Line record and establishes that a
Default Line is not a distinct Line type or a member of that record. It left
the preference's persistence location, ownership, and update operation
unresolved under DG-0021.

The Core needs a portable, repository-owned preference that is distinct from
each client's locally selected/current Line. The Platform may mirror
repository metadata, but must not become the authority for this preference.

## Decision

1. A Project MAY have an optional Default Line. It designates at most one
   existing Line in that Project and is represented conceptually as
   `ProjectId -> optional LineId` in shared repository operational metadata.
2. The preference MUST be persisted in Repository Home as repository-owned
   operational metadata. It is not historical metadata, a Revision root, or
   a member of the Line record. Repository Home is authoritative for it.
3. A Platform MAY mirror the preference as repository-owned metadata. A
   mirror MUST NOT redefine it. Platform/account settings and client-local
   selected/current-Line preferences are separate and MUST NOT overwrite the
   Repository Home value.
4. `SetDefaultLine` is an atomic expected-current-value compare-and-swap
   operation. It takes the Project ID, an optional expected current Line ID,
   and an optional requested new Default Line ID. An absent expected value
   means no Default Line was current; an absent requested value clears the
   preference. The Project MUST exist, and a supplied new Line ID MUST
   identify an existing Line in that Project. A stale expected value is a
   concurrency conflict. Failure leaves the preference unchanged.
5. Setting or clearing the Default Line does not mutate any Line, change a
   Line generation, alter Revision history, or create another history root.
   `CreateLine` MUST NOT implicitly set a Default Line. No separate
   Default-Line generation counter or Line-mutation history is introduced.
6. `DeleteLine` MUST fail atomically while the Line is the current Default
   Line. The caller must first explicitly change or clear the preference.
   The check that the Line is not current Default, Line existence,
   expected-generation comparison, and deletion MUST form one atomic
   decision. This prevents a concurrent `SetDefaultLine` from leaving a
   dangling reference. The Line is not implicitly cleared or reassigned.
7. Clients MAY maintain a locally selected/current Line independently of
   the shared Default Line. Changing such local selection does not change
   the repository preference.

## Rationale

Repository Home already stores current operational metadata, while the
Platform Protocol treats repository-owned metadata as mirrored rather than
Platform-owned. A Project-scoped pointer to stable `line_id` preserves those
boundaries without changing Line identity or historical data.

The expected-current-value check prevents one client from silently replacing
another client's Default Line update. Preventing deletion of the designated
Line, in the same atomic decision as deletion, preserves referential
integrity under concurrent updates without introducing an implicit
fallback.

## Consequences

- Core can expose a deterministic, shared Default Line preference without
  adding a Line member or modifying immutable history.
- Clients can retain local selection independently.
- Platforms can synchronize a mirror without acquiring ownership.
- WORK-0010 can test and implement the preference as part of the Line
  contract.
- This ADR does not determine exact Platform Mirror schema or transport.

## Explicit non-decisions

This ADR does not introduce a Default Line subtype, a `default` Line field,
implicit default assignment, Git branch/HEAD/ref/reflog/tag/merge-base/
force-push semantics, automatic Line deletion retention or pinning, or any
additional Line-mutation history. DEC-CORE-008 remains separate.

## Specification and conformance impact

- Core Specification §§3.2, 16–17, 26, 56–58, 82–83.
- Glossary: Default Line and Operational Metadata.
- Core Invariants: INV-HIST-011.
- Platform Protocol §§9–12.
- Interaction examples that distinguish local selection from the shared
  Default Line.
- WORK-0010, WORK-0013, and `docs/spec-coverage.md`.

Conformance coverage must include optional Project-scoped designation,
same-Project/existing-Line validation, expected-current-value CAS, stale
conflict and atomic failure, clearing, no implicit default on creation,
Line/Revision non-mutation, local selection independence, repository
authority versus Platform mirroring, and atomic prevention of deleting the
current Default Line without changing DEC-CORE-008 retention/pinning policy.

## Notes

This records only the human-approved DG-0021 contract. No additional
semantic decisions were made.
