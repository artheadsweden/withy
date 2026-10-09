# OMVCS Core Invariants Specification
## Draft 0.1

This document defines the non-negotiable architectural rules of the Open Music Version Control System.

It is intentionally narrower than the OMVCS Core Specification.

The **Glossary defines what OMVCS terms mean**.  
This document defines **what must always be true**.

Any later specification, implementation, coding-agent task, test, DAW adapter, storage adapter or platform implementation MUST conform to these invariants.

If a future design conflicts with an invariant in this document, the conflict MUST be resolved explicitly by changing this document first. Implementations MUST NOT silently violate an invariant for convenience.

The words **MUST, MUST NOT, SHOULD, SHOULD NOT and MAY** are normative as defined in the OMVCS Glossary.

---

# 1. Creative history

## INV-HIST-001 — Published history is immutable

A published Revision MUST NOT be modified after creation.

Any change to the creative state represented by a Revision MUST result in a new Revision.

This includes changes to:

- Component States;
- DAW State;
- Resource Objects;
- arrangement state;
- semantic creative metadata that forms part of Project State.

Operational changes that do not alter creative state MUST NOT create or modify a Revision.

---

## INV-HIST-002 — Revision identity depends only on immutable history

A Revision Identifier MUST be derived exclusively from immutable historical data.

Revision identity MUST NOT depend on:

- resource storage locations;
- storage-provider names;
- platform URLs;
- current credentials;
- local filesystem paths;
- temporary access tokens;
- replica availability;
- synchronization status;
- current user-interface state.

A Revision MUST retain the same identity when infrastructure changes.

For OMVCS 0.1, Revision identity is SHA-256 over the canonical bytes of its
closed seven-member historical body defined in Core §14. All and only those
members participate; infrastructure, validation evidence, signatures,
presentation, and reference-object state do not.

---

## INV-HIST-003 — Every Revision identifies one complete Project State

Every Revision MUST reference exactly one complete Project State.

A Revision MUST NOT represent merely a list of changes.

The Project State MUST contain sufficient immutable references to determine the complete logical creative state represented by that Revision.

Every Project State MUST reference exactly one canonical Adapter State metadata object by its Adapter State Identifier. A native Resource Object MAY be referenced by that Adapter State to contain native DAW state, but MUST NOT be referenced directly by the Project State as the complete Adapter State.

Delta storage MAY be used internally for efficiency, but the semantics of a Revision MUST always be equivalent to a complete state.

In OMVCS 0.1, every Project State MUST be a closed object containing exactly the required `schema`, `project_id`, `components`, `adapter_state_id`, and `project_metadata` members. The exact available versioned Project State schema MUST validate the object before historical admission. `components` MUST be a map from canonical Creative Component Identifiers to typed Component State Identifiers; every referenced admitted Component State MUST identify the same Component as its map key. Each referenced Component State metadata object and the referenced valid/admitted Adapter State metadata object MUST be resolvable, but their underlying Resource bytes need not be locally materialised. The generic Creative Component object MUST NOT carry a `project_id` back-reference.

Completeness of a Revision's immutable Project State is distinct from
completeness of metadata held by a local Repository. A local Repository MAY
lack historical metadata, but an absent required target does not satisfy or
waive this invariant's admission requirements.

---

## INV-HIST-004 — Parent relationships are immutable

The parent relationships of a published Revision MUST NOT change.

A Revision graph MUST therefore preserve the original creative ancestry permanently.

Infrastructure events MUST NOT appear as Revision ancestry.

---

## INV-HIST-005 — Creative history and operational history are distinct

Creative-history events and infrastructure events MUST be represented separately.

Examples of creative-history events:

- new bass take;
- changed automation;
- new arrangement;
- contribution integration.

Examples of operational events:

- moving a Resource Object to another provider;
- creating another Resource Replica;
- replacing credentials;
- changing the preferred storage endpoint.

Operational changes MUST NOT rewrite creative history.

## INV-HIST-006 — Hashed collection semantics are explicit and deterministic

Every array-valued collection field included in a hashed OMVCS metadata object MUST declare whether it is ordered or set-like. JSON object maps are not array-valued collections: their insertion order has no semantic significance, duplicate member names MUST be rejected before hashing/canonical serialization, and their canonical serialization MUST use RFC 8785 object-member ordering without additional entry sorting.

Ordered array collections MUST preserve their semantic order. Set-like array collections MUST be canonically sorted before serialization, and duplicate elements MUST be rejected, according to the canonical serialization rules in the Core Specification. Map entries MUST NOT be sorted using the set-like array element-byte rule.

The OMVCS 0.1 Component State historical body MUST contain exactly `schema`, `component_id`, `resources`, and `metadata`, and MAY contain `parents`; no other top-level members are permitted. `schema`, `component_id`, `resources`, and `metadata` are required; `resources` MAY be empty and `metadata` MAY be an empty object map. The Component State Identifier MUST be derived from the canonical body containing all and only these members, including `parents` only when present. The exact versioned Component State schema owns metadata keys, value shapes, meanings, and nested array classifications.

