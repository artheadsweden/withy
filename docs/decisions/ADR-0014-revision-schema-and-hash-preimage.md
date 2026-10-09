# ADR-0014 — Revision schema and canonical hash preimage

Status: ACCEPTED
Date: 2026-10-09
Decision owner: Human
Related Design Gap: DG-0014

## Context

Core Specification §§14–15 defined Revision concepts and a conceptual JSON
example, but did not settle the exact OMVCS 0.1 historical body, field
requiredness and shapes, the direct Project identity reference, provenance
schema, or complete canonical hash preimage. RFC 3339 permits multiple
representations of the same instant, while Revision timestamps participate in
content-derived identity.

These omissions could cause implementations to admit different Revision
bodies or compute different identifiers for equivalent historical meaning.

## Decision

1. The generic OMVCS 0.1 Revision historical body is a closed JSON object
   containing exactly the following REQUIRED top-level members:
   `schema`, `project_state_id`, `parents`, `author_id`, `created_at`,
   `message`, and `provenance`. Unknown top-level members are invalid.
2. `schema` uses the canonical metadata schema-version representation from
   Core §76. The exact versioned Revision schema MUST be known, available, and
   validate the candidate before historical admission. Unknown or unavailable
   schema candidates MAY be preserved outside valid history where supported,
   but MUST NOT produce a valid Revision or Revision Identifier. `schema`
   participates in identity.
3. `project_state_id` is a required typed Project State Identifier and MUST
   resolve to exactly one valid/admitted Project State. The Revision body has
   no direct `project_id`; its Project identity is obtained from that admitted
   Project State. Every parent Revision MUST resolve through its own admitted
   Project State to the same assigned Project Identifier as the current
   Revision's Project State. A parent from another Project prevents
   admission. This decision does not define cross-Project move, copy, fork, or
   identity-preservation semantics.
4. `parents` is a required set-like JSON array of typed Revision Identifiers
   under ADR-0001 and MAY be empty. An empty array represents an initial
   Revision; a normal derived Revision generally has one parent; an
   integration Revision MAY have multiple parents. Omission is invalid.
   Duplicate parent identifiers are invalid, and parent ordering has no
   semantic significance; OMVCS 0.1 has no generic first-parent concept.
   Every parent MUST resolve to a valid/admitted Revision before admission.
   Parent ancestry and provenance are separate; provenance MUST NOT replace
   the parent graph. Parent order MUST NOT encode Line, branch, ref, or Release
   semantics.
5. `author_id` is a required direct ActorId representation under ADR-0002.
   It is not wrapped in an `author` object. The ActorId MUST be valid. Current
   profile, display name, email, Platform username/account, signing key,
   rotation data, credentials, and account-service availability are not
   Revision fields and MUST NOT affect Revision identity or historical
   admission.
6. `created_at` is required, informational, and participates in identity.
   Its one canonical OMVCS 0.1 lexical form is
   `YYYY-MM-DDTHH:MM:SS.nnnnnnnnnZ`: UTC only, uppercase `T` and `Z`, exactly
   nine fractional-second digits, and nanosecond resolution. Numeric offsets,
   omitted fractional seconds, and fewer or more fractional digits are not
   canonical Revision timestamps. Inputs with finer-than-nanosecond precision
   MUST NOT be rounded or truncated into a valid historical timestamp. Parent
   relationships, not timestamps, establish ancestry.
7. `message` is a required JSON string and MAY be empty. Core MUST NOT trim,
   rewrite whitespace, case-fold, or Unicode-normalize it. Existing canonical
   JSON string serialization rules apply, and the exact string value
   participates in identity.
8. `provenance` is a required set-like JSON array under ADR-0001 and MAY be
   empty. Omission is invalid. Each entry is a canonical JSON object. The
   exact versioned Revision schema owns permitted entry members,
   requiredness, value shapes, semantic meanings, relationship kinds, nested
   object schemas, and nested array classifications. Generic Core MUST NOT
   infer provenance meanings from keys or values. Unknown or disallowed
   entry fields, invalid shapes, unclassified nested arrays, or otherwise
   invalid entries prevent admission. Entries are sorted by canonical
   serialized element bytes; duplicate canonical entries are invalid.
   Provenance describes historical relationships/context but does not alter
   or replace parent ancestry. No independent provenance-schema identifier
   is added in OMVCS 0.1; the exact Revision schema is the authority.
9. A Revision candidate may be admitted only if its exact schema validates;
   its Project State resolves as valid/admitted; parent identifiers are
   correctly typed and resolve as valid/admitted Revisions from the same
   Project; `author_id` is valid; `created_at` uses the canonical profile;
   `message` is a string; all provenance entries validate under the exact
   Revision schema; and no unknown top-level member is present. Resource-byte
   materialisation MUST NOT be required merely to validate a Revision,
   Project State, or ancestor metadata when the corresponding admitted
   metadata objects are available.
10. The Revision Identifier is SHA-256 over canonical Revision
    historical-body bytes under WORK-0002/WORK-0003. No type/domain prefix is
    included in the digest input. The canonical body contains exactly the
    seven members listed in item 1, and all seven participate in identity.
