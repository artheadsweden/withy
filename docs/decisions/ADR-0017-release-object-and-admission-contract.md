# ADR-0017 — Release object and admission contract

Status: ACCEPTED
Date: 2026-10-09
Decision owner: Human
Related Design Gap: DG-0017

## Context

The specifications distinguished immutable Releases from mutable Lines and
required target validation, but did not normatively define Release identity,
body, name namespace, Project/Revision relationship, admission, or lifecycle.
The earlier seven-field Core §18 example was expressly conceptual and could
not establish a schema.

## Decision

1. An OMVCS 0.1 Release is immutable, Project-scoped historical repository
   metadata, content-addressed by its complete admitted body. It is distinct
   from a mutable Line and is a repository reachability root for its target
   Revision.
2. The closed Release body contains exactly seven required members:
   `schema`, `project_id`, `name`, `revision_id`, `created_at`,
   `creator_id`, and `description`. The schema is exactly
   `omvcs.release/0.1`; the exact available schema must validate the body
   before admission. Unknown members and `null` substitutions are invalid.
3. The `ReleaseId` is SHA-256 over exactly the UTF-8 RFC 8785/JCS
   serialization of the complete body. All seven members participate; no
   type/domain prefix is included in the digest input. Its typed textual
   form is `omvcs:release:sha256:<64-lowercase-hex-digits>`. The Release
   record still requires canonical serialization even though its fields are
   closed.
4. The Release is owned by `project_id`, which defines the name namespace.
   Its target Revision and that Revision's admitted Project State must
   resolve to the same Project. Cross-Project targets are invalid. Resource
   bytes need not be available or materialized for admission.
5. Release names are non-empty and unique within a Project. Comparison is
   exact string/code-point equality; Core does not case-fold, apply
   locale-sensitive comparison, normalize for a filesystem, apply Git
   reference normalization, or parse/normalize semantic versions. Different
   Projects may reuse a name. No other character restrictions apply beyond
   generic string and serialization rules.
6. `created_at` uses Core §15's exact UTC nanosecond lexical form and is
   informational rather than an ancestry/order rule; it participates in
   identity. `creator_id` is a direct ActorId, independent of Platform
   account/profile/signing-key data and not itself proof of authorization.
   `description` is a required JSON string, may be empty, is not normalized
   beyond canonical JSON string handling, and participates in identity.
7. `CreateRelease` receives ProjectId, name, admitted RevisionId, creator
   ActorId, canonical creation timestamp, and description. It validates
   Project context, the exact Release schema, target admission and
   same-Project association, name availability, ActorId, timestamp, and
   description. Construction, identity derivation, Project/name uniqueness,
   and admission are atomic; failure leaves no partial Release and claims no
   name. Exact duplicate-body creation is idempotent only when the
   ReleaseId, canonical bytes, and Project/name binding all match. A name
   bound to another ReleaseId conflicts; different bytes under an existing
   ReleaseId are an integrity violation.
8. All seven body members are immutable after admission. OMVCS 0.1 defines
   no Release update or deletion operation, but this does not decide what
   future OMVCS versions may define. No name-reuse-after-deletion rule is
   introduced because OMVCS 0.1 defines no Release deletion operation.
9. Reference Render is not a Release body member or a `CreateRelease`
   precondition. This does not decide separate Reference Render policy under
   DEC-CORE-002.
10. An admitted Release is a reachability root with edge
    `Release -> revision_id`. Reachability traversal is specified for
    WORK-0013 and is not part of WORK-0011.
11. OMVCS Release semantics do not inherit Git tag identity, naming,
    lifecycle, deletion, or reference behavior. No semantic-version syntax
    is required.

## Rationale

Content-derived identity and a closed canonical body make an immutable
Release portable and independently verifiable. Project ownership gives the
previously undefined name namespace a stable Core authority and keeps target
validation consistent. Resource-byte independence preserves the distinction
between historical metadata admission and local byte availability.
Explicitly excluding render policy, future deletion semantics, and Git tag
behavior prevents unrelated or unsupported rules from entering the Release
contract.

## Alternatives considered

- **Assigned stable identifier:** not selected; the approved contract makes
  all seven body fields identity-bearing and chooses body-verifiable
  content-derived identity.
- **Git tag/ref identity or lifecycle:** rejected; Git is only a limited
  analogy and supplies no OMVCS Release semantics.
- **Cross-Project target allowed:** rejected; the Release's owning Project
  and the target Revision's admitted Project State must match.
- **Reference Render required or included in Release:** rejected for this
  decision; render is neither a body member nor a creation precondition.
- **Release update or deletion in OMVCS 0.1:** not defined. This decision
  does not assert permanent undeletability in later versions.

## Specification impact

Normative changes:

- `Specs/OMVCS Core Specification.md` §§4.1–4.2, 18, 56, 62, 74, 82–83.
- `Specs/OMVCS Glossary.md`, Release and Release Identifier.
- `Specs/OMVCS Core Invariants Specification.md`, INV-HIST-012.
- `Specs/OMVCS Platform Protocol.md` §§29–30.
- `Specs/OMVCS Interaction Specification.md` §§81–84.

Cross-Spec audit only; no additional Release schema change is needed in
`Specs/OMVCS DAW Adapter Specification.md`,
`Specs/OMVCS Storage Adapter Specification.md`, or
`Specs/Ardour Reference Adapter Design.md`.

DEC-CORE-002 remains OPEN for separate Reference Render policy. Its
remaining scope does not make Reference Render a Release field or
`CreateRelease` precondition.

## Test impact

WORK-0011 must cover exact closed body and unknown-member/null rejection;
schema availability and admission; canonical JCS bytes and an exact
ReleaseId golden vector; identity sensitivity of all seven members;
Project-scoped name uniqueness and exact comparison; same-Project admitted
target resolution without Resource bytes; cross-Project rejection; creator,
timestamp, and description validation; idempotent duplicate creation,
conflict and integrity-failure distinctions; atomic failure/no partial name
claim; whole-body immutability; and absence of Git tag and Reference Render
semantics. WORK-0013 must traverse the Release-to-Revision root edge without
requiring Resource bytes.

## Implementation impact

- WORK-0011 implements Release representation, identity, and admission.
- WORK-0013 consumes each admitted Release as a root and traverses its
  `revision_id` edge.
- No production implementation is authorized by this ADR itself.

## Compatibility / migration impact

No Release schema or admitted Release implementation existed under the
previous unresolved contract. The former Core §18 conceptual example is
replaced by the normative OMVCS 0.1 body; it is not a migration vector.

## Notes

The earlier DG-0017 option analysis and recommendation are non-normative
historical analysis only. This ADR records the human-approved decision.
