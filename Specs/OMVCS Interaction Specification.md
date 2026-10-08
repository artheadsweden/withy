# OMVCS Interaction Specification
## Open Music Version Control System
### Draft 0.1

This document defines the normative interaction model for OMVCS.

It describes **how users experience OMVCS concepts and operations**, especially musicians and collaborators who should not need to understand Git, distributed systems, content addressing, storage topology or DAW internals.

It is subordinate to:

1. **OMVCS Glossary**
2. **OMVCS Core Invariants Specification**
3. **OMVCS Core Specification**
4. **OMVCS DAW Adapter Specification**
5. **OMVCS Storage Adapter Specification**
6. **OMVCS Platform Protocol**

If this specification conflicts with a Core Invariant, the invariant takes precedence.

The governing interaction rule is:

> **The user should work in musical and creative concepts. OMVCS should absorb the version-control and infrastructure complexity underneath.**

The user interface MUST therefore expose clear creative actions while preserving the exact technical semantics defined by OMVCS Core.

---

# 1. Scope

This specification defines the interaction model for:

- creating a Project;
- connecting an existing DAW project;
- opening a Project;
- seeing Project history;
- understanding the current Working State;
- seeing what changed;
- publishing a new version;
- switching between historical versions;
- creating alternate creative directions;
- combining Components from different Revisions;
- contributing to another Project;
- reviewing Contributions;
- integrating Contributions;
- resolving conflicts;
- creating Releases;
- managing storage;
- moving Resources between storage providers;
- restoring missing Resources;
- recovering a Project;
- transferring a Project to another Platform;
- handling missing dependencies;
- handling partial reproducibility;
- working offline;
- communicating failure clearly.

This specification does **not** define:

- visual style;
- fonts;
- colours;
- precise widget layout;
- pixel-level design;
- a specific desktop/web framework.

It defines the **meaning of user actions and states**.

---

# 2. Interaction principles

OMVCS user interfaces MUST follow the following principles.

## 2.1 Creative language before version-control language

The normal musician-facing interface SHOULD prefer concepts such as:

```text
Version
History
Current work
Alternate version
Contribution
Release
Use this bass
Move storage
Restore
```

rather than requiring users to understand:

```text
commit
checkout
branch
pull request
remote
object
blob
ref
```

Technical terminology MAY be exposed in advanced/developer views.

---

## 2.2 Technical precision must survive simplification

Simplifying language MUST NOT change semantics.

For example:

```text
Open this version
```

may map internally to:

```text
MaterialiseRevision
```

but the operation must still satisfy all Core materialisation rules.

---

## 2.3 Creative actions and infrastructure actions must remain distinct

A user changing storage provider MUST NOT appear to create a new musical version.

A user changing the bass recording MUST appear as a creative change.

The interface should reinforce this distinction.

---

## 2.4 No accidental history creation

Routine actions such as:

```text
Save
Close
Autosave
Move storage
Reconnect account
Download Resource
Open Project
Play
```

MUST NOT create new Revisions unless explicitly defined by a separate user action.

---

## 2.5 No hidden destructive behaviour

Operations that may:

```text
discard local changes
remove the final Resource Replica
replace current Working State
remove access
delete a Line
```

MUST disclose the consequence before execution.

---

## 2.6 Users should not need to understand distributed storage

The user SHOULD normally see:

```text
My storage
Anna's storage
Archive
Project storage
```

rather than provider keys, object hashes and Chunk Manifests.

Advanced details MAY be available.

---

## 2.7 Infrastructure details should appear when they matter

The interface MUST surface infrastructure information when it affects:

```text
availability
durability
cost
privacy
permissions
recovery
```

Abstraction must not become concealment.

---

# 3. Interaction vocabulary layers

OMVCS has two vocabulary layers.

## 3.1 Normative technical vocabulary

Used by:

```text
specifications
APIs
coding agents
conformance tests
developer tools
```

Examples:

```text
Revision
Line
Materialisation
Working State
Contribution
Storage Replica
```

---

## 3.2 Musician-facing vocabulary

Used by default UI.

Recommended mapping:

| Technical term | Default UI term |
|---|---|
| Project | Project |
| Revision | Version |
| Line | Creative direction / Version line |
| Working State | Current work |
| Materialise Revision | Open version |
| Publish Revision | Save version / Publish version |
| Contribution | Contribution |
| Component State | Component version |
| Project State | Version state |
| Resource Object | File / Resource |
| Storage Replica | Stored copy |
| Storage Endpoint | Storage location / Storage service |
| Reference Render | Reference mix |
| Creative Integration | Accept / Integrate contribution |
| Selective Integration | Accept selected parts |
| Release | Release |

Exact labels MAY vary by UI context, but semantic meaning MUST remain consistent.

---

# 4. Basic mental model

The normal user-facing model SHOULD communicate:

```text
Project
   |
   +-- Current work
   |
   +-- Version history
   |
   +-- Creative directions
   |
   +-- Contributions
   |
   +-- Releases
```

The user should understand:

> Versions preserve meaningful creative states.

> Current work is editable and not yet necessarily part of history.

> Creative directions let different ideas evolve independently.

> Contributions are other people's proposed work.

> Releases identify meaningful published states.

---

# 5. Project creation

The user action:

```text
Create Project
```

MUST create a new OMVCS Project identity.

The interface SHOULD ask only for information necessary at creation time.

Likely fields:

```text
Project name
optional description
initial storage choice
visibility
DAW association if applicable
```

---

# 6. New empty Project

A user MAY create an OMVCS Project before any DAW project exists.

The Project exists with:

```text
Project identity
metadata
no Revisions yet
```

The UI SHOULD present:

```text
No versions yet
```

rather than pretending an initial Revision exists.

---

# 7. Adopt existing DAW project

A common workflow is:

```text
I already have a song in Ardour.
Make it an Open Music Project.
```

The UI SHOULD offer an action such as:

```text
Add this project to Open Music
```

rather than:

```text
Initialize OMVCS repository
```

---

# 8. Existing DAW adoption workflow

Conceptually:

```text
Detect open DAW project
        |
Inspect project
        |
Find Resources
        |
Suggest Creative Components
        |
Choose storage
        |
Create OMVCS Project
        |
Create first Version
```

The user SHOULD be shown significant findings before publication.

---

# 9. Component suggestion

During adoption, the DAW Adapter MAY propose:

```text
Bass
Lead Vocal
Drums
Guitar
Synth
```

based on DAW structure.

The user MAY:

```text
accept
rename
combine
ignore
reclassify
```

suggested Components before first publication.

---

# 10. Do not expose internal IDs

The default adoption UI SHOULD NOT show:

```text
route UUID
Resource SHA-256
Adapter State hash
Chunk IDs
```

unless the user opens technical details.

---

# 11. Initial version publication

The user SHOULD explicitly confirm creation of the first historical Version.

Recommended action:

```text
Save first version
```

or:

```text
Create first version
```

This maps to:

```text
PublishRevision
```

---

# 12. Project home screen

The Project interface SHOULD make the following immediately visible:

```text
Project name
Current Version
Current Creative Direction
Working State status
latest Release
open Contributions
storage health
```

Not every detail must be shown simultaneously.

The main question should be:

> What am I working on, and is my work safely recorded?

---

# 13. Working State indicator

The interface MUST distinguish at least:

```text
No changes
Unpublished changes
Unknown change state
Conflict
Restore/materialisation incomplete
```

Recommended musician-facing messages:

```text
Up to date with Version 12

You have unpublished changes

Changes could not be fully checked

This project needs attention before it can be opened safely
```

---

# 14. Base Version visibility

The user SHOULD be able to see what Version their current work is based on.

Example:

```text
Current work
Based on Version 12
```

This becomes particularly important after divergent work.

---

# 15. Change summary

The default interface SHOULD show semantic changes where available.

Example:

```text
Changes since Version 12

Bass
    New recording

Lead Vocal
    Volume automation changed

Mix
    Room reverb changed

Project
    New cello track added
```

If only coarse information is available, the UI MUST say so.

Example:

```text
Project state changed
Detailed change information is unavailable for this DAW.
```

---

# 16. Change certainty

The UI MUST distinguish exact and inferred semantic information.

Possible presentation:

```text
Confirmed change
```

versus:

```text
Likely change
```

It MUST NOT present an inferred Adapter conclusion as certain.

---

# 17. Technical details view

Advanced users SHOULD be able to inspect:

```text
Revision ID
Project State ID
Resource IDs
Adapter State
Dependencies
storage replicas
provenance
```

without forcing those details into normal workflows.

---

# 18. Native Save versus Save Version

The interface MUST clearly distinguish:

```text
Save project
```

