# DG-0017 — Release object and admission contract

Status: RESOLVED
Classification: BLOCKS-FEATURE
Discovered by: Spec Guardian
Discovered during: M2 preflight
Date: 2026-10-09

## Relevant specifications

- `Specs/OMVCS Core Specification.md` §§4.1–4.2, 18, 56, 62, 74, 82–83.
- `Specs/OMVCS Glossary.md`, Project, Revision, Line, and Release.
- `Specs/OMVCS Core Invariants Specification.md`, INV-HIST-001–005,
  INV-HIST-009, INV-HIST-012, INV-GC-001, and INV-REC-002.
- `Specs/OMVCS Interaction Specification.md` §§81–87, 262–266.
- `Specs/OMVCS Platform Protocol.md` §§29–30, 67, 203–209.
- Existing `DEC-CORE-002` in `docs/decision-register.md` (Reference Render
  policy; not resolved by this gap).

## Problem

Resolution: the human-approved decision is recorded in ADR-0017 and the
affected specifications. The prior analysis and options below are retained
as historical, non-normative analysis only; ADR-0017 is authoritative.

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

The following analysis is **non-normative**. It records alternatives and a
recommendation for human consideration; nothing here resolves DG-0017 or
authorizes implementation. Citations distinguish explicit normative text
from illustrative examples and omissions.

### Normative evidence and limits

- **Explicit:** Core §18 says a Release “permanently identifies one
  Revision,” its target MUST NOT change, its human-readable name MUST be
  unique “within its namespace,” and another state requires a new Release.
  The seven-field JSON is expressly introduced as “Conceptual structure,”
  not as an admitted schema or closed body.
- **Explicit classification:** Glossary “Historical Metadata” lists
  Releases; Core §3.1 lists Releases in the historical domain; Core §62
  includes Releases among protected roots; Platform §§12 and 29 treat
  Releases as repository-owned and mirrored metadata. Core Invariants
  INV-HIST-001, INV-HIST-005, INV-GC-001 and INV-REC-002 respectively
  protect published history, separate creative from operational events,
  protect objects required by retained Releases, and require Releases to
  survive repository transfer.
- **Explicit target validation, incomplete contract:** Core §56 lists
  “validate Release targets”; Core §82 lists `CreateRelease`. Neither gives
  Release admission rules, target-project consistency, inputs/results, or
  failure behavior. Core §14 establishes that a Revision resolves through
  an admitted Project State and that its Project identity comes from that
  state; §56 says the historical metadata validation path need not
  materialise Resource bytes. This does not itself define Release-to-Project
  consistency.
- **Illustrative fields only:** Core §18's example displays `schema`,
  `project_id`, `name`, `revision`, `created_at`, `creator`, and
  `description`; it establishes no field requiredness, exact shapes,
  closed-member rule, schema authority, canonical hash, or identifier
  class. In particular, the displayed `creator` object is not a normative
  ActorId contract for Releases.
- **Other layers do not close the Core schema:** Platform §29 normatively
  says a mirrored Release MUST retain name, target Revision, creator,
  `created_at`, and description, but does not specify the authoritative
  Core Release body or their Core requiredness/identity. Platform §§199
  and 209 scope retrieval to a Project namespace but do not decide Core
  Release name equality or cross-Project reuse. Interaction §§81–87 give
  creation/display expectations, not a Core schema. Glossary “Release”
  requires immutability and a stable target but defines no serialization.
- **No Git inheritance:** Glossary “Release” gives “immutable tag/release”
  only as an analogy; Core §18 and INV-SPEC-003 make OMVCS semantics
  authoritative. No Git tag, annotated-tag, ref, deletion, or name rule
  follows from that analogy.
- **Reference Render is independent:** Core §53 says Reference Render
  policy remains DEC-CORE-002 and does not define a Project State member.
  DEC-CORE-002 remains OPEN in the decision register; ADR-0012 confirms the
  closed Project State has no extra generic render member. Nothing in the
  Release example makes a render required or associates one with a Release.