## INV-HIST-007 — Actor identity is stable and independent of accounts and keys

An ActorId in OMVCS 0.1 MUST be an assigned UUIDv7 in lowercase canonical textual form.

Display-name, email, username, Platform-account, and signing-key changes, including signing-key rotation, MUST NOT change an ActorId or rewrite historical authorship. Platform/account linkage and proof of control are separate from ActorId.

## INV-HIST-008 — Project State has a closed Project-specific identity

The OMVCS 0.1 Project State historical body MUST contain exactly the required members `schema`, `project_id`, `components`, `adapter_state_id`, and `project_metadata`. No other top-level members are permitted. The exact versioned Project State schema MUST be known and available and MUST validate the body before historical admission or identity calculation. Unknown or unavailable schema candidates MAY be preserved as unchecked data, but MUST NOT produce a valid Project State or Project State Identifier.

`project_id` MUST be the typed assigned identifier of the Project represented and MUST participate in identity. `components` MUST be a required JSON object map from canonical Creative Component Identifier text to typed Component State Identifiers; it MAY be empty. For Project State admission, each referenced Component State MUST be admitted and its `component_id` MUST equal the map key. The referenced Component State and canonical Adapter State metadata objects MUST be resolvable and valid/admitted for Project State admission; the underlying Resource bytes need not be locally materialised. `adapter_state_id` MUST be a typed Adapter State Identifier.

`project_metadata` MUST be a required JSON object map and MAY be empty. The exact versioned Project State schema owns its permitted keys, requiredness, value shapes, meanings, nested schemas, and nested collection classifications. Unclassified nested arrays, invalid shapes, or disallowed keys prevent admission. It MUST NOT serve as an unrestricted container for presentation/UI, local path, storage/Replica, credential, Platform indexing/account, validation-evidence, timestamp, or other operational data.

The Project State Identifier MUST be SHA-256 over the canonical historical body containing all and only those five members; no type/domain prefix is included in the digest input. Every member participates in identity. Its body-derived Identifier can be calculated and verified without local resolution of referenced Component or Adapter State objects; such resolution remains mandatory for historical admission. Missing targets do not alter the Identifier. Validation evidence, operational data, transport wrappers, signatures, credentials, Platform metadata, and unknown extension fields MUST NOT enter the preimage. Map insertion order MUST NOT affect identity; maps use RFC 8785 member ordering solely and duplicate member names MUST be rejected.

---

## INV-HIST-009 — Revision schema and identity are closed and project-consistent

The OMVCS 0.1 Revision historical body MUST contain exactly the required
members `schema`, `project_state_id`, `parents`, `author_id`, `created_at`,
`message`, and `provenance`. Unknown top-level members are invalid. The exact
available versioned Revision schema MUST validate the body before historical
admission or identity calculation.

`project_state_id` MUST be the typed identifier of the required Project
State. A Revision has no direct `project_id`; its Project identity is
supplied by that state when resolved. For Revision admission, the Project
State and every parent Revision MUST resolve as valid/admitted, and each
parent's Project State MUST identify the same Project. A parent from another
Project MUST prevent admission. A declared history boundary classifies an
absent parent for local completeness reporting but does not resolve it or
admit the referring Revision.

`parents` and `provenance` MUST be required set-like arrays and MAY be empty.
Duplicate elements are invalid and element ordering is non-semantic.
Provenance entries MUST validate under the exact versioned Revision schema;
generic Core MUST NOT infer their meanings. Parent ancestry and provenance
are distinct and provenance MUST NOT replace parent relationships.

`author_id` MUST be a valid ActorId. `created_at` MUST use the canonical UTC
RFC 3339 nanosecond form defined in Core §15. `message` MUST be a JSON string
and MAY be empty.

The Revision Identifier MUST be SHA-256 over the canonical historical body
containing all and only the seven required members, without a type/domain
prefix in the digest input. Each member participates in identity.
Its body-derived Identifier can be calculated and verified without local
resolution of the Project State or parent Revisions; those references
remain mandatory for Revision admission. Missing targets do not alter the
Identifier.
Operational, storage, Line, Release, Platform, credential, signature,
validation-evidence, and unknown extension data MUST NOT enter the body or
affect its identity.

## INV-HIST-010 — Line identity and movement are operational

A Line MUST use its stable assigned Line Identifier independently of its
mutable name and target Revision. Line movement and rename MUST NOT modify
Revision ancestry or historical object identity. A Line target MUST resolve
to admitted Revision metadata in the same Project; Resource-byte availability
is not required. Successful target movement MUST use the Core Line
compare-and-swap contract and MUST NOT impose a fast-forward ancestry rule.
Line `generation` MUST be an exact non-negative JSON integer in
`0 ..= 9007199254740991`, with canonical number serialization under the Core
JCS rules. Each successful extant-record mutation MUST increment it exactly
once; failures MUST leave the record unchanged. An increment from the maximum
MUST fail atomically without wrapping, resetting, or silent saturation.