from:

```text
Save version
```

if both are exposed.

Native DAW Save means:

> preserve current editable project locally.

OMVCS Save Version means:

> create new immutable creative history.

---

# 19. Avoid overloaded Save

Where confusion is likely, the OMVCS action SHOULD use:

```text
Save version
```

or:

```text
Publish version
```

rather than simply:

```text
Save
```

---

# 20. Publish Version workflow

The user action:

```text
Save version
```

SHOULD begin with a review of detected changes.

Conceptually:

```text
Review changes
      |
Optional description
      |
Check dependencies/storage
      |
Capture state
      |
Store Resources
      |
Verify durability
      |
Publish Revision
```

---

# 21. Version description

The user SHOULD be encouraged, but not always required, to describe meaningful work.

Example:

```text
New bass take, softer vocal compression
```

This becomes the Revision message.

---

# 22. Automatic description assistance

The UI MAY suggest a description based on semantic changes.

Example:

```text
Suggested:
"New bass recording and vocal reverb adjustment"
```

The user remains free to edit it.

Generated text MUST NOT replace the actual structural change record.

---

# 23. Publication readiness

Before publication, the UI SHOULD summarize relevant readiness:

```text
3 changed Resources
All Resources stored safely
All required plugins available
Reference mix ready
```

or:

```text
1 Resource has no durable stored copy
```

---

# 24. Publication blocking conditions

If Core says publication cannot satisfy durability requirements, the UI MUST NOT show success.

It SHOULD explain the actionable reason.

Example:

```text
This version has not been saved to history yet.

Your project storage is full.

Choose another storage location or free some space.
```

---

# 25. Platform failure during publication

If the Revision is durably published but Platform sync fails, the UI MUST distinguish:

```text
Version saved
Open Music site sync pending
```

from:

```text
Version failed
```

The Platform is not part of the Revision durability boundary.

---

# 26. Publication progress

For large Resources, the UI SHOULD display meaningful phases.

Example:

```text
Preparing project
Uploading new audio
Verifying stored files
Saving version
Updating project page
```

Avoid exposing internal object-level noise by default.

---

# 27. Publication cancellation

If cancellation is possible, the UI MUST explain whether:

```text
current work remains safe
partially uploaded data may remain
no Version will be created
```

---

# 28. Publication success

A successful publication SHOULD result in a clear stable state:

```text
Version 13 saved
Current work matches Version 13
```

The human-facing Version number/label MAY be separate from the content-derived Revision Identifier.

---

# 29. Human-readable Version labels

The UI MAY present Revisions as:

```text
Version 13
Version 14
```

for convenience.

These are presentation labels.

They MUST NOT replace Revision identity.

---

# 30. History view

The Project SHOULD provide a History view.

At minimum it should show:

```text
Version
date
creator
description
Creative Direction
Release status
Contribution relationship where relevant
```

---

# 31. Non-linear history

If history diverges, the UI MUST not falsely pretend it is one simple sequence.

A simplified view MAY group by Creative Direction.

Example:

```text
Main
  Version 10
  Version 11
  Version 12

Acoustic direction
  Version 10
  Acoustic 1
  Acoustic 2
```

---

# 32. Detailed graph view

An advanced History view SHOULD allow visual inspection of the Revision Graph.

This is particularly useful for:

```text
parallel work
Contributions
integration
forks
```

---

# 33. Open historical Version

The user action SHOULD be:

```text
Open this version
```

The system maps it to Materialisation.

Before replacing local Current Work, the UI MUST check unpublished changes.

---

# 34. Opening with no local changes

If Current Work has no unpublished changes:

```text
Open Version 8
```

may proceed directly.

---

# 35. Opening with unpublished changes

If Current Work has changes, the UI MUST NOT silently discard them.

It SHOULD offer semantically clear choices such as:

```text
Save my current work as a version

Keep my current work safely and open the other version

Discard my current changes

Cancel
```

The exact mechanisms are Core/Adapter operations.

---

# 36. Safety checkpoint

The option:

```text
Keep my current work safely
```

MAY create an operational safety checkpoint without creating a public/published Version.

The UI MUST distinguish:

```text
temporary recovery checkpoint
```

from:

```text
saved Version
```

---

# 37. Historical version opening mode

A UI MAY offer:

```text
Open for listening
Open for editing
```

where Adapter capabilities allow.

This can avoid unnecessary Resource materialisation.

---

# 38. Listening without full restore

If Reference Render exists, the user SHOULD be able to audition a Version without reconstructing its full DAW state.

Action:

```text
Play version
```

rather than:

```text
Open version
```

---

# 39. Restore status

After opening a Version, the UI MUST accurately report whether it is:

```text
Fully restored
Restored with warnings
Partially restored
Reference mix only
Unavailable
```

---

# 40. Missing dependency presentation

Example:

```text
Version 7 opened with limitations

Missing:
SuperVerb 4.2

The original plugin settings are preserved, but this machine cannot reproduce them.

Reference mix available.
```

This is preferable to a generic error.

---

# 41. Resource unavailable presentation

Example:

```text
This version cannot be fully opened.

The original cello recording is currently unavailable.

The version still exists in history.
```

The UI MUST NOT imply that history is corrupt merely because storage is unavailable.

---

# 42. Corrupt replica handling

If one Replica is corrupt but another valid one exists, normal operation MAY continue.

The UI SHOULD avoid alarming the user unnecessarily.

It may show:

```text
One backup copy needs repair.
```

---

# 43. Current Working State after opening history

Opening Version 7 SHOULD result in:

```text
Current work
Based on Version 7
```

If the user edits it, new work branches naturally from Version 7.

---

# 44. Creative directions

The user-facing concept for a Line SHOULD be understandable musically.

Recommended labels:

```text
Creative direction
Alternate version
Version line
```

The UI MAY use:

```text
Branch
```

in an advanced/developer mode.

---

# 45. Create Creative Direction

User action:

```text
Create alternate direction
```

or:

```text
Start new direction from this version
```

maps to Line creation.

---

# 46. Creative Direction naming

Users SHOULD name directions descriptively.

Examples:

```text
Acoustic
Dark arrangement
Radio edit
Anna's remix
Live version
```

The name is mutable operational metadata.

---

# 47. Switching Creative Direction

Action:

```text
Switch to Acoustic
```

will ordinarily Materialise the head Revision of that Line.

Unpublished Working State safety rules still apply.

---

# 48. Creative Direction creation does not duplicate Resources

The UI SHOULD avoid implying that creating another direction copies the entire Project.

It creates another historical path.

The storage system deduplicates immutable Resources naturally.

---

# 49. Creative Direction deletion

Deleting a direction MUST disclose:

```text
This removes the named direction.
Its saved versions are not immediately erased.
```

The UI MUST NOT imply destructive content deletion unless garbage collection is separately requested and safe.

---

# 50. Custom Working State

Users SHOULD be able to experiment with combinations from history.

A user-facing action may be:

```text
Try another component version
```

or:

```text
Build a version from history
```

---

# 51. Component History

For a Creative Component such as Bass, the UI SHOULD provide:

```text
Bass history

B1 — Original recording
B2 — Tighter take
B3 — Fretless version
B4 — Anna's contribution
```

---

# 52. Try Component Version

Action:

```text
Use Bass B2
```

may invoke Component Materialisation if supported by the Adapter.

The UI MUST validate compatibility before claiming success.

---

# 53. Component comparison

The UI MAY offer:

```text
Listen to B2
Listen to B4
Use B2 in current work
```

Auditioning need not materialise the full Project.

---

# 54. Custom combination status

If the user combines historical Component States not previously published together, the UI SHOULD indicate:

```text
Custom combination
Not yet saved as a version
```

---

# 55. Publishing custom combination

When saved, the system creates a normal new Revision.

The UI SHOULD preserve provenance automatically.

The user should not need to manually reconstruct where each Component came from.

---

# 56. Provenance display

A Component MAY display:

```text
Cello
Contributed by Anna
Originally from Contribution C17
Integrated in Version 22
```

Provenance presentation MUST NOT imply legal ownership beyond known facts.

---

# 57. Contribution discovery

On a Platform, users may see:

```text
This project welcomes contributions
```

with requested areas:

```text
Cello
Vocals
Mixing
Remix
```

---

# 58. Start Contribution

The musician-facing action SHOULD be:

```text
Contribute to this project
```

rather than:

```text
Fork repository and open pull request
```

---

# 59. Contribution setup

The UI SHOULD ask:

```text
What would you like to contribute?
```

Possible responses:

```text
New part
Replacement part
Alternative arrangement
Remix
Mix change
Something else
```

This becomes Contribution Intent.

---

