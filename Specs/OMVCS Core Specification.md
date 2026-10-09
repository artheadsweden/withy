# OMVCS Core Specification
## Open Music Version Control System
### Draft 0.1

This document defines the core data model, identity model, repository structure and behaviour of OMVCS.

It is subordinate to:

1. **OMVCS Glossary**
2. **OMVCS Core Invariants Specification**

If this specification conflicts with a Core Invariant, the invariant takes precedence.

This document intentionally does **not** define how a particular DAW, storage provider or Open Music platform implements these concepts. Those mappings belong to their respective specifications.

---

# 1. Scope

OMVCS Core is responsible for:

- Project identity;
- Creative Component identity;
- Resource identity;
- immutable historical objects;
- Project States;
- Revisions;
- Revision ancestry;
- Lines;
- Releases;
- Working State tracking;
- Contributions;
- Creative Integration;
- provenance;
- repository metadata;
- storage-location indirection;
- replication;
- storage migration;
- synchronization semantics;
- publication semantics;
- integrity verification;
- reachability;
- garbage collection;
- repository export/import;
- recovery semantics.

OMVCS Core is **not** responsible for:

- interpreting proprietary DAW state;
- authenticating against S3, Drive, OneDrive, WebDAV, etc.;
- presenting musician-facing UI;
- hosting social/discovery functionality;
- implementing DAW-specific merging;
- storing Resource Objects on the Open Music Platform.

Those concerns interact with Core through defined boundaries.

---

# 2. Architectural model

An OMVCS Project exists across three conceptual locations.

```text
                    Open Music Platform
                    -------------------
                    collaboration mirror
                    project discovery
                    contributions
                    public history
                    releases
                    NO RESOURCE BYTES

                            ^
                            |
                       metadata sync
                            |
                            v

Local Client <------> Creator-Controlled Repository
------------           -----------------------------
working state           historical metadata with
metadata                explicit local completeness
current resources       resource history
optional cache          storage map and replica information
```

A Project may use many physical Storage Endpoints:

```text
Project
 |
 +-- Repository Home
 |
 +-- Storage Endpoint A
 |      Bass resources
 |      Drum resources
 |
 +-- Storage Endpoint B
 |      Vocal resources
 |
 +-- Contributor Endpoint C
        Cello resources
```

Neither a Revision nor a Project State knows where those physical resources are located.

They know only **what Resource Objects they require**.

---

# 3. Data domains

OMVCS Core divides information into three domains.

## 3.1 Historical domain

Historical data describes creative history.

Examples:

```text
Project
Creative Components
Component States
Project States
Revisions
Revision ancestry
Provenance
Releases
Contribution lineage
```

Historical data is immutable once published.

---

## 3.2 Operational domain

Operational data describes how the Project currently functions.

Examples:

```text
Storage Endpoints
Storage Locations
Resource Replicas
Availability
preferred replicas
synchronization state
Line targets
optional Project-scoped Default Line preference
Platform synchronization state
```

Operational data may change without changing creative history.

---

## 3.3 Working domain

Working data exists locally and is mutable.

Examples:

```text
materialised audio
modified DAW project state
unpublished recordings
local edits
temporary component combinations
working caches
```

Working data does not become historical merely by existing.
The Core Working State representation is persistent local repository
operational metadata and MAY survive process or DAW restart. It is distinct
from Project State and Revision history, is not Platform-owned, and has no
content-derived historical identifier. Ephemeral runtime details that do not
affect recoverable Working State semantics need not be persisted.

---

# 4. Identifier classes

OMVCS MUST distinguish between **identity assigned to persistent logical entities** and **identity derived from immutable content**.

## 4.1 Assigned identifiers

The following entities use globally unique assigned identifiers:

- Project;
- Creative Component;
- Line;
- Storage Endpoint;
- Contribution;
- Actor.

OMVCS 0.1 uses UUID version 7 for newly generated assigned identifiers.

Example:

```text
019cc17d-1b22-7a41-9fe9-c345c468f82c
```

UUIDs MUST be serialized in lowercase canonical textual form.

---

## 4.2 Content-derived identifiers

Immutable objects use SHA-256 content identity in OMVCS 0.1.

The identifier format is:

```text
omvcs:<object-type>:sha256:<64-lowercase-hex-digits>
```

Examples:

```text
omvcs:resource:sha256:96d2...
omvcs:component-state:sha256:b371...
omvcs:project-state:sha256:84aa...
omvcs:revision:sha256:0ca7...
omvcs:release:sha256:1a2b...
```

Object type is part of the identifier namespace.

An implementation MUST NOT treat:

```text
omvcs:resource:sha256:X
```

and:

```text
omvcs:revision:sha256:X
```

as the same object merely because the digest happens to be identical.

For a metadata object with a known, available exact schema, its
content-derived Identifier is calculated and verified from its canonical
historical body. Local resolution or admission of the objects named by its
typed references is not required to calculate or verify that body hash.
Reference availability and same-Project/admission checks determine whether
the object may be admitted and used as valid history; they do not change its
Identifier. An unknown/unavailable schema or an invalid body still prevents
establishing a valid content-derived Identifier.

---

# 5. Canonical serialization

All hashed OMVCS metadata objects MUST have one deterministic byte representation.

OMVCS 0.1 uses:

- UTF-8;
- JSON;
- JSON Canonicalization Scheme semantics compatible with RFC 8785;
- no semantically irrelevant whitespace;
- canonical JSON number representation;
- lexicographically deterministic object-member ordering;
- Unicode preserved according to canonical JSON rules.

The hash MUST be calculated over the canonical serialized object body.

Storage wrappers, HTTP headers, database keys, signatures and timestamps added outside the historical object MUST NOT affect the identifier unless explicitly defined as fields of the object itself.

This means two implementations given the same semantic object MUST produce the same canonical bytes and therefore the same identifier.

## 5.1 Hashed collection fields

JSON Canonicalization Scheme does not reorder array elements. Before an OMVCS metadata object is serialized for hashing, the schema for every array-valued collection field MUST explicitly declare that the field is either an **ordered sequence** or a **set-like collection**.

- An ordered sequence preserves its specified semantic order. Canonicalization MUST NOT reorder its elements.
- A set-like collection has no semantic element order. Before serializing the containing object, each element MUST be serialized using the same canonical JSON rules, and the elements MUST be sorted in ascending lexicographic order of those canonical serialized bytes. Duplicate elements, determined by identical canonical serialized bytes, are invalid and MUST be rejected.

This normalization is recursive: any collection-valued field within an element MUST itself declare its ordering semantics and be normalized before that element is serialized for sorting. Implementations MUST NOT infer whether a field is ordered or set-like from observed values.

JSON object maps are unordered mappings rather than array-valued collections. Map insertion order has no semantic significance. Before hashing or canonical serialization, maps MUST have unique member names; duplicate names are invalid and MUST be rejected. Their canonical serialization MUST use RFC 8785 object-member ordering solely. OMVCS MUST NOT apply the set-like array element-byte sorting rule, or any other additional entry-sorting transformation, to map entries. Their schemas MUST identify them as maps and define the meaning of their keys; their values are normalized according to their own schemas.

The following collection fields in OMVCS 0.1 schemas have these semantics:

| Object and field | Semantics |
|---|---|
| Resource Reference `properties` | JSON object map keyed by property name. |
| Chunk Manifest `chunks` | Ordered sequence in reconstruction order. |
| Component State `parents` | Optional set-like collection of direct Component State parents. |
| Component State `resources` | REQUIRED set-like collection of Resource References required by that state. |
| Component State `metadata` | REQUIRED JSON object map keyed by metadata property name; the exact versioned Component State schema owns permitted keys, value shapes, meanings, and nested collection classifications. |
| Adapter State `resources` | Set-like collection of native Resource References. |
| Adapter State `component_bindings` | JSON object map keyed by Creative Component Identifier; each value is an adapter-specific binding record. |
| Adapter State `metadata` | JSON object map keyed by metadata property name. |
| Project State `components` | REQUIRED JSON object map from canonical Creative Component Identifier text to typed Component State Identifier; the referenced Component State MUST identify the same Component Identifier as its map key. |
| Project State `project_metadata` | REQUIRED JSON object map keyed by metadata property name; the exact versioned Project State schema owns permitted keys, requiredness, value shapes, meanings, nested schemas, and nested collection classifications. |
| Revision `parents` | REQUIRED set-like collection of direct parent Revisions; MAY be empty. |
| Revision `provenance` | REQUIRED set-like collection of canonical provenance objects; the exact versioned Revision schema owns entry members, requiredness, meanings, nested schemas, and nested collection classifications. It MAY be empty. |

Every additional array-valued collection included in a hashed Core object, Adapter State, or namespaced extension MUST declare its ordering semantics in the schema that defines it. A schema that does not make this declaration is not valid for hashing.

In OMVCS 0.1, `Revision.parents` and `Revision.provenance` are required set-like arrays and MAY be empty. Their element order has no semantic significance; duplicates are invalid and canonical element-byte sorting under ADR-0001 applies. Each provenance entry MUST be an object validated under the exact versioned Revision schema. Nested arrays in provenance entries MUST also declare their ordering semantics in that schema. Generic Core MUST NOT infer provenance entry meaning from field names or values.

`Project State.components` and `Project State.project_metadata` are required JSON object maps and MAY be empty. The exact versioned Project State schema MUST validate each `project_metadata` key, value shape, meaning, nested schema, and nested collection classification. Project metadata MUST NOT be treated as an unrestricted container for presentation/UI, local-path, storage/Replica, credential, Platform indexing/account, validation-evidence, timestamp, or other operational information. Such information is admissible only when an approved Project State schema explicitly defines it as historical Project state and it does not conflict with another Core invariant or ADR.

Resource Reference `properties` values MUST be validated under the exact versioned schema or Adapter context that governs their use in the containing historical object. The applicable context MUST be determinable from the containing object's schema and, where applicable, its Adapter State schema and Adapter identifier; a Resource Reference MUST NOT introduce an independent property-schema identifier for this purpose. Core validates the generic JSON shape, duplicate member names, canonical form, schema-declared value shapes, and nested collection classifications. The applicable schema or Adapter authority validates the semantic admissibility of property values. Core MUST NOT infer semantic admissibility from property names, deny lists, heuristics, or DAW-specific knowledge.

Every versioned containing schema that permits Resource Reference `properties` MUST bind those properties to one exact schema/Adapter validation authority and version. If that authority cannot be uniquely determined from the containing schema/Adapter contract, the candidate remains unchecked and MUST NOT be admitted as valid historical state.

---

# 6. Resource identity

A Resource Object represents immutable raw content.

Its Resource Identifier is calculated directly over the complete raw byte sequence:

```text
resource_id =
    SHA256(resource_bytes)
```

The bytes themselves are not JSON canonicalized.

For example:

```text
Bass_take_03.wav
```

is identified from the actual WAV file bytes.

The filename is irrelevant to Resource identity.

If the file is renamed without changing its contents:

```text
Bass_take_03.wav
```

to:

```text
Final amazing bass.wav
```

the Resource Identifier remains unchanged.

If one byte changes, the Resource Identifier changes.

---

# 7. Resource Reference

Historical structures refer to Resource Objects through a **Resource Reference** embedded directly in the containing historical object. OMVCS 0.1 does not define Resource Manifest as a separate content-addressed historical object.

The generic Resource Reference contains:

- `resource_id` — REQUIRED typed Resource Identifier;
- `byte_length` — REQUIRED JSON number whose mathematical value is the exact byte count of the complete Resource and MUST be an integer in the inclusive range `0 ..= 9007199254740991` (`2^53 - 1`);
- `role` — OPTIONAL semantic role of the Resource in the containing historical state;
- `media_type` — OPTIONAL intended media/content type;
- `properties` — OPTIONAL canonical JSON object map containing immutable interpretation metadata explicitly supplied by the relevant schema or Adapter.

Every field present in a Resource Reference is part of the containing historical object's canonical body and contributes to that object's identity. These fields do not contribute to or change the Resource Identifier, which is calculated only from the complete raw Resource bytes under section 6.

Decoding or preserving a Resource Reference candidate does not by itself validate it for historical use. A Resource Reference without `properties` MAY be validated using the generic Core Resource Reference rules alone. When `properties` is present, including as an empty object, the candidate MUST NOT be admitted as a valid Resource Reference in a historical object until its values have passed validation under the exact applicable, versioned schema or Adapter context governing that use. The applicable context is identified by the containing historical object's schema version and, where the Resource Reference's property meanings are Adapter-owned, the applicable Adapter identity and schema/version declared by that context. It MUST NOT be inferred from the property names or from an unrelated, unknown, or merely latest schema/Adapter version.

The applicable schema or Adapter authority MUST define the valid property meanings and structures for its context and MUST NOT admit properties that encode presentation or logical names, filenames, Chunk or Chunk Manifest information, storage locations, endpoints, Replicas, credentials, provider metadata, or other physical reconstruction/storage information prohibited above. It MUST supply the value-shape and nested collection rules needed by section 5.1. Core is responsible for generic structural and canonical validation, including duplicate object member rejection, but MUST NOT reproduce the schema or Adapter's semantic judgement.

Implementations MAY preserve or transport a property-bearing Resource Reference candidate when its applicable validator or schema context is unknown, unavailable, or non-unique, provided that it remains explicitly unchecked. Such unchecked data MUST NOT be admitted as a valid historical Resource Reference, used to create a valid containing historical object, used to produce a valid historical object identity, or committed into valid OMVCS history until validation succeeds under the applicable context. Preservation does not imply semantic validity.