- **Established design precedents, not Release decisions:** ADR-0002
  defines ActorId as assigned UUIDv7 lowercase canonical text. ADR-0012
  defines a closed, content-addressed Project State; ADR-0014 defines a
  closed, content-addressed Revision, including canonical UTC nanosecond
  `created_at`, direct `author_id`, and exact-schema admission. ADR-0016
  defines Lines as assigned stable IDs plus mutable Project-unique names
  and same-Project admitted Revision targets; it expressly rejects Git
  branch/ref semantics. ADR-0001 and ADR-0005 govern hashed arrays and
  object-map canonicalization, respectively. These decisions demonstrate
  available patterns; none selects a Release pattern.

### Options matrix (all options non-normative)

| Question | Option | Pros | Cons / unresolved consequence | Impact and evidence |
|---|---|---|---|---|
| (1) Release identity | **Assigned stable ID** (candidate class `ReleaseId`; UUIDv7 profile would be consistent with Core §4.1 and ADR-0002) | Distinguishes a Release as a first-class named entity even when body values coincide; can be separately referenced. | Requires a Release ID class/assignment/uniqueness contract and an integrity/admission rule independent of identity; choosing a UUID form is not licensed merely by the example. | Adds identifier model and generation/admission tests; record serialization still needs a specified schema and deterministic representation, even if not hashed for identity. Core §§4.1–4.2, 18; ADR-0002 (ActorId precedent only). |
| | **Content-derived ID** (typed `ReleaseId`, e.g. a Release-specific SHA-256 identifier) | Fits immutable historical objects and the Project State/Revision identity pattern; body identity detects substitution and supports portable verification. | Requires the exact closed body and hash preimage. Any included creator/time/description/name makes those values identity-bearing; re-creating the same body yields the same ID. | Requires a versioned Release schema, canonical hash/admission and exact preimage vectors; use ADR-0001/0005 collection/map rules if applicable. Core §§4.2, 5, 18; ADR-0012/0014 are analogies only. |
| | **Other / opaque repository key** | Could reflect an implementation's object store or external ID. | Risks provider- or implementation-specific identity and weak portability unless a stable interoperable type/profile is specified. | Requires an additional identity format and portability rules; no current Spec defines it. INV-PLAT-005, INV-REC-002; Core §§4, 18. |
| (2) Example fields | **Treat the seven displayed fields as normative** | Simple and superficially aligns with the example. | Contradicts its “Conceptual structure” label; still does not establish types, requiredness, closedness, canonicalization, or field mutability. | Not a safe interpretation. Core §18 is illustrative; compare explicit closed schemas in Core §§10, 13–14 and ADR-0012/0014. |
| | **Define each field by human approval** | Makes every field's status explicit and lets Platform mirroring requirements be satisfied deliberately. | Requires a decision about whether mirror fields are Core fields, derived presentation, or optional values. | Candidate status table below is proposed only. Core §18; Platform §29; Interaction §§81–87; ADR-0002/0014 for ActorId/time precedents. |
| (3) Post-admission mutability | **Immutable whole Release record** | Matches Glossary “Immutable,” Release's historical classification, Core §18 target permanence, and retained-history rules. | Does not by itself decide deletion, retention, or whether a correction is represented as a new Release. | Best supported for contents/identity while admitted; requires no mutable Line generation/CAS model. Glossary “Immutable”/“Release”; Core §§3.1, 18, 73–74; INV-HIST-001/005. |
| | **Only target immutable; metadata updateable** | Would permit correcting names/descriptions without another object. | The Glossary calls Release immutable, so this would need an explicit narrowing/override and operational update contract. | Would blur historical Release and mutable Line semantics; not supported by current text. Glossary “Immutable”/“Release”; Core §18; ADR-0016. |
| (4) Namespace and comparison | **Project namespace; exact string/code-point equality** | Mirrors Line's scoped naming rule; allows two Projects to use e.g. “1.0”; deterministic and locale/filesystem independent. | Release namespace is not explicitly identified as Project; characters/normalization remain a decision. | Proposed pattern only. Core §18 says “within its namespace” without defining it; Platform §§199/209 are project-scoped retrieval only; ADR-0016 is a Line precedent, not Release law. |
| | **Repository-global or platform-global namespace; normalized/case-insensitive comparison** | May simplify one global catalogue. | No such namespace or normalization exists in Core; platform dependence harms repository portability, and normalization rules create compatibility choices. | Requires an explicit namespace owner and comparison algorithm. Core §§18, 29, 44–45; INV-PLAT-004/005; Platform §§14, 22. |
| (5) Project/Revision association | **Release belongs to a Project and targets an admitted Revision of that same Project** | Makes project-scoped name uniqueness and release lookup coherent; prevents an accidental cross-project target; follows Line target validation and Revision's Project derivation via Project State. | “Same Project” is not expressly stated for Release; direct `project_id` may duplicate the target Revision's derived Project identity. | Recommended candidate, not existing law. Core §§14, 18, 56; INV-HIST-009/010; ADR-0014/0016. |
| | **Allow target Revision from another Project** | Could represent a curated cross-Project collection. | No defined ownership, namespace, transfer, or cross-Project reference semantics; creates ambiguity for names, authorization, mirror, roots, and transfer. | Would require explicit cross-Project rules; not implied by Core §18 example. Core §§18, 44–45, 67; INV-REC-002/004. |
| (6) `CreateRelease` | **Validate then atomically admit a new record; fail without partial state** | Fits immutable object creation, no silent success, and existing guarded Core operation style. | Exact inputs/result/error taxonomy and atomicity boundary are absent; atomic name uniqueness against concurrent creation must be addressed. | Recommendation below specifies candidate operation shape without selecting normative behavior. Core §§18, 25–28, 56, 82–83; INV-INT-003; ADR-0016. |
| | **Permit unchecked target/metadata or eventual validation** | Could allow offline or sparse metadata creation. | Conflicts with requested target validation and Core §56's validation duty unless a distinct unchecked state is specified; risks “Release” claims for unresolved objects. | Needs explicit staged-candidate semantics if desired. Core §§14, 18, 26, 56, 76; ADR-0014. |
| (7) Delete/update/permanence | **No update operation; deletion not specified; permanence means target never changes while Release exists** | Preserves explicit target immutability without inventing an undeletable object or undocumented deletion API. | Does not guarantee indefinite existence; GC root applies to retained Releases, and Core §74 leaves deletion distinctions open. | This is the narrowest statement currently supported. Glossary “Immutable”/“Release”; Core §§18, 62, 73–74; INV-GC-001; Platform §30 only forbids Platform presentation removal from mutating the object. |
| | **Release cannot ever be deleted** | Strong durable milestone guarantee. | No explicit prohibition on deleting a Release object; “permanently identifies” addresses its Revision target, not unequivocally its own persistence. Core §62 protects roots and §74 recognizes deletion distinctions. | Requires a separate explicit permanence/deletion decision; do not infer from “permanently.” Core §§18, 62, 74; INV-GC-001/REC-002. |
| (8) Classification | **Historical immutable metadata and Release reachability root** | Directly supported by Glossary, Core domain split and root list. | Lifecycle/deletion and object identity remain open. | Explicit classification/root status. Glossary “Historical Metadata”; Core §§3.1–3.2, 62; INV-HIST-005/GC-001; Platform §§12, 29. |
| (9) Canonical record | **Canonical record serialization with content-derived identity** | Creates one portable body and verified identity. | Requires exact body, requiredness, timestamps, ActorId and description representation decisions before coding. | JCS/RFC 8785 per Core §5; reject duplicate map keys per ADR-0005; arrays need declared ordering per ADR-0001. If no arrays/maps, those rules apply only if introduced. |
| | **Assigned ID with canonical serialized record** | Keeps ID independent of body while preserving deterministic export, signature/transport, and stable record comparison. | Canonicalization does not itself bind bytes to the assigned ID; integrity mechanism and duplicate-ID behavior still need definition. | Exact schema/serialization is still needed even without content-hash identity. Core §§5, 18, 55, 60; ADR-0002/0014 as identity precedents. |
| (10) Reference Render | **Resolve independently under DEC-CORE-002** | Preserves the explicit separation and prevents accidental Release-schema coupling. | Release publication UX may need a later policy. | No render field, requirement, or precondition is selected by DG-0017 analysis. Core §53; DEC-CORE-002; ADR-0012; Interaction §§84–87; Platform §§79, 167–168. |

