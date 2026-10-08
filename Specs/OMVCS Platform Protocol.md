# OMVCS Platform Protocol
## Open Music Version Control System
### Draft 0.1

This document defines the normative protocol between an **OMVCS Project/Repository** and an **Open Music Platform**.

It is subordinate to:

1. **OMVCS Glossary**
2. **OMVCS Core Invariants Specification**
3. **OMVCS Core Specification**
4. **OMVCS DAW Adapter Specification**
5. **OMVCS Storage Adapter Specification**

If this protocol conflicts with a Core Invariant, the invariant takes precedence.

The governing architectural rule is:

> **The Platform coordinates people, visibility, collaboration and discovery. It does not own creative history, Resource storage or Project identity.**

The Platform is therefore a **replaceable collaboration layer**, not the foundation upon which an OMVCS Project depends.

---

# 1. Scope

The Platform Protocol defines how an Open Music Platform may provide:

- user and organisation accounts;
- public and private Project pages;
- Project discovery;
- metadata mirroring;
- Line presentation;
- Release presentation;
- Contribution coordination;
- Contribution discussion;
- permissions;
- notifications;
- identity association;
- Resource-access coordination;
- provenance presentation;
- public playback;
- search/indexing;
- moderation-related metadata;
- platform-to-repository synchronization.

The Platform MUST NOT become authoritative for:

- Revision content;
- Resource bytes;
- Project identity;
- Project State;
- Resource identity;
- Repository Home;
- Storage Map;
- DAW State.

---

# 2. Architectural boundary

Conceptually:

```text
                     OPEN MUSIC PLATFORM

        identity / collaboration / discovery / UI
        contributions / discussions / notifications
        metadata mirror / search / access coordination

                             |
                             |
                     Platform Protocol
                             |
                             v

                    OMVCS PROJECT/CORE

        revisions / lines / releases / provenance
        project state / repository semantics

                             |
                             v

                CREATOR-CONTROLLED STORAGE

                     Resource bytes
```

The Platform may know **what exists**.

It must not become the permanent keeper of **what the Project contains**.

---

# 3. Platform Identifier

Every conforming Platform implementation MUST have a stable Platform Identifier.

Recommended format:

```text
reverse-DNS
```

Examples:

```text
org.openmusic.reference
com.example.openmusic-host
```

The identifier names the Platform implementation/service family, not an individual deployment URL.

---

# 4. Platform Instance

A specific running Platform service MUST have a stable **Platform Instance Identifier**.

Example:

```text
platform_instance_id = 019...
```

The same software may operate several instances.

Instance identity MUST be independent of hostname where practical.

---

# 5. Platform Contract version

Every Platform MUST declare which OMVCS Platform Protocol version it supports.

Example:

```json
{
  "platform_id": "org.openmusic.reference",
  "platform_version": "0.4.0",
  "protocol_version": "omvcs.platform/0.1"
}
```

A client MUST NOT guess compatibility.

---

# 6. Project mirroring model

A Platform does not host the Project itself.

It hosts a **Platform Mirror**.

The Platform Mirror contains enough metadata to:

- display Project history;
- show Lines;
- show Releases;
- coordinate Contributions;
- show provenance;
- allow discovery;
- facilitate authorized playback;
- help users locate collaboration opportunities.

It MUST NOT contain Project Resource bytes.

---

# 7. Platform Mirror contents

A Platform Mirror MAY contain:

```text
Project Identifier
Project title
Project description
public metadata
Line metadata
Revision metadata
Release metadata
Contribution metadata
Creative Component metadata
provenance metadata
Reference Render identifiers
Resource identifiers
availability summaries
adapter/reproducibility summaries
public licensing metadata
discovery tags
permissions metadata
```

The exact set depends on Project visibility and permissions.

---

# 8. Platform Mirror exclusions

The Platform Mirror MUST NOT contain:

```text
raw Resource bytes
Chunk bytes
permanent storage credentials
OAuth refresh tokens
private encryption keys
DAW licence secrets
private local filesystem paths
Repository Home credentials
```

---

# 9. Mirror is non-authoritative

If Platform metadata conflicts with Repository Home historical metadata:

```text
Repository Home historical metadata wins.
```

The Platform MUST NOT silently rewrite the Project to match its mirror.

Instead it must mark the mirror as stale, divergent or invalid.

---

# 10. Mirror generation

Each Project Mirror SHOULD have a monotonically increasing synchronization generation.

Example:

```json
{
  "project_id": "019...",
  "mirror_generation": 188
}
```

This helps detect stale or conflicting updates.

---

# 11. Mirror synchronization

Project metadata may be synchronized:

```text
Repository -> Platform
```

and, for Platform-owned collaboration metadata:

```text
Platform -> local/Core
```

Historical creative data SHOULD flow from OMVCS history to Platform, not the reverse.

---

# 12. Data ownership split

The protocol distinguishes:

**Repository-owned metadata**

Examples:

```text
Revisions
Project States
Line targets
Releases
provenance
Creative Components
```

and:

**Platform-owned collaboration metadata**

Examples:

```text
discussion comments
notifications
follows
stars/bookmarks
moderation flags
public profile presentation
Contribution review conversation
```

The Platform MUST NOT confuse the two.

---

# 13. Mirrored historical objects

When the Platform mirrors immutable historical objects, it SHOULD retain their OMVCS identifiers.

Example:

```text
Revision:
omvcs:revision:sha256:ABC...
```

It SHOULD NOT mint replacement Platform-specific identities for the underlying historical object.

It MAY have an internal database ID in addition.

---

# 14. Platform URLs are secondary

A Project may have:

```text
https://platform.example/project/abc
```

but this URL is not Project identity.

The stable Project Identifier remains authoritative.

---

# 15. Account identity

A Platform may have local user accounts.

Example:

```text
platform_user_id
```

This MUST be distinct from an ActorId. A separate association MAY link the Platform account to an ActorId, but the identifiers MUST NOT be treated as interchangeable.

A Platform username changing MUST NOT alter historical authorship.

---

# 16. Actor identity

Historical objects that identify an actor MUST use the **Actor Identifier (ActorId)** defined in the Glossary and Core Specification. In OMVCS 0.1, ActorId is an assigned UUIDv7 in lowercase canonical textual form.

The Platform may associate:

```text
Actor ID <-> Platform Account
```

This relationship MAY change operationally.

The association and its proof are separate from ActorId. Changing or removing an association MUST NOT change ActorId or historical objects.

---

# 17. Identity association

A Platform MAY verify that a Platform account controls or represents a given Actor Identifier.

Possible mechanisms include:

```text
cryptographic signature
account challenge
trusted external identity
repository-issued authorization
```

The exact authentication implementation is Platform-specific.

Proof-of-control and account-association mechanisms do not define ActorId and remain a separate unresolved Platform Protocol decision.

---

# 18. Display identity

The Platform MAY present:

```text
display name
avatar
bio
links
organisation membership
```

These are mutable presentation data.

They MUST NOT be treated as historical identity.

---

# 19. Anonymous/public actors

OMVCS history MAY contain an Actor Identifier not linked to a current Platform account.

The Platform MUST still be able to display the historical record.