Validation status, validator implementation details, callbacks, timestamps, signatures, and other validation evidence are operational concerns. They MUST NOT become Resource Reference fields or affect canonical historical identity. The property values themselves remain part of the containing object's canonical body after successful validation.

The `byte_length` upper bound is the maximum interoperable JSON safe integer used by the RFC 8785/JCS numeric model. Conforming implementations MUST reject negative, fractional/non-integral, and greater-than-`9007199254740991` values, as well as string or other alternate representations. This bound applies to one Resource only, not repository, Project, Storage Endpoint, or aggregate Project size. Implementations MUST NOT widen the accepted domain based on host-language integer capacity. OMVCS 0.1 defines no decimal-string, tagged-big-integer, or alternate representation for larger values. A future version MAY define a larger representation only through an explicit schema/version decision that specifies compatibility and canonical-identity consequences.

Conformance cases for `byte_length`:

| JSON value | Result |
|---|---|
| `0` | Accept |
| `1` | Accept |
| `9007199254740991` | Accept |
| `-1` | Reject |
| `1.5` | Reject |
| `9007199254740992` | Reject |
| `"1"` | Reject |

Conformance tests for historical admission MUST establish that:

- a Resource Reference without `properties` can pass generic Core validation;
- a property-bearing candidate is admitted only when the exact applicable versioned context approves it, and is rejected for historical admission when that context rejects it;
- an unknown, unavailable, or non-unique context cannot produce a valid historical Resource Reference or containing historical object; preserved unchecked data cannot be committed as valid history;
- Core does not infer property validity from key spelling;
- context-declared nested value shapes and ordered/set-like array rules are enforced, and unclassified arrays are rejected;
- duplicate object member names, including nested names and escaped-equivalent names, are rejected before decoding or canonicalization;
- changing validation status/evidence does not affect canonical historical bytes.

`logical_name`, Friendly Name, and filename are not fields of the generic historical Resource Reference. Chunk structure, Chunk IDs, Chunk Manifest information, Storage Endpoint, Storage Location, Replica information, credentials, provider metadata, and other physical reconstruction or storage details MUST NOT appear in a historical Resource Reference or affect historical object identity. Resource Reference `properties` MUST NOT be used to reintroduce these excluded values.

Renaming a Resource for presentation or local working purposes MUST NOT by itself alter historical creative state. If a DAW requires a particular filename or equivalent naming state for exact native reconstruction, that information belongs in Adapter State.

Conceptually:

```json
{
  "resource_id": "omvcs:resource:sha256:...",
  "byte_length": 183829331,
  "media_type": "audio/wav",
  "role": "primary-audio",
  "properties": {
    "sample_rate": 48000,
    "channels": 1
  }
}
```

Only `resource_id` identifies the Resource Object. The other fields describe the Resource's length or intended interpretation and do not act as physical locators.

---

# 8. Physical representation and chunking

Large music Resources MUST NOT require transfer as one monolithic network object.

OMVCS therefore permits Resource Objects to be physically represented as Chunks.

The **Resource Identifier remains the SHA-256 hash of the reconstructed complete Resource**.

Chunking MUST NOT affect Resource identity.

This gives us an important property:

```text
Resource ABC

stored as:

8 × 8 MiB chunks

or

32 × 2 MiB chunks

or

one complete object
```

is still the same Resource Object.

## 8.1 Chunk identity

Each Chunk is independently content-addressed:

```text
omvcs:chunk:sha256:<digest>
```

---

## 8.2 Chunk Manifest

A Chunk Manifest defines one physical reconstruction representation of a Resource.

Its `chunks` collection is an ordered sequence in reconstruction order. Offsets and lengths describe each element's position and MUST agree with that sequence.

Example:

```json
{
  "resource_id": "omvcs:resource:sha256:ABC...",
  "total_length": 21000000,
  "chunks": [
    {
      "chunk_id": "omvcs:chunk:sha256:111...",
      "offset": 0,
      "length": 8388608
    },
    {
      "chunk_id": "omvcs:chunk:sha256:222...",
      "offset": 8388608,
      "length": 8388608
    },
    {
      "chunk_id": "omvcs:chunk:sha256:333...",
      "offset": 16777216,
      "length": 4222784
    }
  ]
}
```

A Chunk Manifest is storage/reconstruction information.

Historical objects MUST NOT depend upon a particular chunking representation.

Chunk Manifest information is operational physical-reconstruction data. It MUST NOT appear in a historical Resource Reference or affect historical object identity. Different Replicas or Endpoints MAY use different physical chunk layouts for the same Resource without changing its Resource Identifier or creative history.

A Storage Adapter MAY store the same Resource differently while preserving Resource identity.

OMVCS 0.1 SHOULD initially support simple deterministic fixed-size chunking for the reference implementation.

Recommended default:

```text
8 MiB chunks
```

This is deliberately simple.

More sophisticated content-defined chunking MAY later be introduced without changing Resource identity.

---

# 9. Creative Component

A Creative Component is a stable logical identity anchor representing persistent creative meaning rather than physical files. Its normative generic OMVCS 0.1 object contains only the required assigned `component_id`; no other field is part of that object.

Normative generic Creative Component object:

```json
{
  "component_id": "019cc..."
}
```

The globally stable `component_id` MUST remain unchanged throughout the life of the logical Component and is independent of Resource content, Component State parentage, DAW-native identifiers, Project membership, names, descriptive metadata, timestamps, storage, Platform accounts, and locations. It MUST NOT be derived from any of those values.

`project_id`, `kind`, `name`, `created_at`, and comparable descriptive values are not fields of the generic OMVCS 0.1 Creative Component object and MUST NOT affect Component identity. Generic Core MUST NOT infer historical significance for such values. Presentation/friendly names and UI labels are mutable descriptive, local, or Platform metadata unless an approved historical schema explicitly gives them historical meaning. DAW-native names, classifications, identifiers, and reconstruction-relevant values belong in Adapter State when required by that Adapter. A value that an approved Component State schema explicitly defines as creative state belongs in that Component State and affects it according to that schema.

A rename or descriptive metadata change MUST NOT create a new Creative Component Identifier. Generic Core MUST NOT require a new Component State merely because presentation metadata changes; a Component State changes only when its applicable historical schema says a changed value belongs to that state. Implementations MAY keep operational/audit creation timestamps outside the generic Component object, but those timestamps MUST NOT affect Component identity or historical object identity unless another approved schema explicitly includes them elsewhere.

The generic Creative Component object has no `project_id` back-reference. Association is expressed by membership/reference in Project State as specified in section 13. This does not define or imply any cross-Project reuse, copy, import, move, clone, fork, or ownership semantics.

Changing:

```text
Bass take 1
```

to:

```text
Bass take 2
```

normally produces another Component State for the same Component Identifier.

Deleting a component from one Project State does not erase the Component's historical existence.

---

# 10. Component State

A Component State describes one immutable state of a Creative Component.

The closed OMVCS 0.1 Component State historical body is a JSON object containing exactly these top-level members:

| Member | Requirement | Meaning |
|---|---|---|
| `schema` | REQUIRED | Existing versioned schema identifier under section 76. |
| `component_id` | REQUIRED | Typed Creative Component Identifier whose state this object represents. |
| `parents` | OPTIONAL | Direct parent Component State Identifiers, when lineage is asserted. |
| `resources` | REQUIRED | Resource References required by this state; MAY be empty. |
| `metadata` | REQUIRED | Schema-owned creative-state metadata map; MAY be empty. |

No additional top-level members are permitted in OMVCS 0.1. In particular, the fields of a Component State do not become fields of the generic Creative Component object defined in section 9.

The exact versioned schema identified by `schema` owns the permitted metadata keys, required/optional metadata keys, value shapes and types, meanings, nested object schemas, and ordered/set-like classifications of nested arrays. Core MUST enforce generic structure and canonicalization and MUST NOT infer or invent metadata semantics. Unknown metadata keys, invalid shapes, and unclassified nested arrays MUST prevent admission unless the exact versioned schema explicitly permits them.

The `parents` field is optional. When present, it is a set-like collection under section 5.1; element order has no semantic meaning and duplicate elements are invalid. An explicitly empty `parents` array identifies an initial state with zero parents. An omitted `parents` field means parentage is unknown or not asserted; it MUST NOT be interpreted as proof that the state is initial. Implementations MUST NOT fabricate parentage for unknown historical lineage.

Derived states SHOULD record one or more parent Component States when their lineage is known. Parentage is mandatory only when a specific OMVCS operation or provenance rule explicitly requires preserving that derivation.

The required `resources` field is a set-like collection under section 5.1; element order has no semantic meaning and duplicate elements are invalid. It MAY be empty; omission is invalid. Every Resource Reference in `resources` MUST satisfy section 7 before inclusion. A Resource Reference with `properties` MUST be validated under its exact applicable versioned authority before the Component State can be admitted as valid history. Every versioned Component State schema that permits such references MUST bind each property-bearing context to exactly one validation authority/version. That authority MAY be the Component State schema itself or an explicitly bound versioned Adapter/schema authority. The binding MUST be determinable from the Component State schema/Adapter contract and existing context; no independent property-schema field is added to Resource Reference. Unknown, unavailable, or non-unique context leaves the reference unchecked and prevents admission of the containing Component State as valid history. A Resource Reference without `properties` continues to require only generic Core validation.

The required `metadata` field is a JSON object map under section 5.1 and MAY be empty; omission is invalid. Its exact versioned Component State schema owns its values and their semantics. Map insertion order has no semantic significance; duplicate member names are invalid and RFC 8785 ordering applies without additional entry sorting.

The Component State candidate MUST validate against its exact, known and
available `schema` version before a valid content-derived Identifier can be
established or the object admitted as valid historical state. An unknown or
unavailable schema MAY be preserved as uninterpreted candidate data, but
MUST NOT be treated as valid history or used to produce a valid Component
State Identifier. Resolution/admission of referenced historical objects is
required for historical admission, not for calculating or verifying the
canonical body hash. A missing required target leaves the Component State
unadmitted but does not change its body-derived Identifier.

The canonical historical body and Component State hash preimage contain exactly `schema`, `component_id`, `resources`, `metadata`, and `parents` only when `parents` is present. Every present member participates in identity, including `schema`. Under the existing WORK-0002/WORK-0003 rules, the Component State Identifier is SHA-256 over the canonical serialized body bytes; no object-type/domain prefix is included in the digest input. Validation evidence, validator identity, callbacks, timestamps, presentation metadata, storage information, transport wrappers, signatures, credentials, Platform metadata, and unknown extension fields MUST NOT enter the preimage.

OMVCS 0.1 defines no arbitrary top-level extension mechanism for Component State. Schema-specific creative semantics MUST be expressed through the schema-owned `metadata` map and the already-defined `resources` Resource References. Adding a top-level member requires an explicit future versioned specification and compatibility decision.

Normative OMVCS 0.1 body shape:

```json
{
  "schema": "omvcs.component-state/0.1",

  "component_id": "019cc...",

  "parents": [
    "omvcs:component-state:sha256:..."
  ],

  "resources": [
    {
      "resource_id": "omvcs:resource:sha256:...",
      "role": "primary-audio",
      "byte_length": 183829331,
      "media_type": "audio/wav"
    }
  ],

  "metadata": {}
}
```

The `parents` member is shown as a non-empty set-like collection in this example. If it is omitted from a Component State body, lineage is unknown or unasserted; omission does not mean that this is known to be an initial state.

---

# 11. Component lineage

Component State parentage expresses semantic creative derivation.

Parentage is optional in the Component State schema. An initial state has zero parents; when this is explicitly known, it is represented by an empty `parents` array. A missing `parents` field indicates that lineage is unknown or was not asserted, not that the state is known to be initial. Derived states SHOULD record one or more parents when known. A specific operation or provenance rule MAY make preserving derivation mandatory; implementations MUST NOT invent unknown historical lineage.

Example:

```text
Bass State 1
     |
Bass State 2
     |
Bass State 3
```

It MAY also diverge:

```text
                Bass State 2A
               /
Bass State 1
               \
                Bass State 2B
```

Component lineage is separate from Revision ancestry.

This distinction is necessary because a Component may be reused across different Project States and Lines.

---

# 12. Adapter State

DAWs contain state that OMVCS Core must preserve without understanding.

OMVCS represents this through **Adapter State**.

Every Adapter State MUST be a canonical OMVCS metadata object with its own content-derived Adapter State Identifier. It MAY reference one or more opaque Resource Objects containing native DAW state. A native Resource Object MUST NOT serve directly as the complete Adapter State.

The `adapter_id` and `adapter_state_schema` identify the Adapter context governing Adapter-owned Resource Reference properties in this state. Before an Adapter State is admitted as valid historical state, every Resource Reference with `properties` MUST pass semantic validation under that exact Adapter schema/version and generic Core validation under section 7. Unknown, unavailable, or non-unique Adapter schema contexts may be preserved as unchecked data, but MUST NOT be treated as valid historical Adapter State or used to create its valid historical identity.

In the conceptual structure below, `resources` is set-like. `component_bindings` is a JSON object map keyed by Creative Component Identifier and follows RFC 8785 map canonicalization as specified in section 5.1. Each map value is an adapter-specific binding record, which MAY contain fields such as `binding_kind`, `native_ids`, and other metadata permitted by that Adapter State schema. The Creative Component Identifier MUST NOT be duplicated inside the value merely to repeat the map key unless a future schema has a separate justified need. Any array-valued collection in a binding record, adapter-specific metadata, or extensions that participates in Adapter State identity MUST declare its ordering semantics in the Adapter State schema and follow section 5.1.

Conceptually:

```json
{
  "schema": "omvcs.adapter-state/0.1",

  "adapter_id": "org.openmusic.ardour",

  "adapter_state_schema": "org.openmusic.ardour.state/0.1",

  "adapter_format_version": "1",

  "resources": [
    {
      "resource_id": "omvcs:resource:sha256:...",
      "byte_length": 4096,
      "role": "native-session-state"
    }
  ],

  "component_bindings": {
    "019cc-bass": {
      "binding_kind": "ardour-route",
      "native_ids": ["ardour-route-id-817"]
    },
    "019cc-vocal": {
      "binding_kind": "ardour-route",
      "native_ids": ["ardour-route-id-922"]
    }
  },

  "metadata": {}
}
```

OMVCS Core treats adapter-specific metadata as opaque with respect to semantic interpretation. It still enforces generic structural/canonical validation and Resource Reference historical admission rules in section 7.

The DAW Adapter Specification will define exactly how adapters create and restore these objects.

---

# 13. Project State

A Project State defines one complete logical creative state.

The OMVCS 0.1 Project State is a closed JSON object containing exactly these REQUIRED top-level members:

- `schema`
- `project_id`
- `components`
- `adapter_state_id`
- `project_metadata`

No additional top-level members are permitted. The exact versioned Project State schema identified by `schema` MUST be known and available before the candidate can be admitted as valid historical state.

`project_id` is the typed assigned Project Identifier of the Project whose state the object represents. Project State identity is Project-specific: `project_id` participates in canonical identity and is not derived from Component membership, Adapter State, Resource content, storage, Platform identity, or location.

`components` is a REQUIRED JSON object map from canonical typed Creative Component Identifier text to typed Component State Identifier. The map MAY be empty; omission is invalid. Object member ordering and duplicate-member rejection follow section 5.1 and ADR-0005; no set-like array sorting applies to the map. For Project State admission, every referenced Component State MUST be valid and admitted, and its required `component_id` MUST equal the map key. A mismatch prevents Project State historical admission. The Component State metadata object MUST be resolvable and valid for admission, but its underlying Resource bytes need not be locally materialised. This consistency rule does not define clone, copy, import, fork, move, ownership, or cross-Project identity-preservation semantics.

Normative OMVCS 0.1 body example:

```json
{
  "schema": "omvcs.project-state/0.1",
  "project_id": "019cc17d-1b22-7a41-9fe9-c345c468f82c",
  "components": {
    "019cc17d-1b22-7a41-9fe9-c345c468f82d": "omvcs:component-state:sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    "019cc17d-1b22-7a41-9fe9-c345c468f82e": "omvcs:component-state:sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
    "019cc17d-1b22-7a41-9fe9-c345c468f82f": "omvcs:component-state:sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
  },
  "adapter_state_id":
    "omvcs:adapter-state:sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "project_metadata": {}
}
```

This shape example illustrates the complete OMVCS 0.1 top-level body; identifier strings are illustrative valid-format values, not conformance vectors. The `components` and `project_metadata` maps MAY be empty; all five top-level members remain required.

`adapter_state_id` is a REQUIRED typed Adapter State Identifier. It MUST identify exactly one resolvable, valid/admitted canonical Adapter State metadata object under section 12. It MUST NOT identify a native Resource Object as the complete Adapter State. The referenced Adapter State metadata object MUST be resolvable and valid, but Resource bytes beneath it need not be locally materialised.

`project_metadata` is a REQUIRED JSON object map and MAY be empty. Its exact versioned Project State schema owns permitted keys, required and optional keys, value shapes and types, semantic meanings, nested object schemas, and ordered-versus-set-like classification of every nested array. Core enforces generic structure and canonical rules but MUST NOT infer Project metadata semantics. Unknown keys or invalid shapes prevent historical admission unless that exact schema explicitly permits them. Every nested array MUST have an explicit ordering classification under section 5.1; an unclassified nested array prevents historical admission. Project metadata is for schema-approved historical Project-level state only; presentation/UI metadata, local paths, storage/Replica information, credentials, Platform indexing/account information, validation evidence, timestamps, and other operational values MUST NOT enter the Project State merely as generic metadata.

The canonical Project State historical body and hash preimage contain exactly `schema`, `project_id`, `components`, `adapter_state_id`, and `project_metadata`. Every member participates in identity. The Project State Identifier is `SHA-256(canonical Project State historical-body bytes)` under sections 5 and 55; no type/domain prefix is included in the digest input. Changing any member's canonical value changes the canonical body and, barring cryptographic collision, the identifier. Map insertion order alone does not change identity.

Validation evidence, validator implementations, timestamps, presentation/UI metadata, local DAW state, storage information, Resource Replica state, transport wrappers, signatures, credentials, Platform metadata, and unknown extension fields MUST NOT enter the hash preimage. OMVCS 0.1 defines no arbitrary top-level Project State extension mechanism. Adding a top-level member requires an explicit future schema/version and compatibility decision.

A Project State does not describe only what changed.

It describes the complete state.

The Project State membership map is the association between the Project and participating Creative Components. The generic Creative Component object has no `project_id` back-reference.

A Project State candidate MUST NOT be admitted as valid history unless the
exact Project State schema validates the body; `project_id`, every
component-map key/value, and `adapter_state_id` are correctly typed; each
referenced Component State is resolvable, valid/admitted, and has a matching
`component_id`; the referenced Adapter State is resolvable and valid/admitted
under section 12; `project_metadata` validates under the exact Project State
schema; and no unknown top-level member is present. The canonical body
schema/structure and hash may be validated independently of whether those
referenced objects are locally present. Missing required targets prevent
admission but do not change the body-derived Project State Identifier.
Missing Resource bytes alone do not invalidate otherwise valid Resource
References or metadata-complete history.

---

# 14. Revision

A Revision is the principal immutable historical node representing exactly one valid/admitted Project State.

## OMVCS 0.1 historical body

The closed OMVCS 0.1 Revision historical body is a JSON object containing exactly these REQUIRED members:

| Member | Requirement | Meaning |
|---|---|---|
| `schema` | REQUIRED | Exact versioned Revision schema identifier under section 76. |
| `project_state_id` | REQUIRED | Typed identifier of the required Project State; local resolution and admission are required for Revision admission, not for calculating the Revision body Identifier. |
| `parents` | REQUIRED | Set-like array of typed direct parent Revision Identifiers; MAY be empty. |
| `author_id` | REQUIRED | Direct typed ActorId of the author. |
| `created_at` | REQUIRED | Canonical UTC timestamp under section 15. |
| `message` | REQUIRED | JSON string describing the Revision; MAY be empty. |
| `provenance` | REQUIRED | Set-like array of schema-validated canonical provenance objects; MAY be empty. |

No other top-level members are permitted in OMVCS 0.1. In particular, the body has no direct `project_id`: the Revision's Project identity is obtained from its required admitted Project State. Every parent Revision MUST resolve through its own admitted Project State to the same assigned Project Identifier. A parent from another Project prevents admission. This does not define cross-Project move, copy, fork, or identity-preservation semantics.

The exact available versioned Revision schema identified by `schema` MUST
validate the candidate's own body before a valid content-derived Identifier
can be established or the candidate admitted. An unknown or unavailable
schema candidate MAY be preserved outside valid history where supported,
but MUST NOT produce a valid Revision Identifier.

`parents` is REQUIRED. An empty array represents an initial Revision; a normal derived Revision generally has one parent; an integration Revision MAY have multiple parents. Parent order has no semantic significance in generic Core 0.1, there is no generic first-parent concept, and duplicate parent identifiers are invalid. Each parent MUST resolve to a valid/admitted Revision before the referring Revision is admitted. Parent ancestry and provenance are separate: provenance MUST NOT replace the parent graph, and parent order MUST NOT encode Line, branch, ref, or Release semantics.

Where an import intentionally omits a required parent, the unchanged
Revision body and matching declared history boundary MAY be preserved
outside admitted history where supported. The boundary classifies the
omission but does not resolve the parent or waive the admission requirement.
The Revision body-derived Identifier remains valid if its own canonical
body/schema checks pass, but the Revision MUST NOT be admitted as valid
history until its required parents resolve and pass the ordinary checks.

`author_id` is directly encoded as an ActorId; it is not wrapped in an `author` object. The ActorId MUST be valid under section 59. Historical admission MUST NOT require current profile, Platform account, profile service, signing key, or credential availability.

`message` is a required JSON string and MAY be empty. Core MUST NOT trim, rewrite whitespace, case-fold, or Unicode-normalize the value; the exact string value participates in identity under the canonical JSON string rules.

`provenance` is REQUIRED and MAY be empty. Each entry MUST be a canonical JSON object. The exact versioned Revision schema owns the permitted entry members, requiredness, value shapes, semantic meanings, relationship kinds, nested object schemas, and nested array classifications. Unknown or disallowed fields, invalid shapes, unclassified nested arrays, or entries rejected by that schema prevent admission. Provenance entries are set-like, sorted by canonical serialized element bytes, and duplicate canonical entries are invalid. No independent provenance-schema identifier is added in OMVCS 0.1.

The Revision Identifier is SHA-256 over the canonical serialized Revision historical-body bytes under the existing sections 5 and 55 rules; no type/domain prefix is included in the digest input. The canonical body contains exactly `schema`, `project_state_id`, `parents`, `author_id`, `created_at`, `message`, and `provenance`. All seven members participate in identity. Validation evidence, validator implementation details, signatures or signature wrappers, Platform account/profile data, storage/Replica information, local DAW/runtime state, credentials, transport wrappers, Line/ref/branch information, Release information, and unknown extension fields MUST NOT enter the body or affect the Revision Identifier.

Normative OMVCS 0.1 body shape:

```json
{
  "schema": "omvcs.revision/0.1",
  "project_state_id": "omvcs:project-state:sha256:STATE...",
  "parents": [
    "omvcs:revision:sha256:PREVIOUS..."
  ],
  "author_id": "019cc17d-1b22-7a41-9fe9-c345c468f82c",
  "created_at": "2026-10-08T11:02:17.000000000Z",
  "message": "New bass take and vocal automation",
  "provenance": []
}
```

The example illustrates the closed 0.1 shape; exact provenance entry semantics are determined only by the identified, available Revision schema.

---

# 15. Time

A Revision's `created_at` MUST:

- use UTC;
- use RFC 3339 representation in the canonical OMVCS 0.1 Revision form `YYYY-MM-DDTHH:MM:SS.nnnnnnnnnZ`;
- use uppercase `T` and `Z`, and exactly nine fractional-second digits;
- represent an instant at nanosecond resolution; implementations MUST NOT round or truncate finer-than-nanosecond input into an admitted historical timestamp.

Numeric timezone offsets, omitted fractional seconds, and fewer or more than nine fractional digits are not canonical Revision timestamps and MUST be rejected for historical admission. Calendar/time values MUST otherwise be valid under RFC 3339.

Example:

```text
2026-10-08T11:02:17.481000000Z
```

`created_at` is informational historical metadata and participates in Revision identity.

Revision ancestry, not timestamps, defines causal creative history.

Implementations MUST NOT infer ancestry merely from timestamps.

---

# 16. Line

A Line is mutable operational repository metadata, not a content-addressed
historical object. The OMVCS 0.1 Line record is a closed object containing
exactly these REQUIRED members:

| Member | Meaning |
|---|---|
| `line_id` | Assigned Line Identifier using the UUIDv7 lowercase canonical textual form in §4.1; immutable for the Line's lifetime. |
| `project_id` | Typed assigned Project Identifier; immutable for the Line's lifetime. |
| `name` | Current human-readable Line name; mutable. |
| `target_revision` | Typed Revision Identifier of exactly one target Revision. |
| `generation` | Unsigned monotonic version token for this mutable Line record. |

Unknown additional members are invalid in OMVCS 0.1. The identifier is
independent of the Line name and target. A Line MUST NOT move between
Projects. Cross-Project Line transfer and copy semantics are not defined.
The Line record is operational metadata, not a canonical hashed historical
object; it has no `schema` member.

Line names MUST be unique within their Project. Equality uses exact
string/code-point equality; Core MUST NOT case-fold, use locale-sensitive
comparison, normalize for a filesystem, or apply Git-style ref normalization.
Different Projects MAY contain Lines with the same name. No additional
character restrictions apply beyond generic string and serialization rules.

The target MUST resolve as valid/admitted Revision metadata and MUST belong
to the Project named by `project_id`. Resource-byte availability is not
required to validate the target. Multiple Lines MAY point to the same
Revision. A Line always has exactly one target; there is no null or unborn
Line in OMVCS 0.1. `CreateLine` therefore requires an already-admitted
target Revision.

`generation` begins at zero. Every successful mutation of the Line record
increments it by exactly one, including movement and rename. Failed
mutations MUST leave all fields unchanged and MUST NOT increment it.
`generation` MUST be a JSON number representing an exact non-negative
integer in the inclusive range `0 ..= 9007199254740991` (`2^53 - 1`).
Negative, fractional, greater-than-maximum, string, and alternate
representations are invalid. The value's canonical JSON serialization, where
applied, MUST follow RFC 8785/JCS number serialization under §5. Host-language
integer width MUST NOT change the accepted or emitted domain; implementations
MAY use a wider internal type. If the current value is `9007199254740991`,
any operation requiring another increment MUST fail atomically and leave the
Line unchanged. Generation MUST NOT wrap, reset, silently saturate, or reuse
an earlier value. These are Line-specific rules under ADR-0020; ADR-0008's
Resource `byte_length` range does not define a general OMVCS integer profile.

`CreateLine` requires a Project ID, requested name, and admitted target
Revision ID. It succeeds only if the target is valid/admitted in that
Project and the name is unused there. Success atomically creates a fresh
assigned `line_id` and the supplied immutable `project_id`, `name`, and
`target_revision`, with `generation` zero. Failure leaves no partial Line.

