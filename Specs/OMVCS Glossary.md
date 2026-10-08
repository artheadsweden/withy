# OMVCS Glossary
## Open Music Version Control System
### Draft 0.1

This glossary defines the normative vocabulary used by OMVCS specifications, implementations, tests, DAW adapters, storage adapters and platform integrations.

Terms in this glossary are **semantic definitions**, not merely suggested labels.

Implementations MUST NOT silently substitute Git semantics for an OMVCS term merely because a similar concept exists in Git.

Git comparisons in this document are explanatory only.

---

## Normative language

| Term | Meaning |
|---|---|
| **MUST** | An absolute requirement of OMVCS compatibility. |
| **MUST NOT** | An absolute prohibition. |
| **SHOULD** | Expected behaviour unless there is a documented, compelling reason to behave differently. |
| **SHOULD NOT** | Behaviour that should normally be avoided unless there is a documented reason. |
| **MAY** | Optional behaviour. |
| **Immutable** | Once the object has become part of published OMVCS history, its contents and identity can never change. Modification creates a new object. |
| **Authoritative** | The representation that OMVCS treats as defining a particular fact. Other copies may exist, but disagreement is resolved against the authoritative representation according to the relevant specification. |
| **Operational** | Information necessary to operate the repository but which does not itself describe creative history. |
| **Historical** | Information that forms part of the permanent creative lineage of a Project. |

---

# Core creative concepts

## Project

A **Project** is the persistent identity of one creative undertaking managed through OMVCS.

Examples include a song, album project, soundtrack piece or other musical work.

A Project is not synonymous with:

- a local directory;
- a DAW project file;
- a cloud-storage folder;
- an Open Music platform page;
- a particular Revision.

A Project survives changes in all of those things.

A Project MUST have a globally stable identifier that does not change when:

- the Project moves between Open Music platforms;
- its resources move between storage providers;
- another DAW is used;
- the Project is exported and later restored.

A Project MAY contain many Lines, Revisions, Releases, Contributions and Creative Components.

**Git analogy:** repository.

**Important difference:** an OMVCS Project is a logical creative identity independent of both filesystem location and hosting platform.

---

## Creative Component

A **Creative Component** is a persistent semantic part of a Project whose evolution may be tracked independently.

Examples:

- bass;
- lead vocal;
- drums;
- backing vocals;
- guitar;
- synth;
- lyrics;
- arrangement;
- artwork;
- reference video;
- DAW state.

A Creative Component is not merely a filename.

For example, `bass.wav` can change repeatedly while the logical Creative Component remains **Bass**.

A Creative Component MUST have a stable identifier independent of the Resource Objects used to represent its different states.

A Project MAY contain multiple components of the same general kind, such as:

- Lead Guitar;
- Rhythm Guitar;
- Guitar Solo.

**Git analogy:** none.

---

## Component State

A **Component State** is one immutable historical state of a Creative Component.

For example:

> Bass → Tight fingerstyle take

or:

> Lead Vocal → Take 7 with edited timing

A Component State references the Resource Objects required to represent that state.

Creating another bass recording MUST create another Component State rather than modifying the old one.

A Component State MAY have one or more parent Component States to express lineage.

**Git analogy:** loosely comparable to a blob plus semantic history, but there is no direct Git equivalent.

---

## Project State

A **Project State** is an immutable description of the complete logical creative state represented by a Revision.

It identifies exactly which Component States, DAW State, Resources and other relevant objects constitute that creative state.

Example:

```text
Bass        -> Bass State B4
Drums       -> Drums State D7
Lead Vocal  -> Vocal State V3
Guitar      -> Guitar State G9
DAW State   -> Ardour State A12
Reference   -> Render R12
```

A Project State MUST use storage-independent identities.

It MUST NOT contain physical cloud-storage URLs as part of its creative identity.

Two Project States that reference different creative objects are different Project States even if they currently sound identical.

**Git analogy:** tree, but substantially richer and semantically aware.

---

# Historical version concepts

## Revision

