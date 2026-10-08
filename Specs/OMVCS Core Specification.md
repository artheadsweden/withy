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
working state           complete durable metadata
metadata                resource history
current resources       storage map
optional cache          replica information
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
Line heads
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

---

# 4. Identifier classes

OMVCS MUST distinguish between **identity assigned to persistent logical entities** and **identity derived from immutable content**.

## 4.1 Assigned identifiers

The following entities use globally unique assigned identifiers:

- Project;
- Creative Component;
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

The following collection fields in the OMVCS 0.1 conceptual object models have these semantics:

| Object and field | Semantics |
|---|---|
| Resource Reference `properties` | JSON object map keyed by property name. |
| Chunk Manifest `chunks` | Ordered sequence in reconstruction order. |
| Component State `parents` | Optional set-like collection of direct Component State parents. |
| Component State `resources` | Set-like collection of Resource References required by that state. |
| Component State `metadata` | JSON object map keyed by metadata property name. |
| Adapter State `resources` | Set-like collection of native Resource References. |
| Adapter State `component_bindings` | JSON object map keyed by Creative Component Identifier. |
| Adapter State `metadata` | JSON object map keyed by metadata property name. |
| Project State `components` | JSON object map keyed by Creative Component Identifier. |
| Project State `project_metadata` | JSON object map keyed by metadata property name. |
| Revision `parents` | Set-like collection of direct parent Revisions. |
| Revision `provenance` | Set-like collection of provenance relationships. |

Every additional array-valued collection included in a hashed Core object, Adapter State, or namespaced extension MUST declare its ordering semantics in the schema that defines it. A schema that does not make this declaration is not valid for hashing.

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

Historical structures refer to Resource Objects through a **Resource Reference**.

Conceptually:

```json
{
  "resource_id": "omvcs:resource:sha256:...",
  "byte_length": 183829331,
  "media_type": "audio/wav",
  "logical_name": "Bass.wav",
  "role": "primary-audio",
  "properties": {
    "sample_rate": 48000,
    "channels": 1
  }
}
```

Only `resource_id` identifies the Resource Object.

Other fields describe its intended interpretation.

A logical name MUST NOT be used to locate the Resource physically.

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

A Creative Component represents persistent musical meaning rather than physical files.

Conceptual Component record:

```json
{
  "component_id": "019cc...",
  "project_id": "019aa...",
  "kind": "audio-performance",
  "name": "Bass",
  "created_at": "2026-10-08T10:34:11Z"
}
```

`component_id` remains stable throughout the life of the component.

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

The `parents` field is optional. When present, it is a set-like collection: its element order has no semantic meaning, and duplicate elements are invalid. An explicitly empty `parents` array identifies an initial state with zero parents. An omitted `parents` field means parentage is unknown or not asserted; it MUST NOT be interpreted as proof that the state is initial. Implementations MUST NOT fabricate parentage for unknown historical lineage.

Derived states SHOULD record one or more parent Component States when their lineage is known. Parentage is mandatory only when a specific OMVCS operation or provenance rule explicitly requires preserving that derivation.

The `resources` field is a set-like collection: its element order has no semantic meaning, and duplicate elements are invalid. Present collection fields MUST be normalized as specified in section 5.1 before the object is hashed.