It MAY present:

```text
Unlinked contributor
```

rather than pretending authorship is invalid.

---

# 20. Project registration

To expose a Project on a Platform, an authorized actor performs a logical:

```text
RegisterProjectMirror
```

The Platform receives:

```text
Project Identifier
initial public metadata
Repository proof/authorization
mirrorable historical metadata
visibility settings
```

Registration MUST NOT create a new Project identity.

---

# 21. Duplicate registration

If a Project Identifier is already known to the Platform, another actor MUST NOT create an unrelated second Project under the same identity.

The Platform must determine whether the actor is authorized to update the existing Mirror.

---

# 22. Project transfer between Platforms

A Project may be mirrored on:

```text
Platform A
Platform B
```

simultaneously.

This MUST NOT fork the Project.

Both Mirrors refer to the same Project Identifier and history.

---

# 23. Platform departure

A Project owner MAY stop using a Platform.

Removing the Mirror MUST NOT:

- delete Project history;
- delete Resource storage;
- change Project ID;
- alter Revision IDs;
- invalidate Releases.

It removes only Platform-local representation and collaboration state as governed by Platform policy.

---

# 24. Visibility model

A Platform SHOULD support at least:

```text
private
unlisted
public
```

### private

Visible only to authorized participants.

### unlisted

Accessible by direct link or explicit share but not ordinarily discoverable.

### public

Eligible for discovery/search according to Platform policy.

Visibility is operational Platform metadata.

It MUST NOT affect Revision identity.

---

# 25. Fine-grained visibility

A Project MAY expose different objects differently.

Example:

```text
Project page: public
Revision history: public
Working collaboration Lines: private
Release: public
Contribution discussion: contributors only
```

Platform implementations MAY support such granularity.

---

# 26. Line presentation

The Platform may display Lines.

It MUST treat Line targets as mutable operational references.

If:

```text
main -> R10
```

moves to:

```text
main -> R11
```

the Platform updates the mirror pointer.

R10 remains historical.

---

# 27. Line synchronization

Line updates from Repository MUST use guarded semantics.

The Platform SHOULD know:

```text
previous target
new target
generation
```

It MUST reject stale blind updates when they could hide divergence.

---

# 28. Platform cannot force-move repository Lines

A Platform MAY offer a UI action that requests a Line move.

The actual OMVCS operation must still satisfy Core authorization and guarded update rules.

The Platform must not independently invent repository history.

---

# 29. Release presentation

The Platform MAY display Releases prominently.

A mirrored Release MUST retain:

```text
Release name
target Revision
creator
created_at
description
```

The target MUST remain immutable.

---

# 30. Release deletion from Platform

Hiding/removing a Release presentation from the Platform MUST NOT mutate the underlying OMVCS Release object.

If the Project wishes to deprecate a Release, that should be represented separately.

---

# 31. Project metadata

The Platform may hold descriptive metadata such as:

```text
title
short description
long description
genre
tags
language
cover image reference
credits summary
licensing summary
```

Whether these fields belong historically or operationally must be explicitly defined.

Platform-only presentation fields MUST NOT silently become creative-history fields.

---

# 32. Search indexing

The Platform MAY index:

```text
Project title
description
Creative Component names
genre
tempo
instrument tags
contributor names
release metadata
licence metadata
Contribution Intent
```

Only metadata visible to the searching user may be indexed for them.

---

# 33. Private metadata leakage

Search MUST NOT reveal existence or properties of private Projects, Revisions, Components or Contributors to unauthorized users.

---

# 34. Resource identifiers in public index

A Platform SHOULD be cautious about exposing raw Resource hashes publicly when doing so might enable private-content correlation.

Protocol implementations MAY restrict or transform exposure of Resource identifiers in public UI while preserving them internally for authorized synchronization.

---

# 35. Contribution model

A Contribution is an OMVCS object coordinated through the Platform.

The Platform MAY provide:

```text
opening
review
discussion
status presentation
notifications
integration request
closing
superseding
```

But the underlying Contribution must reference actual OMVCS Revision history.

---

# 36. CreateContribution

Logical operation:

```text
CreateContribution
```

Input includes:

```text
Contribution ID
Project ID
Base Revision
Head Revision
Target Line
creator Actor ID
Contribution Intent
message
```

The Platform MUST validate that referenced historical objects are known or retrievable as metadata.

---

# 37. Contribution does not upload files to Platform

Opening a Contribution MUST NOT require uploading its Project Resource bytes to the Platform.

Its head Revision references Resource Objects stored through normal OMVCS storage.

---

# 38. Contribution visibility

A Contribution MAY be:

```text
private
project-participants
public
```

subject to Project and Platform policy.

Visibility does not alter Contribution history.

---

# 39. Contribution discussion

The Platform MAY attach discussion to a Contribution.

Comments are Platform collaboration metadata.

They are not OMVCS Revisions.

Example:

```text
"Love the cello, but could we keep the original drums?"
```

does not itself alter creative history.

---

# 40. Review annotations

A Platform MAY support annotations linked to:

```text
Component
time range
Revision
semantic change
Reference Render position
```

These annotations remain collaboration metadata unless explicitly converted into historical metadata.

---

# 41. Contribution head update

A contributor may advance:

```text
Contribution Head C2 -> C3
```

The Platform MUST use guarded update semantics.

A stale update MUST NOT overwrite newer Contribution history silently.

---

# 42. Contribution Base immutability

The original Contribution Base SHOULD remain fixed for the lifetime of that Contribution.

If a contributor starts again from a substantially different Base, the Platform SHOULD create a new Contribution or explicit superseding relationship rather than silently changing history.

---

# 43. Contribution status

Platform representation MUST support at least:

```text
open
integrated
partially_integrated
closed
superseded
```

matching Core semantics.

Additional Platform display states MAY exist but MUST map unambiguously.

---

# 44. Integration request

The Platform MAY initiate a request:

```text
Integrate Contribution
```

but actual Creative Integration occurs through OMVCS Core/DAW Adapter workflow.

The Platform MUST NOT synthesize a merge Revision without Core.

---

# 45. Selective Integration presentation

The Platform SHOULD be able to present:

```text
Contribution includes:
Cello
Drums
Arrangement
```

and indicate which portions were integrated.

This should be derived from Core provenance where available.

---

# 46. Integrated Contribution provenance

When Core publishes an Integration Revision, the Platform SHOULD update Contribution status by reference to that actual Revision.

Example:

```text
Contribution C integrated into Revision R42
```

not merely:

```text
marked merged
```

without historical evidence.

---

# 47. Contribution closure

Closing a Contribution without integration changes collaboration state only.

Its Revision history remains valid.

---

# 48. Contribution superseding

A Contribution MAY supersede another.

The Platform SHOULD preserve the relationship:

```text
C2 supersedes C1
```

rather than deleting C1.

---

# 49. Fork discovery

When a Project has provenance:

```text
forked_from Project X / Revision R
```

the Platform SHOULD be able to display this relationship.

A fork remains a distinct Project identity.

---

# 50. Project graph

The Platform MAY visualize relationships such as:

```text
forked_from
contributed_to
derived_from
component_taken_from
```

