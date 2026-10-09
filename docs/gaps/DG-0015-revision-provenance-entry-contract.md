# DG-0015 — Operation-specific Revision provenance

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

The OMVCS 0.1 Revision model requires provenance entries, when present, to
be validated by the exact available versioned Revision schema; that schema
owns entry shapes, meanings, and nested collection rules. Generic Core
validates the schema-directed structure and canonical set-like
representation and MUST NOT invent a common provenance vocabulary. This
schema-authority boundary can be implemented generically, as for other
schema-owned historical metadata.

Separately, Core §§23 and 42 and INV-COL-003 require or recommend recording
source Component States and incorporated Contribution scope; Core §68 and
other workflow examples describe recording fork or conversion provenance.
Core §42 shows a conceptual entry with `type`, `source_revision`,
`component_id`, and `component_state`. It remains unspecified which concrete
Revision-schema entries encode these operation-specific facts and exactly
which operations require them. Interaction, Platform Protocol, and DAW
Adapter workflows refer to provenance as part of fork, integration, or
conversion outcomes without supplying a common operation-to-entry mapping.

## Why the current specifications are insufficient

Treating Core §42's example as the complete normative schema or assigning it
operation-specific requiredness would invent semantics. Conversely, the
generic Revision model need not decide those semantics: it can require the
exact versioned schema authority to validate entries and apply the specified
canonical collection rules. The remaining gap is the mapping from operations
to concrete provenance facts/entries, not the generic Revision body,
identity, or schema-authority boundary.

## Affected work

- M6 Contribution integration and selective-integration provenance.
- Fork/import and Adapter-conversion provenance where represented in a
  Revision; assign to the relevant work package when planned.

## Can unaffected work continue?

Yes. WORK-0008 may implement the approved Revision model, including
schema-directed validation and canonicalization of provenance entries, using
an exact-schema authority boundary without hard-coding a generic entry
vocabulary. Tests may exercise that boundary with test-only schemas and must
not present those fixtures as normative OMVCS 0.1 entry semantics. Do not
claim the affected operation-specific provenance requirements are executable
until this gap is resolved. DG-0015 blocks those provenance-dependent
operations in M6, not WORK-0008's generic Revision model.

## Candidate directions

None recorded. Any schema shape, relationship vocabulary, and
operation-specific requiredness would be a semantic choice.

## Required decision

Define how M6 Contribution/integration operations and other applicable
operations map their required or recommended provenance facts to entries
accepted by the exact versioned Revision schema. Confirm whether the existing
Core §42 example is normative for any operation or illustrative only. Do not
change the generic schema-authority boundary already established by ADR-0014.

## Resolution

UNRESOLVED