A **Revision** is an immutable node in OMVCS creative history representing one published Project State.

A Revision MUST contain or reference:

- its Project State;
- zero or more parent Revisions;
- author identity;
- creation timestamp;
- revision message or description;
- any required provenance metadata.

A Revision with no parent is an initial Revision.

A normal Revision generally has one parent.

A Revision created through integration of independent histories MAY have multiple parents.

Once published, a Revision MUST NOT be modified.

Any creative alteration produces a new Revision.

Moving resources between storage locations MUST NOT produce a new Revision.

**Git analogy:** commit.

**Important difference:** OMVCS Revision semantics concern a complete creative state and may contain DAW-specific semantic information that Git does not understand.

---

## Revision Identifier

A **Revision Identifier** is the stable identity of a Revision.

Its calculation MUST depend only on information that forms part of immutable creative history.

It MUST NOT depend on:

- current storage location;
- current platform URL;
- temporary access credentials;
- local filesystem paths;
- replica availability.

This ensures that moving resources or platforms does not rewrite history.

---

## Parent Revision

A **Parent Revision** is a Revision from which another Revision directly descends.

Parent relationships create the **Revision Graph**.

Parentage is historical and immutable.

A later discovery that a resource has moved does not alter parentage.

---

## Revision Graph

The **Revision Graph** is the directed acyclic graph of Revision ancestry within a Project.

It expresses creative lineage.

The Revision Graph MUST NOT be used to encode infrastructure events such as storage migrations.

---

## Line

A **Line** is a named movable reference to the latest Revision in a continuing creative direction.

Examples:

> Main arrangement  
> Acoustic version  
> Experimental remix  
> Anna's direction

Creating new work on a Line moves the Line reference forward to the new Revision.

Moving a Line reference does not modify earlier Revisions.

**Git analogy:** branch.

The term **Line** describes creative lineage rather than implementation mechanics.

---

## Default Line

The **Default Line** is the Project Line presented as the primary current development direction unless another Line is explicitly selected.

There MUST be at most one Default Line at any given time.

Changing the Default Line is operational Project metadata and does not alter Revision history.

---

## Release

A **Release** is an immutable, named reference to a specific Revision intended to identify a meaningful published or milestone state.

Examples:

> Demo 0.1  
> First complete arrangement  
> Album version  
> Release 1.0

A Release MUST continue pointing to the same Revision permanently.

If another state is desired, a new Release MUST be created.

**Git analogy:** immutable tag/release.

---

# Local work concepts

## Working State

A **Working State** is the current locally editable state materialised for a user.

A Working State is not necessarily part of Project history.

It MAY contain:

- checked-out Resource Objects;
- modified files;
- newly created files;
- uncommitted DAW state;
- selectively substituted Component States;
- unresolved contribution work.

A Working State becomes historical only when captured as a Revision.

**Git analogy:** working tree, but DAW- and resource-aware.

---

## Base Revision

The **Base Revision** is the Revision from which a Working State was initially materialised.

OMVCS MUST retain this relationship so that it can determine:

- what changed;
- what is inherited;
- what contribution is being proposed;
- whether upstream history moved meanwhile.

A custom Working State MAY still have a Base Revision even if some components were subsequently substituted from other Revisions.

---

## Materialisation

**Materialisation** is the operation that transforms OMVCS metadata and remote Resource Objects into a usable local Working State.

Materialisation MAY require:

- resolving Resource Objects;
- determining available replicas;
- downloading required resources;
- verifying content hashes;
- constructing local paths;
- restoring DAW State through a DAW Adapter.

Materialisation MUST NOT create a new Revision.

**Git analogy:** checkout, but broader.

---

## Selective Materialisation

**Selective Materialisation** is Materialisation in which only the Resources necessary for the requested Working State are stored locally.

OMVCS assumes full historical metadata can exist locally while historical media remains remote.

This is a fundamental OMVCS behaviour, not merely an optimisation.

**Git analogy:** sparse checkout plus remote large-object retrieval, but central to OMVCS design.