DeleteLine MUST compare the expected generation atomically with Line
existence and removal. A stale-generation conflict MUST leave the Line
unchanged and MUST be distinguishable from a missing Line. Removing a Line
record MUST NOT itself delete the historical objects it referenced.

## INV-HIST-011 — Default Line is shared operational metadata

A Project MAY designate at most one existing Line in that Project as its
Default Line. The optional designation MUST be persisted as repository-
owned operational metadata in Repository Home; it MUST NOT be a Line member,
historical object, or separate history root. Repository Home is authoritative
for the designation, and a Platform mirror MUST NOT redefine it. A
client-local selected/current Line is independent.

`SetDefaultLine` MUST use an atomic expected-current-value comparison;
failure MUST leave the preference unchanged. Setting or clearing the
preference MUST NOT mutate a Line, increment Line generation, or alter
Revision history. `CreateLine` MUST NOT implicitly set it. `DeleteLine` MUST
not remove the currently designated Line; its Default Line check,
existence/generation checks, and deletion MUST be part of one atomic
decision, preventing a dangling designation under concurrent updates.

## INV-HIST-012 — Release identity and body are immutable

An admitted OMVCS 0.1 Release MUST be a closed, seven-member,
Project-scoped historical metadata object with a content-derived ReleaseId
as defined in Core §18. Its exact available versioned schema MUST validate
the body before admission. All seven required members MUST participate in
identity; unknown top-level members MUST be rejected.

The Release's admitted Project and target Revision MUST match, and the
target MUST resolve as valid/admitted metadata. Resource-byte availability
MUST NOT be required for admission. All body members, including the target
and name, are immutable after admission. Release names MUST be unique within
their Project by exact string/code-point equality. Every admitted Release is
a reachability root for its target Revision.

The ReleaseId is derived from the exact closed body and remains verifiable
without local resolution of its Project or target Revision. Resolution and
same-Project checks remain requirements for Release admission; an absent
target does not change the ReleaseId.

OMVCS 0.1 defines creation/admission only; it defines no Release update or
deletion operation. Reference Render is neither a Release body member nor a
`CreateRelease` precondition; other Reference Render policy remains separate
under DEC-CORE-002. A Release MUST NOT acquire Git tag semantics.

---

# 2. Resource identity and storage

## INV-RES-001 — Resource Objects are immutable

A Resource Object MUST be immutable after creation.

If its bytes change, a new Resource Object MUST be created.

---

## INV-RES-002 — Resource identity is content-based

A Resource Identifier MUST be derived deterministically from the Resource Object's content according to the OMVCS hashing specification.

Two byte-identical Resource Objects created under the same applicable OMVCS hashing rules MUST produce the same Resource Identifier.

---

## INV-RES-003 — Resource identity is independent of location

A Resource Object MUST NOT derive its identity from where it is stored.

Moving a Resource Object from Server A to Server B MUST NOT change:

- its Resource Identifier;
- any Component State that references it;
- any Project State that references it;
- any Revision Identifier.

---

## INV-RES-004 — Historical references use identities, not locations

Historical OMVCS objects MUST reference Resource Objects by immutable identity.

Historical objects MUST NOT depend on provider-specific physical paths or URLs to define creative state.

Physical location belongs to operational storage metadata.

In OMVCS 0.1, a generic historical Resource Reference MUST contain a typed Resource Identifier and `byte_length`, a JSON number whose mathematical value is the exact byte count of the complete Resource and an integer in the inclusive range `0 ..= 9007199254740991` (`2^53 - 1`). Negative, fractional/non-integral, greater-than-maximum, string, and other alternate representations MUST be rejected. This bound applies to one Resource only and MUST NOT be widened based on host-language numeric capacity. It does not limit repository, Project, Storage Endpoint, or aggregate Project size. A future larger representation requires an explicit schema/version decision defining compatibility and canonical-identity consequences.

The Resource Reference MAY contain a semantic role, intended media/content type, or schema/Adapter-supplied immutable interpretation properties. Every present Resource Reference field contributes to the containing historical object's canonical identity, but no such field changes the Resource Identifier derived from the complete raw Resource bytes.

A historical Resource Reference MUST NOT contain a logical/Friendly Name or filename, Chunk or Chunk Manifest information, Storage Endpoint or Location, Replica information, credentials, provider metadata, or other physical reconstruction/storage details. Such information MUST NOT affect historical object identity.

When `properties` is present, a Resource Reference MUST be validated under the exact applicable versioned schema or Adapter context before admission into a valid historical object. The applicable context MUST be determinable from the containing object's schema/Adapter contract. Core MUST enforce generic structure and canonicalization; the applicable schema or Adapter authority MUST enforce semantic admissibility and the exclusions above. An unchecked candidate with unknown, unavailable, or non-unique validation context MUST NOT be admitted to valid history or used to produce a valid historical identity. Validation status and evidence MUST NOT affect canonical historical identity.