A Default Line is an optional Project-scoped repository operational
preference represented conceptually as `ProjectId -> optional LineId`. When
present, it MUST designate an existing Line in that Project. The preference
is persisted in Repository Home, which is authoritative for it; a Platform
MAY mirror it but MUST NOT redefine it. It is not a Line member, a distinct
Line type, or a historical object. `CreateLine` MUST NOT implicitly set it.
A client-local selected/current Line is separate and changing that local
selection MUST NOT change the shared preference. `SetDefaultLine` and
deletion constraints are specified in §17.

Moving a Line does not modify either Revision, its ancestry, or other
historical objects.

---

# 17. Line and Default Line updates

Every `expected_generation` input to `MoveLine`, `RenameLine`, and
`DeleteLine` uses the same JSON-number representation and exact integer
domain as the Line `generation` field in §16. An invalid representation is
a validation failure, not a stale-generation conflict.

`MoveLine` is an atomic compare-and-swap operation requiring `line_id`,
expected current `target_revision`, expected current `generation`, and a
new target Revision ID. The operation succeeds only if the Line exists,
both expected values match, and the new target resolves as valid/admitted
metadata in the same Project as the Line. Success changes only
`target_revision` and increments `generation` exactly once. `line_id`,
`project_id`, and `name` remain unchanged.

A stale target or generation is a conflict. A missing Line, invalid target,
or cross-Project target is a validation failure. Every failure is atomic:
no member or generation changes. No fast-forward-only rule applies; a
successful compare-and-swap MAY move the Line to any admitted Revision in
the same Project. Resource bytes need not be available for target validation.
If the new target equals the current target and both expected values match,
the operation still succeeds and increments `generation` exactly once.

`RenameLine` is a Core operation. It requires `line_id`, expected
`generation`, and a requested new name. It succeeds only if the Line exists,
the generation matches, and no other Line in the same Project has that exact
name. Success changes only `name` and increments `generation` exactly once.
A stale generation is a conflict; an unavailable Line or conflicting name
is a failure. A request for the existing name is valid and increments
`generation` once. Failure is atomic and leaves the Line unchanged.

`SetDefaultLine` takes a Project ID, an optional expected current Default
Line ID, and an optional requested new Default Line ID. An absent expected
value means that no Default Line is currently designated; an absent
requested value clears the designation. The operation succeeds only if the
Project exists and the current optional value equals the supplied expected
value. A supplied new Line ID MUST identify an existing Line in the
specified Project. A stale expected value is a concurrency conflict; a
missing Project, missing Line, or cross-Project target is a validation
failure. The compare, target validation, and preference update MUST be
atomic. Any failure leaves the preference unchanged.

Changing or clearing the preference does not mutate a Line, increment a
Line generation, alter Revision history, or create another history root.
There is no separate Default-Line generation counter. A Platform or client
may mirror the shared value, but local selection and Platform/account
preferences MUST NOT overwrite Repository Home authority.

`DeleteLine` removes the mutable Line record. It MUST NOT delete any
Revision, Project State, Component State, or Resource merely because the
Line is removed. It requires `line_id` and `expected_generation`. The Line
MUST exist and its current generation MUST equal the expected generation;
the existence check and generation comparison MUST be part of the same
atomic deletion decision. A stale generation is a concurrency conflict and
MUST leave the Line intact and unchanged. A missing Line MUST be
distinguishable from a stale-generation conflict. Success removes exactly
the Line record and does not need to create a new generation. A Line that
is the current Default Line MUST NOT be deleted; the caller must first
explicitly change or clear the preference. The check that the Line is not
the current Default Line, existence, expected generation, and removal MUST
be part of the same atomic deletion decision. A Default Line is never
implicitly cleared or reassigned. Retry requires the caller to obtain and
reason from current state; Core MUST NOT silently retry using a newly
observed generation. No tombstone, reflog, or Line mutation history is
introduced. Automatic retention or pinning after deletion remains entirely
separate under DEC-CORE-008.

---

# 18. Release

A Release is immutable, Project-scoped historical repository metadata,
content-addressed by its complete closed body, and distinct from a mutable
Line. Every admitted Release is a reachability root for its target Revision.
A Release identifies one Revision for as long as that Release exists.
OMVCS 0.1 defines `CreateRelease`; it defines no Release update or
deletion operation.

## OMVCS 0.1 closed Release body

The Release body MUST contain exactly these seven REQUIRED top-level
members. Unknown additional members are invalid; `null` is not a valid
substitute for any member.

| Member | Requirement | Meaning |
|---|---|---|
| `schema` | REQUIRED | Exact versioned Release schema identifier; OMVCS 0.1 uses `omvcs.release/0.1`. |
| `project_id` | REQUIRED | Typed assigned Project Identifier owning the Release and defining its name namespace. |
| `name` | REQUIRED | Non-empty human-readable Release name, unique within `project_id`. |
| `revision_id` | REQUIRED | Typed Revision Identifier; it MUST resolve to valid/admitted metadata in the same Project before Release admission. |
| `created_at` | REQUIRED | Canonical UTC nanosecond timestamp under §15. |
| `creator_id` | REQUIRED | Direct typed ActorId under §59. |
| `description` | REQUIRED | JSON string; MAY be empty. |

The exact versioned Release schema identified by `schema` MUST be known,
available, and validate the body's own members before its content-derived
Identifier can be established or the Release admitted. An unknown or
unavailable schema candidate MAY be preserved externally where supported,
but MUST NOT produce a valid Release Identifier.
No arbitrary top-level extension members are defined in OMVCS 0.1; future
extensions require an approved schema/version change.

The Project identified by `project_id` MUST exist. It owns the Release and
defines the namespace for Release-name uniqueness. At admission,
`revision_id` MUST resolve to a valid/admitted Revision; the Project State
referenced by that Revision MUST resolve as valid/admitted, and its
`project_id` MUST equal the Release's `project_id`. A cross-Project target
is invalid. Resource bytes need not be available or locally materialised
for Release admission.

Local resolution of the target Revision is required for Release admission,
not for calculating or verifying the Release body's Identifier. If the
target is absent, a validated closed Release body retains its
body-derived ReleaseId but is not an admitted Release or reachability root.

Release-name equality is exact string/code-point equality. Core MUST NOT
case-fold, use locale-sensitive comparison, normalize for a filesystem,
apply Git ref normalization, or parse/normalize semantic versions. Names
MUST NOT be empty; no other character restrictions apply beyond generic
string and serialization rules. Different Projects MAY use the same Release
name. Because Release is immutable, its name cannot subsequently change.

`created_at` MUST use the exact canonical timestamp profile in §15:
`YYYY-MM-DDTHH:MM:SS.nnnnnnnnnZ`, UTC, uppercase `T` and `Z`, and exactly
nine fractional-second digits. Offsets, missing/fewer/more fractional
digits, and inputs with finer precision that would require rounding or
truncation are invalid. The timestamp is informational and MUST NOT
determine ancestry or graph ordering; it participates in Release identity.
The operation input supplies the timestamp; Core validates it.

`creator_id` is a direct ActorId, not a nested creator/profile/account
object. Platform account ID, email, username, display name, signing key,
and credentials are not Release creator identity. `creator_id` records
historical attribution and does not itself prove authorization or current
Platform-account control. It participates in Release identity.

`description` MUST be present as a JSON string and MAY be empty. Omission
and `null` are invalid; the empty string means no descriptive text was
supplied. Core MUST NOT trim, rewrite whitespace, case-normalize, or
Unicode-normalize it beyond canonical JSON string handling. It participates
in Release identity.

Illustrative body shape only (the identifiers below are placeholders, not a
valid Release or canonical/hash test vector):

```json
{
  "schema": "omvcs.release/0.1",
  "project_id": "019cc17d-1b22-7a41-9fe9-c345c468f82e",
  "name": "1.0",
  "revision_id": "omvcs:revision:sha256:0000000000000000000000000000000000000000000000000000000000000000",
  "created_at": "2026-10-09T01:23:45.123456789Z",
  "creator_id": "019cc17d-1b22-7a41-9fe9-c345c468f82c",
  "description": "First public mix"
}
```

## Release identity

`ReleaseId` is a typed content-derived identifier. It is SHA-256 over exactly
the UTF-8 RFC 8785/JCS canonical serialization of the closed Release body
above. All seven members are always present and participate in identity.
The digest input contains no type/domain prefix. The typed textual
representation is:

```text
omvcs:release:sha256:<64-lowercase-hex-digits>
```

No transport envelope, signature, validation evidence, storage information,
Platform metadata, authorization data, or object-type prefix participates in
the digest. The Release body has no arrays; RFC 8785 object-member
canonicalization is the only collection ordering rule needed for OMVCS 0.1.
The body itself MUST still be serialized canonically for hashing and
verification even though its fields are closed.

## CreateRelease

`CreateRelease` takes a ProjectId, non-empty Release name, admitted
RevisionId, creator ActorId, canonical creation timestamp, and description
string. Core constructs the body using the applicable exact Release schema
version.

Preconditions:

- the Project context exists;
- the exact Release schema is known, available, and validates the candidate;
- the target Revision resolves as valid/admitted metadata, and its admitted
  Project State resolves to the same Project as `project_id`;
- the Release name is non-empty and is not bound to a different admitted
  Release in that Project;
- creator_id is a valid ActorId;
- created_at satisfies the canonical timestamp profile;
- description is a JSON string.

Resource bytes, local materialisation, and Reference Render are not
preconditions. Reference Render policy remains separate under
DEC-CORE-002. Authorization is external to the Release body and follows the
repository/Platform authority boundary.

Release body construction, identity derivation, Project/name uniqueness
checking, and admission MUST behave atomically at the Core operation
boundary. Failure MUST leave no partially admitted Release and MUST NOT
claim a name. Success returns the admitted Release and its ReleaseId.

Repeated creation of the exact same already-admitted Release body is
idempotent success only when the computed ReleaseId already exists, its
canonical body bytes are identical, and the Project/name binding refers to
that same Release. It returns the existing Release and identifier without
creating another object. A Project/name already bound to a different
ReleaseId is a name conflict. If an existing ReleaseId resolves to
non-identical canonical body bytes, this is an integrity violation, not a
duplicate success. When one `CreateRelease` request simultaneously encounters
this existing same-ReleaseId/different-canonical-body integrity violation and
a Project/name binding conflict, Core MUST return the integrity violation.
The name conflict MUST NOT mask it. For this overlap, failure MUST remain
atomic and MUST leave all stored objects and Project/name bindings unchanged.
This rule applies only to these two simultaneous conditions; it does not
define a general failure-precedence framework or other precondition ordering.
Failure MUST NOT mutate either existing object or claim the requested name.
Core MUST NOT silently replace either object.

## Immutability and lifecycle

Once admitted, none of the seven Release body members may change. OMVCS 0.1
defines no `RenameRelease`, `MoveRelease`, `UpdateRelease`, mutation
generation, Release CAS update, `DeleteRelease`, or Release reflog. Another
name, description, creator/time assertion, target Revision, or other
historical body requires another Release, subject to Project-scoped name
uniqueness. This 0.1 operation boundary does not decide what later OMVCS
versions may define.

An admitted Release is a repository reachability root with the edge:

```text
Release -> revision_id
```

Reachability then follows the immutable Revision metadata graph. Resource-byte
availability is distinct from metadata reachability. Release reachability
calculation is defined for WORK-0013; WORK-0011 MUST NOT implement traversal.

---

# 19. Working State

The Working State is mutable local Project operational state. The Core
Working State representation is persistent local repository metadata; it
MAY survive process or DAW restart. It is not an immutable historical object,
is not content-addressed, and is not itself a reachability root merely by
existing.

The conceptual Core record identifies the Project and records:

```text
Project
optional Base Revision
optional associated Line
component source mapping
optional AdapterWorkingStateRef for current mutable Adapter-owned content
```

For each Creative Component represented in the Working State, the source map
identifies the `ComponentStateId` from which it was last materialised or
adopted, when such a historical source exists. A locally created component
may have no source Component State. This map is operational state, not
historical provenance.

For reachability, a present Base Revision and each present source
`ComponentStateId` are Working State safety-reference roots as specified in
§62. `line_id`, `AdapterWorkingStateRef`, recovery condition, and Core change
status do not add Working State-derived roots. The conceptual record contains
no separate immutable historical `AdapterStateId` reference.

The conceptual representation above does not define a closed serialized
schema. Core persists the currently authoritative
`AdapterWorkingStateRef`; the Adapter owns the mutable state represented by
that opaque reference. The reference is operational metadata, not historical
identity or provenance. Immutable historical Adapter State MUST NOT be
extended to represent mutable Working State. Core and Adapter ownership,
prepare/commit, and restart recovery requirements are specified in
§§82–83 and the DAW Adapter Contract.
Working State recovery condition is separately reported operational status,
not a member of immutable historical metadata or the Core comparison result.

OMVCS 0.1 does not require a globally stable content-derived Working State
identifier. An internal handle, if needed, MUST NOT be presented as
historical identity or provenance. A Working State is not itself Platform-
owned and is distinct from DAW-native dirty or unsaved state.

---

# 20. Immutable cache and editable working files

Implementations MUST distinguish between:

**immutable local object cache**

and:

**editable Working State files**.

An immutable cached Resource Object MUST NOT be modified in place.

A working file derived from that Resource may be modified.

Example:

```text
.omvcs/cache/resources/ABC...
        immutable

My Song/Audio/Bass.wav
        editable
```

When `Bass.wav` changes, OMVCS hashes the resulting bytes and creates another Resource Object.