---

## Custom Working State

A **Custom Working State** is a local Working State assembled from Component States that did not previously coexist in a single Revision.

Example:

> Bass from Revision 2  
> Drums from Revision 5  
> Current lead vocal  
> Guitar from another Line

A Custom Working State MUST NOT automatically become a Revision.

It exists for experimentation until explicitly published.

If published, OMVCS creates a new Revision describing the resulting Project State.

---

## Local Modification

A **Local Modification** is any detectable difference between the current Working State and its recorded OMVCS state.

Examples:

- audio file changed;
- MIDI changed;
- plugin settings changed;
- routing changed;
- automation changed;
- track added;
- resource removed.

Local Modification is a generic concept.

The detail available depends on DAW Adapter capabilities.

---

## Change Description

A **Change Description** is a semantic representation of one or more Local Modifications.

At minimum an implementation MAY report:

> DAW state changed.

A richer adapter SHOULD report:

> Bass audio changed.  
> Vocal automation changed.  
> Room reverb settings changed.

A Change Description is informational and MUST NOT itself determine content identity.

---

# Resource concepts

## Resource

A **Resource** is any file-like creative or technical data required to represent, reproduce, edit, audition or document a Project State.

Examples:

- WAV;
- FLAC;
- MIDI;
- DAW project state;
- plugin-state blob;
- lyrics;
- artwork;
- sample;
- video;
- reference mix.

A Resource is a logical concept.

Its immutable stored representation is a Resource Object.

---

## Resource Object

A **Resource Object** is an immutable content-addressed representation of a Resource.

Its identity MUST be derived from its content using the OMVCS-defined hashing mechanism.

If one byte changes, a new Resource Object MUST be created.

A Resource Object MUST NOT know or encode where it is physically stored.

**Git analogy:** blob, but designed for potentially very large creative files.

---

## Resource Identifier

A **Resource Identifier** is the content-derived stable identifier of a Resource Object.

The same content MUST produce the same identifier under the same OMVCS specification version.

Physical copies of the same Resource Object on multiple storage systems share the same Resource Identifier.

---

## Chunk

A **Chunk** is an immutable content-addressed subsection of a Resource Object used for efficient storage, deduplication and transfer.

Large Resource Objects MAY be represented physically as a sequence of Chunks.

Chunk boundaries MUST be deterministic according to the relevant OMVCS storage specification.

A Chunk is an implementation/storage concept and is normally invisible to the musician.

---

## Chunk Manifest

A **Chunk Manifest** identifies the ordered Chunks required to reconstruct a Resource Object.

The Chunk Manifest MUST permit verification that reconstructed bytes equal the Resource Identifier.

A Resource Object may conceptually remain one object even though its physical storage consists of many Chunks.

---

## Resource Manifest

A **Resource Manifest** is immutable metadata describing a Resource Object.

It MAY include:

- Resource Identifier;
- content length;
- media type;
- chunk structure;
- original/friendly filename;
- technical format;
- sample rate;
- channel count;
- other format-specific metadata.

The Resource Manifest MUST NOT contain mutable physical storage location as part of content identity.

---

## Friendly Name

A **Friendly Name** is a human-readable label used when presenting or materialising a Resource.

Example:

`Bass.wav`

Friendly Names are not Resource Identifiers.

Different Resource Objects MAY use the same Friendly Name at different points in history.

---

# Storage concepts

## Storage Endpoint

A **Storage Endpoint** is a configured physical or logical storage service capable of storing OMVCS repository content.

Examples:

- S3-compatible bucket;
- WebDAV server;
- Google Drive adapter;
- OneDrive adapter;
- NAS;
- local archival filesystem.

A Storage Endpoint has a stable OMVCS identity distinct from its current credentials.

Credentials MUST NOT be stored in immutable creative history.

---

## Resource Replica

A **Resource Replica** is one verified physical copy of a Resource Object at a Storage Endpoint.

A Resource Object MAY have zero, one or many known replicas.