---

## INV-RES-005 — One Resource Object may have many replicas

A Resource Object MAY exist simultaneously on multiple Storage Endpoints.

All valid Resource Replicas of the same Resource Object MUST verify to the same Resource Identifier.

The existence of additional replicas MUST NOT alter creative history.

---

## INV-RES-006 — Moving storage is not a creative change

Storage Migration MUST NOT create a Revision unless the creative content itself changes.

A migration such as:

```text
Bass resource ABC
Server A -> Server B
```

MUST leave all creative history unchanged.

---

## INV-RES-007 — A missing replica does not erase history

If every known Resource Replica for a historical Resource Object becomes unavailable, historical references to that object MUST remain intact.

OMVCS MUST distinguish:

> resource historically exists

from:

> resource is currently retrievable.

Unavailability MUST NOT be represented by deleting or rewriting history.
Missing local historical metadata is a separate completeness condition and
must not be conflated with unavailable or corrupt Resource bytes.

---

## INV-RES-008 — Property-bearing Resource References require contextual validation

A Resource Reference with `properties` MUST pass generic Core validation and semantic validation under the exact applicable versioned schema or Adapter context before it is admitted into a valid historical object.

The containing schema/Adapter contract MUST identify the applicable context unambiguously. Core owns generic structural and canonical validation; the applicable schema or Adapter authority owns semantic admissibility and MUST enforce the exclusions in INV-RES-004. Core MUST NOT infer semantic admissibility from property names or heuristics.

A candidate whose applicable validation context is unknown, unavailable, or non-unique MAY be preserved as unchecked data, but MUST NOT be admitted into valid historical state, used to produce a valid historical object identity, or committed as valid OMVCS history. Validation status and evidence MUST NOT affect canonical historical identity.

For a property-bearing Resource Reference in Component State, the exact versioned Component State schema MUST deterministically bind the permitted context to exactly one validation authority/version. The authority MAY be that Component State schema or an explicitly bound versioned Adapter/schema authority, and MUST be determinable from the containing schema/Adapter contract. An unknown, unavailable, or non-unique binding leaves the Resource Reference unchecked and MUST prevent admission of the containing Component State. No independent property-schema field is added to Resource Reference.

---

# 3. Project and component identity

## INV-PROJ-001 — Project identity survives infrastructure changes

A Project MUST retain the same Project Identifier when:

- moved between storage providers;
- moved between Open Music platforms;
- reconstructed on another machine;
- imported by another conforming implementation;
- opened through another DAW Adapter.

---

## INV-PROJ-002 — Creative Components have stable semantic identity

A Creative Component MUST be distinguishable from the files currently representing it.

Replacing `bass.wav` with a new recording MUST NOT automatically create a new logical Bass component.

Instead, the existing Creative Component normally receives a new Component State.

The Creative Component Identifier MUST be an assigned identifier independent of Resource content, Component State parentage, DAW-native identifiers, Project membership, names, descriptive metadata, timestamps, storage, Platform accounts, and locations. In OMVCS 0.1, the generic Creative Component object contains only its `component_id`; Project association is represented by Project State membership/reference, not by a `project_id` field on the Component.

`kind`, `name`, `created_at`, and comparable descriptive values MUST NOT be fields of the generic Creative Component object or affect Component identity. A presentation rename MUST NOT create a new Component Identifier. Generic Core MUST NOT require a new Component State for a descriptive change unless the applicable historical Component State schema defines the changed value as part of that state.

This invariant does not define cross-Project reuse, copy, import, move, clone, fork, or ownership semantics.

---

## INV-PROJ-003 — Filenames are not identities

OMVCS MUST NOT use human-readable filenames as authoritative Resource, Component or Project identities.

Names MAY be used for presentation and local materialisation.

Renaming a Resource for presentation or local working purposes MUST NOT by itself alter historical creative state. If a DAW requires a particular name for exact native reconstruction, that naming state belongs in Adapter State, not the generic historical Resource Reference.

---

## INV-PROJ-004 — Project State and Working State are different domains

A Project State is immutable historical state.

A Working State is mutable local Project operational state, distinct from
immutable history and not content-addressed historical metadata.

The Core Working State representation is persistent local operational
metadata and may survive process or DAW restart. It has no required
content-derived historical identifier.

Changes in Working State MUST NOT alter historical Project State until a new Revision is explicitly created.

---

## INV-PROJ-005 — Custom combinations do not become history automatically

A user MAY construct a Working State from Component States originating from different Revisions.

This MUST NOT create a Revision automatically.

A new Revision exists only when the user explicitly publishes that Working State.
The Working State's component-source mapping is operational metadata, not
historical provenance. Selecting a component from another Revision does not
change the Working State Base Revision.

---

# 4. Local working state