provided it preserves the underlying provenance semantics.

---

# 51. Provenance presentation

The Platform SHOULD distinguish:

```text
created by
contributed by
derived from
stored by
hosted on
published by
```

These are not interchangeable.

Storage custody MUST NOT be presented as authorship.

---

# 52. Licensing metadata

A Platform MAY display licensing metadata associated with:

```text
Project
Revision
Release
Creative Component
Contribution
Resource
```

where defined by Open Music licensing rules.

Licensing is metadata about permitted use.

The Platform MUST NOT infer ownership merely from storage location or upload action.

---

# 53. Licence preservation

Where a historical object carries a licence declaration, the Platform MUST preserve that declaration accurately.

It MUST NOT silently replace the creator's licence with Platform Terms as the licence governing the creative work.

---

# 54. Platform Terms versus work licence

Platform Terms govern use of the Platform service.

They MUST remain distinct from the licence applied to creative works.

Using the Platform MUST NOT automatically transfer ownership of Project content to the Platform.

---

# 55. Permissions model

The Platform SHOULD represent permissions as actions rather than vague role labels.

Examples:

```text
view_project
view_private_history
comment
create_contribution
update_own_contribution
review_contribution
request_integration
integrate_contribution
move_line
create_release
manage_project_visibility
manage_project_permissions
request_resource_access
```

Roles MAY group these permissions.

---

# 56. Roles

A Platform MAY define roles such as:

```text
viewer
contributor
reviewer
integrator
project_steward
```

But protocol authorization MUST ultimately map to explicit permitted operations.

---

# 57. Project Steward

A **Project Steward** MAY be a Platform role with broad coordination rights.

Stewardship MUST NOT be presented as ownership unless legal ownership is separately established.

---

# 58. Permission source

Permissions MAY originate from:

```text
Project policy
Platform account relationships
Repository authorization
organisation policy
explicit grant
```

The Platform MUST know which authority granted the permission where relevant.

---

# 59. Permission changes

Changing permissions is operational.

It MUST NOT rewrite creative history.

---

# 60. Revoking contributor access

Revocation prevents future authorized operations.

It MUST NOT remove already published historical contributions or provenance.

---

# 61. Historical author after account deletion

If a Platform account is deleted, existing historical attribution MUST remain.

The Platform may show:

```text
Former/removed account
```

but MUST NOT erase authorship.

---

# 62. Platform authentication

The Platform chooses authentication mechanism.

Examples:

```text
password
passkey
OAuth
federated login
enterprise SSO
```

Authentication method is not OMVCS creative identity.

---

# 63. Repository authorization proof

Before accepting repository-affecting metadata updates, the Platform SHOULD require proof that the caller is authorized by the Project.

Possible models include:

```text
signed request
repository-issued token
linked Project key
trusted Platform permission synchronized from Repository
```

Exact cryptographic design remains to be frozen within this protocol.

---

# 64. Signed historical objects

If Revisions or other OMVCS objects have signatures, the Platform SHOULD preserve and present signature status.

It MUST NOT modify the object to add its own signature.

Additional attestations remain separate.

---

# 65. Platform attestation

The Platform MAY attest:

```text
"I observed Revision R at time T"
```

or:

```text
"Actor A controlled this Platform account"
```

Such attestations MUST remain distinct from Revision identity.

---

# 66. Metadata synchronization direction

The protocol recognizes:

```text
Repository-owned -> Platform
Platform-owned -> clients
```

For a field, ownership MUST be clear.

Two masters for the same mutable field SHOULD be avoided.

---

# 67. Mirror update request

A repository/client may submit:

```text
UpdateProjectMirror
```

containing:

```text
Project ID
expected mirror generation
new mirror metadata
new historical object references
Line movements
Release additions
provenance additions
```

The Platform returns a new mirror generation.

---

# 68. Stale mirror update

If expected generation does not match current state, the Platform MUST return conflict rather than blindly overwrite.

The client can then fetch new mirror state and reconcile.

---

# 69. Immutable object synchronization

If the Platform already has immutable object metadata under a given OMVCS identifier, re-sending identical content is idempotent.

If different metadata appears under the same content-derived identifier, the Platform MUST report integrity failure.

---

# 70. Line update conflict

Suppose Platform Mirror knows:

```text
main -> R10
generation 20
```

Client A updates:

```text
main -> R11
```

Client B later tries:

```text
main -> R12
expected previous target R10
```

The Platform MUST reject B's stale update.

---

# 71. Repository remains authoritative

If Repository Home already accepted R12 through another route but Platform Mirror has R11, the next synchronization should update Platform to R12 after authorization and consistency checks.

The Platform MUST NOT demand repository history be changed to match its stale mirror.

---

# 72. Offline work

A Project MUST remain usable while Platform is offline.

Users may:

```text
edit
capture
publish to Repository Home
create local Revisions
move Lines
create Releases
```

subject to Core/storage availability.

Platform synchronization occurs later.

---

# 73. Offline Contribution preparation

A contributor MAY create Revision history locally/offline.

Opening or updating the Platform Contribution can occur when connectivity returns.

---

# 74. Platform outage during publication

A failed Platform sync does not invalidate a durable Revision.

The local state should become:

```text
platform_sync = pending
```

---

# 75. Platform sync retry

Platform synchronization SHOULD be idempotent and safely retryable.

Repeated transmission of the same immutable historical metadata MUST not create duplicates.

---

# 76. Mirror stale indicator

A Platform SHOULD record last successful synchronization time/generation.

The UI MAY indicate when displayed Project data is known to be stale.

---

# 77. Resource access coordination

The Platform may help an authorized user access a Resource without storing it.

Workflow:

```text
listener/client
    |
    | request access
    v
Platform
    |
    | verify authorization
    | request/coordinate temporary grant
    v
Storage Endpoint
    |
    | direct Resource stream
    v
listener/client
```

---

# 78. No permanent credential exposure

The Platform MUST NOT give users permanent creator-storage credentials merely to access a Resource.

Use:

```text
temporary grant
scoped token
public read
provider share capability
```

where appropriate.

---

# 79. Reference Render playback

For a publicly playable Project/Release, the Platform MAY coordinate direct access to its Reference Render.

It SHOULD prefer Reference Render rather than arbitrarily rendering or transforming DAW state itself.

---

# 80. Platform media proxy

The reference architecture SHOULD NOT route ordinary Resource media bytes through the Platform.

A temporary technical proxy MAY be permitted only where provider constraints require it and if:

- bytes are not retained as Project storage;
- the behaviour is explicit;
- privacy/security requirements are preserved.

It must not become the normal storage architecture.

---

# 81. Preview derivatives

The Platform MAY store or cache separately generated preview derivatives only if the Project's policies permit it.

Such preview objects MUST be clearly classified as Platform operational/cache data, not authoritative Project Resource Objects.

This is especially important for thumbnails, waveforms and lossy previews.

---

# 82. Cached preview invalidation

A preview cache may be regenerated or deleted freely.

Its lifecycle MUST NOT alter Project history.

---

# 83. Public playback authorization

The Platform MUST respect Project visibility/licensing/access settings.