All valid replicas of the same Resource Object MUST reconstruct to the same Resource Identifier.

---

## Storage Location

A **Storage Location** describes where one Resource Replica can currently be retrieved.

It identifies:

- the Resource Object;
- Storage Endpoint;
- provider-specific object key/path;
- verification/availability state;
- optional operational information.

Storage Location is mutable operational metadata.

It MUST NOT influence Revision identity.

---

## Storage Map

The **Storage Map** is the current mapping between Resource Objects and their known Resource Replicas.

The Storage Map answers:

> Where can this object currently be obtained?

It does not answer:

> What object belongs to this Revision?

That information belongs to creative history.

---

## Storage Migration

A **Storage Migration** moves or replicates one or more Resource Objects between Storage Endpoints without modifying creative history.

A correct migration generally follows:

> copy → verify → register new replica → optionally remove old replica.

Storage Migration MUST NOT create a new Revision merely because physical location changed.

---

## Replica Addition

A **Replica Addition** records a newly verified Resource Replica.

It changes operational metadata only.

---

## Replica Removal

A **Replica Removal** removes a known Storage Location from the Storage Map.

It MUST NOT remove the historical Resource Object identity from any Revision.

Removal of the final replica MAY make historical content unavailable, but does not alter history.

---

## Availability State

**Availability State** describes whether a Resource Object or Project State can currently be retrieved.

Possible states will be formally specified later, but may include concepts such as:

- Available;
- Partially Available;
- Temporarily Unavailable;
- Missing;
- Corrupt.

Availability is operational, not historical.

---

## Project Storage

**Project Storage** is the set of Storage Endpoints and Resource Replicas collectively providing durable storage for a Project.

A single Project MAY span multiple Storage Endpoints and multiple owners.

Project Storage is therefore logical rather than synonymous with one bucket or folder.

---

# Repository concepts

## OMVCS Repository

An **OMVCS Repository** is the complete version-controlled representation of a Project.

Conceptually it consists of:

- creative-history metadata;
- operational repository metadata;
- Resource Object history through one or more Storage Endpoints.

The Repository is independent of any particular Open Music platform.

A repository MUST be recoverable without relying on a single hosted Open Music service.

---

## Repository Metadata

**Repository Metadata** is the complete metadata required to understand and reconstruct OMVCS history and repository operations.

It includes historical and operational subsets.

---

## Historical Metadata

**Historical Metadata** is immutable or historically versioned information describing creative lineage.

Examples:

- Revisions;
- parent relationships;
- Project States;
- Component States;
- provenance;
- Releases.

Historical Metadata MUST NOT be rewritten merely because infrastructure changes.

---

## Operational Metadata

**Operational Metadata** describes current repository operation.

Examples:

- Storage Map;
- Resource Replica status;
- preferred Storage Endpoint;
- synchronization state.

Operational Metadata MAY change without creating a Revision.

---

## Portable Repository

A **Portable Repository** is a representation containing enough metadata and access to Resource Objects for another conforming OMVCS implementation to reconstruct and continue the Project without depending on the original Open Music platform.

Portability is a fundamental OMVCS requirement.

---

## Repository Export

A **Repository Export** produces a portable representation of Project metadata and, depending on selected mode, Resource Objects.

A complete export SHOULD permit fully offline archival.

A metadata-only export MAY depend on Resource Replicas that remain externally available.

---

## Repository Import

A **Repository Import** reconstructs a Project from a Portable Repository or compatible repository source.

Import MUST preserve stable identities and historical relationships.

---

# Platform concepts

## Open Music Platform

An **Open Music Platform** is a collaboration and discovery service implementing the Open Music/OMVCS platform protocol.

It MAY provide:

- accounts;
- public Project pages;
- discovery;
- Contribution coordination;
- discussion;
- Line visibility;
- Releases;
- provenance presentation.

An Open Music Platform MUST NOT be the sole custodian of a Project's Resource Objects or essential repository history.

---

## Platform Mirror