## INV-WORK-001 — Materialisation does not alter history

Materialising, opening or switching to a Revision MUST NOT create a new Revision.

Successful full materialisation sets the Working State Base Revision to the
target Revision and establishes the represented Components' source mapping
from its Project State. It does not mutate the source historical objects.
Materialisation or publication completion MUST NOT advance the Base Revision
implicitly; a change requires an explicit successful rematerialisation.

---

## INV-WORK-002 — Historical metadata may exist without historical media locally

An OMVCS client MUST be capable in principle of knowing Project history without storing every historical Resource Object locally.

Resource-sparse local operation is a core architectural requirement.
Local historical metadata MAY also be incomplete. Its completeness MUST be
reported separately; Resource sparsity does not imply metadata sparsity.

---

## INV-WORK-003 — Local resource presence is not historical truth

The fact that a Resource Object is currently absent from local storage MUST NOT imply that it does not belong to Project history.

Local storage is a materialised working/cache environment, not the authoritative definition of history.
Likewise, the absence of a required historical metadata target alone does
not prove corruption: a matching declared history boundary identifies
intentional omission, while an absent target without one is unresolved.

---

## INV-WORK-004 — Unpublished work remains separate from published history

Local modifications MUST remain distinguishable from the Base Revision until explicitly captured and published.

An implementation MUST NOT silently incorporate local work into an existing Revision.

Base Revision is not inferred from or advanced with an associated Line's
target. A Working State MAY have no Line association. Core comparison status
is derived as `unchanged`, `changed`, or `unknown` and is distinct from
DAW-native dirty/unsaved state.

---

## INV-WORK-005 — Destructive replacement requires scoped authorization

Working State replacement MUST default to preserving local work. If Core
comparison status is `changed` or `unknown`, replacement MUST NOT proceed
without explicit authorization for that invocation. Authorization MUST NOT
be remembered or reused. Refusal MUST leave persisted Working State and live
Adapter state unchanged. This authorization is distinct from DAW-native
dirty-state acknowledgement and does not require a temporary checkpoint.

---

## INV-WORK-006 — Adapter Working State is operational and recoverable

Mutable Adapter-owned Working State MUST be referenced by an opaque
`AdapterWorkingStateRef` in persistent Core operational metadata. It MUST
NOT be represented as immutable historical Adapter State or create
historical identity/provenance merely by being durable. Adapter preparation
failure MUST leave the previously committed Core record authoritative.
Core MUST commit the Adapter reference and related Working State metadata
atomically. Invalid or unavailable references and partial Adapter failures
MUST be reported as recovery conditions, distinct from Core comparison
status; exact recovery MUST NOT be claimed without validation.

## INV-WORK-007 — Explicit Working State safety references protect history

Reachability MUST include a present Working State Base Revision and every
present `ComponentStateId` in its component-source mapping as roots into
immutable history. An absent reference contributes no root. Working State
itself, its Line association, `AdapterWorkingStateRef`, recovery condition,
and Core change status MUST NOT add roots. The current persisted record
determines these roots; Line movement alone MUST NOT change them. Root
convergence MUST be deduplicated. These roots do not create provenance or
authorize deletion.

---

# 5. DAW independence

## INV-DAW-001 — OMVCS Core is DAW-independent

OMVCS Core MUST NOT require Ardour-specific classes, files, terminology or lifecycle rules.

Ardour is the reference DAW implementation, not the definition of OMVCS.

---

## INV-DAW-002 — DAWs integrate through the DAW Adapter Contract

All DAW-specific behaviour MUST cross the defined DAW Adapter boundary.

DAW implementations MUST NOT require OMVCS Core to contain DAW-specific conditional logic such as:

```text
if Ardour ...
if Logic ...
if Ableton ...
```

Such behaviour belongs in adapters.

---

## INV-DAW-003 — OMVCS owns history

Revision graphs, Lines, Releases, Contributions, Resource identity and Project history belong to OMVCS.

A DAW Adapter MUST NOT maintain a competing authoritative history model.

Native DAW snapshots MAY exist, but OMVCS MUST remain authoritative for OMVCS history.

---

## INV-DAW-004 — DAW Adapters own interpretation

The DAW Adapter is responsible for interpreting DAW-specific state.

OMVCS Core MUST NOT need to understand the internal meaning of:

- Ardour routes;
- Logic regions;
- Ableton devices;
- plugin-specific automation structures.

The Adapter translates those concepts into the OMVCS contract.

For Resource Reference `properties` supplied under an Adapter schema, that Adapter owns semantic validation under the exact applicable versioned context and MUST reject properties that encode information prohibited by INV-RES-004. Core owns generic structural and canonical validation and MUST NOT infer Adapter-specific meanings. An Adapter MUST NOT claim a containing historical object valid when required property validation has not succeeded.

---

## INV-DAW-005 — Unsupported capabilities must be explicit

A DAW Adapter MUST declare its capabilities.