---

# 21. Materialisation

Materialising a Revision consists conceptually of:

```text
1. Resolve Revision
2. Resolve Project State
3. Resolve required Component States
4. Determine required Resource Objects
5. Check local immutable cache
6. Resolve missing Resource Replicas
7. Retrieve missing Resources
8. Verify hashes
9. Create/update editable Working State
10. Ask DAW Adapter to restore Adapter State
11. Validate resulting state
12. Record Base Revision
```

Failure at steps 6–11 MUST NOT silently report successful materialisation.

Required historical metadata that is absent MUST NOT be treated as resolved
or admitted solely because a matching declared history boundary exists.
Materialisation MUST fail rather than claim success when the target state or
other required metadata cannot be resolved; validation may separately report
whether the omission is declared or unresolved.

Before replacing an existing Working State, Core MUST compare its current
derived status with the recorded source/base state. If status is `changed`
or `unknown`, replacement MUST be refused unless this invocation supplies
explicit `discard_current_working_state` authorization. The default is
`preserve`. Authorization is operation-scoped and MUST NOT be remembered or
reused. This rule does not replace any separate DAW-native dirty-state
acknowledgement.

On successful full materialisation, Core records the target Revision as the
Base Revision, establishes the represented Components' source mapping from
that Revision's Project State, and begins Core comparison status as
`unchanged`. Materialisation MUST NOT mutate the source Revision, Project
State, Component States, or Resources, and MUST NOT create a Revision.
Resource bytes need not be available where the applicable selective
materialisation contract permits their absence.

An optional Line association is set or retained only as explicitly specified
by the invoking operation. Movement of the associated Line MUST NOT advance
the Base Revision or silently rematerialise the Working State.

Core prepares the complete recoverable Adapter-owned state before committing
the new Core Working State record. The record's Adapter reference and
associated Base Revision, Line, and component-source mapping changes are
committed atomically. A failure before this commit leaves the old record
authoritative. If Adapter work partially changes live state and cannot be
completed or rolled back, Core MUST return
`adapter_partial_failure_recovery_required`, retain the last committed
record as authoritative, and report recovery as `recovery_required`; it MUST
NOT automatically retry or report normal success.

After restart, Core MUST restore or validate the committed Adapter reference.
If it is missing, invalid, unavailable, or unrestorable, Core MUST report
the recovery condition and MUST NOT fabricate state or silently fall back
to historical Adapter State while claiming exact recovery.

---

# 22. Selective Materialisation

A client MAY materialise only Resources required for the current Working State.

Historical Resource Objects that are not required need not exist locally.

Example:

A Project has 400 GB of historical media.

Current Revision requires 11 GB.

A valid local Working State MAY therefore contain:

```text
complete or explicitly incomplete local metadata history
+
11 GB current resources
+
optional cache
```

rather than 400 GB.

Selective materialisation does not change the Base Revision. The component
source map describes each represented Component's immutable source when one
exists; local components without an admitted historical source have no
source `ComponentStateId`.

Resource sparsity and metadata-history completeness are independent. Local
metadata omissions MUST be classified as declared or unresolved under
sections 55–56; missing historical metadata is not implied by the absence of
Resource bytes.

---

# 23. Custom Working State

A user MAY combine Component States from different Revisions.

Example:

```text
Bass    Revision 12
Drums   Revision 18
Vocal   Revision 21
Guitar  Revision 14
```

OMVCS materialises this as a Custom Working State.

Until explicitly published, it has no Revision Identifier.

When published:

```text
Custom Working State
      |
      v
new Project State
      |
      v
new Revision
```

Provenance SHOULD record the source Component States. This provenance records known origins and MUST NOT be populated with fabricated lineage.

The current Working State's component-source map is mutable operational
metadata and is not Revision provenance. Selecting a source Component State
from another Revision does not change the Working State's Base Revision.
Custom Working State has no Revision Identifier until the resulting state is
explicitly published.

---

# 24. Change detection

Core Working State comparison status is derived and has exactly these
results:

```text
unchanged
changed
unknown
```

`unchanged` means Core can establish semantic equality with the applicable
recorded source/base comparison state. `changed` means Core can establish a
relevant difference. `unknown` means available evidence is insufficient.
This status MUST NOT be treated as immutable persisted truth. Implementations
MAY cache comparison evidence or results operationally, but stale cached
status MUST NOT override the current comparison.

DAW-native dirty/unsaved state is distinct from Core Working State comparison
status. Neither MUST be silently substituted for the other. Adapter evidence
may contribute to Core comparison only under the applicable Core/Adapter
comparison contract.

Core defines three levels of possible change information.

## Level 1 — Resource change

Core can determine:

```text
Resource X != Resource Y
```

through Resource identifiers.

---

## Level 2 — Component change

Core can determine:

```text
Bass Component State B4 != B5
```

---

## Level 3 — Semantic DAW change

Only an appropriate DAW Adapter may determine:

```text
Room reverb changed
Automation changed
Routing changed
Tempo map changed
```

Core MUST NOT fabricate semantic detail that the adapter did not report.

---

# 25. Publishing a Revision

Publishing a Revision is a coordinated operation.

For an online durable publication, the logical order MUST be:

```text
Capture working state
        |
Create Resource Objects
        |
Upload missing Resource Objects
        |
Verify durable replicas
        |
Create Component States
        |
Create Adapter State
        |
Create Project State
        |
Create Revision
        |
Persist validated metadata required by the new Revision
        |
Atomically update Line
        |
Update local repository state
        |
Mirror permitted metadata to platform
```

Platform mirroring is not required for Revision durability.

Publication completion alone MUST NOT advance or replace the current Working
State's Base Revision. A Base Revision change requires an explicit successful
rematerialisation operation subject to this section's replacement
authorization and failure contract.

---

# 26. Durability boundary

A Revision MUST NOT be reported as durably published until:

1. its historical metadata is durably stored in the Project's creator-controlled repository;
2. every Resource Object required to reconstruct the Revision satisfies the configured minimum replica policy;
3. every required Resource Replica has been verified;
4. the target Line update succeeds.

Failure to update the Open Music Platform after this point does not invalidate the Revision.

Instead:

```text
Revision: durable
Platform mirror: pending synchronization
```

---

# 27. Publication failure

Consider:

```text
Resource upload succeeds
Revision metadata upload fails
```

The uploaded Resource may temporarily become an Orphan Object.

It MUST NOT appear as a published Revision.

A retry MAY reuse the already uploaded object because Resource identity is content-addressed.

---

# 28. Idempotent publication

Publication steps SHOULD be idempotent.

Examples:

Uploading Resource ABC twice MUST result logically in:

```text
Resource ABC exists
```

not:

```text
Resource ABC
Resource ABC duplicate
```

Creating an immutable object whose identifier already exists MUST verify that the existing bytes/object match the identifier and then treat the operation as satisfied.

---

# 29. Repository Home

Every actively published Project MUST designate at least one creator-controlled location as its **Repository Home**.

The Repository Home stores the available durable OMVCS metadata and its
operational completeness information. It MAY contain complete or
intentionally incomplete local historical metadata. An intentional omission
of a required historical reference MUST have a matching declared history
boundary under section 55; an absent target without one is unresolved.

It is not necessarily the same place that stores every Resource Object.

Example:

```text
Repository Home:
    user's S3 bucket

Resource replicas:
    S3
    OneDrive
    collaborator's server
```

The Repository Home MUST retain the historical metadata it stores without
rewriting immutable objects, together with current operational metadata
sufficient to resolve Project storage and any applicable local completeness
and declared-boundary records.
This operational metadata includes the optional Project-scoped Default Line
preference defined in §§16–17.

It MUST NOT contain credentials in historical metadata.

---

# 30. Repository Home migration

Moving Repository Home does not change creative history.

Migration consists conceptually of:

```text
1. Create destination repository
2. Copy available metadata and applicable completeness/boundary records
3. Verify metadata integrity
4. Copy or retain storage map
5. Verify resource resolvability
6. Make destination authoritative
7. Update clients/platform mirror
8. Optionally retire old Repository Home
```

Revision identifiers and Project Identifier remain unchanged.
Migration MUST NOT silently turn an incomplete source into a complete claim.
Any newly intentional omission MUST be declared; undeclared missing targets
remain unresolved.

---

# 31. Storage Endpoint descriptor

A Storage Endpoint is represented by an operational descriptor.

Example:

```json
{
  "endpoint_id": "019ef...",

  "adapter_type": "s3",

  "owner": "019cc17d-1b22-7a41-9fe9-c345c468f82c",

  "display_name": "Joakim project storage",

  "capabilities": [
    "read",
    "write",
    "delete",
    "range-read"
  ],

  "configuration_reference":
    "local-secret-store:s3-main"
}
```

Secrets MUST NOT appear in Repository historical metadata.

Configuration references MAY differ between clients.

---

# 32. Resource Replica record

A replica record conceptually contains:

```json
{
  "replica_id": "019f...",

  "resource_id":
    "omvcs:resource:sha256:ABC...",

  "endpoint_id":
    "019ef...",

  "object_key":
    "objects/96/d2/...",

  "representation":
    "chunked",

  "availability":
    "available",

  "verification":
    {
      "state": "verified",
      "verified_at":
        "2026-10-08T14:00:00Z"
    }
}
```

The replica record is operational metadata.

Changing `object_key` does not change Resource identity.

---

# 33. Storage Map

The Storage Map is conceptually:

```text
Resource ABC
    -> Endpoint A / Replica 1
    -> Endpoint B / Replica 2

Resource DEF
    -> Endpoint C / Replica 3
```

The Storage Map MUST support multiple replicas.

Replica ordering MAY express preferred retrieval order but preference MUST NOT alter Resource identity.

---

# 34. Storage Map updates

Storage Map mutation MUST be versioned operationally.

Each mutation SHOULD carry:

```text
operation identifier
previous generation
new generation
actor
timestamp
operation type
affected replica
```

Example:

```text
generation 91
ADD_REPLICA ABC -> Server B

generation 92
REMOVE_REPLICA ABC -> Server A
```

This operational history MUST NOT become Revision history.

---

# 35. Storage migration

Moving Resource ABC from Server A to Server B proceeds:

```text
ABC on A
   |
copy
   v
ABC candidate on B
   |
verify full resource hash
   v
ABC verified on B
   |
register replica
   v
Storage Map: A + B
   |
optional removal
   v
Storage Map: B
```

At no point is a new Revision required.

If verification on B fails, A MUST remain authoritative if it was previously the only valid replica.

---

# 36. Replica policy

A Project MAY define an operational durability policy.

Example:

```json
{
  "minimum_verified_replicas": 1
}
```

A more conservative project could use:

```json
{
  "minimum_verified_replicas": 2
}
```

Publication MUST satisfy the configured policy for newly required Resource Objects.

Replica count is an operational property.

Changing:

```text
1 replica
```

to:

```text
2 replicas
```

does not create a Revision.

---

# 37. Contributor-controlled storage

A Contribution MAY reference Resource Objects stored under contributor custody.

Example:

```text
Joakim Project

Bass -> Joakim storage
Drums -> Joakim storage
Cello -> Anna storage
```

This is valid.

Integration does not inherently require transferring Anna's cello into Joakim's storage.

However, the integrating user SHOULD be offered an option to create an additional Project-controlled replica.

This may be strongly recommended for Releases.

---

# 38. Contribution

A Contribution is a first-class object.

Conceptually:

```json
{
  "contribution_id":
    "019fa...",

  "project_id":
    "019aa...",

  "base_revision":
    "omvcs:revision:sha256:BASE...",

  "head_revision":
    "omvcs:revision:sha256:HEAD...",

  "target_line":
    "019-line-main",

  "creator":
    "019cc17d-1b22-7a41-9fe9-c345c468f82c",

  "intent": {
    "kind":
      "component-replacement",

    "components": [
      "019-bass"
    ],

    "message":
      "Alternative bass part"
  },

  "state":
    "open"
}
```

The Contribution does not contain copied creative content.

It references OMVCS history.

---

# 39. Contribution state

Core recognizes at minimum:

```text
open
integrated
partially_integrated
closed
superseded
```

Platform Protocol may define richer collaborative presentation.

Contribution state is collaboration metadata and does not alter Revision identity.

---

# 40. Contribution evolution

A contributor MAY add more Revisions to the same Contribution.

Example:

```text
Target R10
    |
Contributor C1
    |
Contributor C2
    |
Contributor C3
```

The Contribution's `head_revision` may therefore move.

This movement MUST use guarded update semantics similar to Line movement.

---

# 41. Creative Integration

Creative Integration always produces an ordinary Revision.

Suppose:

```text
Main:
M1 -> M2 -> M3

Contribution:
      \
       C1 -> C2
```

Integrating C2 may produce:

```text
M1 -> M2 -> M3 -----\
                      I1
       C1 -> C2 -----/
```

where `I1` has:

```text
parents = [M3, C2]
```

and contains the complete resulting Project State.

---

# 42. Selective Integration

Suppose C2 changes:

```text
Cello
Drums
```

and the user accepts only:

```text
Cello
```

the resulting Project State contains:

```text
Cello -> contributed Component State
Drums -> Main's Component State
```

The new Revision MAY still reference both branch heads as parents, but provenance MUST state what was actually incorporated.

Conceptually:

```json
{
  "provenance": [
    {
      "type":
        "component-integration",

      "source_revision":
        "C2",

      "component_id":
        "CELLO",

      "component_state":
        "CELLO_STATE_C2"
    }
  ]
}
```

This prevents the graph alone from being mistaken for precise component-level provenance.

---

# 43. Provenance