### Example-field status table (evidence-based, not a proposed final schema)

“Illustrative only” below means the field's appearance in Core §18 does
not itself make it normative. “Required concept” identifies separate
normative behavior, not an approved member spelling or schema status.

| Field shown in Core §18 example | Current status in the Specs | What is still undecided |
|---|---|---|
| `schema` | Illustrative only; no Release schema-version authority is stated. | Whether required, exact version identifier/profile, available-schema validation, closed-member rule. |
| `project_id` | Illustrative only as a Release member; Release Project namespace is not expressly defined. | Required member or project association derived from target; consistency with target Project; namespace. |
| `name` | Human-readable Release name and namespace uniqueness are explicit in Core §18; member spelling `name` appears only in the conceptual example. | Requiredness at creation, exact namespace and equality/normalization, whether any metadata exception permits later change. |
| `revision` | A Release's specific Revision target and target immutability are explicit in Core §18; member spelling `revision` is illustrative. Core §56 calls for target validation. | Exact type/member name; admitted-metadata preconditions; same-Project requirement and failure result. |
| `created_at` | Illustrative in the Release example. Core §15 canonical timestamp rule is specifically for Revision; it does not silently extend to Release. | Required/optional; format/time authority; operational vs identity-bearing status. |
| `creator` | Illustrative in the Release example. Platform §29 requires creator information in a mirror, but does not define the Core field. ADR-0002 supplies ActorId when an ActorId is used, not Release creator requiredness. | Required/optional; exact ActorId representation; whether creator is admission input or derived metadata. |
| `description` | Illustrative in Core §18. Platform §29 requires a mirrored description field to be retained; Interaction §§81/87 present description as optional. Neither settles Core body requiredness. | Omitted vs empty vs null; requiredness; mutability; identity participation. |