A public Project page does not automatically imply every historical Resource is downloadable.

---

# 84. Audition versus download

The Platform MAY distinguish:

```text
play/audition permission
```

from:

```text
raw Resource download permission
```

where storage/provider mechanisms support this distinction.

---

# 85. Resource access failure

If playback Resource is unavailable, the Platform SHOULD accurately state:

```text
temporarily unavailable
missing
access denied
```

rather than implying the Release itself never existed.

---

# 86. Availability display

A Platform MAY display aggregate status such as:

```text
fully available
partially available
render available
editable state unavailable
```

These must be derived from Core/Adapter status, not guessed.

---

# 87. Reproducibility display

The Platform SHOULD distinguish:

```text
auditionable
editable
exactly reproducible
partially reproducible
```

where metadata supports it.

---

# 88. DAW compatibility presentation

The Platform MAY display:

```text
Created with Ardour
Exact restore supported with Adapter X
Reference Render available
Foreign DAWs may import portable Components
```

It MUST NOT claim cross-DAW compatibility unsupported by Adapter capabilities.

---

# 89. Discovery metadata

Projects MAY publish discovery information such as:

```text
genre
BPM
key
instrumentation
mood
language
licence
Contribution wanted
component availability
```

These fields should use standardized vocabularies where practical.

---

# 90. Contribution discovery

A Platform MAY expose Projects seeking:

```text
bass
cello
vocals
mixing
mastering
lyrics
remix
```

This is Platform discovery metadata.

It does not create Contributions itself.

---

# 91. Contribution request

A Project MAY publish an explicit **Contribution Request** such as:

```text
Looking for cello arrangement
Target Revision R18
Desired style: sparse acoustic
```

This is collaboration metadata.

A submitted response becomes an actual Contribution only once tied to OMVCS Revision history.

---

# 92. Search ranking neutrality

The Platform may rank discovery results however it chooses, subject to its own policy.

Ranking MUST NOT change OMVCS provenance or ownership.

---

# 93. Fertility/discovery concepts

Future Open Music platforms MAY expose measures such as:

```text
number of forks
number of Contributions
reuse count
derivative Project count
```

These are derived Platform metrics.

They MUST NOT be treated as creative ownership percentages.

---

# 94. Metrics

A Platform MAY maintain:

```text
plays
views
follows
forks
Contributions
downloads
reuse events
```

These are Platform operational analytics.

They MUST NOT be inserted into immutable Revision identity.

---

# 95. Metric portability

Metrics are not necessarily portable between Platforms.

Historical creative provenance is portable.

Platform popularity metrics are not part of the OMVCS Core Project.

---

# 96. Discussion portability

Discussion comments MAY be exportable, but are not required for reconstructing creative history.

A Project moving Platform must not depend on them for understanding its Revisions.

---

# 97. Moderation

A Platform MAY moderate:

```text
accounts
comments
discoverability
Project presentation
illegal content
abuse
spam
```

Moderation MUST remain distinct from rewriting underlying OMVCS history.

---

# 98. Project removal for policy reasons

A Platform MAY remove or hide a Project Mirror according to its rules.

This MUST NOT imply destruction of creator-controlled Repository/Resources.

---

# 99. Platform ban

If an account is banned:

```text
Platform access may end
```

but the user's OMVCS Project identity and external Repository remain independent.

This is a crucial anti-lock-in property.

---

# 100. Abuse reporting

Reports and moderation metadata are Platform-local unless exported.

They MUST NOT become creative provenance.

---

# 101. Illegal or unavailable content

If Platform policy prevents presentation of a Resource or Project, the Platform may refuse access/presentation.

It MUST distinguish:

```text
Platform unavailable
```

from:

```text
historically nonexistent
```

---

# 102. Notification model

Platforms MAY notify users about:

```text
Contribution opened
Contribution updated
comment added
Contribution integrated
Line updated
Release created
Project forked
permission changed
storage access required
```

Notifications are operational and ephemeral.

---

# 103. Notification deduplication

Retrying the same synchronization event SHOULD NOT generate duplicate user notifications unnecessarily.

The Platform SHOULD use stable event IDs where possible.

---

# 104. Event model

The Platform SHOULD consume/publish structured events such as:

```text
project_registered
mirror_updated
line_moved
release_created
contribution_opened
contribution_updated
contribution_integrated
visibility_changed
permission_changed
```

Events describe state transitions.

They are not creative history.

---

# 105. Event ordering

Platform event timestamps MUST NOT determine Revision ancestry.

The underlying OMVCS graph remains authoritative.

---

# 106. Webhook or subscription support

A Platform MAY support:

```text
webhooks
event streams
polling
notifications API
```

for integrations.

These mechanisms are optional implementation choices.

---

# 107. Idempotency keys

State-changing Platform operations SHOULD support idempotency identifiers.

Example:

```text
CreateContribution request ID
UpdateMirror request ID
```

Retrying a timed-out request should not create duplicate objects.

---

# 108. API error model

Every Platform Protocol error SHOULD include:

```text
error code
error class
operation
Project ID where relevant
recoverability
human-readable summary
machine-readable context
```

---

# 109. Error classes

At minimum:

```text
authentication
authorization
not_found
conflict
integrity
validation
rate_limit
temporarily_unavailable
unsupported
visibility
storage_access
cancelled
internal
```

---

# 110. Conflict is not generic failure

Concurrent state changes SHOULD return a distinct conflict result.

Clients must be able to tell:

```text
retry same request
```

from:

```text
fetch/reconcile changed state
```

---

# 111. Rate limiting

Platforms MAY rate limit API use.

They SHOULD return structured retry information.

Rate limits MUST NOT corrupt repository state.

---

# 112. Project deletion semantics

The Platform MUST distinguish at least:

```text
remove Platform Mirror
hide Project
archive Project
revoke Platform access
request Repository deletion
```

These are not equivalent.

The Platform cannot assume authority to delete creator-controlled storage.

---

# 113. Project archival

A Platform MAY mark a Mirror archived/read-only.

This affects Platform interaction.

It does not alter OMVCS historical identity.

---

# 114. Resource deletion requests

If a user asks the Platform to delete a Resource, the Platform MUST NOT directly pretend it owns that storage.

It may initiate an authorized OMVCS/storage operation where supported.

Core reachability and storage rules still apply.

---

# 115. Right-to-delete account data

Platform account/personal data deletion must be handled separately from immutable creative history.

Where legal/privacy rules require removal of personal profile data, historical Actor references may need pseudonymous/unlinked presentation while preserving Project integrity.

Exact legal implementation is outside OMVCS semantics.

---

# 116. Organisation accounts

A Platform MAY support organisations or collectives.

Projects MAY be associated operationally with an organisation.

This MUST NOT replace Project identity.

---

# 117. Organisation membership changes

Leaving an organisation MUST NOT alter historical Revisions authored while the actor was a member.

---

# 118. Project stewardship transfer

Platform stewardship may transfer from one account to another.

This is an authorization change.

It does not create a new Project.

---

# 119. Multi-Platform mirroring

One Project MAY synchronize to several conforming Platforms.

Example:

```text
Project P
  -> Platform A
  -> Platform B
```