A **Platform Mirror** is the copy of Project and OMVCS metadata made available to an Open Music Platform.

The Platform Mirror MUST NOT contain project Resource Object bytes.

It MAY contain sufficient metadata to display history and coordinate collaboration.

The authoritative Project MUST remain recoverable without the Platform Mirror.

---

## Platform Independence

**Platform Independence** is the property that a Project can move to or be reconstructed on another Open Music Platform without rewriting its creative history.

Platform URLs MUST NOT be embedded into immutable creative identities.

---

# DAW concepts

## DAW State

**DAW State** is the versioned state required by a Digital Audio Workstation to reproduce the editing and production configuration associated with a Project State.

It may include:

- arrangement;
- track structure;
- routing;
- buses;
- plugin configuration;
- automation;
- clip positioning;
- tempo maps;
- mixer state.

DAW State MAY be represented by one or more Resource Objects.

A native Ardour session is an example of DAW State.

---

## DAW Adapter

A **DAW Adapter** is the implementation layer connecting OMVCS Core to a specific DAW.

The Adapter translates between:

> OMVCS concepts

and:

> the DAW's native project/session model.

The DAW Adapter MUST NOT implement its own competing version-history semantics.

OMVCS owns history.

The DAW Adapter owns interpretation of DAW state.

---

## DAW Adapter Contract

The **DAW Adapter Contract** is the normative interface that all DAW integrations implement.

It defines:

- required operations;
- optional capabilities;
- lifecycle;
- error behaviour;
- state capture;
- state restoration;
- Resource enumeration;
- semantic change reporting;
- validation.

Ardour will provide the reference implementation of this contract.

---

## Adapter Capability

An **Adapter Capability** is a formally declared optional or required feature supported by a DAW Adapter.

Potential capabilities include:

- full state capture;
- full state restoration;
- Resource enumeration;
- semantic diff;
- component-level substitution;
- selective integration;
- dependency reporting;
- reference rendering.

Clients MUST NOT assume a capability that the Adapter has not declared.

---

## Dependency

A **Dependency** is external functionality required to reproduce a Project State but not itself fully contained in the repository.

Examples:

- commercial plugin;
- external sample library;
- specific DAW version;
- system font;
- hardware device.

Dependencies SHOULD be reported by DAW Adapters where possible.

---

## Reproducibility State

**Reproducibility State** describes how completely a Project State can currently be reconstructed in a given environment.

Exact states will be defined later, but conceptually they may include:

- Exact;
- Exact with available dependencies;
- Partial;
- Render-only;
- Unavailable.

Reproducibility MUST NOT be confused with Resource Availability.

A Project may have every file available yet still be incompletely reproducible because a required plugin is unavailable.

---

## Reference Render

A **Reference Render** is an immutable audio representation documenting how a Revision was intended to sound when published.

The Reference Render serves as an auditory reference even when exact editable reproduction is impossible.

A Reference Render is not a substitute for editable DAW State.

For DAW-based musical Revisions, OMVCS SHOULD strongly encourage or require one according to later specification.

---

# Collaboration concepts

## Contribution

A **Contribution** is a first-class OMVCS collaboration object representing work proposed for consideration by another Project/Line.

A Contribution references actual Revision history.

It is not merely a discussion message or uploaded file.

A Contribution MAY contain one or many Revisions.

**Git analogy:** pull request, but richer and part of OMVCS itself rather than merely a hosting-platform concept.

---

## Contribution Base

The **Contribution Base** is the Revision against which a Contribution was originally created.

It establishes what the contributor started from.

---

## Contribution Head

The **Contribution Head** is the latest Revision included in a Contribution.

---

## Contribution Intent

**Contribution Intent** describes the contributor's intended creative role for the Contribution.

Examples:

- replacement bass;
- additional cello;
- alternate vocal;
- arrangement suggestion;
- remix;
- complete alternative direction.

Contribution Intent is semantic metadata.

It does not alter content identity, but SHOULD assist UI presentation and integration decisions.

---

## Creative Integration