11. Revision identity MUST NOT include or depend on a direct `project_id`,
    Line/ref/branch information, Release information, validation evidence,
    validator implementation details, signatures or signature wrappers,
    Platform account/profile data, storage/Replica information, local
    DAW/runtime state, credentials, transport wrappers, or unknown extension
    fields. Adding a top-level member requires an explicit future
    schema/version and compatibility decision.

## Rationale

The closed seven-member body gives OMVCS 0.1 one interoperable Revision
identity. Project identity is obtained through the already admitted
Project-specific state instead of duplicating `project_id`. The canonical
timestamp spelling avoids multiple lexical representations for an instant.
Schema-owned provenance permits versioned semantics without inventing a
generic relationship vocabulary, while the already approved set-like rule
keeps provenance order from changing identity.

## Alternatives considered

- Include a direct `project_id`: rejected; the required admitted Project State
  already supplies Project identity, and a duplicate would introduce an
  additional consistency field.
- Represent the author as a nested `author.actor_id`: rejected; the approved
  historical fact is one stable ActorId.
- Omit required empty collections: rejected; `parents` and `provenance` each
  have one required representation, including their valid empty case.
- Use variable RFC 3339 timestamp spellings: rejected for OMVCS 0.1
  historical identity; one exact UTC nanosecond profile is selected.
- Define one generic provenance entry vocabulary: rejected; exact
  versioned Revision schemas own permitted provenance semantics and shapes.
- Add Line/Release state or a first-parent order: rejected; those are distinct
  reference concepts, and parent order is set-like in OMVCS 0.1.

## Specification impact

- `Specs/OMVCS Core Specification.md` §§5.1, 14–15, 43, 56, 59–60, 76–77.
- `Specs/OMVCS Glossary.md`: Revision, Revision Identifier, Parent Revision,
  Revision Graph, Provenance, and Provenance Link.
- `Specs/OMVCS Core Invariants Specification.md`: INV-HIST-002,
  INV-HIST-003, INV-HIST-004, INV-HIST-006, INV-HIST-007, and new
  INV-HIST-009.
- Interaction and Platform examples were searched; their message,
  Contribution, Release, and presentation fields refer to separate UI or
  Platform objects and do not conflict with the generic Revision body. Their
  operation-level provenance requirements depend on a non-empty provenance
  entry contract tracked separately by DG-0015.
- DAW Adapter Capture Result, Storage Adapter, and Ardour examples were
  searched; no conflicting generic Revision body was found. Adapter workflow
  provenance requirements are also within DG-0015's cross-spec impact.

## Test impact

WORK-0008 conformance tests MUST cover:

- all seven required members, omission rejection, unknown top-level rejection,
  exact schema availability, and exact canonical preimage;
- typed and admitted Project State resolution; absence of direct `project_id`;
- valid empty, single-parent, and multi-parent cases; typed parent IDs,
  resolution/admission, same-Project enforcement, permutation invariance, and
  duplicate rejection;
- direct typed `author_id` validation and independence from mutable
  profile/account/email/key state;
- canonical timestamp spelling, rejection of offsets and non-nine-digit
  fractions, and rejection rather than rounding/truncation of finer precision;
- informational timestamp behavior separate from ancestry;
- required `message`, valid empty string, unchanged text, and exact
  value participation in identity;
- required empty `provenance`, schema-owned entry shape/semantics, nested
  collection declarations, permutation invariance, duplicate canonical-entry
  rejection, and rejection of unclassified arrays;
- identical canonical bodies yielding identical IDs, and valid changes to
  each of the seven members changing canonical identity;
- exclusion of operational, storage, Line, Release, Platform,
  signature-wrapper, validation-evidence, credential, and unknown fields;
- metadata-object admission without requiring local Resource-byte
  materialisation.

These requirements are recorded in WORK-0008 and `docs/spec-coverage.md`.

## Implementation impact

- `crates/omvcs-model/`: Revision candidate/admission/model and focused
  conformance tests under WORK-0008, only after its revalidated package is
  handed to the Core Engineer.
- WORK-0008 consumes the already admitted Project State and verified
  identifier, canonicalization, and hashing contracts.
- Lines, Releases, repository mutation, publication, storage, contribution
  integration, and branch/ref mechanics are outside this decision.

## Compatibility / migration impact

No production Revision objects are known in this bootstrap workspace. Any
Draft 0.1 data using a direct `project_id`, nested `author`, a noncanonical
timestamp spelling, missing required members, or a different provenance
shape is not the approved OMVCS 0.1 Revision body and requires an explicit
future compatibility or migration treatment before interoperability.

## Notes

The exact versioned Revision schema remains the authority for provenance
semantics. This ADR does not select a generic provenance vocabulary or make
every illustrative provenance relationship mandatory.

The cross-specification review found that Core §§23 and 42 and
INV-COL-003 impose or recommend operation-specific provenance facts, while
their non-empty OMVCS 0.1 Revision encoding remains undefined. DG-0015 records
this separate issue; it does not alter the approved Revision body or hash
preimage, but blocks implementation of non-empty provenance until resolved.