Both may host discussions independently.

Creative history remains shared.

---

# 120. Multi-Platform collaboration metadata

Platform-local comments, follows and moderation state need not synchronize across Platforms.

OMVCS historical objects and Project identifiers can.

---

# 121. Contribution across Platforms

A contributor on Platform B MAY create an OMVCS Contribution targeting a Project primarily presented on Platform A, provided authorization and metadata exchange permit it.

The Contribution is an OMVCS concept, not owned by one hosting site.

---

# 122. Cross-Platform Contribution identifier

Contribution Identifier MUST remain stable if the Contribution is mirrored across Platforms.

Each Platform MAY have its own local discussion around it.

---

# 123. Project relocation

Moving public presentation from Platform A to Platform B SHOULD be possible without:

```text
new Project ID
new Revision IDs
new Release IDs
new Resource IDs
```

The new Platform rebuilds its Mirror from repository metadata.

---

# 124. Platform bootstrap from Repository

A Platform SHOULD be able to register an existing Project by importing mirrorable metadata from Repository Home.

This is important for Platform replacement.

---

# 125. Platform reconstruction

If all Platform database data is lost but Repository Home survives, the Platform SHOULD be able to reconstruct:

```text
Project identity
Revisions
Lines
Releases
Components
provenance
Contribution historical references where persisted in Repository
```

Platform-local comments/follows may be lost unless separately backed up.

---

# 126. Contribution persistence boundary

Core Contribution metadata necessary to understand historical collaboration SHOULD be recoverable outside the Platform.

Pure discussion/comments MAY remain Platform-local.

---

# 127. Platform cache

The Platform MAY cache immutable metadata aggressively.

Because historical objects are content-addressed, cached immutable objects require no semantic invalidation.

Mutable pointers require generation-aware invalidation.

---

# 128. Mirror database model

The Platform MAY use any internal database model.

It MUST preserve OMVCS identities and semantics.

Database row IDs MUST NOT replace OMVCS object IDs in protocol-facing behaviour.

---

# 129. Search database

Search indexes are derivative.

They may be rebuilt entirely from Platform Mirror data.

Loss of the search index MUST NOT affect Project history.

---

# 130. CDN use

The Platform MAY use a CDN for:

```text
Platform assets
thumbnails
preview derivatives
possibly temporary Resource delivery coordination
```

A CDN cache MUST NOT become the authoritative Repository.

---

# 131. Reference Render CDN caching

If Project policy allows, a temporary CDN cache of a Reference Render MAY exist.

The authoritative Resource remains creator-controlled storage.

Cache eviction must not affect Project availability if another valid Replica exists.

---

# 132. Cache integrity

Cached media SHOULD preserve or be verifiable against the original Resource identity if presented as the actual Reference Render.

If transcoded, it becomes a preview derivative and MUST be labelled as such.

---

# 133. Platform-generated transformations

A Platform may create:

```text
waveforms
thumbnails
spectrograms
low-bitrate previews
```

These are derivative operational objects.

They MUST NOT replace the Project's immutable Resource Objects.

---

# 134. No silent transcoding identity

If WAV Resource `ABC` is transcoded to MP3, the MP3 MUST NOT be exposed as Resource `ABC`.

It is a distinct derivative.

---

# 135. Project cover art

Cover art may itself be an OMVCS Resource or Platform presentation asset.

The Platform MUST know which.

If historical, it should reference the OMVCS Resource Identifier.

If Platform-only, it remains operational.

---

# 136. Public API

A conforming Platform SHOULD expose a protocol-capable API sufficient for independent OMVCS clients.

Core collaboration functionality SHOULD NOT require using one proprietary Platform GUI.

---

# 137. GUI independence

A user may interact through:

```text
Platform web UI
DAW-integrated UI
desktop Open Music client
CLI
agent
third-party client
```

The protocol must support all of them.

---

# 138. Non-interactive operation

Protocol operations MUST support non-interactive API use for:

```text
coding agents
CI
background synchronization
alternative clients
```

When human approval is needed, the API must return a structured requirement.

---

# 139. Coding-agent use

An agent may:

```text
sync mirror
open Contribution
update Contribution
query history
request integration
```

only with the same authorization semantics as a human client.

Agents receive no implicit privilege.

---

# 140. Structured user decision

Operations needing human choice SHOULD return structured options.

Example:

```json
{
  "decision_required": {
    "kind": "integration_conflict",
    "options": [...]
  }
}
```

The Platform should not bury core protocol decisions exclusively in web UI.

---

# 141. Platform capability declaration

The Platform MUST declare supported capabilities.

Example:

```json
{
  "capabilities": {
    "public_projects": true,
    "private_projects": true,
    "contributions": true,
    "discussion": true,
    "temporary_resource_grants": true,
    "multi_platform_mirroring": true,
    "organisations": true
  }
}
```

---

# 142. Capability truthfulness

A Platform MUST NOT claim support for:

```text
portable Contributions
```

if its Contribution model depends on proprietary state that cannot be exported.

---

# 143. Minimum Platform conformance

A Platform calling itself OMVCS-compatible MUST support at minimum:

```text
Project Mirror registration
Project identity preservation
immutable historical metadata mirroring
Line mirroring
Release mirroring
Contribution coordination
permissions
mirror synchronization
platform-independent Project removal
no Resource-byte custody
```

---

# 144. Optional capabilities

Optional Platform capabilities include:

```text
discussion
social following
search
advanced discovery
organisations
moderation tools
analytics
temporary playback grants
public profile pages
federation
```

---

# 145. Federation

Future Platforms MAY federate Project metadata and discovery.

Federation MUST preserve stable Project/Revision identifiers.

Federation is optional in 0.1.

---

# 146. Federation must not redefine identity

If Platform A learns about Project P from Platform B, it MUST still refer to:

```text
Project P
```

not mint a new identity merely because the source Platform differs.

---

# 147. Platform trust

Clients SHOULD treat mirrored metadata as:

```text
Platform's current view of repository state
```

not as stronger evidence than signed/verified Repository history.

---

# 148. Tampered mirror

If mirrored immutable metadata fails hash verification:

```text
mirror integrity failure
```

must be reported.

The client SHOULD retrieve authoritative metadata from Repository Home if available.

---

# 149. Platform compromise

Compromise of an Open Music Platform MUST NOT enable an attacker to silently rewrite immutable Project history without detection if clients validate OMVCS object identities.

This is a fundamental benefit of content-addressed history.

---

# 150. Malicious Line movement

A compromised or unauthorized Platform actor might attempt to move a mirrored Line.

The repository-side guarded and authorized Line update must still be authoritative.

Platform display alone MUST NOT redefine the Project's Line.

---

# 151. Security of access grants

Temporary storage grants MUST be treated as secrets until expiry.

Platforms MUST avoid logging or exposing them unnecessarily.

---

# 152. Grant scoping

Temporary access SHOULD be scoped as narrowly as provider capability permits.

Example:

```text
one Resource
read-only
30 minutes
```

rather than:

```text
entire storage account
```

---

# 153. Resource abuse prevention

The Platform MAY limit repeated temporary access requests to protect creator storage costs and bandwidth.