OMVCS provenance is structured historical metadata. In OMVCS 0.1, the exact versioned schema of the containing historical object owns the permitted provenance entry fields, shapes, meanings, and nested collection rules. Generic Core does not infer a provenance relationship kind from a key or value.

Examples of provenance relationship kinds that a versioned schema MAY permit include:

```text
created_by
derived_from
integrated_from
component_taken_from
forked_from
```

A provenance relation MUST identify its source through stable OMVCS identity whenever possible, as required by the exact schema that permits that relation.

Free-text attribution MAY supplement structured provenance but MUST NOT replace it where structured identity is available.

---

# 44. Forking a Project

Repository copying and Project forking are different operations.

## Copy/transfer

Preserves:

```text
Project Identifier
history
Revision identifiers
```

## Fork

Creates:

```text
new Project Identifier
```

while preserving provenance:

```text
forked_from:
    Original Project
    Original Revision
```

This lets a creator deliberately establish an independent Project lineage without pretending it has no origin.

---

# 45. Platform Mirror

The Platform Mirror contains only metadata permitted by the Platform Protocol.

Core requires that it MUST NOT contain:

- Project Resource bytes;
- Chunk bytes;
- storage credentials;
- secret access tokens.

It MAY contain:

- Project identity;
- component metadata;
- Revisions;
- Lines;
- Releases;
- Contribution metadata;
- provenance;
- Resource identifiers;
- public availability information.

The exact mirror schema belongs to the Platform Protocol.

---

# 46. Synchronization model

OMVCS separates:

```text
metadata synchronization
resource synchronization
platform synchronization
```

A single UI button MAY trigger all three.

The architecture MUST nevertheless treat them independently.

---

# 47. Metadata pull

A metadata pull retrieves new repository historical and operational information without necessarily retrieving Resource Objects.

Example:

```text
Remote history:
R1 -> R2 -> R3 -> R4 -> R5

Local history:
R1 -> R2
```

A metadata pull MAY retrieve information about R3–R5 while downloading none of their WAV files.

---

# 48. Resource retrieval

Resources are retrieved lazily when required by:

- Materialisation;
- audition;
- verification;
- explicit pinning;
- migration;
- release durability policy.

The client resolves:

```text
Resource Identifier
        |
    Storage Map
        |
available replicas
        |
Storage Adapter
        |
verified Resource
```

---

# 49. Replica selection

If several replicas exist, an implementation MAY choose according to:

- locality;
- cost;
- speed;
- user's preference;
- provider availability.

Such selection is operational.

It MUST NOT affect creative history.

---

# 50. Platform-first retrieval is prohibited

The Open Music Platform MUST NOT secretly become the Resource transport/storage layer by proxy.

A platform MAY issue or coordinate temporary authorization information where required, but Project media transfer should occur directly between the appropriate Resource Storage Endpoint and consumer.

Conceptually:

```text
User
 |
 | metadata/auth coordination
 v
Open Music Platform

User
 |
 | Resource bytes
 v
Creator Storage
```

not:

```text
Creator Storage
      |
      v
Open Music Platform
      |
      v
User
```

as the normal reference architecture.

---

# 51. Availability model

Core defines resource availability conceptually as:

```text
available
degraded
temporarily_unavailable
missing
corrupt
unknown
```

### Available

At least one verified retrievable replica is known.

### Degraded

The Resource remains retrievable but the configured durability policy is no longer satisfied.

### Temporarily unavailable

A known replica exists but cannot presently be accessed.

### Missing

No retrievable valid replica is known.

### Corrupt

A located replica fails verification.

### Unknown

Availability has not been established.

A Resource may have several replicas with different individual states.

Overall Resource availability is derived from them.

---

# 52. Reproducibility

Core MUST treat Resource availability separately from DAW reproducibility.

Example:

```text
All audio resources: available
Ardour session: available
Required commercial plugin: unavailable
```

Resource availability:

```text
available
```

Reproducibility:

```text
partial
```

The DAW Adapter determines reproducibility details.

---

# 53. Reference Render

A Revision MAY include a Reference Render Resource in its publication. This section does not define a Reference Render field in the Project State body: OMVCS 0.1 Project State has only the members defined in section 13. A Reference Render may participate in Project State identity only through a member explicitly defined as historical by the exact applicable versioned Project State schema. The remaining Reference Render policy and association rules are tracked by DEC-CORE-002.

Where present it MUST itself be an immutable Resource Object.

Example:

```text
Reference Render
    omvcs:resource:sha256:MIXABC...
```

A Reference Render does not replace the editable Project State.

The UI MUST be able to distinguish:

```text
play intended sound
```

from:

```text
reconstruct editable project
```

---

# 54. Dependency records

Core supports immutable dependency declarations produced by DAW Adapters.

Conceptually:

```json
{
  "kind":
    "audio-plugin",

  "identifier":
    "com.vendor.superreverb",

  "version":
    "4.2",

  "required":
    true
}
```

Core does not decide whether the dependency can actually be installed.

It merely preserves and presents the adapter's declaration.

---

# 55. Integrity verification

Verification exists at several levels.

## Resource

```text
SHA256(reconstructed bytes)
    ==
Resource Identifier
```

## Chunk

```text
SHA256(chunk bytes)
    ==
Chunk Identifier
```

## Metadata object

```text
SHA256(canonical metadata)
    ==
Object Identifier
```

## Revision ancestry

Every referenced parent must either:

- be available;
- be absent under an exact declared history boundary according to the
  supported import rules;
- or cause validation failure.

An absent declared parent is not resolved or admitted by the declaration.
The referring Revision MUST NOT be admitted while a required parent remains
unresolved. Validation reports the declared incompleteness separately from
metadata corruption. The reference implementation SHOULD default to complete
ancestry.

---

# 56. Repository validation

A full Project validation MAY include:

```text
validate canonical object hashes
validate each Revision against its exact available schema and closed 0.1 member set before historical admission or identity calculation
resolve the referenced Project State and require its valid/admitted status for Revision admission
resolve each parent Revision and require valid/admitted status and matching Project identity for Revision admission
validate canonical `created_at`, typed `author_id`, `message`, and schema-owned `provenance`
validate Project State references
validate each Project State against its exact available schema and closed 0.1 member set before identity calculation or historical admission
resolve and validate every Component State referenced by Project State, including equality between each map key and the referenced state's `component_id`, for Project State admission
resolve the referenced Adapter State and require its valid/admitted status under section 12 for Project State admission
validate `project_metadata` under the exact Project State schema, including nested collection classifications
validate Component State references
validate each Component State against its exact available schema and closed 0.1 member set before identity calculation or historical admission
validate Resource Reference structure and applicable schema/Adapter property admission, plus applicable operational reconstruction manifests
validate each Release against its exact available `omvcs.release/0.1`
  schema and closed seven-member body before historical admission or identity
  calculation; verify its content-derived ReleaseId
resolve its Project and target Revision as valid/admitted metadata and require
  the target Revision's Project State to identify the same Project
require exact Project-scoped Release-name uniqueness
verify an identical ReleaseId resolves to byte-identical canonical body
  bytes; treat a different body under an existing ReleaseId as an integrity
  violation
validate each Line against its exact closed member set, assigned Line
  Identifier profile, and approved generation profile; require unique Line
  Identifiers and names within each Project; resolve its Project and target
  Revision and require matching Project identity (Resource bytes are not
  required)
validate an optional Default Line preference resolves to an existing Line
  in its Project
validate provenance references
validate replica records
verify available Resource content
```

Repository validation MUST NOT automatically download historical Resources
at any verification depth.

The full Project validation above is optional as a repository-wide
operation. `ValidateRepository` is specified in section 83. A validation
result MUST distinguish metadata integrity, history completeness, Resource
state, and requested-scope/provider coverage. A `complete` history result
requires sufficient coverage, resolution of all required references in the
requested scope, and no applicable declared history boundary.

When a required target is absent, validation MUST query local operational
metadata for a declaration matching the exact referring object, normative
edge kind, and target Identifier. A match is a `declared_incomplete`
finding; no match is an `unresolved` finding. Provider failure MUST be
reported as incomplete coverage, not treated as no declaration. A boundary
does not resolve, validate, fabricate, rewrite, or admit the missing target.
The applicable historical-object admission requirements remain in force.

This declared-boundary lookup applies to required references from identified
historical metadata objects. A missing Working State safety-reference root
target MUST be reported as `unresolved`; Core MUST NOT substitute another
Identifier for a Working State Identifier or invent a root edge kind for a
boundary lookup. See ADR-0031.

Reachability-related validation coverage MUST include the applicable
Working State safety-reference roots defined in §62. Their support does not
make repository-wide reachability complete while other applicable §62 root
classes are unsupported.

A content-derived Identifier is calculated or verified from the exact
available schema and canonical body; reference resolution is not required
for that body hash. An implementation MUST perform the Revision schema/member
checks whenever it calculates or verifies a Revision Identifier, and MUST
additionally resolve/admit its Project State and
parents before admitting the Revision as valid history. It MUST perform the
Project State schema/body checks for its Identifier and additionally
resolve/admit referenced Component and Adapter State objects before
admitting the Project State. The same distinction applies to Component
State and Release Identifiers: a valid body-derived Identifier does not
itself establish historical admission or resolve absent references.

Resource verification depth MUST be explicit. `metadata_only` does not
inspect Resource bytes. `verify_available_resources` verifies bytes already
available through the supplied verification boundary and reports its
method/strength. `deep_resources` requires full-content verification of
bytes obtainable without automatic fetching or materialisation. Resource
unavailability is not corruption.

---

# 57. Repository operation log

Mutable operational information MUST be auditable without contaminating creative history.

The Repository Home therefore SHOULD maintain an append-only **Repository Operation Log**.

Example entries:

```text
ADD_REPLICA
REMOVE_REPLICA
MIGRATE_REPOSITORY_HOME
MOVE_LINE
CREATE_RELEASE
UPDATE_PLATFORM_SYNC_STATE
```

An operation-log record conceptually includes:

```json
{
  "operation_id":
    "019op...",

  "generation":
    188,

  "actor_id":
    "019cc17d-1b22-7a41-9fe9-c345c468f82c",

  "timestamp":
    "...",

  "operation":
    "ADD_REPLICA",

  "payload":
    {}
}
```

The operation log is operational history.

It is not creative Revision history.

---

# 58. Operational reconciliation

Repository operational metadata MUST use generation/version semantics to avoid silent last-write-wins data loss.

If two clients modify the same operational record from the same previous
generation, the second mutation MUST detect divergence where the changes
cannot safely commute. For a Line move, the expected current target and
expected generation are both compared atomically as specified in §17. For
Line deletion, the existence check, expected-generation comparison, and
Default Line reference check are atomic with record removal as specified
in §17. `SetDefaultLine` compares the expected current optional Line ID
with the repository value atomically before changing or clearing that
value.

Some operations may commute safely.

Example:

```text
Client A:
add replica Server B

Client B:
add replica Server C
```

Both may be retained.

But:

```text
Client A:
move line main to R10

Client B:
move line main to R11
```

requires divergence resolution.

---

# 59. Identity and authorship claims

Core records historical actors by their Actor Identifier (ActorId), as defined in the Glossary. An ActorId is an assigned UUIDv7 serialized in lowercase canonical textual form.

Every Revision MUST record its author in the required direct `author_id` member as an ActorId. OMVCS 0.1 MUST NOT wrap it in a generic `author` object. Any other historical object that records an actor MUST use an ActorId rather than a display name, email address, username, Platform account identifier, or signing key.

ActorId is independent of display name, email, username, Platform account, and signing keys. Changing any of those values, including rotating a signing key, MUST NOT change the ActorId or rewrite historical authorship. Platform/account linkage and proof that an account controls or represents an ActorId are separate Platform concerns.

Core MUST NOT infer legal ownership from authorship metadata.

Cryptographic signing MAY be attached to historical objects without becoming part of their content-derived identity.

ActorId is not a signing identity or proof of authorization. The Platform Protocol defines association and authentication behaviour separately.

---

# 60. Signatures

A signature, when present, MUST sign the immutable object identifier rather than alter the object's identifier.

Revision signatures, signature wrappers, and validation evidence are not members of the OMVCS 0.1 Revision historical body and MUST NOT affect its content-derived identifier.

Conceptually:

```text
Revision Object
      |
      v
Revision ID
      |
      +------ signed by Actor A
      |
      +------ signed by Actor B
```

This permits additional attestations without rewriting Revision identity.

---

# 61. Authorization boundary

Before performing operations such as:

- move Line;
- create Release;
- alter Storage Map;
- publish Revision;
- integrate Contribution;

Core SHOULD invoke the configured authorization mechanism.

Core defines what operation is being requested.

Platform/storage integrations determine whether the actor may perform it.

Core MUST NOT encode one specific hosted-service role model into historical data.

---

# 62. Garbage-collection roots

An object is protected from garbage collection if reachable from any protected root.

Core roots include at minimum:

```text
current Line targets
Releases
open Contributions
configured archival pins
Working State safety references
pending publication transactions
```

The Working State itself is not a root merely because it exists. Only
the following explicit references in the current persisted Working State
record are safety roots:

- a present Base Revision, through the edge `Working State -> Revision`;
- each present `ComponentStateId` in the component-source mapping, through
  the edge `Working State -> Component State`.

An absent Base Revision or component source contributes no edge. Traversal
follows the normal historical graph from each referenced object, and the
reached set remains deduplicated where roots or traversals converge. A custom
Working State may therefore root Component States outside its Base Revision;
no synthetic Revision is created. A pre-first-Revision Working State with no
historical component sources may contribute zero roots.