**Creative Integration** is the process of incorporating all or part of a Contribution into another creative Line.

This is deliberately not defined as byte-level merging.

Creative Integration may require:

- selecting one component;
- accepting several components;
- manually reconciling DAW state;
- re-recording;
- re-arranging;
- creating an entirely new Project State.

The result of a successful Creative Integration MUST be represented by a new Revision.

**Git analogy:** merge, but only loosely.

---

## Selective Integration

**Selective Integration** incorporates only some parts of a Contribution.

Example:

> accept Anna's cello but not her drum replacement.

The architecture MUST support provenance for the accepted parts.

Selective Integration is a first-class design requirement.

---

## Integration Conflict

An **Integration Conflict** occurs when independent creative changes cannot be automatically combined according to available adapter semantics.

Examples:

- two conflicting vocal arrangements;
- drums recorded against incompatible song structures;
- two incompatible plugin-routing changes.

An Integration Conflict MUST NOT be resolved automatically by arbitrary file preference.

The DAW Adapter or user must determine a valid creative result.

---

## Provenance

**Provenance** is the preserved record of where creative material or state came from.

Provenance MAY express:

- original creator;
- contributor;
- source Revision;
- source Component State;
- Contribution;
- Integration event;
- derivation relationship.

Provenance does not imply ownership.

That distinction is central to Open Music.

---

## Provenance Link

A **Provenance Link** is a structured relationship connecting one OMVCS historical object to its creative origin.

Examples:

> Component State X derived from Component State Y.

> Revision R incorporated Contribution C.

> Cello State Z was created by Anna.

Provenance Links are historical metadata.

---

# Publication and synchronization concepts

## Capture

**Capture** is the operation through which a DAW Adapter examines the current DAW Working State and produces the information required for OMVCS to create a Revision.

Capture does not necessarily publish anything.

---

## Publish Revision

**Publish Revision** is the user operation that transforms selected Working State changes into a new immutable Revision and makes the required objects and metadata durable according to OMVCS rules.

It may internally include:

- Capture;
- creation of new Resource Objects;
- Resource upload;
- verification;
- Revision creation;
- repository metadata update;
- Platform Mirror update.

This is intentionally broader than Git `commit`.

---

## Push

Until final UI terminology is chosen, **Push** refers technically to synchronization of new local OMVCS history and Resources to their required remote locations.

A Push may have two distinct destinations:

**Creator-controlled storage**
receives Resource Objects and complete repository metadata.

**Open Music Platform**
receives permitted Platform Mirror metadata only.

A Push MUST NOT upload Project Resource Objects to the Open Music Platform.

---

## Pull

Until final terminology is chosen, **Pull** refers to synchronizing repository metadata and relevant remote state into the local OMVCS environment.

Pull does not necessarily materialise Resource Objects.

---

## Synchronization

**Synchronization** is the broader process of bringing local metadata, creator-controlled cloud metadata and Platform Mirror metadata into the expected consistent state.

Synchronization MUST distinguish creative history from operational storage changes.

---

# Integrity and failure concepts

## Content Verification

**Content Verification** is the operation of hashing retrieved or stored bytes and confirming that they match the expected immutable object identity.

A Resource Replica MUST NOT be marked verified until Content Verification succeeds.

---

## Corrupt Replica

A **Corrupt Replica** is a physical Resource Replica whose contents no longer match the expected Resource Identifier.

OMVCS MUST reject corrupt replicas as valid sources.

Other valid replicas remain usable.

---

## Orphan Object

An **Orphan Object** is a stored Resource Object or Chunk that is no longer reachable from any retained historical or operational reference.

This may occur after:

- interrupted Push;
- abandoned work;
- removed branches;
- failed transaction.

Orphan status does not imply immediate deletion.

---

## Reachability

**Reachability** describes whether an object can be reached by traversing OMVCS references from protected roots such as:

- active Lines;
- Releases;
- retained Contributions;
- configured archival roots.

Reachability is used for garbage collection.

---

## Garbage Collection