This is operational policy.

---

# 154. Storage-cost awareness

A Project MAY configure playback/download policy based on provider cost.

The Platform MAY present:

```text
playback unavailable due to creator storage policy
```

without altering history.

---

# 155. Project access policy

A Project MAY express Platform-facing access policy such as:

```text
public Reference Render
raw stems contributors only
full DAW state collaborators only
```

The Platform MUST enforce it when coordinating access.

---

# 156. Access policy versus licence

Access control and legal licence are distinct.

A publicly downloadable file may still carry licence restrictions.

A legally reusable Resource may still require authentication to retrieve.

---

# 157. Notifications do not define truth

A missed notification does not alter Contribution or Revision state.

Clients must query actual protocol state when correctness matters.

---

# 158. Platform time

Platform timestamps MAY record:

```text
comment time
notification time
mirror receipt time
```

They MUST NOT replace Revision timestamps or ancestry semantics.

---

# 159. Time zones

Protocol timestamps SHOULD use UTC RFC 3339.

Presentation MAY localize to user timezone.

---

# 160. Project activity feed

A Platform MAY build an activity feed from:

```text
new Revision
Line moved
Release created
Contribution opened
Contribution integrated
Project forked
```

The feed is derivative presentation.

---

# 161. Activity deletion

Removing an activity-feed item MUST NOT delete the underlying OMVCS object.

---

# 162. Project history view

The Platform SHOULD render the Revision Graph accurately.

It MUST NOT linearize divergent history in a way that falsely implies ancestry.

A simplified UI MAY hide complexity, but detailed inspection must remain possible.

---

# 163. Contribution history view

The Platform SHOULD preserve:

```text
Base Revision
Contribution Revision chain
Head Revision
Integration Revision if any
```

This gives users meaningful creative provenance.

---

# 164. Component history view

If Platform Mirror contains sufficient Component State metadata, the Platform MAY show:

```text
Bass B1 -> B2 -> B3
```

independently of overall Revision history.

This must be derived from actual Component lineage.

---

# 165. Semantic diff presentation

If DAW Adapter-generated semantic change metadata is available, the Platform MAY display it.

It MUST distinguish:

```text
exact semantic diff
```

from:

```text
inferred semantic diff
```

according to Adapter confidence.

---

# 166. Adapter absence on Platform

The Platform does not need the DAW Adapter installed merely to display Project history.

It may be unable to produce rich semantic descriptions without precomputed metadata.

This is acceptable.

---

# 167. Reference Render requirement

The Platform MAY strongly encourage Reference Renders for public Releases.

Whether they are mandatory is ultimately governed by Core/project publication policy.

The Platform MUST NOT fabricate one if absent.

---

# 168. Release playback

When a Release has a Reference Render, the Platform SHOULD associate playback specifically with that immutable Resource.

This ensures:

```text
Release 1.0
```

continues to sound like Release 1.0 even if later work progresses.

---

# 169. Current Project playback

A Project page MAY also play the Reference Render of the current Default Line head.

The UI should distinguish this from an immutable named Release.

---

# 170. Public history availability

A Project MAY choose whether historical Reference Renders are publicly playable.

Visibility is independent from historical existence.

---

# 171. Download bundles

The Platform MAY initiate creation or retrieval of export bundles.

It SHOULD NOT become the long-term store of those bundles unless explicitly operating as a permitted temporary cache.

---

# 172. Repository Export coordination

A Platform MAY offer:

```text
Export Project
```

but the actual export should be generated from Repository/Storage data via OMVCS Core.

The Platform itself is not the source of truth.

---

# 173. Project recovery link

The Platform SHOULD expose enough information to help an authorized user locate:

```text
Project Identifier
Repository Home descriptor/reference
```

without exposing secrets.

This helps migration/recovery.

---

# 174. Repository Home changes

When Repository Home migrates, Platform operational metadata MAY be updated.

No historical object changes.

---

# 175. Storage Map privacy

The Platform does not necessarily need the full private Storage Map.

It may only need:

```text
availability status
access broker information
public/authorized retrieval mechanism
```

Exact exposure should follow least privilege.

---

# 176. Endpoint disclosure

A creator may not want users to know:

```text
which provider
which bucket
which account
```

stores a Resource.

The Platform Protocol SHOULD allow access coordination without exposing unnecessary Endpoint detail.

---

# 177. Temporary retrieval abstraction

The Platform MAY request from the Resource-access layer:

```text
GetTemporaryResourceAccess(Resource ID, requester, purpose)
```

and receive an opaque grant.

It need not understand provider internals.

---

# 178. Resource purpose

Temporary access MAY specify purpose such as:

```text
audition
download
DAW materialisation
verification
```

Policy may distinguish these.

---

# 179. Public forks

When a user explicitly forks a Project, the Platform SHOULD create/present:

```text
new Project ID
forked_from provenance
```

It MUST NOT treat ordinary local copying as a fork automatically.

---

# 180. Derivative discovery

Platforms MAY show:

```text
Projects derived from this Revision
Projects using this Component
```

where provenance allows.

This is one of the strengths of Open Music.

---

# 181. Provenance privacy

Private derivative relationships MUST NOT be exposed publicly unless permitted.

---

# 182. Contribution attribution

The Platform SHOULD show contributed Components and their creators where provenance supports this.

It MUST NOT calculate legal royalty percentages from provenance unless a separate licensing/payment system explicitly defines them.

---

# 183. No hidden monetisation semantics

The Platform Protocol 0.1 does not define:

```text
payment splits
royalties
sales
subscription entitlement
commercial licensing transactions
```

A future layer may do so.

The Platform MUST NOT pretend provenance percentages are payment percentages.

---

# 184. External payment layers

A Platform MAY integrate external payment systems later, but those remain outside OMVCS creative history unless explicitly recorded through a future defined standard.

---

# 185. Project notices

A Project MAY publish notices such as:

```text
seeking collaborators
maintenance paused
archived
reference implementation
experimental
```

These are Platform-facing metadata.

---

# 186. Experimental Platform notice

The Open Music reference Platform SHOULD make clear that it is replaceable and may not be permanent.

This is especially important for the project's philosophical goal.

Users should understand:

> Project survival does not depend on this site.

---

# 187. Exportability requirement

A conforming Platform MUST allow users to retrieve or reconstruct all Platform Mirror metadata necessary to move their Project elsewhere.

It MUST NOT deliberately trap portable OMVCS metadata behind proprietary export restrictions.

---

# 188. Collaboration metadata export

Where practical, the Platform SHOULD allow export of:

```text
Contribution discussions
review comments
Project descriptions
social metadata belonging to user/Project
```

but these are not required for core Project reconstruction.

---

# 189. Platform lock-in prohibition

A conforming Open Music Platform MUST NOT introduce proprietary requirements that make existing OMVCS Project history unusable elsewhere.

Extensions MAY exist, but Core Project operation must remain portable.

---

# 190. Platform extensions

A Platform MAY attach namespaced extensions.

Example:

```json
{
  "extensions": {
    "org.example.platform": {
      "custom_discovery_score": 0.72
    }
  }
}
```

