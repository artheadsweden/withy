# DG-0014 — Revision schema and canonical hash preimage

Status: RESOLVED
Classification: BLOCKS-MILESTONE
Discovered by: OMVCS Lead
Discovered during: WORK-0008 preflight
Date: 2026-10-09

## Relevant specifications

- `Specs/OMVCS Core Specification.md` §§4–5, 14–15, 43, 55–56, 59–60, 76–77.
- `Specs/OMVCS Glossary.md`: Revision, Revision Identifier, Parent Revision,
  Revision Graph, Line, Release, Provenance, and Provenance Link.
- `Specs/OMVCS Core Invariants Specification.md`: INV-HIST-001–007,
  INV-RES-002, and INV-PROJ-001.
- Related accepted decisions: ADR-0001 (Revision parents and provenance are
  set-like), ADR-0002 (author references use stable ActorId), ADR-0004–0005,
  ADR-0009–0012 (upstream object identity, canonicalization, admission, and
  Project State contracts).

## Problem

The specifications establish that a Revision is immutable, identifies exactly
one complete Project State, has zero or more parents, an ActorId author, a
creation timestamp, a message or description, and any required provenance.
They establish that an initial Revision has no parents, normal Revisions
generally have one parent, integration MAY have multiple parents, and parents
and provenance are set-like collections. A canonical metadata object includes
a schema version, and its identifier is the hash of its canonical form.

The specifications do not establish the complete OMVCS 0.1 Revision member
allowlist or the exact body and hash preimage. In particular:

- `project_id` appears in the conceptual JSON in Core §14, but no normative
  text says whether it is a required Revision member, whether a Revision is
  Project-specific only through its referenced Project State, or whether both
  references are required and must agree.
- The author is normatively an ActorId, but the conceptual example nests it as
  `author.actor_id`; the exact serialized shape is not otherwise fixed.
- The Revision message-or-description requirement does not select a member
  name or exact shape.
- Provenance is described as structured historical metadata and several
  relationship kinds are named, but its entry schema, authority, and
  requiredness are not specified. ADR-0001 classifies a `provenance` array as
  set-like, but does not define the array's entries or whether the field must
  be present.
- The exact required/optional field set, top-level extension policy, and
  corresponding complete canonical hash preimage are not stated.

Core §15 specifies UTC RFC 3339 timestamps with sufficient precision to
preserve ordering where available and explicitly says time is informational,
not causal ancestry. The precise lexical profile for equivalent timestamp
representations is not separately normalized; this must be considered when
the complete hash body is selected.

## Why the current specifications are insufficient

The §14 JSON is labelled a conceptual structure and cannot alone settle the
closed member set, requiredness, or admission schema. The prose requires a
Revision message or description and “any required provenance metadata” but
does not define which provenance is required or how it is represented.
Likewise, an example containing `project_id` is not an unambiguous normative
choice when the referenced Project State already contains a required
Project Identifier under ADR-0012.

Choosing whether to add or omit `project_id`, how to serialize the author and
message, what a provenance record contains, or which optional members are
hashed changes canonical Revision bytes and identifiers. Implementing any
one interpretation would therefore invent historical semantics.

## Affected work

- WORK-0008: Revision schema, admission, ancestry validation, identity, and
  acceptance tests.
- M1: the normative data-model milestone cannot be complete until the
  Revision historical contract is executable.
- `crates/omvcs-model/`: only after the schema and admission contract is
  resolved.

WORK-0001 through WORK-0007 are unaffected and remain verified.

## Can unaffected work continue?

Yes. Verified work through WORK-0007 and unrelated repository maintenance may
continue. WORK-0008 implementation and its final conformance tests must stop
until the Revision body and provenance contract are resolved. Do not begin
Lines, Releases, repository mutation, publication, storage, contribution
integration, or branch/ref mechanics under this gap.

## Candidate directions

None selected. Possible body profiles and provenance representations are
discussion material only and are not approved decisions.

## Required decision

Approve an exact OMVCS 0.1 Revision contract that specifies:

1. the closed or extensible top-level member set and each field's requiredness;
2. whether a Revision has a direct `project_id` member or is Project-specific
   only through its required Project State, including any consistency rule;
3. exact serialized forms for the required schema, Project State reference,
   parent Revision identifiers, ActorId author, timestamp, and message;
4. provenance field requiredness, entry schema, identity authority, and
   applicable canonicalization;
5. exact canonical hash preimage and treatment of unknown fields; and
6. any lexical timestamp normalization needed for interoperable identity,
   while preserving Core §15's informational-time rule.

The decision must preserve the already specified complete Project State
reference, immutable ancestry, ActorId stability, UTC RFC 3339 timestamp
semantics, set-like parent/provenance ordering, and infrastructure independence.

## Resolution

Resolved by the human-approved [ADR-0014 — Revision schema and canonical hash
preimage](../decisions/ADR-0014-revision-schema-and-hash-preimage.md), with
corresponding updates to the Core Specification, Glossary, Core Invariants,
WORK-0008, and specification coverage.

The OMVCS 0.1 Revision body is closed and contains exactly required
`schema`, `project_state_id`, `parents`, `author_id`, `created_at`, `message`,
and `provenance` members. Project identity is obtained through the admitted
Project State; no direct `project_id` is present. Parent Revision references
must resolve to admitted Revisions for that same Project. Provenance entry
semantics are owned by the exact versioned Revision schema. The canonical
timestamp profile is UTC RFC 3339 with uppercase `T` and `Z` and exactly nine
fractional digits. The canonical hash preimage is exactly the seven-member
body, with all members participating.

No new Design Gap was discovered. The existing examples elsewhere in the
specification set concern distinct Platform, Contribution, Release, or Adapter
objects and do not conflict with the generic Revision body.