The recommended field-status candidate (below) is a possible human decision,
not a reading of this table as normative text.

### Non-normative recommended model for human consideration

Recommended direction: define Release as a **closed, immutable,
Project-scoped historical metadata object**, distinct from a Line and not
assumed to be a Git tag. Prefer a typed **content-derived Release
Identifier** with SHA-256 over one explicit canonical body; this is a
coherent match for immutable history and the already approved Project
State/Revision model. This is a recommendation only; an assigned stable
UUIDv7 ReleaseId remains a viable alternative and requires human selection.

Candidate field disposition for that proposed model:

| Member | Recommended candidate status | Candidate meaning |
|---|---|---|
| `schema` | Required | Exact versioned Release schema; body closed against unknown members. |
| `project_id` | Required | Typed assigned Project ID and Release namespace. It must equal the Project derived from the target Revision's admitted Project State. |
| `name` | Required | Human-readable name, exact code-point uniqueness within `project_id`; immutable after admission. |
| `revision` | Required | Typed admitted Revision ID in the same Project; immutable after admission. |
| `created_at` | Required | Canonical UTC RFC 3339 nanosecond form matching Core §15/ADR-0014; informational but included in Release identity. This deliberately reuses an existing profile and still needs approval. |
| `creator` | Required | Direct ActorId string under ADR-0002, not a nested account/profile object; included in identity. Platform account/profile is not the actor identity. |
| `description` | Optional | Omitted when absent; otherwise a JSON string (including the empty string); `null` is invalid. If present, immutable and included in identity. This exact candidate representation still requires approval. |