Reachability uses the currently persisted record. A change to its Base
Revision or component-source mapping changes the next calculation. Line
movement alone does not change these roots; `line_id` does not create an
additional root because the Line is rooted separately. `AdapterWorkingStateRef`,
recovery condition, and Core change status are operational and are not roots.
The record contains no separate immutable historical Adapter State reference.

These root edges protect immutable historical objects for the current
unpublished Working State. They do not create provenance, mutate history, or
authorize deletion of objects outside the reached set. Temporary checkpoint
root semantics remain separate under DEC-INTERACTION-004. Contributions,
configured archival pins, and pending publication transactions remain
separate Core §62 root classes.

Each admitted Release is a root whose edge is `Release -> revision_id`.
Traversal then follows the Revision metadata graph. Release metadata and
Revision metadata reachability MUST NOT depend on local Resource-byte
availability.

Reachability traverses:

```text
Revision
  -> Project State
      -> Component States
          -> Resource Objects
      -> Adapter State
          -> Resource Objects
```

---

# 63. Resource garbage collection

A Resource Object MAY be eligible for physical deletion only when:

1. it is unreachable from all protected roots relevant to the endpoint;
2. it is not required by another Project sharing the storage namespace;
3. no active transaction depends on it;
4. the retention period has expired;
5. the caller has authority over the replica.

OMVCS MUST prefer leaking storage over destroying uncertain creative history.

When safety cannot be proven, garbage collection MUST retain the object.

---

# 64. Metadata garbage collection

Historical metadata reachable from any protected root defined in §62 MUST
NOT be garbage-collected.

Temporary operational records MAY be compacted according to defined retention policies, provided current state and required auditability remain reconstructable.

---

# 65. Local cache eviction

Local immutable cache objects may be evicted independently from Project history.

Eviction MUST NOT:

- remove working files currently required by the user;
- modify historical metadata;
- unregister remote replicas.

Cache eviction is not garbage collection of the Project itself.

---

# 66. Repository export

Core defines two export classes.

## Metadata export

Contains:

```text
available historical metadata
operational metadata sufficient for location resolution
local completeness and declared-boundary information
Project identity
```

but MAY omit Resource bytes. An export MUST NOT claim complete history if
the exported scope is incomplete or not assessed.

---

## Archival export

Contains:

```text
complete historical metadata
required operational metadata
all selected Resource Objects
verification information
```

and SHOULD be capable of fully offline reconstruction within the limitations of external software dependencies.

An export whose source lacks required historical metadata MUST NOT be
represented as a complete archival export. It MAY be emitted as a metadata
export that preserves the source's completeness and boundary information.

---

# 67. Repository import

Import MUST verify:

- object identities;
- Revision ancestry;
- Project identity;
- Resource identity where bytes are present.

An import MAY intentionally omit required historical targets only when the
corresponding omissions are represented by exact local declared history
boundaries under section 55. Otherwise absent targets are unresolved.
Import MUST preserve the original historical object bytes and MUST NOT
treat a boundary declaration as resolving or admitting its missing target.
Implementations MAY preserve imported objects with unresolved required
references outside admitted history; they MUST NOT report them as valid
admitted objects until the ordinary admission requirements are met. The
boundary persistence and transport representation is not specified here.

Ordinary import preserves Project identity.

A user explicitly requesting a fork creates another Project Identifier and records provenance.

---

# 68. Recovery scenarios

OMVCS Core MUST support the following architectural recovery cases.

## Local computer lost

Recover:

```text
Repository Home metadata
+
Resource storage
```

onto another machine.

Creative identity remains unchanged.

---

## Open Music Platform lost

Recover directly from:

```text
Repository Home
+
Resource storage
```

No creative history changes.
Recovery is limited to locally available history. If the Repository Home is
declared incomplete, the missing targets remain missing and the recovered
Project MUST NOT be represented as complete.

---

## Repository Home lost but local metadata survives

A new Repository Home MAY be established.

Resource locations are preserved or re-established.

No creative history changes. Any incompleteness in the surviving local
metadata remains explicit; recovery does not establish missing targets or
turn incomplete history into a complete claim.

---

## One Resource Endpoint lost

Resources with alternative replicas remain recoverable.

Resources without alternatives become unavailable but remain referenced in history.

---

## Storage provider migration

Copy and verify Resources to another endpoint, update Storage Map, retire old endpoint.

No creative history changes.

---

# 69. Partial failure recovery

Long-running operations MUST be resumable.

An implementation SHOULD persist transaction state for:

```text
publication
storage migration
repository-home migration
bulk replication
large materialisation
repository import
```

Example publication transaction:

```text
PREPARING
RESOURCES_UPLOADING
RESOURCES_VERIFIED
METADATA_PERSISTED
LINE_UPDATED
PLATFORM_PENDING
COMPLETE
```

After restart, Core MUST determine the last confirmed state and continue or safely roll back operational changes where possible.

---

# 70. Platform outage during publication

If:

```text
Resources durable
Metadata durable
Line updated
Platform unavailable
```

the Revision remains valid and durable.

The platform synchronization state becomes:

```text
pending
```

and may be retried later.

The Open Music Platform is therefore never part of the creative transaction's durability boundary.

---

# 71. Storage outage during publication

If required Resource durability cannot be achieved, the Revision MUST NOT be reported as durably published.

The Working State remains intact.

Already transferred Resource Objects may remain as reusable orphan candidates until cleanup.

---

# 72. Divergent creative work

Suppose local Line state is:

```text
R5 -> Local R6
```

while remote Line became:

```text
R5 -> Remote R7
```

Core MUST NOT silently choose one.

It must report divergence.

The user or DAW workflow may then:

```text
integrate
create another Line
abandon local work
rebase-equivalent operation if later specified
```

No implicit destructive resolution is allowed.

---

# 73. History rewriting

OMVCS 0.1 does not permit rewriting already published Revision objects.

A mutable-reference update or removal MUST NOT destroy published historical
objects.

A Line MAY deliberately be redirected subject to permissions and policy, but the previously referenced Revision continues to exist.

Releases remain immutable.

---

# 74. Deletion semantics

Deleting a Project, Line, Contribution or Resource requires distinguishing:

```text
remove reference
hide from platform
revoke access
garbage-collect content
destroy historical object
```

These MUST NOT be treated as equivalent.

`DeleteLine` removes exactly the mutable Line record after the atomic
existence, expected-generation, and not-current-Default-Line checks in §17.
The operation MUST NOT delete the referenced Revision or any other
historical object. This rule does not decide automatic retention or pinning,
which remains governed by DEC-CORE-008.

Removing a Line does not immediately destroy its Revision history.
Removing a Line record MUST NOT itself delete any Revision, Project State,
Component State, or Resource. Automatic pin/retention behavior remains
governed separately by DEC-CORE-008.

OMVCS 0.1 defines no `DeleteRelease` or `UpdateRelease` operation. Hiding or
removing a Platform presentation does not delete or mutate the Release body.
This omission does not define Release deletion for future OMVCS versions.

Deleting a platform page does not delete creator-controlled storage.

---

# 75. Unknown object types

OMVCS implementations SHOULD preserve unknown immutable extension objects where possible.

They MUST NOT silently discard unknown data and then claim lossless repository transfer.

This supports future compatibility.

---

# 76. Schema versioning

Every canonical metadata object MUST identify its schema version.

Example:

```text
omvcs.revision/0.1
```

An implementation MUST reject or preserve-but-not-interpret schema versions it cannot safely understand.

It MUST NOT reinterpret unknown versions according to a guessed older schema.

For Component State, an unknown or unavailable schema version MAY be preserved only as uninterpreted candidate data. Such preservation MUST NOT establish a valid historical Component State or a valid Component State Identifier.

For Project State, an unknown or unavailable schema version MAY be preserved only as unchecked, uninterpreted candidate data where supported. Such preservation MUST NOT establish a valid historical Project State or a valid Project State Identifier. A Project State MUST be validated against the exact available versioned schema before admission, including validation of `project_metadata` keys, values, nested schemas, and collection classifications.

For Revision, an unknown or unavailable schema version MAY be preserved only outside valid history where supported. Such preservation MUST NOT establish a valid historical Revision or a valid Revision Identifier. A Revision MUST be validated against its exact available versioned schema before admission, including its schema-owned provenance entries and nested collection classifications.

Preserving an unknown or unavailable schema version does not make a candidate Resource Reference with `properties` valid for historical admission. Such a candidate MUST remain unchecked and MUST NOT be used to produce or commit a valid historical object until the exact applicable schema or Adapter context can validate it under section 7.

---

# 77. Protocol evolution

Future OMVCS versions may add object types and capabilities.

They MUST NOT redefine the meaning of already published object schemas.

A new semantic interpretation requires a new schema version.

Changing the admissibility or interpretation of Resource Reference `properties` requires a new version of the applicable containing schema or Adapter schema. Implementations MUST NOT apply a newer property's validation rules to an older historical schema version.

The OMVCS 0.1 Component State top-level member set is closed. A future top-level member or extension mechanism requires an explicit versioned specification and compatibility decision; implementations MUST NOT silently accept or hash unknown top-level Component State members as 0.1.

The OMVCS 0.1 Project State top-level member set is closed and contains exactly required `schema`, `project_id`, `components`, `adapter_state_id`, and `project_metadata`. A future top-level member or extension mechanism requires an explicit versioned specification and compatibility decision; implementations MUST NOT silently accept or hash unknown top-level Project State members as 0.1. All five members participate in Project State identity.

The OMVCS 0.1 Revision top-level member set is closed and contains exactly required `schema`, `project_state_id`, `parents`, `author_id`, `created_at`, `message`, and `provenance`. A future top-level member or extension mechanism requires an explicit versioned specification and compatibility decision; implementations MUST NOT silently accept or hash unknown top-level Revision members as 0.1. All seven members participate in Revision identity.

This ensures old creative history remains interpretable.

---

# 78. Core capability model

Core itself exposes capabilities independent from DAW capabilities.

Examples:

```text
immutable-resources
revision-history
lines
releases
contributions
selective-integration-model
multi-endpoint-storage
replication
migration
sparse-materialisation
repository-export
repository-recovery
```

A conforming implementation MUST advertise which optional OMVCS capabilities it supports.

Required minimum conformance will be fixed once the complete 0.1 specification set is stabilized.

---

# 79. DAW Adapter boundary

From the point of view of Core, a DAW Adapter conceptually provides operations such as:

```text
identify_project
capture_state
enumerate_resources
detect_changes
describe_changes
restore_state
validate_state
report_dependencies
render_reference
```

Core receives structured results.

It MUST NOT reach around the Adapter and manipulate DAW-specific internal state directly.

Exact signatures belong to the DAW Adapter Specification.

---

# 80. Storage Adapter boundary

Core conceptually requests operations such as:

```text
put_resource
get_resource
verify_resource
delete_replica
list_replica
copy_replica
range_read
```

Core determines:

```text
what resource
why
required verification
repository semantics
```

The Storage Adapter determines:

```text
how this provider performs the physical operation
```

Exact signatures belong to the Storage Adapter Specification.

---

# 81. Platform boundary

Core provides the platform with mirrorable metadata.

The Platform Protocol determines:

```text
authentication
public/private visibility
discovery
permissions
contribution discussion
notifications
presentation
```

Core history MUST remain valid without the platform.

---

# 82. Core operation set

OMVCS 0.1 Core should expose at least the following logical operations:

```text
CreateProject
CreateInitialWorkingState
OpenProject
ValidateRepository

CreateComponent
UpdateComponentMetadata

MaterialiseRevision
MaterialiseCustomState
AssociateWorkingStateLine
InspectWorkingState
DetectChanges
PrepareWorkingState
RestoreWorkingState

PublishRevision

CreateLine
MoveLine
RenameLine
DeleteLine
SetDefaultLine

CreateRelease

CreateContribution
UpdateContribution
CloseContribution
IntegrateContribution

AddReplica
RemoveReplica
MigrateReplica
VerifyReplica

SynchronizeMetadata
RetrieveResource

ExportRepository
ImportRepository
ForkProject

RunGarbageCollection
RecoverRepository
```

Names shown here are normative technical concepts, not necessarily UI labels.
The Line operation contracts, including `SetDefaultLine`, are specified in
§§16–17.
The Release body, admission, identity, and `CreateRelease` contract are
specified in §18. WORK-0013 owns reachability traversal from admitted
Release roots.
Working State operations MUST follow §§19–24 and the operation-specific
contracts below. Temporary safety checkpoints remain separate under
DEC-INTERACTION-004.

---

# 83. Operation contract rule

Every Core operation defined in implementation-facing documentation MUST specify:

```text
Inputs
Preconditions
Authorization requirement
Historical effects
Operational effects
Working-state effects
Failure modes
Retry behaviour
Idempotency behaviour
Result
```

Coding agents MUST NOT implement operations whose failure semantics are left implicit.

## `ValidateRepository`

`ValidateRepository` is a diagnostic operation and MUST be strictly
read-only. It MUST NOT repair metadata, update availability or verification
records, fetch or materialise Resources, change Lines, Releases, or Working
State, pin or archive objects, or delete data.

### Preconditions and effects

The repository context MUST be open/readable and the requested scope and
Resource verification depth MUST be valid. Any ordinary repository access
authorization continues to apply; no mutation authorization is introduced.
The operation has no historical, operational, or Working State effects.

An invocation failure caused by an invalid request or inability to establish
the requested repository context returns an invocation error. Provider or
enumeration failures after report construction begins are reported inside an
explicit incomplete report as specified below.