Other Platforms must be able to ignore these without losing OMVCS Project meaning.

---

# 191. Unknown extensions

Unknown optional Platform metadata MUST NOT prevent clients from interpreting standard OMVCS history.

---

# 192. Extension preservation

When exporting Platform metadata, unknown namespaced extension data SHOULD be preserved where feasible.

---

# 193. Public API stability

A Platform claiming protocol compatibility SHOULD maintain stable versioned API semantics.

Breaking changes require another protocol/API version.

---

# 194. Pagination

Large collections such as:

```text
Revision history
Contribution lists
search results
discussion threads
```

MUST support deterministic pagination or continuation mechanisms.

---

# 195. Stable ordering

When API ordering matters, sort rules MUST be explicit.

Historical ordering MUST NOT rely solely on client-local time.

---

# 196. Project query

The Platform SHOULD support retrieving a Project by stable Project Identifier.

Lookup by title is insufficient.

---

# 197. Revision query

The Platform SHOULD support retrieval by Revision Identifier.

If the object is mirrored and visible to the requester, it should be uniquely resolvable.

---

# 198. Contribution query

Contributions SHOULD be retrievable by stable Contribution Identifier.

---

# 199. Release query

A Release SHOULD be retrievable within its Project namespace.

---

# 200. Platform operation set

The normative logical Platform operations for OMVCS 0.1 include at minimum:

```text
GetPlatformInfo
GetCapabilities

RegisterProjectMirror
RemoveProjectMirror
GetProjectMirror
UpdateProjectMirror

GetRevisionMetadata
GetLine
ListLines
GetRelease
ListReleases

CreateContribution
GetContribution
UpdateContributionHead
UpdateContributionStatus
ListContributions

AddContributionComment
ListContributionComments

GetPermissions
GrantPermission
RevokePermission

RequestResourceAccess

SearchProjects

SubscribeProjectEvents
ListProjectEvents
```

Optional operations include:

```text
FollowProject
UnfollowProject
CreateOrganisation
ManageOrganisationMembership
CreateContributionRequest
ListContributionRequests
CreatePlatformAttestation
ExportPlatformMetadata
```

Exact transport syntax is not fixed here.

---

# 201. GetPlatformInfo

Returns:

```text
Platform ID
Instance ID
Platform version
Protocol version
```

---

# 202. GetCapabilities

Returns exact supported feature set.

Capabilities MAY vary by account or Project policy in addition to Platform-wide support.

---

# 203. RegisterProjectMirror

Preconditions:

```text
Project exists
requester authorized
metadata valid
```

Historical effects:

```text
none
```

Platform effects:

```text
new Mirror created
```

---

# 204. RemoveProjectMirror

Removes Platform representation.

MUST NOT imply:

```text
delete Repository
delete Resources
rewrite history
```

---

# 205. GetProjectMirror

Returns current visible Mirror plus:

```text
mirror generation
sync status
visibility
```

---

# 206. UpdateProjectMirror

Uses guarded generation semantics.

It must clearly distinguish:

```text
immutable historical additions
mutable operational updates
```

---

# 207. GetRevisionMetadata

Returns mirrored immutable Revision metadata.

It MUST retain OMVCS identifier.

---

# 208. GetLine/ListLines

Returns current mirrored Line references and generations.

---

# 209. GetRelease/ListReleases

Returns immutable Release metadata visible to requester.

---

# 210. CreateContribution

Creates Platform coordination state around an existing/new OMVCS Contribution object.

It MUST reference actual Base/Head Revisions.

---

# 211. UpdateContributionHead

Moves Contribution head under guarded semantics.

It MUST NOT rewrite old Contribution Revision history.

---

# 212. UpdateContributionStatus

Changes collaboration state.

If status becomes `integrated`, the request SHOULD include the actual Integration Revision.

---

# 213. AddContributionComment

Creates Platform discussion metadata.

No OMVCS historical effects.

---

# 214. GetPermissions

Returns effective Platform/project permissions relevant to requested operations.

---

# 215. GrantPermission/RevokePermission

Mutate operational authorization only.

Historical provenance remains untouched.

---

# 216. RequestResourceAccess

Input conceptually:

```text
Project ID
Resource ID
requester
purpose
desired duration
```

Output may be:

```text
temporary grant
public URL
authorization required
denied
temporarily unavailable
```

No permanent storage credentials may be returned.

---

# 217. SearchProjects

Returns Projects visible to requester according to search and privacy rules.

Search result identity MUST use stable Project IDs.

---

# 218. SubscribeProjectEvents

Optional persistent event subscription.

The protocol must allow resumption after disconnect without assuming notifications are the source of truth.

---

# 219. Platform event IDs

Events SHOULD have stable IDs to support deduplication and continuation.

---

# 220. Platform conformance testing

Every conforming Platform SHOULD pass automated tests covering:

```text
Project registration
Project mirror rebuild
immutable Revision mirroring
Line guarded updates
Release immutability
Contribution creation
Contribution head update conflict
Contribution integration status
Project visibility
permission enforcement
account deletion attribution preservation
Resource access coordination
no raw Resource storage
Platform outage recovery
Platform migration
duplicate sync idempotency
stale mirror conflict
unknown extension preservation
public/private search separation
```

---

# 221. Reference test Project

The protocol suite SHOULD use a canonical Project containing:

```text
several Revisions
two Lines
one Release
one open Contribution
one integrated Contribution
provenance
Reference Render
private Resource
public Resource
```

This lets Platforms prove behaviour against known state.

---

# 222. No Resource storage conformance test

A crucial test MUST verify that normal Platform operation does not require uploading Project Resource bytes to Platform storage.

---

# 223. Platform replacement test

Test:

```text
Project mirrored on Platform A
export/remove A
register same Project on Platform B
```

Expected:

```text
same Project ID
same Revisions
same Releases
same provenance
same storage Resources
```

Only Platform-local collaboration state may differ.

---

# 224. Platform loss test

Simulate total Platform database loss.

Given Repository Home remains intact, a fresh Platform instance must be able to reconstruct a valid Mirror from OMVCS metadata.

---

# 225. Stale Mirror test

Repository:

```text
main -> R12
```

Platform:

```text
main -> R10
```

Synchronization MUST advance Platform toward repository truth.

It MUST NOT demand history rewrite.

---

# 226. Contribution portability test

A Contribution created on Platform A is mirrored/read on Platform B.

Expected:

```text
same Contribution ID
same Base
same Head
same Project
same underlying history
```

Platform-specific comments may differ.

---

# 227. Permission revocation test

Actor contributes Revision C1.

Later their write permission is revoked.

Expected:

```text
C1 remains historical
attribution remains
future unauthorized updates rejected
```

---

# 228. Account deletion test

Actor deletes Platform account.

Expected:

```text
historical Actor ID remains
existing Revisions remain
display account association may disappear
```

---

# 229. Resource access test

Public Reference Render resides in creator-controlled storage.

Listener presses Play.

Expected:

```text
Platform authorizes/coordinates
Storage Adapter issues access
listener receives bytes directly from storage
Platform does not persist them as Project Resource storage
```

---

# 230. Private Resource access test

Unauthorized user requests private stem.