For the recommended content-derived candidate, the **candidate exact
preimage**, subject to approval, is the canonical UTF-8 RFC-8785/JCS
serialization of exactly the closed body members `schema`, `project_id`,
`name`, `revision`, `created_at`, `creator`, and optional `description`
(present as a string or absent; never `null`). `creator` is a direct typed
ActorId string. SHA-256 of those canonical bytes is rendered as
`omvcs:release:sha256:<64-lowercase-hex-digits>`. All present members,
including `schema` and optional `description`, participate; no type/domain
prefix is added to the digest input, following ADR-0012/0014. There are no
arrays in this candidate body, so ADR-0001 array sorting is not invoked; any
future array must declare ordered/set-like semantics. Object maps (if
introduced) follow ADR-0005 only. This is an exact proposal, not an approved
preimage or schema.

Candidate `CreateRelease` inputs: Project ID, unique name, admitted target
Revision ID, creator ActorId, canonical creation timestamp, and optional
description. Under this proposal, the Release ID is computed, not supplied.
Preconditions: Project exists; target Revision metadata and its Project
State resolve as valid/admitted under Core §14/ADR-0014; target Project
matches supplied Project ID; requested name is unused in that Project; and
the exact Release schema is available and validates. Resource bytes,
Reference Render production/availability, and local materialisation are
not preconditions. Result: the admitted immutable Release object and its
identifier. Any validation failure or duplicate-name/concurrency conflict
leaves no partial Release. This operation contract is a candidate; Core
§§56/82 do not currently define these inputs/results/atomicity.

This model describes an admitted Release's contents and target as immutable.
It **does not** claim that a Release can never be deleted: no `DeleteRelease`
or `UpdateRelease` contract exists. Whether delete is prohibited, permitted,
or governed by later retention policy requires an explicit decision. Until
then, WORK-0011 must not implement delete/update. The existing protected-root
rule applies to retained Releases, not necessarily a promise that every
Release object can never be removed.

Reference Render remains entirely outside the proposed Release body and
`CreateRelease` preconditions unless a separate approved resolution of
DEC-CORE-002 explicitly establishes a Release relationship or requirement.
The conceptual example is not evidence for a render field.

### Exact human decisions still required

1. Choose assigned stable ReleaseId, content-derived ReleaseId, or another
   explicitly specified class/format; decide whether identity must bind the
   Release body cryptographically.
2. Approve the exact closed Release member set and schema/version authority,
   including for each of `schema`, `project_id`, `name`, `revision`,
   `created_at`, `creator`, and `description` whether it is required,
   optional, or absent; settle exact member names/types and empty/absent/null
   semantics.
3. Decide whether Release identity is Project-scoped and whether its target
   Revision MUST be in that same Project; specify how Project identity is
   obtained and validated.
4. Define the name namespace, comparison/equality and normalization rules;
   state whether different Projects may reuse a name. Decide name reuse
   after deletion only if a DeleteRelease operation is defined.
5. Decide whether all admitted body fields are immutable; state separately
   whether any update operation exists, whether Release deletion exists, and
   what “permanently” guarantees beyond an immutable target. Do not infer
   deletion or retention from target permanence.
6. Define `CreateRelease` inputs, authorization point, preconditions,
   duplicate-name conflict behavior, result, and atomic failure behavior.
   In particular decide metadata admission vs Resource-byte availability,
   Reference Render preconditions only under DEC-CORE-002, creator identity,
   timestamp, and description handling.
7. If content-derived identity is selected, approve the exact schema,
   ordered preimage member list, optional-member omission rules, digest/type
   format, and canonicalization. If assigned identity is selected, approve
   the ID class/format and whether/how canonical record serialization and
   independent integrity verification are required.
8. Confirm Release's historical classification and its role as a
   reachability root; define any deletion effect on root status only if
   deletion is explicitly selected.
9. Keep Reference Render policy/association as a separate decision under
   DEC-CORE-002; DG-0017 must not resolve or subsume it.