# 60. Contribution Base selection

By default, a Contribution SHOULD start from the current target Version/Line.

Advanced users MAY choose another Base.

The selected Base MUST be visible.

---

# 61. Contribution Working State

The contributor should receive a normal editable Working State.

The UI MAY indicate:

```text
You are working on a contribution to Project X
Based on Version 18
```

---

# 62. Contribution storage

The contributor SHOULD be able to use their own configured storage.

The UI SHOULD not imply that contribution files must be handed over to the Project owner immediately.

---

# 63. Contribution privacy

Before opening/submitting a Contribution, the UI SHOULD explain its intended visibility where relevant.

Example:

```text
Visible to project collaborators
```

or:

```text
Public contribution
```

---

# 64. Submit Contribution

Recommended action:

```text
Submit contribution
```

This requires actual Revision history to exist.

If contributor Working State is unpublished, the UI SHOULD first create a Version.

---

# 65. Contribution summary

The submission UI SHOULD summarize:

```text
Based on Version 18

Changes:
    New cello Component
    Drum arrangement changed

Intent:
    Add cello arrangement

Storage:
    Resources available
```

---

# 66. Contribution discussion

The user MAY discuss the Contribution without modifying creative history.

Comments SHOULD be visually distinct from Version history.

---

# 67. Update Contribution

A contributor MAY continue working.

UI action:

```text
Update contribution
```

moves its Head to newer contributed Revision history.

---

# 68. Reviewer experience

A reviewer SHOULD be able to:

```text
play Reference Render
inspect changes
inspect Components
compare with target Version
open contribution in DAW
comment
accept all
accept selected parts
close
```

subject to Adapter capabilities.

---

# 69. Safe review

The UI SHOULD permit review of:

```text
metadata
Reference Render
change summary
```

without immediately loading executable plugin/script state from an untrusted Contribution.

---

# 70. Contribution comparison

A Contribution review SHOULD distinguish:

```text
Target
Contribution
```

and show differences.

Example:

```text
Cello
    Added

Drums
    Different arrangement

Vocals
    Unchanged
```

---

# 71. Semantic limitations

If semantic diff is unavailable:

```text
Detailed DAW-level comparison unavailable.
2 project Resources and DAW state changed.
```

The UI MUST not invent a rich comparison.

---

# 72. Accept entire Contribution

Action:

```text
Accept contribution
```

SHOULD build and validate an Integration Working State first.

It MUST NOT create a historical result before Adapter/Core validation.

---

# 73. Accept selected parts

If supported, the UI SHOULD allow:

```text
Accept cello
Keep current drums
```

This maps to Selective Integration.

---

# 74. Integration Plan presentation

Before applying a complex selective integration, the UI MAY show:

```text
Will use:
    Anna's cello

Will keep:
    Current drums
    Current vocals

Also required:
    Anna's cello bus routing

No conflicts found
```

---

# 75. Integration conflict presentation

Conflict messages SHOULD describe creative meaning where possible.

Bad:

```text
Object merge failed.
```

Better:

```text
The contributed cello was arranged against a different tempo map.

Current project: 120 BPM
Contribution: 95 BPM

The cello cannot be inserted safely without choosing which tempo structure to use.
```

---

# 76. Unknown conflict

If Adapter cannot explain:

```text
These project states cannot be safely combined automatically.

Open both versions and resolve the difference manually.
```

This is preferable to guessing.

---

# 77. Integration does not immediately imply publication

Applying an Integration Plan creates Working State.

The user SHOULD be able to inspect/listen before saving the new Version.

---

# 78. Publish Integration

Once satisfied:

```text
Save integrated version
```

creates the Integration Revision.

The resulting provenance is handled automatically.

---

# 79. Contribution status after integration

The UI SHOULD show:

```text
Integrated in Version 23
```

or:

```text
Partially integrated in Version 23
```

rather than merely:

```text
Closed
```

---

# 80. Close Contribution without integration

User action:

```text
Close contribution
```

should clearly mean:

> Stop considering this proposal.

It does not delete contributor history.

---

# 81. Release workflow

A user action:

```text
Create Release
```

identifies a meaningful immutable Version.

The UI SHOULD require selecting:

```text
Version
Release name
optional description
visibility
```

---

# 82. Release immutability

The UI MUST communicate that the target Version cannot later be silently changed.

Example:

```text
Release 1.0 will permanently refer to Version 28.
```

---

# 83. Fixing a Release

If an error is found after Release 1.0, the user creates another Version and Release:

```text
1.0.1
```

or another chosen name.

The UI MUST NOT offer:

```text
replace contents of Release 1.0
```

as ordinary behaviour.

---

# 84. Reference mix for Release

If a Reference Render is missing and Project policy recommends or requires it, the UI SHOULD prompt:

```text
Create reference mix
```

before completing Release.

---

# 85. Release readiness

The UI SHOULD show:

```text
All Resources available
Reference mix available
2 required external plugins
1 storage copy per Resource
```

and any durability warnings.

---

# 86. Release durability warning

If Releases are configured to require stronger replication:

```text
This Release has only one stored copy of the master audio.
Create another backup copy before releasing?
```

---

# 87. Release page

A Release view SHOULD show:

```text
Release name
Version
Reference Render
creator
date
description
provenance/credits
reproducibility status
licence where applicable
```

---

# 88. Storage overview

Users need a simple Storage view.

Recommended conceptual presentation:

```text
Project Storage

Main storage        Healthy
Backup storage      Healthy
Anna's contribution Available
Local cache         12 GB

Project safety:
All published Resources have at least 1 verified copy
```

---

# 89. Storage terminology

The normal UI SHOULD use:

```text
Storage location
Stored copy
Backup copy
```

rather than:

```text
Endpoint
Replica
```

unless in advanced mode.

---

# 90. Storage health summary

The user SHOULD see overall status such as:

```text
Healthy
Needs attention
Some files unavailable
Storage authentication required
Backup policy not satisfied
```

---

# 91. Add storage location

Action:

```text
Add storage
```

may guide users through:

```text
Local folder
S3-compatible storage
WebDAV
OneDrive
Google Drive
other supported provider
```

The exact supported set depends on Storage Adapters.

---

# 92. Storage authentication

Authentication SHOULD be handled as a provider connection step.

The UI MUST NOT expose secrets unnecessarily.

Example:

```text
Connect OneDrive
```

not:

```text
Paste OAuth refresh token
```

for normal users.

---

# 93. Storage capabilities

If a provider cannot support Repository Home but can store Resources, the UI should describe the consequence clearly.

Example:

```text
This location can store project files but cannot host the project's main metadata.
```

---

# 94. Move Resource storage

The user may want:

```text
Move my bass recordings from Server A to Server B.
```

The UI action SHOULD be:

```text
Move storage
```

or:

```text
Move selected files to...
```

---

# 95. Storage migration preview

Before migration, show:

```text
Moving:
    14 bass Resources
    8.3 GB

From:
    Server A

To:
    Server B

Creative history will not change.
```

That final line reinforces the architecture.

---

# 96. Migration execution

UI progress SHOULD correspond to:

```text
Copying
Verifying
Updating storage records
Removing old copies
```

It SHOULD NOT display:

```text
Creating new version
```

because none occurs.

---

# 97. Verification before source deletion

The UI MUST NOT report successful move before destination verification.

If verification fails:

```text
Move failed.

The original files remain safely stored on Server A.
```

---

# 98. Keep both copies

Users SHOULD be able to choose:

```text
Copy to B and keep A
```

which is replication rather than migration.

Default wording:

```text
Create another backup copy
```

may be more understandable.

---

# 99. Remove storage copy

Before deleting a Replica, the UI MUST determine whether another valid copy exists and whether policy permits removal.

If it is the last valid copy:

```text
This is the only verified copy of 23 Resources.

Removing it would make those historical versions unavailable.
```

The UI SHOULD block or require exceptional confirmation according to Core policy.

---

# 100. Storage location disappears

If provider access fails:

```text
Main storage needs sign-in
```

should be shown before:

```text
Files missing
```

when absence has not been established.

---

# 101. Authentication expiry flow

Recommended:

```text
Storage connection expired

Reconnect
```

After successful authentication, OMVCS probes availability again.

---

# 102. Degraded storage

Example:

```text
Project still works, but your backup policy is no longer satisfied.

14 Resources have only one verified copy.
```

This distinguishes availability from durability.

---

# 103. Missing Resource

The interface SHOULD identify:

```text
what is missing
which Versions are affected
whether another copy might exist
```

Example:

```text
Cello Take 3 is unavailable.

Affected:
Version 18
Version 19
Release 1.0

Last known storage:
Anna's OneDrive
```