OMVCS MUST NOT assume that every Adapter supports:

- semantic diff;
- selective Component replacement;
- automatic integration;
- dependency inspection;
- reference rendering.

Unsupported behaviour MUST be reported explicitly rather than approximated silently.

---

## INV-DAW-006 — State loss must never be silent

If a DAW Adapter cannot capture, restore or interpret part of a Project State, it MUST report that limitation.

It MUST NOT silently discard unknown or unsupported creative state while claiming successful exact reproduction.

---

# 6. Storage adapters

## INV-STOR-001 — Storage provider behaviour is abstracted

OMVCS Core MUST interact with physical storage through the Storage Adapter Contract.

Provider-specific logic MUST NOT be embedded throughout OMVCS Core.

---

## INV-STOR-002 — Credentials are never historical content

Credentials, OAuth tokens, temporary access keys and equivalent secrets MUST NOT form part of immutable creative history.

Changing credentials MUST NOT change Project, Revision or Resource identity.

---

## INV-STOR-003 — A Project may span multiple Storage Endpoints

OMVCS MUST support the possibility that different Resources in one Project are stored under different custodians and at different Storage Endpoints.

A Project MUST NOT require one universal physical storage root.

---

## INV-STOR-004 — Resource retrieval is location-resolved

Clients MUST resolve a Resource Identifier into one or more currently known Resource Replicas through operational storage information.

Historical objects MUST NOT need to know which replica will be used.

---

## INV-STOR-005 — Migration uses verify-before-remove semantics

When moving the only known valid replica of a Resource Object, OMVCS MUST NOT treat the destination as valid until content verification succeeds.

The old valid replica SHOULD NOT be removed before the new replica has been verified.

---

# 7. Platform independence

## INV-PLAT-001 — The Open Music Platform is not the repository

An Open Music Platform is a coordination, collaboration and discovery layer.

It MUST NOT become synonymous with the OMVCS Repository.

---

## INV-PLAT-002 — The platform must never be the sole source of project reconstruction

Loss of the Open Music Platform MUST NOT inherently destroy a Project.

Sufficient Project metadata and Resource access MUST exist independently to permit recovery according to the later Core Specification.

---

## INV-PLAT-003 — Project Resource Objects do not live on the Open Music Platform

The Open Music reference architecture MUST NOT store creators' Project Resource Object payloads on the central Open Music Platform.

The platform MAY store or mirror permitted metadata.

Auditioning media SHOULD be delivered from creator-controlled storage or other explicitly defined non-platform resource storage.

---

## INV-PLAT-004 — Platform migration does not rewrite creative history

Moving a Project from one Open Music-compatible platform to another MUST NOT require changing:

- Project Identifier;
- Revision Identifiers;
- Component State identities;
- Resource Identifiers;
- Release targets.

---

## INV-PLAT-005 — Platform URLs are not creative identity

No immutable creative object MUST depend on the URL, hostname or account identifier of a particular Open Music Platform for its identity.

---

# 8. Collaboration and contributions

## INV-COL-001 — Contributions reference real OMVCS history

A Contribution MUST reference actual Revisions and their ancestry.

It MUST NOT be merely an uploaded ZIP, message or detached file collection pretending to be version-controlled work.

---

## INV-COL-002 — Contributions do not transfer ownership automatically

Submitting or integrating a Contribution MUST NOT inherently imply transfer of copyright, authorship or ownership.

OMVCS provenance and legal licensing are separate concerns.

---

## INV-COL-003 — Contribution provenance survives integration

When contributed material is incorporated into Project history, OMVCS MUST preserve enough provenance to identify its origin according to the later Core Specification.

Integration MUST NOT erase the historical origin of accepted work.

---

## INV-COL-004 — Selective integration must be representable

OMVCS MUST support the architectural possibility of accepting only part of a Contribution.

Example:

> accept the cello contribution but reject the drum replacement.

Whether a particular DAW Adapter can automate this is capability-dependent, but the OMVCS history model MUST be able to represent the result.

---

## INV-COL-005 — Creative conflicts are not arbitrary file conflicts

When creative states cannot be safely combined, OMVCS MUST NOT resolve the conflict merely by arbitrary file precedence.

For example:

> "latest timestamp wins"

MUST NOT be considered a valid general creative merge rule.

Creative Integration MAY require human or DAW-specific resolution.

---

# 9. Provenance

## INV-PROV-001 — Provenance is distinct from possession

Provenance records where creative material came from.

It MUST NOT be treated as proof of legal ownership merely because OMVCS records a creator or contributor relationship.

---

## INV-PROV-002 — Provenance is historical

Provenance associated with published historical objects MUST remain part of historical metadata.

Changing storage custody MUST NOT alter provenance.

---

## INV-PROV-003 — Derivation must not erase origin

If a Component State or Revision is derived from earlier OMVCS material, the architecture MUST permit the derivation relationship to remain discoverable.