**Garbage Collection** is controlled deletion of unreachable Resource Objects, Chunks or metadata after required retention rules are satisfied.

Garbage Collection MUST NOT delete objects still reachable from protected history.

---

## Repository Recovery

**Repository Recovery** is reconstruction of usable OMVCS state after loss of one component of the system.

Examples:

- central platform lost;
- local computer lost;
- one storage provider lost.

Recovery requirements will be specified explicitly and are fundamental to OMVCS.

---

# Ownership and custody concepts

## Creator-Controlled Storage

**Creator-Controlled Storage** is Storage for which the creator or authorised Project participants, rather than the Open Music Platform, control the account or storage relationship.

The Open Music reference architecture requires creative Resources to live in Creator-Controlled Storage.

---

## Custody

**Custody** describes practical control over a physical Resource Replica.

Custody does not imply creative ownership.

A contributor MAY retain custody of their contributed Resource Object while permitting its use in a Project.

---

## Project Custody

**Project Custody** describes responsibility for maintaining sufficient Resources and metadata for continued Project operation.

It MUST NOT imply exclusive ownership of contributions created by others.

---

# Terms we should explicitly avoid conflating

These distinctions should appear prominently in the agent instructions because confusing them would create architectural bugs.

**Project ≠ DAW project file**

**Revision ≠ Resource Object**

**Creative Component ≠ filename**

**Component State ≠ Project State**

**Project State ≠ Working State**

**Revision ≠ Release**

**Resource Identifier ≠ Storage Location**

**Resource Object ≠ Resource Replica**

**Historical Metadata ≠ Operational Metadata**

**Availability ≠ Reproducibility**

**Contribution ≠ arbitrary uploaded file**

**Creative Integration ≠ byte-level merge**

**Open Music Platform ≠ OMVCS Repository**

**Creator-Controlled Storage ≠ Open Music Platform storage**

**DAW Adapter ≠ OMVCS Core**

---

# Current Git-to-OMVCS mapping

For developers and coding agents familiar with Git:

| Git concept | OMVCS concept | Equivalence |
|---|---|---|
| Repository | Project / OMVCS Repository | Partial |
| Commit | Revision | Strong conceptual similarity |
| Tree | Project State | Partial |
| Blob | Resource Object | Partial |
| Branch | Line | Strong |
| Tag | Release | Strong |
| Working tree | Working State | Partial |
| Checkout | Materialisation | Partial |
| Sparse checkout | Selective Materialisation | Partial |
| Remote | No single equivalent | Storage and platform are deliberately separate |
| Push | Push | Partial, multiple destination classes |
| Pull/fetch | Pull/Synchronization | Partial |
| Pull request | Contribution | Similar intent, first-class OMVCS concept |
| Merge | Creative Integration | Weak |
| Cherry-pick | Selective Integration may sometimes resemble it | Weak |
| Git LFS | Resource storage subsystem | Some conceptual overlap |
| Clone | Repository reconstruction + metadata sync | Weak |
| Object database | OMVCS immutable object/chunk store | Strong idea, different implementation requirements |

The final specification should warn agents:

> **The Git analogy may be used to understand intent, but Git behaviour is never normative unless OMVCS explicitly states that it is.**

---

# Likely future glossary areas

There are several terms we can already see coming but should avoid freezing until we design those subsystems properly:

**Identity and trust**
author identity, signing identity, trusted contributor, signature, delegation.

**Permissions**
reader, contributor, integrator, Project steward, storage access grant.

**Network/offline reconciliation**
remote head, divergence, synchronization conflict, stale metadata.

**Resource access**
access grant, temporary retrieval token, public preview, protected resource.

**Plugin and dependency portability**
portable dependency, opaque dependency, open dependency, substituted dependency.

**Integration semantics**
integration plan, component mapping, semantic replacement, arrangement compatibility.

**Storage transactions**
pending replica, verified replica, migration transaction, repository operation log.

I would deliberately leave those names provisional until we design their underlying behaviour.