Expected:

```text
denied
```

No storage locator or secret should leak.

---

# 231. Mirror integrity test

Platform stores metadata under Revision ID R but bytes/fields do not hash to R.

Expected:

```text
integrity failure
Mirror object rejected/quarantined
```

---

# 232. Multi-Platform test

Same Project mirrored on A and B.

A disappears.

Expected:

```text
Project continues on B
Repository unchanged
```

---

# 233. Agent implementation rules

Coding agents implementing the Platform MUST obey:

> Do not store Project Resource bytes as the Platform's authoritative media storage.

> Do not mint new Project identities merely because a Project moves Platform.

> Do not treat Platform database IDs as OMVCS identities.

> Do not rewrite immutable history to resolve Platform conflicts.

> Do not infer authorship from storage custody.

> Do not infer ownership from account creation or upload action.

> Do not erase historical attribution when a Platform account disappears.

> Do not use blind last-write-wins for Line or Contribution head updates.

> Do not make Platform availability part of Revision durability.

> Do not expose permanent creator-storage credentials.

> Do not represent Platform comments as creative history.

> Do not silently convert Platform extensions into Core semantics.

> Do not make proprietary Platform metadata necessary to reconstruct the Project.

> Do not make Platform Terms silently replace the work's licence.

> Do not claim a Contribution is integrated without a corresponding OMVCS historical result where integration occurred.

---

# 234. Example: publish while Platform is offline

Local/Core successfully publishes:

```text
R21
```

to Repository Home.

Platform sync fails.

Local status:

```text
Revision R21: durable
main -> R21
Platform sync: pending
```

Later:

```text
UpdateProjectMirror
```

sends R21 metadata.

Platform updates.

No new Revision is created.

---

# 235. Example: Contribution

Main:

```text
R1 -> R2 -> R3
```

Anna creates:

```text
R3 -> A1 -> A2
```

Platform Contribution:

```text
base = R3
head = A2
intent = cello contribution
```

Joakim integrates only cello and publishes:

```text
R4
parents = [R3, A2]
```

Platform updates Contribution:

```text
partially_integrated
integration_revision = R4
```

Provenance shows the cello came from A2.

---

# 236. Example: Platform move

Project P exists on:

```text
openmusic.example
```

The site closes.

Joakim registers P on:

```text
anotheropenmusic.example
```

The new Platform retrieves authorized repository metadata.

It reconstructs:

```text
Project P
R1...R87
Lines
Releases
provenance
Contributions retained in repository metadata
```

The new Platform creates new URLs.

Nothing in the Project's creative identity changes.

---

# 237. Example: public playback

Release:

```text
Enough 1.0
Revision R30
Reference Render RR30
```

RR30 lives on Joakim's S3-compatible storage.

User clicks Play.

Platform requests temporary access for RR30.

Storage returns short-lived read grant.

Browser streams RR30 directly.

If the Platform disappears tomorrow, RR30 and R30 still exist independently.

---

# 238. Example: private stems

Project page is public.

Reference mix is public.

Individual stems are collaborators-only.

The Platform may show:

```text
Stems available to contributors
```

but unauthorized users cannot receive Resource grants.

Public Project visibility does not equal unrestricted Resource access.

---

# 239. Example: account rename

Anna's username changes from:

```text
anna_cello
```

to:

```text
anna_music
```

Historical Actor ID remains identical.

Past contributions display the new current profile name if desired, but authorship does not change.

---

# 240. Example: account deletion

Anna later removes her Platform account.

Her historical cello Contribution remains:

```text
Actor ID X
```

The Platform may display:

```text
Contributor account no longer linked
```

It MUST NOT change:

```text
Created by Joakim
```

or remove provenance.

---

# 241. Example: mirror tampering

Platform database row for R30 is modified manually.

Client fetches R30.

Canonical metadata no longer hashes to R30.

Client rejects mirrored object and retrieves R30 from Repository Home.

The Platform can be wrong without the Project becoming wrong.

That distinction is central.

---

# 242. Example: storage provider hidden

The listener does not need to know:

```text
Joakim uses Provider X, bucket Y
```

Platform requests an opaque temporary Resource grant.

Storage details remain private.

The architectural separation survives without leaking infrastructure.

---

# 243. Platform architecture summary

The Platform boundary can be summarized as:

```text
              OMVCS REPOSITORY

       What is the Project?
       What history exists?
       What are the Lines?
       What are the Releases?
       What are the Contributions?
       Where did material come from?

                    |
                    v

              PLATFORM MIRROR

       Who can see it?
       Who can collaborate?
       How is it discovered?
       What is being discussed?
       Who needs notification?
       How can an authorized user
       obtain temporary Resource access?

                    |
                    v

                   USER
```

The Platform knows the **social and collaborative context**.

The Repository knows the **creative truth**.

---

# 244. Architectural statement

The most important rules of this protocol are:

> **The Platform is a mirror and coordination layer, not the Project.**

> **Project identity, Revision identity and Resource identity survive Platform replacement.**

> **Resource bytes remain outside the Platform's authoritative storage.**

> **Creative history flows from OMVCS into the Platform, not from Platform UI state into history by accident.**

> **Contributions are backed by actual Revision history.**

> **Platform discussions, permissions, discovery and social data remain distinct from creative history.**

> **The Platform may disappear without the Project disappearing.**

---

# 245. Unresolved Platform Protocol decisions for 0.1

The following questions remain inside this document and should be resolved before freezing version 0.1:

1. Exact mechanism by which Platform accounts prove control of or association with an ActorId.
2. Exact cryptographic authorization/signing model for repository-to-Platform updates.
3. Which Contribution metadata must also be persisted in Repository Home for Platform-independent recovery.
4. Exact Platform Mirror schema.
5. Exact capability vocabulary.
6. Whether public Project Mirroring requires a Reference Render.
7. Exact public/private exposure rules for Resource identifiers.
8. Exact temporary Resource-access request/response schema.
9. Whether public playback requires Storage Adapters to support direct temporary grants or whether limited proxy fallback is allowed.
10. Exact permission vocabulary and default role mappings.
11. Exact rules for Project-level versus Revision-level visibility.
12. Whether Contribution discussions have a standard export format.
13. Exact event schema and event continuation semantics.
14. Whether multi-Platform synchronization/federation is part of 0.1 conformance or only optional.
15. Exact Platform Mirror recovery requirements after total Platform loss.
16. Whether licensing metadata belongs in Core historical metadata, Platform metadata, or a combination depending on scope.
17. Exact protocol transport for the reference implementation, likely HTTP/JSON but not yet frozen.
18. Exact account-deletion behaviour where historical attribution intersects privacy/legal requirements.

These remain finite questions within the **OMVCS Platform Protocol**. They do not justify additional top-level specifications.

---

## Document status

**Document:** OMVCS Platform Protocol  
**Version:** 0.1 Draft  
**Normative:** Draft normative  
**Depends on:** OMVCS Glossary, Core Invariants Specification, OMVCS Core Specification, OMVCS DAW Adapter Specification, OMVCS Storage Adapter Specification  
**Next fixed document:** **OMVCS Interaction Specification**