---

# 104. Find another copy

The UI SHOULD offer:

```text
Look for another copy
```

where recovery scanning is possible.

---

# 105. Repair Replica

If another valid Replica exists:

```text
Repair missing backup
```

may recreate it.

This MUST NOT appear as a new creative Version.

---

# 106. Repository Home view

The user need not normally understand `Repository Home`.

The default UI might present:

```text
Project metadata location
```

or:

```text
Main project storage
```

with technical details available.

---

# 107. Repository Home migration

User action:

```text
Move project metadata
```

or:

```text
Change main project storage
```

SHOULD explain:

```text
Your versions and IDs will stay exactly the same.
```

---

# 108. Platform view

The user SHOULD understand the distinction between:

```text
Project itself
```

and:

```text
Open Music site
```

The UI may present:

```text
Published on:
Open Music Reference Platform
```

as one external presentation channel.

---

# 109. Disconnect Platform

Action:

```text
Remove from Open Music site
```

MUST explain:

```text
This removes the public/collaborative page.
Your Project, history and stored files remain intact.
```

---

# 110. Move to another Platform

Workflow:

```text
Connect new Platform
        |
Register same Project
        |
Sync metadata
        |
Verify Project page
        |
Optionally remove old Platform mirror
```

No new Project should be created.

---

# 111. Multi-Platform presentation

Advanced users MAY see:

```text
Project is mirrored on:
Platform A
Platform B
```

Each Mirror may have its own sync status.

---

# 112. Platform sync status

Recommended statuses:

```text
Up to date
Sync pending
Platform unavailable
Permission required
Mirror conflict
```

---

# 113. Platform conflict

If Platform Mirror disagrees with Repository:

```text
The Open Music site has outdated project information.

Project history itself is safe.

Resynchronise
```

This avoids alarming the user about creative history.

---

# 114. Project visibility

Visibility controls SHOULD use familiar language:

```text
Private
Unlisted
Public
```

The UI SHOULD explain consequences.

---

# 115. Resource visibility

Project visibility MUST NOT automatically imply all Resources are downloadable.

The UI SHOULD allow separate policy such as:

```text
Reference mix: Public
Stems: Contributors
DAW project: Collaborators
```

---

# 116. Contribution permissions

The Project UI SHOULD make clear:

```text
Who can contribute?
Anyone
Approved users
Invite only
Nobody
```

subject to Platform permission semantics.

---

# 117. Collaboration roles

Musician-facing roles MAY include:

```text
Listener
Contributor
Reviewer
Project steward
```

The interface should provide an advanced view of actual permissions where needed.

---

# 118. Stewardship language

The UI SHOULD avoid implying that `Project Steward` means sole creative owner.

It is primarily a coordination role.

---

# 119. Permission changes

When access is revoked, the UI SHOULD distinguish:

```text
Cannot make future changes
```

from:

```text
Past contributions remain credited in history
```

---

# 120. Account removal

If a contributor's Platform account disappears, historical attribution SHOULD remain visible.

Possible presentation:

```text
Anna
Account no longer linked to this Platform
```

or:

```text
Unlinked historical contributor
```

depending on privacy/identity information available.

---

# 121. Offline mode

Users SHOULD be able to continue working without Platform connectivity.

The UI SHOULD distinguish:

```text
Working offline
```

from:

```text
Project broken
```

---

# 122. Offline publication

If Repository Home/storage is reachable but Platform is not:

```text
Version saved
Site sync pending
```

If durable storage is also unavailable:

```text
Your work is saved locally but has not yet been published as a durable version.
```

---

# 123. Local-only checkpoint

The UI MAY allow:

```text
Keep a local checkpoint
```

for temporary safety.

It MUST not call this a published Version if it lacks Core durability requirements.

---

# 124. Sync later

Offline changes SHOULD be resumable.

The user should not need to repeat publication manually if the operation can safely continue.

---

# 125. Divergent work

Suppose two collaborators independently continue from Version 12.

The UI MUST not silently overwrite.

It SHOULD say:

```text
Two creative directions have continued from Version 12.

Your work: Version 13A
Remote work: Version 13B
```

---

# 126. Divergence choices

Possible actions:

```text
Keep both as separate directions
Combine them
Open the other version
Abandon my unpublished/local direction
```

The UI should avoid technical `non-fast-forward` terminology by default.

---

# 127. Automatic safe combination

If DAW Adapter determines changes can combine safely, the UI MAY offer:

```text
Combine automatically
```

but SHOULD still summarize the planned result.

---

# 128. Conflict-first design

When automatic combination is unsafe, OMVCS should preserve both histories and ask the user rather than choosing.

The interface should communicate:

> Nothing has been lost. We need to decide how these changes should fit together.

---

# 129. Recovery centre

The application SHOULD provide a coherent recovery area rather than scattering recovery operations.

Possible title:

```text
Project Recovery
```

It may include:

```text
Recover after computer loss
Reconnect storage
Find missing Resources
Restore Repository Home
Repair backups
Import archive
```

---

# 130. Recover after local machine loss

On a new machine:

```text
Open existing Open Music Project
```

should allow using:

```text
Repository Home
Portable Repository export
known Project identity + authorized storage
```

---

# 131. Recovery workflow

Conceptually:

```text
Locate Project metadata
        |
Validate history
        |
Connect required storage
        |
Check Resource availability
        |
Choose Version
        |
Materialise
```

---

# 132. Platform not required for recovery

The UI MUST support Project recovery without logging into the original Platform when Repository data is otherwise available.

---

# 133. Repository archive import

Action:

```text
Import Project Archive
```

SHOULD preserve Project identity by default.

---

# 134. Fork on import

If the user wants an independent derivative:

```text
Import as new Project
```

may create a new Project ID and `forked_from` provenance.

The default must not accidentally fork.

---

# 135. Export workflow

Users SHOULD have:

```text
Export Project
```

with clear modes.

---

# 136. Metadata-only export

Description:

```text
Project history and metadata only.
Large media files remain in connected storage.
```

---

# 137. Archival export

Description:

```text
Project history plus selected/all Resource files.
Designed for offline recovery.
```

---

# 138. Export completeness

The UI SHOULD show:

```text
Complete archive
```

only if all required selected Resources were successfully included and verified.

---

# 139. Missing Resource during export

Example:

```text
Archive incomplete

2 historical Resources could not be retrieved.

You can:
Retry
Export without them
Cancel
```

If continuing, the result MUST be labelled incomplete.

---

# 140. Project portability message

The UI SHOULD reinforce:

```text
This Project is portable.
You can open it with another conforming OMVCS implementation.
```

This is philosophically important but should be factual, not marketing exaggeration.

---

# 141. DAW compatibility screen

The user SHOULD be able to inspect:

```text
Native DAW
Adapter
Adapter State version
Reproducibility
Dependencies
Portable Components
```

---

# 142. Foreign DAW Project opening

If an Ardour user opens a Project last published through Ableton:

```text
This version was created in Ableton Live.

Ardour cannot recreate the complete Ableton project state.

Available:
Reference mix
Audio Components
MIDI Components

You may import compatible parts into a new Ardour working state.
```

---

# 143. Cross-DAW conversion warning

Any conversion MUST state known losses before publication.

Example:

```text
The following cannot be transferred:
Ableton device chains
2 automation lanes

Audio and MIDI will be preserved.
```

---

# 144. Cross-DAW conversion creates new state

The UI SHOULD explain that conversion:

```text
does not change old versions
```

and:

```text
creates a new working state
```

which becomes history only if saved.

---

# 145. Dependency manager

The UI MAY provide a dependency overview:

```text
Available
Missing
Different version
Unknown
```

grouped by Revision or current Working State.

---

# 146. Missing optional dependency

Example:

```text
Optional analyser plugin missing.
The project can still be reproduced exactly.
```

if Adapter semantics support that conclusion.

---

# 147. Missing required dependency

Example:

```text
Required synth missing.
The MIDI and synth settings are preserved, but the original sound cannot currently be recreated.
```

---

# 148. Reference mix comparison

Where reproducibility is partial, the UI MAY let the user:

```text
Play original reference mix
Play current reconstructed mix
```

This could be very useful for diagnosing missing dependencies.

---

# 149. Reproducibility status

Recommended user-facing categories:

```text
Exact
Exact with external tools
Partial
Reference mix only
Unavailable
Unknown
```

The UI SHOULD provide explanation rather than only an icon.

---

# 150. Security and untrusted Projects

Opening another user's DAW state may activate:

```text
plugins
scripts
external devices
```

The UI SHOULD offer safe review where supported.

---

# 151. Safe inspection

Possible action:

```text
Inspect safely
```

Meaning:

```text
do not activate executable dependencies
use Reference Render and metadata first
```

---

# 152. Trust prompt

When activation is required:

```text
This Project contains plugin/script state from another user.

Open with external components disabled
Open normally
Cancel
```

The exact choice depends on DAW capabilities.

---

# 153. Secrets

The UI MUST never display permanent credentials in ordinary Project views.

Access tokens, OAuth secrets and signed URLs SHOULD be hidden/redacted.

---

# 154. Error design

Errors SHOULD answer four questions:

```text
What happened?
Is my creative work safe?
What is affected?
What can I do next?
```

---

# 155. Example error

Bad:

```text
500 storage error
```

Better:

```text
Version not published

The new bass recording could not be stored because your S3 account is temporarily unavailable.

Your recording is still safe in your current working project.

Reconnect storage and try again.
```

---

# 156. Integrity failure

A serious integrity error SHOULD be explicit.

Example:

```text
Stored copy failed verification

The file retrieved from Server B does not match the version recorded in history.

A valid copy still exists on Server A.

OMVCS will not use the corrupted copy.
```

---

# 157. Unknown integrity condition

If verification is incomplete:

```text
This copy has not yet been fully verified.
```

not:

```text
File is safe
```

---

# 158. User cancellation

Cancellation should describe what remains.

Example:

```text
Move cancelled.

Server A still contains the original files.
3 partially transferred files on Server B will be cleaned up later.
```

---

# 159. Long-running operations

The UI SHOULD expose meaningful progress without drowning users in file-level details.

Default:

```text
Uploading new recordings
6.4 GB of 12.1 GB
```

Advanced details MAY show individual Resources.

---

# 160. Background operations

Safe operations such as:

```text
replication
verification
Platform sync
cache download
```

MAY continue in background.

The user should be able to inspect status.

---

# 161. Recording/performance priority

Background OMVCS work SHOULD avoid affecting active recording/playback.

The UI MAY pause transfers automatically during recording.

Example:

```text
Storage sync paused while recording
```

---

# 162. Bandwidth control

Users MAY configure:

```text
Unlimited
Limit background transfers
Pause on metered connection
Manual only
```

These are operational preferences.

---

# 163. Local cache

The UI MAY expose:

```text
Local cache size
Clear unused cache
Keep this version offline
```

Clearing cache MUST explain:

```text
Published history will not be deleted.
Files can be downloaded again from Project storage.
```

---

# 164. Keep offline

Action:

```text
Keep this version available offline
```

should pin required Resource Objects locally.

This does not create another Version.

---

# 165. Cache eviction warning

If a Resource has no durable remote copy, it is not merely cache.

The UI MUST NOT offer to delete it as disposable cache.

---

# 166. Project status model

A Project SHOULD have independent status dimensions rather than one ambiguous green/red state.

Possible dimensions:

```text
Creative state
Storage safety
Platform sync
Reproducibility
Collaboration
```

---

# 167. Example Project status

```text
Creative:
2 unpublished changes

Storage:
Healthy

Platform:
1 update pending

Reproducibility:
Exact

Contributions:
2 open
```

This is much more informative than:

```text
Project: warning
```

---

# 168. Notification priorities

The UI SHOULD distinguish:

```text
informational
action recommended
action required
critical data risk
```

Storage durability failure deserves higher priority than social notification.

---

# 169. Critical warning

Example:

```text
Data at risk

23 published audio Resources currently have only one known copy.

Create another backup.
```

---

# 170. Non-critical warning

Example:

```text
Open Music site is temporarily unavailable.
Your Project and saved versions are unaffected.
```

---

# 171. Undo model

Not every OMVCS operation should be described as `undo`.

For immutable history:

```text
Create a new version based on an older one
```

is often more accurate than:

```text
Undo Version 12
```

---

# 172. Revert workflow

A user MAY choose:

```text
Make Version 8 the current direction
```

or:

```text
Create new version using Version 8's state
```

The UI must distinguish pointer movement from creating new history.

---

# 173. Line head rollback

If a user intentionally moves a Line to an earlier Revision, the UI MUST warn that later Versions will no longer be the Line head but remain in history.

---

# 174. No hidden force push

The interface MUST NOT provide an innocuous action that destroys or hides divergence equivalent to Git force-push without clear explanation.

---

# 175. Deletion model

The UI must distinguish:

```text
Remove from current version
Remove creative direction
Remove public page
Remove stored copy
Delete local cache
Delete Project
```

These are different operations.

---

# 176. Delete Project

A full Project deletion request is complex.

The UI SHOULD identify separately:

```text
Platform Mirror
Repository metadata
Resource storage
local Working State
shared Resources
```

It MUST not imply that one button necessarily erases all distributed copies.

---

# 177. Shared Resource warning

A physical Resource shared by another Project MUST NOT be deleted just because one Project is deleted.

This should usually remain invisible unless relevant.

---

# 178. Garbage collection

Ordinary users SHOULD not need to understand reachability graphs.

A UI action may be:

```text
Clean unused project storage
```

---

# 179. Garbage collection preview

Before deleting substantial data, show:

```text
Unused storage that can be safely removed:
8.2 GB

Published versions and Releases will not be affected.
```

---

# 180. Uncertain garbage collection

If safety cannot be proven:

```text
OMVCS cannot confirm these files are unused.
They will not be removed.
```

This matches the conservative Core policy.

---

# 181. Automatic garbage collection

Automatic cleanup MAY remove:

```text
expired temporary uploads
old cache files
safe orphan data after retention
```

according to policy.

It MUST NOT silently delete reachable historical content.

---

# 182. Search in Project history

Users SHOULD be able to search by:

```text
Version description
Component
creator
Contribution
Release
date
```

where metadata permits.

---

# 183. Search by musical Component

Example:

```text
Show me versions where Bass changed
```

This is a valuable OMVCS-specific history interaction.

---

# 184. Compare versions

The UI SHOULD support:

```text
Compare Version 12 with Version 16
```

Results MAY include:

```text
Components changed
Resources changed
semantic DAW differences
dependencies changed
Reference Render comparison
```

according to Adapter capability.

---

# 185. Compare arbitrary directions

Users SHOULD be able to compare:

```text
Main Version 20
```

with:

```text
Acoustic Version 7
```

even if neither directly descends from the other.

---

# 186. Compare Components

For Component history:

```text
Compare Bass B4 and B7
```

may offer:

```text
metadata
waveform
audio audition
source provenance
```

---

# 187. Project graph complexity

The UI SHOULD progressively reveal complexity.

Default:

```text
simple history
```

Advanced:

```text
graph view
Component lineage
storage topology
provenance graph
```

---

# 188. Storage topology view

Advanced users MAY see:

```text
Resource A
  -> Server A
  -> Server B

Resource B
  -> Anna OneDrive
```

This is valuable for diagnosis but should not be necessary for everyday use.

---

# 189. Provenance graph

A Project MAY expose a visual graph of:

```text
forks
Contributions
derived Components
integrations
```

This should reflect real structured provenance.

---

# 190. Project credits

The UI MAY derive a credits view from provenance.

It MUST distinguish:

```text
created
contributed
integrated
derived
```

It MUST NOT invent ownership percentages.

---

# 191. Attribution persistence

If a contributor's Platform account changes or disappears, Project credits SHOULD remain grounded in historical Actor/Provenance data.

---

# 192. Accessibility

Core OMVCS workflows SHOULD be usable without relying solely on:

```text
colour
drag-and-drop
hover
visual graph interpretation
```

Alternative textual navigation SHOULD exist.

---

# 193. Keyboard access

Desktop interfaces SHOULD support keyboard operation for major actions.

This is especially important in DAW-integrated workflows.

---

# 194. Screen-reader semantics

Important states such as:

```text
unpublished changes
storage failure
integration conflict
missing Resource
```

SHOULD have explicit accessible labels.

---

# 195. No colour-only status

For example:

```text
green dot
red dot
```

without text is insufficient for critical state.

---

# 196. Confirmation design

Confirmation prompts SHOULD be reserved for meaningful destructive/risky actions.

Over-confirmation trains users to ignore warnings.

---

# 197. Low-risk operations

Actions such as:

```text
play Reference Render
open history view
inspect Contribution
add backup replica
```

should not need unnecessary confirmation.

---

# 198. High-risk operations

Actions such as:

```text
discard unpublished work
remove final verified Resource copy
delete Repository Home
remove Project storage
```

require explicit confirmation.

---

# 199. Context preservation

After completing an OMVCS operation, the user SHOULD remain in a sensible creative context.

Example:

After Save Version:

```text
return to DAW / Current Work
```

rather than sending user into technical repository administration.

---