Component State parentage is optional. Derived states SHOULD record parent Component States when lineage is known, and a specific OMVCS operation or provenance rule MAY require preserving derivation. Unknown historical lineage MUST NOT be fabricated. Exact provenance rules will be defined in the Core Specification.

---

# 10. Integrity

## INV-INT-001 — Retrieved immutable content must be verifiable

A client MUST be able to verify that retrieved immutable content matches its expected identifier.

A Resource Object that fails verification MUST NOT be accepted as the requested Resource Object.

---

## INV-INT-002 — Corrupt replicas are not valid replicas

If a physical replica fails content verification, OMVCS MUST mark or treat that replica as invalid.

The existence of a corrupt replica MUST NOT invalidate other verified replicas of the same Resource Object.

---

## INV-INT-003 — Failure must not fabricate success

A failed or incomplete operation MUST NOT leave the system reporting a successfully published Revision, migration, replica or checkout if required guarantees were not satisfied.

Exact transaction semantics will be specified later.

For `ValidateRepository`, a completed report with findings MUST be
distinguished from failure to perform the requested validation. Incomplete
coverage MUST NOT be reported as a clean or complete validation.

---

## INV-INT-004 — Retryable operations should be idempotent where practical

Operations involving storage transfer, synchronization and publication SHOULD be designed so that repeating an interrupted operation does not duplicate or corrupt logical state.

---

## INV-INT-005 — Repository validation is read-only and coverage-honest

`ValidateRepository` MUST NOT repair or mutate historical or operational
state, automatically fetch or materialise Resources, or change Lines,
Releases, Working State, pinning, archival, or deletion state. Its result
MUST distinguish metadata integrity, local history completeness, Resource
availability/verification, and requested-scope/provider coverage.

A partial or unavailable provider result MUST NOT be represented as
complete history or global reachability. A validation finding is not, by
itself, deletion authority.

---

# 11. Recovery and portability

## INV-REC-001 — No single Open Music service may be necessary forever

An OMVCS Project MUST NOT depend permanently on one particular Open Music implementation or company for continued interpretation of its history.

---

## INV-REC-002 — Repository transfer must preserve identity

Exporting and importing a conforming Project MUST preserve:

- Project identity;
- Revision identity;
- Revision ancestry;
- Resource identity;
- Component identity;
- Releases;
- provenance.

Transfer MUST NOT create a logically new Project unless the user explicitly requests a fork/new identity.

Transfer of an intentionally incomplete local history MUST preserve its
historical object bytes and any declared-boundary information needed to
classify the omissions. Operational declarations do not alter historical
identity or provenance.

---

## INV-REC-003 — Platform loss must be recoverable

If the central Open Music Platform disappears while creator-controlled
repository/storage data remains available, the locally available history
MUST be recoverable by another conforming OMVCS implementation. Any declared
or unresolved local metadata omissions remain explicit; recovery MUST NOT
claim that absent historical objects were reconstructed.

---

## INV-REC-004 — Local machine loss must not inherently destroy published history

A correctly synchronized published Project MUST be recoverable after loss
of the local working machine, assuming required durable storage remains
available. If its synchronized Repository Home is intentionally incomplete,
recovery preserves and reports that completeness state and does not claim
the absent history was recovered.

---

## INV-REC-005 — Storage-provider loss must be detectable

If required Resource Objects become unavailable due to provider loss, OMVCS MUST report the resulting availability problem accurately.

It MUST NOT claim successful reconstruction when required Resource Objects cannot be retrieved.

---

## INV-REC-006 — Local history completeness is explicit

A local Repository MAY contain incomplete historical metadata. An
intentional omission of a required historical reference MUST be represented
by a machine-readable local operational declaration identifying the
referring object, normative edge kind, omitted target Identifier, and
intentional-omission classification. An absent target without a matching
declaration is unresolved.

A declaration MUST NOT resolve, validate, fabricate, rewrite, admit, or
permanently exclude its target, and MUST NOT alter historical identity,
provenance, Platform truth, or deletion authority. If the target later
resolves, it MUST be validated normally.

---

# 12. Reproducibility

## INV-REPRO-001 — Availability and reproducibility are different

OMVCS MUST distinguish:

> all required Resource Objects are available

from:

> the original creative state can be reproduced exactly in this environment.

A missing commercial plugin may make a fully available Project only partially reproducible.

---

## INV-REPRO-002 — Reproduction limitations must be explicit

A DAW Adapter MUST NOT report exact reproducibility if required dependencies or state cannot be restored.

---

## INV-REPRO-003 — Reference renders do not replace editable state

A Reference Render MAY preserve how a Revision sounded.

It MUST NOT be treated as equivalent to complete editable DAW State.

A Revision that can only be auditioned through a render MUST be distinguishable from one that can be fully reconstructed.

---

# 13. Synchronization

## INV-SYNC-001 — Metadata synchronization and resource transfer are separable

OMVCS MUST NOT require downloading all Resource Objects merely to synchronize Project history.