Canonical conceptual structure:

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
      "logical_name": "Bass.wav",
      "role": "primary-audio",
      "byte_length": 183829331,
      "media_type": "audio/wav"
    }
  ],

  "metadata": {
    "description": "Fingerstyle bass take"
  }
}
```

The Component State Identifier is the hash of the canonical representation.

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

In the conceptual structure below, `resources` is set-like. `component_bindings` is a JSON object map keyed by Component Identifier and follows RFC 8785 map canonicalization as specified in section 5.1. Any array-valued collection in adapter-specific metadata or extensions that participates in Adapter State identity MUST declare its ordering semantics in the Adapter State schema and follow section 5.1.

Conceptually:

```json
{
  "schema": "omvcs.adapter-state/0.1",

  "adapter_id": "org.openmusic.ardour",

  "adapter_format_version": "1",

  "resources": [
    {
      "resource_id": "omvcs:resource:sha256:...",
      "role": "native-session-state"
    }
  ],

  "component_bindings": {
    "019cc-bass": "ardour-route-id-817",
    "019cc-vocal": "ardour-route-id-922"
  },

  "metadata": {}
}
```

OMVCS Core treats adapter-specific metadata as opaque.

The DAW Adapter Specification will define exactly how adapters create and restore these objects.

---

# 13. Project State

A Project State defines one complete logical creative state.

The `components` member is a JSON object map keyed by Creative Component Identifier. Its canonical serialization uses RFC 8785 object-member ordering solely, as specified in section 5.1.

Conceptual structure:

```json
{
  "schema": "omvcs.project-state/0.1",

  "project_id": "019aa...",

  "components": {
    "019-bass": "omvcs:component-state:sha256:BASS4...",
    "019-drums": "omvcs:component-state:sha256:DRUM7...",
    "019-vocal": "omvcs:component-state:sha256:VOC3..."
  },

  "adapter_state":
    "omvcs:adapter-state:sha256:ARDOUR12...",

  "project_metadata": {
    "title": "Example Song"
  }
}
```

The Project State Identifier is content-derived.

A Project State does not describe only what changed.

It describes the complete state.

The `adapter_state` member MUST reference exactly one canonical Adapter State metadata object by its Adapter State Identifier. It MUST NOT reference a native Resource Object directly as the complete Adapter State.

---

# 14. Revision

A Revision is the principal immutable historical node.

The `parents` and `provenance` arrays are set-like collections: their element order has no semantic meaning, and duplicate elements are invalid. They MUST be normalized as specified in section 5.1 before the Revision is hashed.

Conceptual structure:

```json
{
  "schema": "omvcs.revision/0.1",

  "project_id": "019aa...",

  "parents": [
    "omvcs:revision:sha256:PREVIOUS..."
  ],

  "project_state":
    "omvcs:project-state:sha256:STATE...",

  "author": {
    "actor_id": "019cc17d-1b22-7a41-9fe9-c345c468f82c"
  },

  "created_at": "2026-10-08T11:02:17Z",

  "message": "New bass take and vocal automation",

  "provenance": []
}
```

The Revision Identifier is the hash of this canonical object.

---

# 15. Time

Historical timestamps MUST:

- use UTC;
- use RFC 3339 representation;
- include sufficient precision to preserve ordering where available.

Example:

```text
2026-10-08T11:02:17.481Z
```

Time is informational historical metadata.

Revision ancestry, not timestamps, defines causal creative history.

Implementations MUST NOT infer ancestry merely from timestamps.

---

# 16. Line

A Line is a mutable named reference.

Conceptually:

```json
{
  "line_id": "019dd...",
  "project_id": "019aa...",
  "name": "main",
  "target_revision":
    "omvcs:revision:sha256:ABC...",
  "generation": 17
}
```

The Line object itself is operational metadata.

Moving:

```text
main -> Revision 17
```

to:

```text
main -> Revision 18
```

does not modify either Revision.

---

# 17. Concurrent Line updates

Line movement MUST use compare-and-swap semantics.

A client attempting:

```text
main:
Revision 17 -> Revision 18
```

must declare that it believes the current target is Revision 17.

If another user already moved it:

```text
Revision 17 -> Revision 19
```

the update MUST fail as divergent rather than silently replacing Revision 19.

The caller must then reconcile:

```text
Revision 18
     \
      integration
     /
Revision 19
```

or create another Line.

This prevents lost creative history.

---

# 18. Release

A Release permanently identifies one Revision.

Conceptual structure:

```json
{
  "schema": "omvcs.release/0.1",

  "project_id": "019aa...",

  "name": "1.0",

  "revision":
    "omvcs:revision:sha256:ABC...",

  "created_at":
    "2026-10-08T13:42:00Z",

  "creator": {
    "actor_id": "019cc17d-1b22-7a41-9fe9-c345c468f82c"
  },

  "description":
    "First public mix"
}
```

A Release target MUST NOT be changed.

The human-readable Release name MUST be unique within its namespace.

If `1.0` already exists, another target requires another Release such as:

```text
1.0.1
```

or:

```text
1.0-remaster
```

---

# 19. Working State

The Working State exists locally.

It has:

```text
Project
Base Revision
currently materialised Project State
local Resource paths
local modifications
unpublished resources
adapter working state
```

Conceptually:

```json
{
  "project_id": "019aa...",

  "base_revision":
    "omvcs:revision:sha256:BASE...",

  "materialised_state":
    "omvcs:project-state:sha256:STATE...",

  "modified": true,

  "custom_component_sources": {}
}
```

The Working State is not content-addressed because it is intentionally mutable.

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

---

# 22. Selective Materialisation

A client MAY materialise only Resources required for the current Working State.

Historical Resource Objects that are not required need not exist locally.

Example:

A Project has 400 GB of historical media.

Current Revision requires 11 GB.

A valid local Working State MAY therefore contain:

```text
complete metadata history
+
11 GB current resources
+
optional cache
```

rather than 400 GB.

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

---

# 24. Change detection

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
Persist complete historical metadata
        |
Atomically update Line
        |
Update local repository state
        |
Mirror permitted metadata to platform
```