# 200. Embedded DAW UI

A DAW-integrated OMVCS interface SHOULD prioritise:

```text
Current work
Save version
History
Compare
Contribute
Review
```

Storage administration may live in a separate Project settings area.

---

# 201. Standalone Open Music client

A standalone client MAY provide broader functionality:

```text
Project discovery
history
storage
recovery
Platform management
Contribution review
```

using the same underlying semantics.

---

# 202. Web Platform UI

The Platform web UI SHOULD prioritise:

```text
listen
discover
history
Contributions
discussion
Release presentation
```

and delegate deep DAW editing to compatible clients.

---

# 203. CLI

A CLI MAY exist for:

```text
developers
automation
advanced users
testing
coding agents
```

The existence of a CLI MUST NOT mean GUI users need to understand it.

---

# 204. CLI terminology

The CLI MAY use normative OMVCS technical language.

Example:

```text
omvcs revision publish
omvcs line create
omvcs materialise
```

It SHOULD avoid pretending to be Git if semantics differ.

---

# 205. Coding-agent interaction

Coding agents interacting with OMVCS MUST use normative API semantics rather than guessing from UI labels.

Example:

UI:

```text
Save version
```

Agent operation:

```text
PublishRevision
```

---

# 206. Automation and agents

Agent-driven actions that may alter Project history MUST observe the same authorization and validation requirements as human actions.

---

# 207. Agent confirmation

If an action requires explicit human approval due to destructiveness, an agent MUST not bypass that boundary.

---

# 208. Non-interactive ambiguity

When run non-interactively, unresolved ambiguity MUST return structured failure/decision-required status.

It MUST not choose arbitrarily.

---

# 209. Interaction operation mapping

Every major UI action MUST map to explicit OMVCS operations.

Examples:

| User action | Core/Adapter operation |
|---|---|
| Create Project | CreateProject |
| Add existing DAW project | AdoptProject + CreateProject |
| Save version | CaptureState + PublishRevision |
| Open version | MaterialiseRevision + RestoreState |
| Try old bass | MaterialiseComponent |
| Start alternate direction | CreateLine |
| Submit contribution | PublishRevision + CreateContribution |
| Accept contribution | Integration Plan + PublishRevision |
| Create release | CreateRelease |
| Add backup copy | AddReplica |
| Move storage | MigrateReplica |
| Reconnect storage | OpenEndpoint/Auth flow |
| Export Project | ExportRepository |
| Recover Project | RecoverRepository |
| Move to another Platform | RegisterProjectMirror |

This mapping MUST remain deterministic.

---

# 210. No undefined UI magic

If a UI feature cannot be described in terms of Core/Adapter/Platform operations, its architecture is incomplete.

Coding agents MUST NOT implement hidden direct mutations merely to make a UI button work.

---

# 211. Workflow: everyday editing

Typical flow:

```text
Open Project
    |
Edit in DAW
    |
OMVCS detects changes
    |
Save version
    |
Continue editing
```

The user should not need to think about storage unless something needs attention.

---

# 212. Workflow: alternate idea

```text
Current Version 12
    |
Start alternate direction
    |
"Acoustic"
    |
Edit
    |
Save Version A1
```

Main remains untouched.

---

# 213. Workflow: revisit old idea

```text
History
    |
Version 6
    |
Play
    |
Open
    |
Start new direction from this version
```

No history rewrite occurs.

---

# 214. Workflow: swap one component

```text
Current work
    |
Bass history
    |
Choose B4
    |
Preview compatibility
    |
Use B4
    |
Custom Working State
    |
Save version if desired
```

---

# 215. Workflow: contribution

```text
Discover Project
    |
Contribute
    |
Choose intent
    |
Materialise Base Version
    |
Work in DAW
    |
Save contributor Version
    |
Submit Contribution
```

---

# 216. Workflow: selective integration

```text
Open Contribution
    |
Compare changes
    |
Select Cello
    |
Build Integration Plan
    |
Resolve dependencies/conflicts
    |
Apply to Current Work
    |
Listen/edit
    |
Save integrated Version
```

---

# 217. Workflow: release

```text
Choose Version
    |
Check Reference Render
    |
Check storage durability
    |
Create Release
    |
Publish Platform metadata
```

---

# 218. Workflow: move storage

```text
Storage settings
    |
Select Server A
    |
Move Resources
    |
Choose Server B
    |
Copy
    |
Verify
    |
Remove old copies
```

No Version created.

---

# 219. Workflow: recover after computer loss

```text
Install OMVCS client
    |
Open existing Project
    |
Connect Repository Home
    |
Connect required storage
    |
Validate history
    |
Select current Version
    |
Materialise
```

---

# 220. Workflow: Platform disappears

```text
Project continues locally
    |
Choose new Platform
    |
Register existing Project
    |
Sync Mirror
```

No history migration is necessary.

---

# 221. Workflow: missing provider

```text
Project opens with warning
    |
Provider inaccessible
    |
Reconnect
or
Find another Replica
or
Restore from archive
```

---

# 222. Workflow: missing plugin

```text
Open Version
    |
Dependency check
    |
Plugin missing
    |
Open partially
    |
Play Reference Render
    |
Optionally install/replace dependency
```

The original historical state remains intact.

---

# 223. Workflow: Project fork

User action:

```text
Create my own Project from this version
```

maps to:

```text
ForkProject
```

The UI SHOULD state:

```text
A new Project will be created.
Its history will remember where it came from.
```

---

# 224. Fork versus Contribution

The UI SHOULD help users understand:

```text
Contribute
```

means:

> propose work back to this Project.

```text
Create my own Project
```

means:

> create an independent Project with preserved provenance.

---

# 225. Fork workflow

```text
Choose source Revision
    |
Create new Project identity
    |
Preserve fork provenance
    |
Choose storage
    |
Start Working State
```

---

# 226. Licensing visibility

Where licence metadata exists, users SHOULD see it before:

```text
forking
reusing
downloading
contributing
```

The UI MUST not infer permissions absent from actual licence metadata.

---

# 227. Rights uncertainty

If OMVCS knows provenance but no licence:

```text
The origin of this work is known, but reuse permission is not specified.
```

This is better than guessing.

---

# 228. Public Project without reuse licence

Public visibility MUST NOT be interpreted automatically as permission to reuse.

The UI should keep:

```text
Can view
```

and:

```text
Can reuse
```

separate.

---

# 229. Project notices

The Project may display notices such as:

```text
Experimental Project
Reference implementation
No longer actively maintained
Contributions welcome
```

These are presentation metadata.

---

# 230. Reference implementation warning

For the Open Music reference Platform/client, the UI SHOULD make clear:

```text
This service is experimental and is not intended to be the only place your Project survives.
```

This directly supports the anti-lock-in philosophy.

---

# 231. First-run onboarding

Onboarding SHOULD communicate only the essential mental model.

Possible message:

> Open Music keeps the history of your music while letting you store the actual project files where you choose.

> Save meaningful versions, explore alternate directions, and collaborate without giving one platform control of your work.

Avoid explaining hashes, DAGs and replica topology initially.

---

# 232. First storage setup

The UI SHOULD explain:

```text
Your project files stay in storage you control.

Choose where Open Music should keep them.
```

---

# 233. First Platform setup

Separately:

```text
Connect to an Open Music Platform if you want discovery and collaboration.
```

This reinforces that Platform connection is optional infrastructure.

---

# 234. Advanced mode

An advanced mode MAY expose:

```text
Revision IDs
Line generations
Adapter capability status
Storage Replica topology
verification timestamps
Repository Home
Operation Log
```

---

# 235. Diagnostic mode

A diagnostic view SHOULD help support/debugging without requiring raw database access.

Possible information:

```text
Project ID
current Revision
Adapter
Storage Endpoint status
Platform sync generation
last failed operation
integrity status
```

Secrets must remain hidden.

---

# 236. Copy diagnostic report

Users MAY create a diagnostic report suitable for bug reports.

It MUST redact:

```text
credentials
private access URLs
tokens
unnecessary local paths
```

---

# 237. Operation history

The UI MAY expose Repository Operation Log events such as:

```text
Storage moved
Backup added
Platform synced
Line moved
```

This SHOULD be visually separate from creative Version history.

---

# 238. Why separate histories matter

A user should not see:

```text
Version 18
Version 19
Moved S3 bucket
Version 20
Reauthenticated OneDrive
```

as one creative timeline.

Infrastructure activity belongs elsewhere.

---

# 239. Creative timeline

Creative timeline should contain:

```text
Versions
Contributions
Integrations
Releases
forks
```

not routine storage operations.

---

# 240. Infrastructure timeline

Advanced view may show:

```text
Replica added
Replica removed
Storage migrated
Repository Home changed
Platform synced
```

---

# 241. Recoverable failure

Errors that can be retried SHOULD retain enough operation state for:

```text
Resume
```

rather than forcing restart.

---

# 242. Resume transfer

Example:

```text
Storage move interrupted at 62%.

Resume
Start over
Cancel
```

If provider/Adapter supports resumption.

---

# 243. Non-resumable transfer

If operation cannot resume:

```text
This provider cannot resume interrupted uploads.
The transfer will restart.
```

---

# 244. Background retry

Temporary failures MAY retry automatically.

The UI SHOULD show:

```text
Retrying in background
```

and allow manual action if prolonged.

---

# 245. Do not spam users

Transient storage/API failures that recover automatically need not produce permanent alerts.

Persistent or data-risk conditions must.

---

# 246. Notification hierarchy

Suggested categories:

```text
Creative
Collaboration
Storage
Platform
System
```

so a new comment does not compete visually with data-loss risk.

---

# 247. Privacy controls

Users SHOULD be able to understand:

```text
what Project metadata is public
what Resources are public
what storage provider details are hidden
```

without reading protocol documentation.

---

# 248. Raw storage details

By default, other collaborators should not see more physical storage detail than necessary.

Example:

```text
Resource available
```

may be sufficient.

Advanced authorized users may see Endpoint information.

---

# 249. Temporary Resource access

Users receiving temporary access SHOULD not need to understand presigned URLs or tokens.

They simply:

```text
Play
Download
Open in DAW
```

---

# 250. Expired access

If a temporary grant expires mid-operation:

```text
Access expired.
Requesting fresh access...
```

should be automatic where safe.

---

# 251. Project owner absent

A Project should remain interpretable even if original creator stops using the Platform.

The UI SHOULD preserve:

```text
history
provenance
Releases
```

according to available data and access.

---

# 252. Contributor Resource at risk

If an integrated Version depends solely on contributor-controlled storage and that storage is becoming unavailable, the Project steward SHOULD see:

```text
Version 22 depends on Anna's storage.

Create a Project-controlled backup copy?
```

---

# 253. Release dependency on contributor storage

For Releases, the UI SHOULD strongly encourage durable Project-controlled replication of essential Resources.

---

# 254. Storage custody display

When relevant:

```text
Cello recording
Stored by Anna
Project backup available
```

Custody is not authorship.

---

# 255. Project transfer of stewardship

A stewardship transfer SHOULD not create new Project identity.

The UI may show:

```text
Transfer project stewardship
```

with explicit permission changes.

---

# 256. History after stewardship transfer

Past versions retain original authors.

Future stewardship does not rewrite them.

---

# 257. Multiple creators

Project UI SHOULD avoid assuming one owner/creator where provenance shows many contributors.

---

# 258. Credits versus permissions

Someone may have:

```text
historical credit
```

but:

```text
no current Project write permission
```

The UI must distinguish these.

---

# 259. Search discoverability

The Project owner SHOULD control whether public metadata is searchable.

`Unlisted` should mean:

```text
accessible by link
not normally included in discovery
```

---

# 260. Search result transparency

Search result cards may show:

```text
Project
Release
seeking contributions
licence
DAW compatibility
```

but must not reveal private history.

---

# 261. Component-level discovery

Future Platform UI MAY allow:

```text
Find Projects needing bass
Find openly reusable drum Components
Find 122 BPM rock projects
```

provided metadata and licences support it.

This interaction model is allowed but not required for 0.1.

---

# 262. Version pinning

A user MAY mark a Version as:

```text
Favourite
Milestone
Keep offline
```

These are operational/presentation states unless represented as Release.

They MUST NOT be confused with immutable Release semantics.

---

# 263. Milestone versus Release

A UI MAY support a lightweight milestone marker.

If so, it MUST clearly distinguish:

```text
Milestone
```

from:

```text
Release
```

if milestone target is mutable.

---

# 264. Human-readable history numbering

Presentation may number versions sequentially within a Line.

Example:

```text
Main 12
Main 13
Main 14
```

These labels are UI conveniences.

Revision IDs remain authoritative.

---

# 265. Numbering across divergence

The UI must avoid labels that imply false ordering.

For divergent directions:

```text
Main 14
Acoustic 3
```

is better than globally implying:

```text
Version 15 happened after everything else
```

where chronology/ancestry is ambiguous.

---

# 266. Time presentation

Dates may be localized.

Ancestry must never be inferred from display time.

---

# 267. Current Version indicator

The UI should identify:

```text
Current direction: Main
Current saved version: Main 14
Current work: 3 unpublished changes
```

This is a very useful summary.

---

# 268. Current work after Platform sync

Platform updates do not change Current Work unless the user explicitly materialises another Revision or integration result.

---

# 269. Automatic background metadata pull

The client MAY fetch remote metadata in background.

If it discovers divergent history, it should notify rather than changing Working State.

---

# 270. Automatic Resource download

The client MAY prefetch likely Resources.

Prefetching MUST NOT alter Working State or history.

---

# 271. Smart prefetch

Possible useful behaviour:

```text
download Reference Render for history preview
prefetch likely next Version
prefetch Contribution audio
```

This is optimization only.

---

# 272. Metered connections

Users SHOULD be able to prevent large automatic transfers.

---

# 273. Storage cost warning

If an operation may incur significant egress/provider cost and the Adapter knows this:

```text
Moving 180 GB may incur storage-provider transfer charges.
```

This is advisory.

---

# 274. Quality of service

The UI may prioritize:

```text
recording > playback > current Working State > background replication
```

to protect musical work.

---

# 275. Reference Render creation

The user MAY explicitly request:

```text
Update reference mix
```

for current Working State.

It becomes historical only if included when a Version is published.

---

# 276. Stale Reference Render warning

If Working State changed since the last temporary render:

```text
Reference mix no longer matches current work.
```

---

# 277. Historical Reference Render immutability

A Reference Render attached to an existing Revision MUST not be silently regenerated and replaced.

A new render means new historical state if it is part of Project State.

---

# 278. Preview render

The UI may create temporary preview renders freely.

They must be labelled as preview, not historical reference.

---

# 279. Plugin substitution

If a missing plugin is replaced manually:

```text
Original SuperVerb unavailable
Using OpenVerb instead
```

this is Working State.

Saving produces new history.

The original Version remains unchanged.

---

# 280. Dependency repair

Installing the missing original plugin may allow exact restoration without creating new history, because environment changed, not Project State.

The UI should understand this distinction.

---

# 281. Environment versus history

The user may see:

```text
Version unchanged
Reproducibility improved from Partial to Exact
```

after installing a Dependency.

No Revision needed.

---

# 282. Adapter upgrade

Installing a newer DAW Adapter may improve:

```text
semantic diff
restore compatibility
dependency detection
```

without changing history.

---

# 283. Native format upgrade

Opening an old Version in a newer DAW may create upgraded Working State.

The UI SHOULD warn before saving:

```text
This project was converted to Ardour 11 format.
Saving a new version will preserve the converted state while leaving the original version untouched.
```

---

# 284. Adapter failure

If Adapter cannot interpret old state:

```text
This version is valid, but your current Ardour adapter cannot open its native project state.
```

Avoid saying:

```text
Version invalid
```

unless integrity actually failed.

---

# 285. Unknown extension data

Users generally need not see unknown extension metadata.

If it prevents lossless operations:

```text
This project contains information created by another OMVCS implementation that this client cannot fully interpret.

It will be preserved, but some editing features are unavailable.
```

---

# 286. Conformance indicator

Advanced UI MAY show:

```text
OMVCS 0.1 compatible
DAW Adapter Level C
Storage Endpoint supports Repository Home
```

This is useful for diagnostics and developers.

---

# 287. Project compatibility check

Before major migration/import, the UI SHOULD run:

```text
Check Project
```

which can validate:

```text
metadata
Resource availability
integrity
Adapter support
Dependencies
storage policy
```

---

# 288. Health check

Recommended action:

```text
Check project health
```

rather than:

```text
Run repository validation
```

for normal users.

---

# 289. Health report

Example:

```text
Project history        Healthy
Current Resources      Healthy
Historical Resources   2 unavailable
Backups                Needs attention
DAW dependencies       Exact
Platform               Up to date
```

---

# 290. Repair actions from health report

Each actionable issue SHOULD link directly to:

```text
Reconnect
Repair
Create backup
Install dependency
Restore from archive
Resync Platform
```

---

# 291. Conformance testability

All major workflows defined here MUST be testable without relying solely on screenshots/manual visual inspection.

Tests SHOULD verify resulting OMVCS operations and states.