### Expected impact if the recommended direction is approved

| Area | Expected impact (conditional on human approval) |
|---|---|
| Core Specification | Replace the conceptual-only Core §18 structure with the exact Release object, identity/preimage, schema admission, Project/name namespace, immutability, and `CreateRelease` contract. Extend §§56 and 82–83 with target validation, operation result/failures, and canonical identity/admission. Clarify §62 root resolution and §74 deletion wording only to the extent the approved decisions require. Do not add Reference Render semantics through DG-0017. |
| Glossary | Clarify Release as a named immutable historical object/reference, identity term and namespace/target relationship; preserve the explicit distinction from mutable Line and state any approved deletion boundary without importing Git tag semantics. |
| Core Invariants | Consider a narrow Release-specific invariant if the approved identity/target/immutability rule is not fully entailed by INV-HIST-001/005 and INV-GC-001. Do not weaken existing invariants. |
| Other five Specs | Audit Interaction §§81–87/262–266 and Platform §§29–30, 67, 203–209 against the approved field names, creator/time/description requiredness, mirror source, namespace and immutability. Check Storage, DAW Adapter, and Ardour design for any Release references; do not introduce render requirements. |
| ADR/register | A human-approved ADR must record the decision and alternatives; only then resolve DG-0017 and update this already-indexed register entry. DEC-CORE-002 remains open unless separately resolved by its own approval. |
| Tests / WORK-0011 | Add closed-schema and exact identity/preimage vectors (if content-derived), field requiredness, duplicate name and namespace cases, target admission/project consistency, Resource-byte-independent validation, atomic create failures, and immutability tests. Assigned IDs require ID-profile and serialization/integrity tests instead. |
| WORK-0013 | Release-root resolver must resolve the approved Release identity to its admitted target Revision, then traverse the specified metadata graph. It must not require Resource bytes, and must not invent a Release edge or delete/pin policy. A delete decision, if any, changes root membership only as explicitly specified. |

### WORK-0011 and Line conformance boundary

At the time of this analysis, WORK-0011 was **BLOCKED — DG-0017** and
unstarted. Its allowed scope and acceptance criteria expressly require the
approved Release contract and exclude Line lifecycle. When later started, it should reuse only the verified
WORK-0010 principles where relevant: stable assigned Line identity,
Project-scoped exact name equality, same-Project admitted Revision target
validation without Resource-byte availability, atomic failure, and no
Git-derived semantics. These are evidence/analogy, not automatic Release
rules; the human-approved Release ADR controls.

Release conformance must additionally test the approved Release-specific
identity, schema, field mutability, namespace, creation/admission, and target
relationship. It must never mutate/rename a Line in order to create or
represent a Release, equate Release with Line, or infer Git tag behavior.
Under ADR-0017, Reference Render is not a Release field or
`CreateRelease` precondition. DEC-CORE-002 remains separate, and WORK-0011
must not add render-dependent Release behavior. This gap analysis does not
start the work.

### Separate Design Gap check

No additional distinct gap is recommended from this analysis. The deletion/
update/permanence uncertainty is within DG-0017's Release lifecycle scope;
DEC-CORE-002 separately owns render policy; DEC-CORE-008 concerns Line
retention, not Release identity. Record no new gap unless later evidence
establishes a separate, blocking semantic question not covered by those
boundaries.

## Resolution

Resolved by human-approved ADR-0017 and the affected specification changes.
OMVCS 0.1 Release is a content-addressed immutable Project-scoped historical
metadata object with the seven required members and hash preimage, exact
Project-scoped name rules, same-Project admitted Revision target, atomic
`CreateRelease` contract, no 0.1 update/delete operation, and a
`Release -> revision_id` reachability edge. Reference Render remains
separate under DEC-CORE-002 and is neither a Release member nor a creation
precondition. No additional Design Gap was identified.

WORK-0011 is planned and no implementation has started. DEC-CORE-002 does
not block WORK-0011 under ADR-0017.
