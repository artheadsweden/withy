# DG-0015 — Revision provenance entry contract

Status: OPEN
Classification: BLOCKS-FEATURE
Discovered by: Spec Guardian
Discovered during: WORK-0008 preflight after ADR-0014
Date: 2026-10-09

## Relevant specifications

- `Specs/OMVCS Core Specification.md` §§14, 23, 42–43, 68, 76.
- `Specs/OMVCS Core Invariants Specification.md` INV-COL-003–004.
- `Specs/OMVCS Glossary.md`, Provenance and Provenance Link.
- `Specs/OMVCS Interaction Specification.md` §§56, 134.
- `Specs/OMVCS Platform Protocol.md` §§46, 49, 51.
- `Specs/OMVCS DAW Adapter Specification.md` §206.
- `docs/decisions/ADR-0014-revision-schema-and-hash-preimage.md`.

## Problem

ADR-0014 establishes the closed Revision body and makes the exact versioned
Revision schema authoritative for provenance entry fields, meanings,
relationship kinds, nested schemas, and collection classifications. It also
requires `provenance` to be a set-like array that MAY be empty.

The specifications do not define any permitted non-empty provenance entry
shape or the OMVCS 0.1 schema rules for such entries. At the same time, Core
§§23 and 42 and INV-COL-003 require or recommend recording source Component
States and incorporated Contribution scope; Core §68 and other workflow
examples describe recording fork or conversion provenance. Core §42 also
shows a conceptual entry with `type`, `source_revision`, `component_id`, and
`component_state`. It is not established whether that example is valid under
the exact OMVCS 0.1 Revision schema or whether those operation-level
requirements can be represented by it. Interaction, Platform Protocol, and
DAW Adapter workflows refer to provenance as part of fork, integration, or
conversion outcomes without supplying a common Revision-entry schema.

## Why the current specifications are insufficient

The exact Revision schema is identified as `omvcs.revision/0.1`, but its
allowed provenance entry members, value shapes, semantic meanings, and
operation-specific requiredness are not specified. Treating the Core §42
example or other illustrative relationships as a valid schema would invent
semantics. Treating every non-empty provenance array as invalid would also
choose semantics not stated by ADR-0014. The cross-spec requirements cannot
therefore be validated or implemented consistently for non-empty provenance.

## Affected work

- WORK-0008: Revision provenance admission and non-empty provenance
  canonicalization/conformance tests.
- Contribution integration and selective integration provenance.
- Fork/import and Adapter-conversion provenance where represented in a
  Revision.

## Can unaffected work continue?

Yes. The approved Revision body, identity, timestamp, author, message,
Project State reference, and parent rules are settled. The provenance field's
required empty representation and generic set-like canonicalization rules are
also settled. Do not implement or accept non-empty Revision provenance, infer
entry schemas from examples, or claim the affected workflows' provenance
requirements are executable until this gap is resolved. WORK-0008 as a
complete package must not be handed off until its scope and acceptance tests
can be made executable without guessing.

## Candidate directions

None recorded. Any schema shape, relationship vocabulary, and
operation-specific requiredness would be a semantic choice.

## Required decision

Define the permitted non-empty provenance entry contract for the exact
OMVCS 0.1 Revision schema, including entry members and meanings, nested
collection rules, and which Core operations require or recommend particular
provenance facts. Confirm whether the existing Core §42 example is normative,
must be revised, or remains illustrative only.

## Resolution

UNRESOLVED
