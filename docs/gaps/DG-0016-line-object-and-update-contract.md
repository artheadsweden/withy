# DG-0016 — Line object and update contract

Status: RESOLVED
Classification: BLOCKS-FEATURE
Discovered by: Spec Guardian
Discovered during: M2 preflight
Date: 2026-10-09

## Relevant specifications

- `Specs/OMVCS Core Specification.md` §§4.1–4.2, 16–17, 34, 56–58, 74, 82–83.
- `Specs/OMVCS Glossary.md`, Line and Default Line.
- `Specs/OMVCS Core Invariants Specification.md`, INV-HIST-005, INV-HIST-010,
  INV-GC-001, INV-UX-002.
- `Specs/OMVCS Interaction Specification.md` §§44–49, 173, 209, 264–267; unresolved DEC-INTERACTION-001 and DEC-INTERACTION-003.
- `Specs/OMVCS Platform Protocol.md` §§26–28, 67–71.
- Existing `DEC-CORE-008` in `docs/decision-register.md` and `docs/milestones.md` M2.

## Problem

The Core Specification calls a Line mutable operational metadata, gives a
conceptual object containing `line_id`, `project_id`, `name`,
`target_revision`, and `generation`, and normatively requires compare-and-swap
for movement. The Glossary defines a Line as a named movable reference, and
the Core operation list names `CreateLine`, `MoveLine`, and `DeleteLine`.
However, the conceptual example is not a closed schema and the operation
contracts are not supplied.

The Specs do not establish the Line identifier class or generation rules;
which fields and metadata are authoritative or mutable; whether names are
unique and in what namespace; whether two Lines may target the same Revision;
the validation rule tying a Line and its target Revision to the same Project;
how initial/default Lines are created or changed; or the exact state changes
for create, move, and delete. Section 17 specifies the expected-current-target
precondition and conflict outcome for movement, but does not say whether a
generation token is required, how it changes, or how target and generation
interact in the compare-and-swap.

Line deletion itself is mentioned in Core §74, but the separate question of
whether deletion creates an automatic archival/pin period is already
`DEC-CORE-008`. This gap does not duplicate that policy question.

## Why the current specifications are insufficient

Choosing an identifier, schema, name uniqueness rule, duplicate-head rule,
generation progression, or creation/deletion side effect would establish
observable repository semantics not determined by the conceptual example.
Treating the example fields as a normative schema, or importing Git branch,
ref, detached-HEAD, force-push, or reflog behaviour, would invent semantics.
CAS on the expected target is specified; adding a required generation token
or choosing its mutation semantics is not.

## Affected work

- M2 Line model, creation, movement/CAS, default-Line handling, reference
  validation, and deletion (subject to DEC-CORE-008).
- M2 validation of Line references and project association.
- M2 Line/history presentation only to the extent of unresolved
  DEC-INTERACTION-001 and DEC-INTERACTION-003.
- Later Platform Line mirroring consumes the Core contract.

## Can unaffected work continue?

Yes. Immutable Revision and Project State models, same-Project parent
validation, full-graph ancestry traversal, and independent Release work may
continue within their specified boundaries. The expected-target CAS conflict
condition can be tested as a stated invariant, but a complete Line record or
mutation implementation must wait for a decision on the missing fields and
generation semantics. Do not implement deletion retention/pinning without
the separate DEC-CORE-008 decision.

## Candidate directions

None recorded. Possible identity and generation designs would be choices,
not consequences of the current Specs.

## Required decision

Define the OMVCS 0.1 Line record and its identity; required and mutable
members; name uniqueness namespace; whether duplicate target Revisions are
allowed; same-Project reference validation; default-Line creation/change;
generation/CAS token and increment semantics; and create, move, and delete
effects. Resolve or explicitly defer the Line-deletion pin policy separately
under DEC-CORE-008.

## Resolution

Resolved by [ADR-0016](../decisions/ADR-0016-line-object-and-update-contract.md)
and the corresponding Core, Glossary, Core Invariants, Interaction, and
Platform Protocol updates.

The approved contract defines the closed five-member Line record, stable
assigned UUIDv7 identity, Project-scoped exact name uniqueness, same-Project
admitted Revision targets, generation/CAS behavior, CreateLine, MoveLine,
RenameLine, and deletion's separation from historical-object removal.
Generation's numeric domain is resolved separately by ADR-0020, and
DeleteLine's expected-generation concurrency contract is resolved separately
by ADR-0022. Default Line persistence and the `SetDefaultLine` operation are
resolved separately by ADR-0021.

DEC-CORE-008 remains separate and unresolved for automatic pin/retention
after Line deletion. DEC-INTERACTION-001 and DEC-INTERACTION-003 remain open
presentation decisions.