Historical metadata SHOULD be independently synchronizable from large media resources.

---

## INV-SYNC-002 — Platform synchronization and storage synchronization are distinct

Synchronizing metadata with an Open Music Platform and synchronizing Resources with creator-controlled storage are logically distinct operations.

An implementation MAY present them as one user action, but the architecture MUST preserve the distinction.

---

## INV-SYNC-003 — Resource upload precedes dependent publication

A Revision MUST NOT be exposed as successfully durable if required Resource Objects have not reached the durability guarantees defined by the Core Specification.

Exact publication ordering will be specified later, but published metadata MUST NOT knowingly point to resources that were never successfully persisted.

---

# 14. Garbage collection and retention

## INV-GC-001 — Reachable history must never be garbage-collected

Objects required by retained Revisions, Releases, protected Contributions,
Working State safety references, or other defined roots MUST NOT be deleted
by automatic garbage collection.

---

## INV-GC-002 — Unreferenced does not mean immediately disposable

An object becoming unreachable MUST NOT necessarily cause immediate deletion.

Retention and recovery rules MUST permit interrupted operations and reasonable recovery windows.

---

## INV-GC-003 — Garbage collection must operate on object reachability, not filenames

Garbage collection MUST use OMVCS identity and graph semantics.

It MUST NOT decide that a Resource Object is unused merely because a matching working file is absent.

An incomplete or partial validation/reachability result MUST NOT establish
that an object is unreachable or authorize its deletion. All applicable
Core §62 root classes must be covered before a complete reachability claim.

---

# 15. Security and authority boundaries

## INV-SEC-001 — Secrets are operational data

Authentication secrets MUST remain outside immutable creative history.

They MUST be replaceable without rewriting historical objects.

---

## INV-SEC-002 — Access authority and creative identity are distinct

The fact that a user currently has permission to modify, read or store a Project MUST NOT alter the immutable identity of historical Project objects.

---

## INV-SEC-003 — Storage custody and authorship are distinct

The person paying for or controlling the Storage Endpoint containing a Resource Replica MUST NOT automatically be considered the author of the Resource.

---

# 16. User interaction boundaries

## INV-UX-001 — Technical implementation may not dictate musician-facing terminology

The underlying implementation MAY use concepts analogous to Git internally.

The musician-facing UI MUST be free to express those operations in domain-appropriate language.

The architecture MUST NOT require users to understand Git terminology.

---

## INV-UX-002 — User actions must map deterministically to architectural operations

Important user actions such as:

- save/publish version;
- open historical version;
- move storage;
- contribute;
- integrate;
- release;
- recover;

MUST map to well-defined OMVCS operations.

The UI MUST NOT depend on undefined side effects to perform core repository operations.

---

## INV-UX-003 — Infrastructure operations must not masquerade as creative operations

Moving files, adding replicas or refreshing platform metadata MUST NOT appear in creative Revision history merely because they were initiated through the GUI.

---

# 17. Specification and implementation discipline

## INV-SPEC-001 — The Glossary is normative

Terms defined in the OMVCS Glossary MUST be interpreted according to that glossary throughout all OMVCS specifications and implementations.

Git analogies MUST NOT override OMVCS definitions.

---

## INV-SPEC-002 — The adapter interfaces are architectural boundaries

DAW-specific and storage-specific functionality MUST remain behind their respective adapter contracts unless the specification explicitly defines a core concept.

Coding agents MUST NOT bypass these boundaries merely because direct access appears easier.

---

## INV-SPEC-003 — Core semantics must not depend on the reference implementation

Behaviour observed in the Ardour reference implementation does not automatically become OMVCS semantics.

Only behaviour defined in the normative OMVCS specifications is normative.

---

## INV-SPEC-004 — No silent fallback semantics

When a required capability cannot be provided exactly, an implementation MUST either:

1. report the limitation; or
2. use an explicitly specified fallback.

It MUST NOT invent silent behaviour.

---

# 18. Architectural summary

All later OMVCS work should preserve these fundamental separations:

```text
Creative identity
    != physical storage location

Creative history
    != infrastructure history

Project
    != DAW project file

Project State
    != Working State

Resource Object
    != Resource Replica

Availability
    != reproducibility

Provenance
    != ownership

Open Music Platform
    != OMVCS Repository

OMVCS Core
    != DAW Adapter

OMVCS Core
    != Storage Adapter
```

And the architecture can be reduced to four particularly important rules:

> **History identifies what existed.**

> **Storage metadata identifies where its bytes currently exist.**

> **OMVCS owns creative history; adapters translate external systems into it.**

> **No platform, DAW or storage provider may become necessary to interpret the Project forever.**

---

# Status

**Document:** OMVCS Core Invariants Specification  
**Version:** 0.1 Draft  
**Normative:** Yes, once accepted  
**Depends on:** OMVCS Glossary  
**Next document:** OMVCS Core Specification