---

# 292. Interaction test harness

A reference UI/test harness SHOULD simulate:

```text
user action
expected Core call
expected Adapter call
expected resulting state
expected visible message category
```

This is not a new specification.

---

# 293. Golden workflow tests

At minimum, interaction conformance SHOULD test:

```text
create Project
adopt existing DAW project
save first Version
edit and save Version
open old Version
protect unpublished work
create alternate direction
switch direction
try historical Component
build Custom Working State
submit Contribution
review Contribution
selectively integrate
create Release
add backup
move storage
storage authentication expiry
missing Resource
missing Dependency
Platform offline
recover Project
move Platform
export archive
import archive
fork Project
```

---

# 294. Destructive workflow tests

Tests MUST verify that the UI does not silently:

```text
discard local work
remove last valid Resource copy
overwrite divergent Line
delete historical Versions
replace immutable Release
```

---

# 295. Failure workflow tests

The UI/test harness SHOULD simulate:

```text
storage upload fails
verification fails
Platform unavailable
DAW restore fails
Dependency missing
auth expires
quota exceeded
concurrent Line update
Contribution conflict
```

and verify correct user-facing state.

---

# 296. Terminology consistency tests

The reference UI SHOULD be checked for accidental Git language where musician-facing terms are intended.

Technical detail screens MAY use normative OMVCS vocabulary.

---

# 297. Agent implementation rules

Coding agents implementing OMVCS UI MUST obey:

> Do not expose Git terminology merely because the underlying concept resembles Git.

> Do not hide important failure or data-risk conditions behind technical abstraction.

> Do not create a Revision from Save, Close or Autosave unless explicitly instructed by OMVCS workflow.

> Do not discard unpublished work when Materialising another Version.

> Do not present storage migration as creative history.

> Do not claim a Version is published when Core durability requirements failed.

> Do not claim the Project is broken because the Platform is offline.

> Do not claim a Resource is missing when authentication merely failed.

> Do not claim exact reproduction when Adapter reports partial or unknown.

> Do not silently accept all parts of a Contribution when the user selected only some.

> Do not treat Platform account identity as historical Actor identity.

> Do not treat custody as authorship.

> Do not turn UI labels into new Core semantics.

> Do not bypass Core/Adapter APIs to mutate repository state directly.

> Do not invent conflict resolution.

> Do not hide destructive operations behind friendly wording.

---

# 298. Example: ordinary musician experience

A user opens their Project.

The main screen says:

```text
My Song

Main
Version 14

Current work:
2 unpublished changes

Changes:
Bass recording changed
Vocal automation changed

Storage:
Healthy

Open contributions:
1
```

The user presses:

```text
Save version
```

and enters:

```text
"Tighter bass and softer vocal ending"
```

OMVCS handles:

```text
capture
hashing
Resource upload
verification
Revision creation
Line update
Platform sync
```

The musician need not see any of that unless something fails.

That is the intended experience.

---

# 299. Example: storage migration

User opens:

```text
Storage
```

and sees:

```text
Old S3
82 GB

New S3
14 GB
```

They choose:

```text
Move project files from Old S3 to New S3
```

The UI says:

```text
82 GB will be copied and verified before the old copies are removed.

Your versions and releases will not change.
```

After completion:

```text
Move complete
All files verified
```

No Version appears in History.

---

# 300. Example: Contribution

Anna submits:

```text
Cello idea
```

Joakim sees:

```text
Anna contributed to Version 18

Added:
Cello

Changed:
Drums

Anna's note:
"I liked the original arrangement but tried a lighter cello line."
```

He chooses:

```text
Use cello only
```

The DAW Adapter detects that cello requires one routing bus.

UI says:

```text
The cello uses an additional Cello Bus.
It can be added safely.

Drum changes will not be used.
```

He applies it, listens, adjusts one level and saves:

```text
Version 19
"Anna's cello with original drums"
```

History/provenance records the source automatically.

---

# 301. Example: missing plugin

User opens Version 6.

UI says:

```text
Version 6 opened with one limitation

Missing plugin:
VintageEcho 2.1

The original plugin state is preserved.

You can:
Play the original reference mix
Continue with the plugin disabled
Install the plugin and try again
```

The user understands that Version 6 still exists intact.

---

# 302. Example: Platform disappears

The Open Music Platform cannot be reached.

User opens Project locally.

UI says:

```text
Open Music site unavailable

Your Project and stored versions are unaffected.

You can continue working and saving versions.
Site updates will sync later.
```

That message embodies one of the most important architectural goals.

---

# 303. Example: conflicting work

Joakim and Anna both continue from Version 20.

Joakim later synchronizes.

UI:

```text
Two creative directions continued from Version 20.

Your version:
New vocal mix

Anna's version:
New cello arrangement

Nothing has been overwritten.

Choose:
Keep as separate directions
Combine them
Review Anna's version
```

This is far more appropriate to music than:

```text
non-fast-forward push rejected
```

---

# 304. Example: Custom Working State

Joakim opens Bass History.

He listens to B2, B5 and B8.

He selects:

```text
Use B5
```

Current screen changes to:

```text
Current work
Custom combination

Bass:
Version B5 from Main 9

Drums:
Current

Vocals:
Current

Not yet saved as a version
```

After listening he presses:

```text
Save version
```

OMVCS preserves where B5 came from.

---

# 305. Example: Project recovery

New computer.

User chooses:

```text
Open existing Open Music Project
```

and connects Repository Home.

UI finds:

```text
My Song
87 Versions
3 Releases
2 Creative Directions
```

It validates storage and reports:

```text
Current Version available
All Resources healthy
```

User clicks:

```text
Open current version
```

The Project is reconstructed.

No Platform involvement is required.

---

# 306. Interaction architecture summary

The interaction model can be reduced to:

```text
MUSICIAN

"I changed my bass."
"I want to save this version."
"I want to try the old drums."
"I want Anna's cello."
"I want to move my files."
"I want to release this."
"I want my project back."

        |
        v

OMVCS INTERACTION LAYER

translates intent into precise operations

        |
        +--> OMVCS Core
        +--> DAW Adapter
        +--> Storage Adapter
        +--> Platform Protocol
```

The user expresses **creative intent**.

The system performs **technical operations**.

The UI must never confuse the two.

---

# 307. Architectural statement

The most important requirements of this specification are:

> **The default OMVCS experience is musician-first, not Git-first.**

> **Creative history, current work and infrastructure operations must remain visibly distinct.**

> **The UI simplifies mechanisms, not semantics.**

> **Important destructive or lossy behaviour must never be hidden behind abstraction.**

> **Users should be able to collaborate, explore history and recover Projects without understanding hashes, DAGs or storage topology.**

> **Every important user action must map to a defined OMVCS operation.**

> **If an action cannot be mapped cleanly, the underlying architecture is not yet finished.**

---

# 308. Unresolved Interaction Specification decisions for 0.1

The following remain intentionally open inside this document:

1. Final musician-facing term for **Line**: `Creative Direction`, `Version Line`, or another tested term.
2. Final musician-facing distinction between `Save Version` and `Publish Version`.
3. Whether human-readable Version numbers are global per Project or scoped per Line.
4. Exact treatment of temporary local safety checkpoints.
5. Whether storage administration lives primarily inside DAW-integrated UI or standalone Project management UI.
6. Exact minimum information shown before Publish Version.
7. Exact default Contribution Intent vocabulary.
8. Exact UI model for Custom Working States.
9. Exact conflict presentation categories.
10. Exact visibility/access presets for Project, Reference Render, stems and DAW state.
11. Exact terminology for Repository Home in musician-facing UI.
12. Exact default replication warnings for Releases.
13. Exact Reference Render UX for ordinary Versions versus Releases.
14. Exact safe-inspection workflow for untrusted Contributions.
15. Exact treatment of Platform-offline mode in the reference application.
16. Whether a Component history view is mandatory for OMVCS reference UI or only required when the Adapter supports Component mapping.
17. Exact minimum recovery workflow required by the reference implementation.
18. Exact advanced/developer mode content.
19. Whether the UI should expose storage custody by default for contributed Resources.
20. Exact interaction design for cross-DAW import/conversion.

These are finite questions inside the **OMVCS Interaction Specification**. They do not create additional top-level documents.

---

## Document status

**Document:** OMVCS Interaction Specification  
**Version:** 0.1 Draft  
**Normative:** Draft normative  
**Depends on:** OMVCS Glossary, Core Invariants Specification, OMVCS Core Specification, OMVCS DAW Adapter Specification, OMVCS Storage Adapter Specification, OMVCS Platform Protocol  
**Next fixed document:** **Ardour Reference Adapter Design**