Validation is safe to invoke again because it is read-only. Each retry is a
new invocation that re-reads repository/provider state; reports are not
guaranteed to remain identical if that state changes. Core MUST NOT hide
provider failure with automatic retry or an empty success-shaped result.
With unchanged inputs and underlying state, repeated invocations are
observationally equivalent and require no operation identifier for
deduplication.

### Inputs

- `scope`: either the whole local Repository or one identified Project.
  Another narrower scope MAY be used only when a normative specification
  already defines it.
- `resource_verification_depth`: exactly one of `metadata_only`,
  `verify_available_resources`, or `deep_resources`. There is no implicit
  fetch or materialisation at any depth.

The result MUST echo both inputs. Validation MUST NOT imply checks outside
the requested scope.

### Completed report

A completed invocation returns a machine-readable report, not a pass/fail
Boolean. The report MUST contain:

- `metadata_integrity`: `valid`, `invalid`, or `indeterminate`;
- `history_completeness`: `complete`, `declared_incomplete`, `unresolved`,
  or `not_assessed`;
- Resource state for each assessed Resource: `available`,
  `unavailable_or_not_locally_materialised`, `corrupt`, or `not_checked`;
  checked Resources also report verification method and strength;
- requested-scope coverage by object enumeration and each required root
  provider, with `complete`, `partial`, or `unavailable` status and
  unavailable capabilities; and
- typed findings, including metadata integrity failure,
  schema/admission failure, identity/Project/reference mismatch, cycle,
  unresolved metadata, declared history boundary, Resource unavailable or
  corrupt, incomplete coverage, and provider/enumeration failure.

`valid` means all required metadata checks within sufficient requested-scope
coverage passed and their required targets were resolved. `invalid` means
an actually checked metadata integrity, schema, identity,
or graph invariant failed. Missing targets, declared omissions, unavailable
schemas/providers, unrequested Resource checks, and incomplete coverage
alone MUST NOT be reported as `invalid`. When absence prevents an admission
check, the referring object remains unadmitted and metadata integrity is
`indeterminate`, not `invalid` solely due to that absence. `indeterminate`
means required metadata or provider coverage prevents a conclusive
integrity determination. A declaration does not validate an absent target.
An unresolved or declared-missing reference MUST be reported with its own
finding and MUST NOT be misreported as a body-hash or schema failure solely
because the required target is absent.

History completeness is `complete` only when coverage is sufficient for the
requested scope, all required references in that scope resolve, and no
applicable declared boundary exists. A known undeclared missing reference
yields `unresolved`; otherwise a known declared omission yields
`declared_incomplete`; when neither is known but the available providers do
not permit a meaningful determination, the result is `not_assessed`.
Partial coverage MUST NOT be reported as complete and MUST NOT imply global
unreachability.

Resource verification is independent of metadata integrity and history
completeness:

- `metadata_only` does not inspect Resource bytes;
- `verify_available_resources` verifies bytes already available through
  the supplied boundary, reporting its method and strength;
- `deep_resources` performs full-content hash verification for bytes
  obtainable through the supplied boundary without automatic fetch or
  materialisation.

Unavailable Resource bytes are not corrupt. Resources not checked at the
requested depth or because the required capability is unavailable are
`not_checked`, with the reason reflected in findings/coverage.

### Provider boundary and failures

Core MUST use provider-neutral boundaries to enumerate scoped objects and
required roots, query a declared history boundary by the exact tuple
`(referring object Identifier, normative edge kind, target Identifier)`,
and request Resource verification. A boundary query returns a matching
declaration, no matching declaration, or an explicit provider failure.
Provider failure MUST NOT be converted into an empty result or “no
declaration”.

A provider/enumeration failure after report construction begins is a typed
finding with partial or unavailable coverage; the report remains an
explicit incomplete result. An invalid invocation or failure before a
report can be constructed returns an invocation error, not a completed
report. A report with negative findings means validation ran; it does not
mean the Repository passed.

The report MUST identify coverage for every required root/provider class in
section 62. A provider that is unsupported or whose semantics remain
unresolved is partial or unavailable. The WORK-0013 Line/Release traversal
MAY be used only with its explicitly partial scope; this operation MUST NOT
claim complete Core §62 reachability or global unreachable status from it.

`ValidateRepository` MUST NOT clean up obsolete declared-boundary records
when a target later becomes available. It validates the actual target and
its references normally.

For the Working State operation set, the following additional contracts
apply:

- `CreateInitialWorkingState` requires valid Project context and no existing
  Working State. Otherwise it returns `working_state_already_exists`
  without mutation.
- `InspectWorkingState` is read-only and returns persisted Core metadata,
  derived Core comparison status, and recovery condition. Repeated reads
  are idempotent; comparison may differ when mutable content changes.
- `AssociateWorkingStateLine` changes only the optional association.
  Repeating the current association succeeds as a no-op. It MUST NOT change
  Base Revision, Adapter content, or history.
- Full and custom/selective materialisation validate Project and admitted
  target/source metadata before Adapter replacement. Full rematerialisation
  updates Base Revision only on successful Core commit; custom/selective
  materialisation does not change Base Revision and updates selected
  component-source mappings and the AdapterWorkingStateRef as required by
  the operation.
- When replacing `changed` or `unknown` state, materialisation defaults to
  preserving it and requires per-invocation
  `discard_current_working_state` authorization. Refusal returns
  `replacement_requires_authorization`, changes neither persisted Core
  metadata nor live Adapter state, and creates no history.
- Adapter preparation failure leaves the previous Core record authoritative.
  A partial destructive Adapter failure returns
  `adapter_partial_failure_recovery_required`; the committed Core record
  remains authoritative, recovery status is `recovery_required`, and no
  automatic retry is permitted.
- Invalid, unavailable, or unrestorable `AdapterWorkingStateRef` results
  MUST be explicit and MUST NOT be presented as exact recovery.
- A retry is a new invocation: it re-reads current state, revalidates
  metadata and authorization, and does not inherit authorization from a
  previous attempt. Destructive materialisation has no general idempotency
  guarantee; it is a successful no-op only when Core can establish that no
  destructive Adapter work is required.
- Recovery condition is separate from Core comparison status and has at
  least `confirmed`, `unconfirmed`, and `recovery_required` values.
- Operation results MUST distinguish at least invalid/missing Project
  context; missing, unadmitted, or cross-Project Revision/Component State
  sources; replacement requiring authorization; Adapter preparation/capture
  failure; Adapter restore failure; Adapter partial failure requiring
  recovery; invalid, missing, unavailable, or unrestorable
  AdapterWorkingStateRef; Working State already existing when initial
  creation requires absence; successful operation; and defined successful
  no-op.
- A failed operation before its Core commit point MUST leave the previously
  committed Core Working State record authoritative. If failure occurs
  after that commit point, the newly committed record remains authoritative.
  Any mismatch between the committed record and live Adapter state MUST be
  represented by the recovery condition, not hidden by rollback claims or
  Core comparison status.

These are logical contract requirements. They do not prescribe provider
storage, transaction-log, retention, or M3/M4 recovery mechanics.

---

# 84. Example complete history

Consider:

```text
Project P

Components:
Bass B
Drums D
Vocal V
```

Initial states:

```text
B1
D1
V1
```

Project State S1:

```text
B -> B1
D -> D1
V -> V1
```

Revision R1:

```text
parents: []
state: S1
```

User changes Bass:

```text
B1 -> B2
```

Project State S2:

```text
B -> B2
D -> D1
V -> V1
```

Revision R2:

```text
parent: R1
state: S2
```

Then Anna contributes cello:

```text
Component C
State C1
```

Her Project State S3:

```text
B -> B2
D -> D1
V -> V1
C -> C1
```

Contribution Revision C-R1:

```text
parent: R2
state: S3
```

Joakim accepts cello but also changes vocal to V2.

Result S4:

```text
B -> B2
D -> D1
V -> V2
C -> C1
```

Integration Revision R3:

```text
parents:
    R2
    C-R1

state:
    S4

provenance:
    C1 came through Anna's contribution
```

This is a complete OMVCS creative-history representation.

---

# 85. Example storage independence

R3 references:

```text
Bass B2 -> Resource A
Drums D1 -> Resource B
Vocal V2 -> Resource C
Cello C1 -> Resource D
```

At first:

```text
A -> Joakim Server 1
B -> Joakim Server 1
C -> Joakim Server 1
D -> Anna OneDrive
```

Later:

```text
A -> Joakim Server 2
B -> Joakim Server 1
C -> Joakim Server 1
D -> Anna OneDrive
     Joakim Server 2
```

R3 remains byte-for-byte identical.

Its Revision Identifier remains identical.

No new Revision has occurred.

---

# 86. Example platform disappearance

Before disappearance:

```text
Local
    metadata
    working resources

Repository Home
    complete metadata
    Resource history/location map

Open Music Platform
    collaboration mirror
```

The Open Music Platform vanishes permanently.

Another OMVCS implementation connects to Repository Home.

It reconstructs:

```text
Project
Lines
Revisions
Component States
Provenance
Releases
Storage Map
```

and materialises required Resource Objects.

The Project continues with exactly the same identities.

This example assumes the Repository Home contains complete metadata. If its
local history is incomplete, only the available history can be reconstructed
and its declared or unresolved omissions remain explicit.

This is required behaviour, not an emergency workaround.

---

# 87. Design constraints for implementation agents

Coding agents implementing OMVCS Core MUST follow these rules:

> Do not introduce DAW-specific behaviour into Core.

> Do not encode storage location into content-addressed historical objects.

> Do not use filenames as stable identity.

> Do not mutate immutable objects.

> Do not use timestamps to determine ancestry.

> Do not use last-write-wins for divergent Line updates.

> Do not make Platform availability part of Revision durability.

> Do not silently replace unavailable resources with alternatives.

> Do not silently discard unknown DAW state.

> Do not interpret Contribution integration as generic filesystem merging.

> Do not delete uncertain historical content during garbage collection.

> Do not invent fallback semantics where this specification defines failure.

---

# 88. Decisions deliberately left for the remaining fixed specifications

This Core Specification intentionally stops at boundaries already assigned to the established documentation set.

The following are **not new specifications**.

They remain tasks for the documents we already agreed upon.

**DAW Adapter Specification**

Will define the exact API for capture, restore, semantic change information, component mappings, dependencies and DAW merge capabilities.

**Storage Adapter Specification**

Will define the concrete storage operations, capabilities, authentication boundary, range/chunk support and provider error semantics.

**Platform Protocol**

Will define identity, authentication, permissions, visibility, collaboration synchronization and discovery.

**Interaction Specification**

Will define how musicians experience all these operations without requiring knowledge of OMVCS internals.

**Ardour Reference Adapter Design**

Will map the generic DAW contract onto Ardour.

---

# 89. Matters that remain unresolved inside Core 0.1

There are a few areas where we should deliberately avoid pretending we have made a decision when we have not.

They belong inside this Core Specification and should be resolved before Core 0.1 is frozen:

1. Whether fixed 8 MiB chunking remains the reference storage representation or we adopt deterministic content-defined chunking.
2. Remaining Reference Render policy and association rules, excluding a
   Release body member or `CreateRelease` precondition as specified in §18.
3. Exact minimum durability policy for published Revisions.
5. Exact retention period before unreachable Resources become eligible for garbage collection.
6. Whether signing becomes mandatory for published Revisions in 0.1.
7. Whether Line deletion requires an automatic archival/pin period.

Former items 4 and 8 are resolved by ADR-0027 and ADR-0028 respectively.
`ValidateRepository` is resolved by ADR-0029. These six remaining items are
the unresolved Core decisions; their existing item numbers are retained for
the Decision Register.
Those are now a finite list of **Core decisions**, not invitations to create ten more documents.

---

# 90. Core model in one diagram

```text
PROJECT
  |
  +---------------------------------------------+
  |                                             |
  v                                             v
LINES                                        RELEASES
  |                                             |
  v                                             |
REVISION <--------------------------------------+
  |
  +---- parents[] --------> REVISION
  |
  v
PROJECT STATE
  |
  +---- COMPONENT ID ------> COMPONENT STATE
  |                              |
  |                              +--> RESOURCE ID
  |                              +--> RESOURCE ID
  |
  +---- COMPONENT ID ------> COMPONENT STATE
  |                              |
  |                              +--> RESOURCE ID
  |
  +---- ADAPTER STATE
                                 |
                                 +--> RESOURCE ID


RESOURCE ID
   |
   v
STORAGE MAP
   |
   +--> REPLICA -> STORAGE ENDPOINT A
   |
   +--> REPLICA -> STORAGE ENDPOINT B
```

The upper half describes:

> **what the music was.**

The lower half describes:

> **where the bytes currently are.**

Those two worlds meet at the immutable **Resource Identifier** and nowhere else.

---

# 91. Core architectural statement

The central OMVCS Core model can therefore be stated very compactly:

> A Project is an enduring creative identity.

> A Revision is an immutable historical statement that a particular complete Project State existed.

> A Project State contains its Project identity, Component-to-Component-State map, Adapter State reference, and schema-defined Project metadata; it identifies immutable Component States and DAW Adapter State.

> Those states refer to immutable Resource Objects by content identity.

> Resource Objects may be replicated, migrated, cached, lost and restored without altering creative history.

> Lines identify evolving creative directions.

> A Release immutably identifies a meaningful historical state for as long
> as that Release exists.

> Contributions introduce independently developed history.

> Integration produces new history while preserving provenance.

> Storage, DAWs and platforms are replaceable infrastructure around that history.

---

## Document status

**Document:** OMVCS Core Specification  
**Version:** 0.1 Draft  
**Normative:** Draft normative  
**Depends on:** OMVCS Glossary, Core Invariants Specification  
**Next fixed document:** OMVCS DAW Adapter Specification
