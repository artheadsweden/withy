# DG-0017 — Release object and admission contract

Status: OPEN
Classification: BLOCKS-FEATURE
Discovered by: Spec Guardian
Discovered during: M2 preflight
Date: 2026-10-09

## Relevant specifications

- `Specs/OMVCS Core Specification.md` §§4.1–4.2, 18, 56, 74, 82–83.
- `Specs/OMVCS Glossary.md`, Project, Revision, Line, and Release.
- `Specs/OMVCS Core Invariants Specification.md`, INV-HIST-001–005,
  INV-HIST-009, INV-GC-001, and INV-REC-002.
- `Specs/OMVCS Interaction Specification.md` §§81–87, 262–266.
- `Specs/OMVCS Platform Protocol.md` §§29–30, 67, 203–209.
- Existing `DEC-CORE-002` in `docs/decision-register.md` (Reference Render
  policy; not resolved by this gap).

## Problem

The Glossary normatively distinguishes an immutable Release, which continues
to identify the same Revision, from a mutable Line. Core §18 requires an
unchanged Release target and uniqueness of a human-readable Release name
“within its namespace”; it gives a conceptual example with `schema`,
`project_id`, `name`, `revision`, `created_at`, `creator`, and `description`.
Core §§56 and 82 require target validation and list `CreateRelease`.

The normative Release identity class, closed member set, schema/version
authority, identifier and canonical identity (if any), name namespace and
comparison/uniqueness rules, metadata requiredness and mutability, creation
preconditions, and same-Project relationship among Release, Project, and
target Revision are not defined. The conceptual example cannot determine
which fields are historical, operational, required, or identity-bearing.
Nor does the Core operation list define the creation result or failure
semantics.

`DEC-CORE-002` separately leaves Reference Render policy open. This gap does
not select a Release render requirement or add a render member.

## Why the current specifications are insufficient

Release immutability and target immutability do not define Release identity
or admission. “Unique within its namespace” does not identify that namespace
or the name comparison rule. Treating the example as a complete schema,
assuming Release IDs are hashes or assigned IDs, or assuming tag semantics
would invent identity and compatibility rules.

## Affected work

- M2 Release representation, creation, validation, name uniqueness, and
  immutable-target enforcement.
- M2 Line/Release distinction in reference validation and repository
  history.
- Reference Render and Release policy only through the existing
  DEC-CORE-002 decision.

## Can unaffected work continue?

Yes. The immutable Revision model, Line work not dependent on Release
semantics, and generic repository validation of already specified Revision,
Project State, Component State, and Resource identity rules may continue.
Do not admit or create a normative Release object from the illustrative
shape, and do not infer a render requirement from the Release terminology or
examples.

## Candidate directions

None recorded. Choosing an assigned versus content-derived Release identity,
or a field schema, would be a semantic decision.

## Required decision

Define the OMVCS 0.1 Release identity and closed schema; required and mutable
fields; name uniqueness scope and comparison rules; creation preconditions;
validation that the target Revision and Release belong to the stated Project;
and failure behavior. Decide separately under DEC-CORE-002 whether and how a
Reference Render is associated with or required for a Release.

## Resolution

UNRESOLVED