Platform mirroring is not required for Revision durability.

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

The Repository Home stores the complete durable OMVCS metadata necessary to reconstruct Project history.

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

The Repository Home MUST contain complete historical metadata and current operational metadata sufficient to resolve Project storage.

It MUST NOT contain credentials in historical metadata.

---

# 30. Repository Home migration

Moving Repository Home does not change creative history.

Migration consists conceptually of:

```text
1. Create destination repository
2. Copy complete metadata
3. Verify metadata integrity
4. Copy or retain storage map
5. Verify resource resolvability
6. Make destination authoritative
7. Update clients/platform mirror
8. Optionally retire old Repository Home
```

Revision identifiers and Project Identifier remain unchanged.

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

OMVCS provenance is structured historical metadata.

Core supports relationships including:

```text
created_by
derived_from
integrated_from
component_taken_from
forked_from
```

A provenance relation MUST identify its source through stable OMVCS identity whenever possible.

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

A Revision MAY include a Reference Render Resource as part of its Project State publication.

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
- be explicitly shallow/unavailable according to supported import rules;
- or cause validation failure.

The reference implementation SHOULD default to complete ancestry.

---

# 56. Repository validation

A full Project validation MAY include:

```text
validate canonical object hashes
validate Revision references
validate Project State references
validate Component State references
validate Resource manifests
validate Release targets
validate Line targets
validate provenance references
validate replica records
verify available Resource content
```

Metadata validation MUST NOT require downloading every historical Resource unless deep Resource verification is requested.

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

If two clients modify the same operational record from the same previous generation, the second mutation MUST detect divergence where the changes cannot safely commute.

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

Every Revision MUST record its author as an ActorId. Any other historical object that records an actor MUST use an ActorId rather than a display name, email address, username, Platform account identifier, or signing key.

ActorId is independent of display name, email, username, Platform account, and signing keys. Changing any of those values, including rotating a signing key, MUST NOT change the ActorId or rewrite historical authorship. Platform/account linkage and proof that an account controls or represents an ActorId are separate Platform concerns.

Core MUST NOT infer legal ownership from authorship metadata.

Cryptographic signing MAY be attached to historical objects without becoming part of their content-derived identity.

ActorId is not a signing identity or proof of authorization. The Platform Protocol defines association and authentication behaviour separately.

---

# 60. Signatures

A signature, when present, MUST sign the immutable object identifier rather than alter the object's identifier.

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

Published historical metadata referenced by retained creative history MUST NOT be garbage-collected.

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
complete historical metadata
operational metadata sufficient for location resolution
Project identity
```

but MAY omit Resource bytes.

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

---

# 67. Repository import

Import MUST verify:

- object identities;
- Revision ancestry;
- Project identity;
- Resource identity where bytes are present.

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

---

## Repository Home lost but local complete metadata survives

A new Repository Home MAY be established.

Resource locations are preserved or re-established.

No creative history changes.

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

Operations analogous to Git force-push MUST NOT destroy published historical objects.

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

Removing a Line does not immediately destroy its Revision history.

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

---

# 77. Protocol evolution

Future OMVCS versions may add object types and capabilities.

They MUST NOT redefine the meaning of already published object schemas.

A new semantic interpretation requires a new schema version.

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
OpenProject
ValidateRepository

CreateComponent
UpdateComponentMetadata

MaterialiseRevision
MaterialiseCustomState
InspectWorkingState
DetectChanges

PublishRevision

CreateLine
MoveLine
DeleteLine

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
2. Exact rules for reference renders: optional globally, required for Releases, or adapter-dependent.
3. Exact minimum durability policy for published Revisions.
4. Whether complete metadata history is mandatory locally or may itself be sparse.
5. Exact retention period before unreachable Resources become eligible for garbage collection.
6. Whether signing becomes mandatory for published Revisions in 0.1.
7. Whether Line deletion requires an automatic archival/pin period.
8. Exact treatment of shallow/incomplete history imports.
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

> A Project State is composed of immutable Component States and DAW Adapter State.

> Those states refer to immutable Resource Objects by content identity.

> Resource Objects may be replicated, migrated, cached, lost and restored without altering creative history.

> Lines identify evolving creative directions.

> Releases permanently identify meaningful historical states.

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
