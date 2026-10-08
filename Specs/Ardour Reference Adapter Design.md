# Ardour Reference Adapter Design
## Open Music Version Control System
### Draft 0.1

This document defines the reference mapping between the generic **OMVCS DAW Adapter Specification** and Ardour.

It is subordinate to:

1. **OMVCS Glossary**
2. **OMVCS Core Invariants Specification**
3. **OMVCS Core Specification**
4. **OMVCS DAW Adapter Specification**
5. **OMVCS Storage Adapter Specification**
6. **OMVCS Platform Protocol**
7. **OMVCS Interaction Specification**

This is the final document in the fixed OMVCS top-level specification set.

The purpose of this document is **not** to make Ardour part of OMVCS Core.

Its purpose is to prove that the generic DAW Adapter Contract can be implemented deeply enough in a real DAW to support the intended Open Music workflow.

The governing rule is:

> **Ardour is the reference implementation of the OMVCS DAW Adapter Contract. Ardour does not define OMVCS semantics.**

This design is based on the current Ardour source architecture inspected during specification work. Ardour already exposes session-state save/load concepts, snapshot naming/state, save locks, archive behaviour and state-management machinery, giving us a useful native foundation rather than requiring OMVCS to invent a parallel project-state system inside Ardour.  The `Session` class also carries persistent session UUID/path/name, current snapshot state and synchronization around state saving, which are relevant integration points for capture and restoration. 

---

# 1. Design objective

The Ardour Adapter MUST allow OMVCS to perform at minimum:

```text
Identify Ardour Project
Capture complete Ardour state
Enumerate required Resources
Map meaningful Ardour entities to Creative Components
Detect changes
Publish Versions through OMVCS Core
Restore historical Versions
Validate restoration
Report Dependencies
Render Reference Mixes
Support component-level historical experimentation
Support Contribution review/integration where safely possible
```

The reference implementation SHOULD eventually demonstrate the richest practical OMVCS integration level.

It should therefore aim for:

> **OMVCS Native DAW Adapter**

rather than merely minimum compatibility.

---

# 2. Non-goals

The reference Adapter MUST NOT:

- replace Ardour's audio engine;
- replace Ardour's internal session model;
- implement a separate Version graph inside Ardour;
- modify OMVCS Core to understand Ardour;
- treat Ardour snapshots as OMVCS Revisions automatically;
- require all future DAWs to behave like Ardour;
- make Ardour project format the Open Music interchange format;
- assume every future OMVCS Project is Ardour-based.

---

# 3. High-level architecture

The recommended architecture is:

```text
+---------------------------------------------------+
|                     ARDOUR                        |
|                                                   |
| Session / Routes / Playlists / Regions / Sources |
| Processors / Plugins / Automation / Tempo / etc. |
+--------------------------+------------------------+
                           |
                           v
+---------------------------------------------------+
|            ARDOUR OMVCS ADAPTER                  |
|                                                   |
| Ardour state capture                              |
| Ardour Resource discovery                        |
| Component bindings                               |
| Semantic change detection                        |
| Restore/materialisation                          |
| Integration planning                             |
| Reference rendering                              |
+--------------------------+------------------------+
                           |
               OMVCS DAW Adapter Contract
                           |
                           v
+---------------------------------------------------+
|                   OMVCS CORE                      |
|                                                   |
| Project / Versions / Lines / Contributions       |
| Resources / Provenance / Storage / Releases      |
+---------------------------------------------------+
```

The Adapter MAY be compiled into the Ardour reference distribution, but logically it remains a distinct architectural layer.

---

# 4. Implementation strategy

The reference implementation SHOULD use a combination of:

```text
Ardour library integration
+
Ardour-native UI integration
+
OMVCS Core library
```

rather than controlling Ardour externally through filesystem observation alone.

Deep integration is desirable because OMVCS needs access to:

- stable native entity identities;
- source/reference graphs;
- current state;
- recording lifecycle;
- semantic changes;
- dependencies;
- routing;
- automation;
- plugin state.

A purely external tool could support basic snapshotting but would lose much of the reason for using Ardour as the reference DAW.

---

# 5. Adapter identifier

The reference Adapter identifier SHOULD be:

```text
org.openmusic.ardour
```

Adapter state schema initially:

```text
org.openmusic.ardour.state/0.1
```

Adapter Contract version:

```text
omvcs.daw-adapter/0.1
```

---

# 6. Ardour project identity versus OMVCS Project identity

Ardour already has its own Session identity and project/session naming.

OMVCS MUST keep these distinct.

Conceptually:

```text
OMVCS Project ID
    !=
Ardour Session UUID
```

The Adapter maintains a binding:

```text
OMVCS Project ID
        ↕
Ardour Session UUID
```

The Ardour Session UUID is valuable as a stable native identity but MUST NOT become the OMVCS Project Identifier.

---

# 7. Project association persistence

The Adapter SHOULD persist lightweight OMVCS association metadata inside the Ardour session where practical.

Conceptually:

```xml
<OMVCS
    project-id="019..."
    binding-version="1"
    base-revision="omvcs:revision:sha256:..."/>
```

Exact XML placement must be selected only after detailed Ardour-state inspection.

The association MUST survive:

```text
save
close
reopen
rename
move project directory
```

The Project ID MUST NOT depend on the directory path.

---

# 8. Sidecar fallback

If inserting OMVCS metadata directly into Ardour's native session XML would risk incompatibility or require invasive changes, the reference Adapter MAY use a sidecar such as:

```text
.omvcs/ardour-binding.json
```

The sidecar must remain bound to the Ardour Session UUID.

A moved project must therefore still be recognizable.

Preferred order:

```text
Native Ardour extension metadata
        ↓
OMVCS sidecar if necessary
        ↓
Never filename inference
```

---

# 9. Association ambiguity

The Adapter MUST detect cases such as:

```text
Ardour Session UUID X
claims OMVCS Project A

Local OMVCS metadata
claims Session UUID X belongs to Project B
```

This MUST produce:

```text
invalid_or_ambiguous_association
```

rather than silently choosing one.

---

# 10. Ardour Session as native Adapter State root

The Ardour `Session` is the principal native state root.

The reference Adapter SHOULD capture a saved Ardour session snapshot as a native-state Resource referenced by the canonical Adapter State metadata object. The Project State references that Adapter State object, not the native-state Resource directly.

Ardour's state subsystem already exposes session save and possible saved state/snapshot handling, and `Session::save_state` accepts options relating to snapshot, archive and used assets. 

The Adapter MUST nevertheless determine all dependent Resource Objects separately.

The `.ardour` state file alone is not assumed to contain every required media byte.

---

# 11. Native state capture strategy

The reference Adapter SHOULD use Ardour's own state serialization rather than building a separate full Ardour serializer.

Conceptually:

```text
Current Ardour Session
       |
Ardour native save/snapshot machinery
       |
stable session-state representation
       |
OMVCS Resource Object
```

This minimizes duplication and maximizes fidelity.

---

# 12. Capture consistency

Capture MUST represent one coherent Ardour state.

Recommended strategy:

```text
1. Request capture
2. Ensure safe Ardour state point
3. Prevent incompatible session-state mutation during state serialization
4. Serialize native state
5. Enumerate dependent Sources and state Resources
6. Verify source stability
7. Release capture protection
```

Ardour already serializes state under explicit save-related synchronization, including `save_state_lock` and related state-save fields inside `Session`, which is promising for this boundary. 

---

# 13. Capture during recording

OMVCS Publish Version SHOULD NOT finalize capture while Ardour is actively recording.

Possible initial behaviour:

```text
Recording active
        |
Save Version requested
        |
"Finish recording before saving this version."
```

After recording completes, the newly created source must be finalized before hashing/capture.

Later versions MAY support sophisticated recording-safe snapshots if Ardour guarantees them.

0.1 SHOULD prefer correctness over cleverness.

---

# 14. Capture during playback

Capture MAY be permitted during playback if Ardour can serialize a coherent state safely without disturbing the audio engine.

Expensive Resource hashing/upload MUST occur outside the real-time audio thread.

---

# 15. Native Session Resource

Each captured Ardour state SHOULD include an immutable Resource representing the Ardour session state.

Example:

```text
role:
ardour-session-state
```

Logical name might be:

```text
My Song.ardour
```

but Resource identity derives from bytes. If an exact native filename is required for restoration, the required naming state belongs in Adapter State, not the generic historical Resource Reference.

---

# 16. Supporting native Resources

The Adapter must determine whether additional Ardour-native state files are required for exact restoration.

Possible categories include:

```text
session XML/state
plugin state
MIDI source files
audio Sources
automation/native support state
embedded metadata
other referenced session assets
```

The exact inventory MUST be established during implementation reconnaissance.

---

# 17. Disposable Ardour files

The Adapter SHOULD exclude Resources that Ardour can regenerate without creative loss.

Likely examples may include:

```text
peak/waveform cache
temporary export state
lock files
reconstructable analysis cache
```

However, exclusion MUST be proven.

A coding agent MUST NOT classify a file as disposable merely because its filename looks temporary.

---

# 18. Source enumeration

The Adapter SHOULD use Ardour's own source/reference graph rather than simply including every file found under the session directory.

Ardour's session-state implementation already reasons about Sources associated with project/session state and provides archive/save behaviour around used assets. 

This is preferable to:

```text
scan project folder
include everything
```

because a folder may contain:

- unused media;
- abandoned takes;
- backups;
- exports;
- unrelated files.

---

# 19. Required inactive state

Ardour may contain creatively relevant state that is not currently audible.

Examples:

```text
alternate playlists
inactive processors
muted tracks
hidden Routes
unused but retained takes
automation
regions outside current playback range
```

If Ardour considers those objects part of the saved Session, full-state capture MUST preserve them.

OMVCS MUST NOT reduce history to "what was audible at the moment".

---

# 20. Source IDs

Ardour-native stable Source IDs SHOULD be preserved inside Adapter State where available.

Binding example:

```text
Ardour Source ID
        ↕
OMVCS Resource ID
```

This supports change detection and Resource reuse.

Core MUST not interpret the Ardour Source ID.

---

# 21. Creative Component mapping strategy

The first reference implementation SHOULD map OMVCS Creative Components primarily to Ardour Routes/Tracks where this corresponds naturally to musical meaning.

Example:

```text
Ardour Track "Bass"
    ↕
OMVCS Creative Component "Bass"
```

The quoted names are illustrative presentation labels, not fields of the generic OMVCS 0.1 Creative Component object. A Track and a Creative Component are not formally identical.

---

# 22. Why Route/Track mapping is only the default

A musician may have:

```text
Kick
Snare
Overheads
Room
```

as four Ardour Tracks but conceptually think of them as:

```text
Drums
```

one Creative Component.

Or one Track may contain multiple creatively meaningful layers.

Therefore the Adapter MUST permit explicit user-level Component grouping.

---

# 23. Component binding table

Adapter State SHOULD contain a stable mapping such as:

```json
{
  "component_bindings": {
    "019-bass...": {
      "binding_kind": "ardour-route",
      "native_ids": ["route-id-123"]
    },
    "019-drums...": {
      "binding_kind": "ardour-route-group",
      "native_ids": [
        "route-kick",
        "route-snare",
        "route-overheads"
      ]
    }
  }
}
```

Each map key is a Creative Component Identifier; each value contains the Ardour-specific binding record. Do not repeat the Component Identifier inside the value. The exact Ardour ID representation must be taken from native stable identities, not route names.

`component_bindings` is a JSON object map: insertion order has no semantic significance, and its canonical serialization follows RFC 8785 object-member ordering solely. Duplicate member names MUST be rejected before hashing or canonical serialization; no additional entry-sorting transformation applies. Each binding's `native_ids` array is set-like: its member order carries no semantics, and duplicate members are invalid. Any other array-valued collection introduced in a later schema MUST be classified explicitly and normalized according to Core Specification section 5.1.

---

# 24. Track rename

If:

```text
Bass
```

becomes:

```text
Fretless Bass
```

while Ardour's stable native Route identity is unchanged:

```text
same OMVCS Creative Component
```

The Adapter reports the display metadata change.
The displayed route name is not a field of the generic OMVCS 0.1 Creative Component object. Keep it as presentation metadata unless an approved historical schema gives it historical meaning; if required for Ardour reconstruction, represent it in Adapter State. A route rename does not change the Component Identifier or by itself require a new Component State.

---

# 25. Track duplication

If an Ardour Track is duplicated, the Adapter MUST determine that the new Route is a new native entity.

It SHOULD propose:

```text
new Creative Component candidate
```

rather than inheriting the original component identity automatically.

---

# 26. Grouped Components

The OMVCS Ardour UI SHOULD eventually allow:

```text
Select tracks
        |
"Treat as one Open Music Component"
```

Example:

```text
Kick
Snare
OH-L
OH-R
Room
        ↓
Creative Component:
Drums
```

This mapping becomes Adapter State.

---

# 27. Component split

The user SHOULD be able to split a prior OMVCS Component mapping.

Example:

```text
Drums
```

into:

```text
Drum Kit
Percussion
```

The Adapter must propose known lineage rather than erase history, but MUST NOT fabricate unknown historical parentage. Confirmed derived Component States SHOULD record their parent Component States, except where a specific operation or provenance rule makes this mandatory.

---

# 28. Component merge

Likewise multiple prior Components may be grouped into a new semantic Component.

History of source Components remains preserved.

---

# 29. Ardour structural objects not necessarily Components

The Adapter SHOULD NOT automatically create user-visible Creative Components for every native object.

Examples may include:

```text
Master Bus
Monitor Section
Utility Bus
Control-only Route
Sidechain support
technical routing object
```

They may remain entirely in Adapter State.

---

# 30. Creative versus technical state

The Adapter has two responsibilities:

```text
map meaningful creative units to OMVCS Components
```

and:

```text
preserve all other required Ardour state in Adapter State
```

Not everything needs to become a Creative Component for OMVCS to preserve it.

---

# 31. Adapter State structure

A conceptual Ardour Adapter State:

```json
{
  "schema": "org.openmusic.ardour.state/0.1",

  "session": {
    "ardour_session_uuid": "...",
    "native_format": "...",
    "ardour_version": "..."
  },

  "native_state_resources": [
    {
      "resource_id": "omvcs:resource:sha256:...",
      "byte_length": 4096,
      "role": "ardour-session-state"
    }
  ],

  "component_bindings": {
    "...creative-component-id...": {
      "binding_kind": "ardour-route",
      "native_ids": ["..."]
    }
  },

  "dependencies": [],

  "extensions": {}
}
```

The conceptual structure is a canonical OMVCS metadata object. `component_bindings` is keyed by Creative Component Identifier; the binding value MUST NOT duplicate that identifier merely to repeat the key. The real schema must be canonical and deterministic, and the Project State references this object rather than any native-state Resource directly.

For this conceptual Adapter State schema, `native_state_resources` and each binding's `native_ids` are set-like arrays: member order carries no semantics, and duplicate elements are invalid. `component_bindings` is a JSON object map, not an array collection, and follows RFC 8785 object-member ordering solely; duplicate member names are invalid and MUST be rejected before hashing or canonical serialization. Any other array-valued collection added to a hashed Ardour Adapter State schema MUST explicitly declare its ordering semantics and follow Core Specification section 5.1. Ardour-native order that has creative meaning remains represented in the native Session Resource, not in the set-like identity of these references.

The Ardour Adapter implementation is responsible for defining and validating any Resource Reference `properties` under the exact Ardour Adapter State schema version that governs the containing historical use. It MUST enforce ADR-0007's excluded naming, storage, credential, and physical-reconstruction semantics without relying on Core to interpret Ardour property meanings. The Core still performs generic structural, duplicate-member, canonicalization, and declared-collection validation. This reference design does not define additional generic Resource Reference property meanings.

---

# 32. Session path independence

Adapter State MUST NOT require the original absolute Ardour Session path.

Example:

```text
C:\Users\Joakim\Music\Song
```

may restore to:

```text
/Users/Anna/Music/Song
```

The Adapter rewrites/restores local Source paths appropriately.

---

# 33. Resource materialisation layout

For restoration, Core provides local paths for Resources.

The Ardour Adapter determines a valid session layout.

Recommended strategy:

```text
materialised OMVCS Resources
        ↓
Ardour-compatible session directory
        ↓
session state rewritten/resolved where necessary
```

The exact folder layout SHOULD follow Ardour-native conventions where practical.

---

# 34. Working State versus immutable cache

The Adapter MUST never let Ardour edit OMVCS immutable cache objects directly.

Correct model:

```text
OMVCS immutable object cache
        |
materialise/copy/reflink
        v
Ardour Working Session
        |
editable
```

---

# 35. Reflinks/hardlinks

The local implementation MAY use copy-on-write/reflink optimizations.

It MUST ensure Ardour editing cannot mutate the immutable cached Resource.

Hard links that permit in-place mutation SHOULD be avoided unless immutability can be guaranteed.

---

# 36. State capture after local editing

When publishing:

```text
Ardour Working Resource
        |
hash
        |
same bytes?
   /          \
 yes           no
 |             |
reuse ID     new Resource Object
```

This naturally deduplicates unchanged Sources.

---

# 37. Incremental Resource hashing

To avoid repeatedly hashing very large audio Sources, the Adapter SHOULD maintain a local fingerprint cache using:

```text
native Source ID
path
size
mtime as hint
previous content hash
```

But before asserting a new immutable Resource identity, actual content correctness remains authoritative.

mtime MUST never be identity.

---

# 38. Stable recorded Sources

Once Ardour has finalized an audio recording and the file is no longer being written, the Adapter can hash it.

If Ardour is still writing the Source:

```text
capture blocked or source staged safely
```

---

# 39. External Source handling

If Ardour references audio outside the Session directory, the Adapter MUST detect it.

The normal OMVCS publication path SHOULD capture it as a Resource Object.

The user should not be required to remember the external path forever.

---

# 40. Ardour Save As integration

Ardour already has Save As/session copying behaviour and options relating to media inclusion/copying. That machinery may be useful for constructing isolated Working States or restoration staging. 

However:

> Ardour Save As is not OMVCS Version publication.

It is an implementation mechanism the Adapter MAY use.

---

# 41. Ardour archive integration

Ardour provides an `archive_session` implementation in its session-state code. 

The Adapter SHOULD evaluate whether archive/consolidation logic can help:

- discover required assets;
- create portable state;
- validate source completeness.

It MUST NOT simply treat an Ardour archive as the OMVCS Repository.

---

# 42. Native snapshots

Ardour snapshots are potentially very useful implementation primitives.

But:

```text
Ardour Snapshot
    !=
OMVCS Revision
```

A snapshot may help Capture/Restore.

Only OMVCS Core creates the Revision.

---

# 43. Snapshot naming

Internal OMVCS-created snapshots SHOULD use a reserved naming scheme if native snapshots are employed.

For example:

```text
.omvcs-capture-<operation-id>
```

These SHOULD be hidden from normal Ardour snapshot UI where possible or clearly marked internal.

---

# 44. Snapshot cleanup

Temporary internal snapshots SHOULD be removed after successful capture unless required for local recovery.

They MUST NOT clutter creative history.

---

# 45. Existing user-created snapshots

User-created Ardour snapshots form part of the Ardour project state if needed for full reconstruction.

The Adapter SHOULD preserve them as native state.

They MUST not automatically become separate OMVCS Revisions.

---

# 46. Change detection architecture

The Ardour Adapter SHOULD combine several signals:

```text
Ardour dirty/state flags
native object IDs
Source identities
saved-state comparison
semantic object comparison
OMVCS previous Adapter State
```

No one mechanism needs to carry the full responsibility.

---

# 47. Minimum change detection

The Adapter MUST determine:

```text
unchanged
changed
unknown
```

relative to Base Revision.

---

# 48. Rich semantic diff

The reference Adapter SHOULD aim to report semantic changes for at least:

```text
Routes/Tracks
Sources
Regions
Playlists
Processor chains
Plugin state
Automation
Routing
Sends
Tempo/meter
Markers/locations
track addition/deletion
track rename
```

Exact feasibility must be established against Ardour APIs.

---

# 49. Semantic diff implementation principle

The Adapter SHOULD compare structured Ardour state rather than raw XML text whenever feasible.

Raw XML difference may report meaningless changes due to:

```text
ordering
volatile fields
UI state
format-version noise
```

The semantic layer should filter that.

---

# 50. Raw state diff fallback

If a particular Ardour subsystem cannot yet be semantically interpreted, Adapter may report:

```text
Ardour state changed: processor configuration
```

rather than fabricating more detail.

---

# 51. UI-only state exclusion

The Adapter SHOULD identify Ardour fields that affect only local editing presentation.

Examples may include:

```text
window geometry
selected editor tab
scroll position
zoom
meter presentation
```

Such fields SHOULD NOT create creative Version churn unless necessary for exact user-state restoration policy.

---

# 52. Creative UI state ambiguity

Some UI-like state may have creative relevance.

Example:

```text
track visibility
```

may be merely editor preference or deliberately part of a session setup.

The Adapter design must explicitly classify these fields after Ardour inspection.

Do not guess.

---

# 53. Semantic normalization

Before hashing Adapter State, non-creative volatile native metadata SHOULD be normalized or excluded.

Goal:

```text
open project
make no creative change
save Version
```

should not produce a meaningless new Adapter State merely because Ardour updated a last-opened timestamp.

---

# 54. Repeat-capture determinism

Given unchanged meaningful Ardour state:

```text
Capture A
Capture B
```

SHOULD produce identical:

```text
Resource IDs
Adapter State ID
Project State ID
```

excluding intentionally historical Revision metadata such as author/time/message.

This must become a major conformance test.

---

# 55. Dependency reporting

The Ardour Adapter SHOULD enumerate external dependencies such as:

```text
LV2 plugins
VST2/VST3 plugins where supported
AudioUnit on supported systems
instrument plugins
external hardware insert dependencies
external sample libraries where detectable
scripts/extensions where relevant
```

---

# 56. Plugin identity

Dependency records SHOULD use Ardour/native plugin identifiers where stable.

The Adapter must distinguish:

```text
plugin unique identity
plugin display name
plugin version
plugin format
```

Names alone are insufficient.

---

# 57. Plugin state

Where Ardour persists plugin state inside Session XML or ancillary files, the Adapter MUST ensure those bytes/state are included in captured native state.

If Ardour stores plugin state externally, the corresponding Resources must be enumerated.

---

# 58. Plugin availability

During restore:

```text
state preserved
```

and:

```text
plugin executable available
```

must be reported separately.

---

# 59. Missing plugin

The Adapter SHOULD allow Ardour's normal missing-plugin handling where safe.

Reproducibility result may be:

```text
partial
```

while historical state remains valid.

---

# 60. Plugin version mismatch

Where plugin identity matches but version differs, the Adapter SHOULD report:

```text
dependency version mismatch
```

rather than automatically claiming exact reproducibility.

---

# 61. Unknown plugin compatibility

The Adapter MUST NOT assume:

```text
same plugin ID + newer version = exact
```

unless Ardour/plugin semantics provide a reliable guarantee.

It may report:

```text
available_with_version_difference
```

and validate further if possible.

---

# 62. Hardware dependencies

External hardware inserts or MIDI devices may affect reproduction.

The Adapter SHOULD report these where Ardour exposes them as project dependencies.

---

# 63. Audio device independence

Local playback device selection SHOULD normally be local-only.

Example:

```text
Focusrite interface
Built-in audio
```

must not become creative dependency unless the Project explicitly depends on hardware processing.

---

# 64. Routing preservation

Ardour routing is creative state.

The Adapter MUST preserve:

```text
Track -> Bus
Bus -> Master
Sends
Sidechains
Processor order
```

through native state.

Semantic diff SHOULD expose meaningful routing changes.

---

# 65. Automation

Automation is creative state.

The Adapter MUST preserve it.

Semantic diff SHOULD be able to report at least:

```text
automation changed
```

and preferably:

```text
which parameter
which Component/Route
time range
```

---

# 66. Tempo and meter

Tempo/meter state is Project-level creative state.

It SHOULD reside in Adapter State, not arbitrarily assigned to one Creative Component.

Changes must be reflected in semantic diff.

---

# 67. Locations/markers

Markers and locations SHOULD be preserved as Ardour native state.

Whether changes appear in default semantic diff may depend on relevance.

---

# 68. Regions

Regions are important semantic Ardour objects.

The Adapter SHOULD preserve native identities and, where practical, produce semantic changes such as:

```text
Region moved
Region trimmed
Region gain changed
Region added
Region removed
```

---

# 69. Playlists

Playlists/alternate track content can represent real creative alternatives.

They MUST be preserved if part of Ardour's project state.

The reference Adapter SHOULD eventually expose meaningful Playlist changes.

---

# 70. Source versus Region distinction

The Adapter MUST distinguish:

```text
Source:
underlying media
```

from:

```text
Region:
usage of Source on timeline
```

The same Resource Object may appear in many Regions.

Changing Region placement must not create a new audio Resource Object.

---

# 71. Example

Audio source:

```text
Bass.wav
Resource ABC
```

Region A:

```text
ABC from 0:00–0:30 at timeline 0:00
```

Region B:

```text
ABC from 0:20–0:40 at timeline 1:10
```

Moving Region B changes Ardour Adapter State.

It does **not** change Resource ABC.

This distinction is extremely important for efficient versioning.

---

# 72. Non-destructive editing

Ardour's non-destructive editing model is a good match for OMVCS.

Many creative edits should produce:

```text
new Adapter State
```

without:

```text
new audio Resource
```

Examples:

```text
trim
region movement
routing
plugin parameter
automation
```

This can make OMVCS substantially more storage-efficient.

---

# 73. Destructive/source-changing operations

If Ardour actually writes new media bytes through:

```text
recording
bounce
consolidate
render
destructive processing
```

a new Resource Object is created.

---

# 74. Reference Render

The Ardour Adapter SHOULD implement:

```text
RenderReference
```

using Ardour's own export/mix engine.

The resulting audio becomes an OMVCS Resource.

---

# 75. Reference Render preset

The reference implementation SHOULD define a deterministic default render profile.

Possible initial default:

```text
full session range
main master output
WAV
same project sample rate
24-bit PCM
no loudness normalization unless part of project intent
```

Exact defaults remain to be finalized.

---

# 76. Render-range selection

The Adapter should derive the project/song range from Ardour session range where available.

If no sensible range exists, user choice may be required.

---

# 77. Reference Render during Publish Version

Recommended flow:

```text
Capture coherent Ardour state
        |
Render from same captured state
        |
Verify captured state did not diverge
        |
publish both
```

The reference mix must correspond to the Version.

---

# 78. Render interruption

If the user changes the project materially while a reference render is being generated, the Adapter SHOULD detect state divergence and either:

```text
restart render
```

or:

```text
warn that render is stale
```

It MUST NOT attach a knowingly stale render.

---

# 79. Restore architecture

Materialisation of an Ardour Version should proceed:

```text
Core resolves Revision
        |
Core retrieves required Resource Objects
        |
Core provides materialised Resources
        |
Ardour Adapter constructs/restores session directory
        |
Adapter restores native session state
        |
Ardour opens Session
        |
Adapter validates
```

---

# 80. Restore should avoid overwriting Current Work

Historical Versions SHOULD normally be materialised into an isolated Working State/session location first.

This is safer than overwriting the current session in place.

---

# 81. Possible Working Session layout

Conceptually:

```text
OpenMusic/
  Project-P/
    working/
       current/
       historical-R12/
```

The exact directory design belongs to implementation, but isolated restore is preferred.

---

# 82. Open historical Version read/write behaviour

Once materialised, the user MAY edit the historical state.

The Adapter then treats it as:

```text
Working State based on historical Revision
```

The historical Version itself remains immutable.

---

# 83. Ardour native upgrade on restore

If Ardour upgrades an older session format when opened:

```text
old Adapter State remains unchanged
```

The upgraded state exists only in Working State.

Saving a new OMVCS Version captures the upgraded native state.

---

# 84. Restore validation

After Ardour opens a materialised Version, the Adapter MUST validate at least:

```text
Session opened
expected Session identity/binding established
required Sources linked
expected Routes/Components exist where mappings are available
Dependencies assessed
```

---

# 85. Exact restoration validation

For stronger validation, Adapter SHOULD compare:

```text
captured normalized Ardour state
```

against:

```text
normalized restored Ardour state
```

allowing explicitly defined harmless transformations such as local paths or native-format migration.

---

# 86. Restoration result

The Adapter returns:

```text
exact
exact_with_external_dependencies
partial
render_only
unavailable
unknown
```

according to the generic specification.

---

# 87. Component materialisation

The reference Adapter SHOULD ultimately support:

```text
MaterialiseComponent
```

This is one of the key OMVCS differentiators.

Example:

```text
Use Bass from Version 7
```

without restoring every other Component.

---

# 88. Component materialisation strategy

For a simple Track-bound Component:

```text
Locate target Ardour Route
Locate historical source Route/Component state
Analyze dependencies
Construct replacement/import state
Apply to Working Session
Validate
```

This is significantly harder than full-session restoration and should be implemented after basic OMVCS integration is proven.

---

# 89. Route export/import

Ardour's session-state implementation already includes route-state export functionality. 

This is particularly interesting for OMVCS Component-level operations.

The reference design SHOULD investigate whether Ardour's existing Route export/import mechanisms can serve as a foundation for:

```text
Component materialisation
Contribution integration
track-level state transfer
```

rather than inventing custom serialization.

---

# 90. Route state as component transport

Potential mapping:

```text
OMVCS Creative Component
        |
Ardour Route export
        |
native route-state Resource / structured Adapter state
```

This must be tested thoroughly before becoming normative.

We must determine whether Route export includes all required:

```text
Sources
playlists
processors
routing dependencies
automation
```

for an independently portable Component.

---

# 91. Component dependency graph

For Component-level operations, Ardour Adapter needs a native dependency graph.

Example:

```text
Lead Vocal Track
    |
    +--> Vocal Bus
    |
    +--> Reverb Send
            |
            +--> Reverb Bus
                    |
                    +--> Plugin X
```

The Adapter must distinguish:

```text
Component-owned state
```

from:

```text
shared Project state
```

---

# 92. Shared dependencies

A Component cannot simply "own" a shared Master/Reverb Bus if several tracks use it.

Component materialisation may therefore require:

```text
reuse existing compatible dependency
create dependency
map to alternative
report conflict
```

---

# 93. Dependency compatibility

The Adapter SHOULD compute a compatibility plan.

Example:

```text
Historical Bass uses Bus "Band Bus"

Current project already contains matching Band Bus
    -> reuse

Historical Bass uses unique distortion bus missing locally
    -> import support state

Historical Bass expects incompatible routing object
    -> conflict
```

---

# 94. Custom Working State

The Ardour Adapter SHOULD support assembling:

```text
Bass from R4
Drums from R7
Vocals from R10
```

where structurally compatible.

The result is ordinary Ardour Working State.

---

# 95. Custom state is not generated through XML splicing blindly

The implementation MUST NOT simply concatenate/edit XML fragments without using Ardour-aware semantics unless that approach has been proven correct.

Prefer Ardour's own object/state APIs.

---

# 96. Contribution review

For an Ardour-based Contribution, the user SHOULD be able to:

```text
Play Reference Render
View semantic changes
Open entire contributed session
Try selected Components
```

without losing Current Work.

---

# 97. Safe Contribution inspection

Initial inspection SHOULD avoid enabling potentially unsafe external plugin/script execution where Ardour permits.

At minimum, Reference Render and metadata can be reviewed safely.

---

# 98. Full contribution materialisation

If the user chooses:

```text
Open contribution in Ardour
```

it should materialise into a separate Working Session based on the Contribution Head.

---

# 99. Selective integration

Long-term reference Adapter goal:

```text
CreateIntegrationPlan
ApplyIntegrationPlan
```

using Ardour semantic structures.

---

# 100. Integration Plan example

Contribution:

```text
New cello Route
Changed Drums Route
Changed Tempo Map
```

User selects:

```text
Cello only
```

Adapter might report:

```text
Cello Route:
safe

Required:
Cello audio Source
Cello processing chain
Cello bus

Tempo Map:
not required

Drum changes:
excluded
```

---

# 101. Integration conflict example

Contribution Cello Regions depend on tempo changes.

Adapter reports:

```text
Cello cannot be integrated independently because its region timing
depends on the Contribution's tempo map.
```

Possible choices:

```text
Include tempo changes
Import cello as absolute-time audio
Open contribution manually
Cancel
```

Only offer choices that are technically valid.

---

# 102. Absolute-time fallback

The Adapter MAY offer a rendered audio fallback for otherwise incompatible state.

But this produces a different creative representation.

It MUST be explicit.

Example:

```text
Import rendered cello audio instead of editable cello track
```

That result becomes new Working State and provenance should record the transformation.

---

# 103. Semantic integration categories

The Ardour Adapter should eventually recognize conflicts such as:

```text
Route identity conflict
Source conflict
Playlist conflict
Region conflict
Tempo conflict
Routing conflict
Processor conflict
Automation conflict
Location conflict
Shared-bus conflict
```

---

# 104. No automatic creative preference

The Adapter MUST NOT resolve:

```text
Joakim drums
vs
Anna drums
```

based on:

```text
newest timestamp
filename
user identity
```

unless the user explicitly chooses.

---

# 105. Component-level provenance

When an imported historical Component is used:

```text
OMVCS provenance
```

records the source Component State.

The Ardour Adapter does not own this provenance.

It merely reports which native state was incorporated.

---

# 106. Current Work change mapping

The Adapter should maintain a binding between:

```text
Ardour native object
```

and:

```text
OMVCS Component
```

so semantic changes can be attributed correctly.

Example:

```text
Route ID X changed processor parameter
        ↓
Creative Component Bass
        ↓
"Distortion settings changed on Bass"
```

---

# 107. Project-level semantic changes

Not all changes belong to one Component.

Examples:

```text
Tempo map
Master bus
global markers
project range
monitor/routing architecture
```

These SHOULD be classified:

```text
Project-level change
```

rather than arbitrarily attributed.

---

# 108. Shared semantic changes

A shared bus change may affect several Components.

The Adapter MAY report:

```text
Shared Reverb Bus changed
Affects: Vocal, Guitar
```

without pretending the bus is owned by either.

---

# 109. Project dependency graph

The Adapter SHOULD eventually expose a structured dependency graph for:

```text
Routes
Buses
Processors
Sources
Components
```

This graph is essential for safe selective materialisation.

---

# 110. Change recording architecture

The first implementation SHOULD NOT attempt to log every Ardour edit event permanently.

Instead:

```text
working activity notifications
        +
capture-time structured comparison
```

is likely safer.

Events can be used for cache invalidation/optimization.

Captured state remains authoritative.

---

# 111. Why not event sourcing

Ardour's editor action stream is not guaranteed to be a stable OMVCS history representation.

OMVCS wants meaningful snapshots/Revisions, not every mouse movement.

---

# 112. Adapter event use

Useful internal events:

```text
Route added
Route deleted
Source created
Source changed
Processor changed
Automation changed
Tempo changed
```

can mark semantic areas dirty.

At Publish Version, the Adapter confirms state.

---

# 113. Native Undo history

Ardour's Undo/Redo history is Working State behaviour.

It SHOULD NOT automatically become OMVCS Revision history.

It MAY be captured as native state if required for exact session restoration, but OMVCS does not interpret it as creative ancestry.

---

# 114. Session history save/restore

Ardour's source contains explicit session history save/restore concepts, which may be useful for preserving or reconstructing native editing state, but this remains distinct from OMVCS history. This separation is important because native DAW edit history and OMVCS creative Revision history serve different purposes. 

---

# 115. Autosave

Ardour autosave may assist Working State recovery.

The Adapter MUST NOT publish autosaves as OMVCS Versions.

---

# 116. Crash recovery

If Ardour recovers unsaved work:

```text
Recovered Ardour state
        ↓
OMVCS Working State
```

The user may then Save Version.

OMVCS history is not retroactively rewritten.

---

# 117. Ardour UI integration areas

The reference fork SHOULD provide OMVCS UI in at least:

```text
main project/session interface
history/version panel
current changes
Contribution review
storage/project settings entry point
```

Exact placement belongs to implementation.

---

# 118. Primary Ardour OMVCS control

A compact status/control should probably show:

```text
Open Music
Main · Version 14
2 unpublished changes

[Save Version]
[History]
```

This keeps everyday workflow simple.

---

# 119. Changes panel

Potential Ardour panel:

```text
Changes since Main 14

Bass
  New recording

Lead Vocal
  Automation changed

Room Reverb
  Wet 18% -> 24%
```

with:

```text
Save Version
```

---

# 120. History panel

The Ardour-integrated history should allow:

```text
play Reference Render
open Version
compare
start alternate direction
inspect changes
```

without requiring the browser Platform.

---

# 121. Component history in Ardour

Right-click/context action on a Track mapped to a Component MAY expose:

```text
Open Music
  Component history
  Try another version
  View provenance
```

This is a powerful native integration.

---

# 122. Contribution integration UI

Inside Ardour:

```text
Open Music Contributions
```

might show:

```text
Anna — Cello idea

Added:
Cello

Changed:
Drums

[Play]
[Open]
[Use Cello]
[Review all changes]
```

---

# 123. Storage UI placement

Deep storage administration SHOULD probably not dominate Ardour.

Ardour can expose:

```text
Storage healthy
```

and direct the user to Project settings for:

```text
migration
replicas
provider configuration
```

---

# 124. Threading model

The Adapter MUST respect Ardour's real-time architecture.

The following MUST NOT run on the audio process thread:

```text
Resource hashing
cloud transfer
JSON canonicalization
repository synchronization
Platform requests
large semantic comparisons
garbage collection
```

---

# 125. Background worker architecture

OMVCS integration SHOULD use background workers for:

```text
hashing
upload
download
verification
Platform sync
semantic diff where safe
```

Ardour UI receives status asynchronously.

---

# 126. Native-state access threading

Any Ardour state inspection/mutation requiring its main/UI thread must be scheduled there.

The Adapter boundary should hide this from Core.

---

# 127. Recording priority

While recording:

```text
network/background work MAY continue only if demonstrably safe
```

but CPU/disk-heavy verification SHOULD be throttled or paused.

---

# 128. Disk I/O contention

Hashing 50 GB of audio while recording to the same disk could cause dropouts.

The reference client SHOULD implement:

```text
recording-aware I/O throttling
```

or pause background Resource processing.

---

# 129. Publication pipeline

Recommended Ardour Publish Version sequence:

```text
User presses Save Version
        |
Adapter checks recording state
        |
Adapter captures coherent native state
        |
Adapter enumerates Resources
        |
Core identifies changed/new Resources
        |
background hashing/storage
        |
Adapter creates Reference Render if requested
        |
Core verifies durability
        |
Core creates Project State + Revision
        |
Core moves Line
        |
Platform mirror sync
        |
Ardour UI shows Version saved
```

---

# 130. Publication should not freeze DAW unnecessarily

Only the state-capture boundary should require brief synchronization.

The long upload process should not block music editing if Core can safely preserve the captured snapshot.

---

# 131. Editing while publication uploads

This is desirable:

```text
capture Version candidate
        |
user continues editing
        |
background publication completes
```

But we then need to distinguish:

```text
captured Version state
```

from:

```text
newer Working State
```

The UI might say:

```text
Version 15 is being saved in background.
You have made 2 newer changes.
```

---

# 132. Background publication transaction

The Adapter must preserve captured state/resources long enough for background publication to complete even if Current Work changes.

This argues strongly for immutable staging/snapshot capture.

---

# 133. Publication completion while Working State moved on

If Version 15 completes after new edits:

```text
Current Working State base may logically advance to Version 15
```

only if the subsequent local changes can still be understood as descendants of the captured state.

The Core Specification may need careful implementation logic here.

The Adapter MUST NOT reset current work.

---

# 134. Simple 0.1 alternative

For initial implementation, we MAY constrain:

```text
Save Version waits until local capture and durable Resource staging complete
```

before returning control.

Uploads to remote storage could continue later only if local durable transaction semantics are sound.

Correctness first.

---

# 135. Local capture staging

A local OMVCS staging area may hold:

```text
captured session state
new Resource bytes
Reference Render
transaction metadata
```

until publication finishes.

It must be recoverable after crash.

---

# 136. Capture recovery

If Ardour crashes after Capture but before publication completion:

```text
local OMVCS transaction
```

should permit:

```text
Resume publication
Discard unpublished capture
```

without losing the original DAW work.

---

# 137. Adapter errors

Ardour-specific error codes should be namespaced.

Examples:

```text
OMVCS_ARDOUR_SESSION_NOT_BOUND
OMVCS_ARDOUR_RECORDING_ACTIVE
OMVCS_ARDOUR_STATE_CAPTURE_FAILED
OMVCS_ARDOUR_SOURCE_CHANGED_DURING_CAPTURE
OMVCS_ARDOUR_PLUGIN_MISSING
OMVCS_ARDOUR_ROUTE_BINDING_AMBIGUOUS
OMVCS_ARDOUR_RESTORE_FAILED
OMVCS_ARDOUR_COMPONENT_INTEGRATION_CONFLICT
```

They map to generic Adapter error classes.

---

# 138. Error detail

Example:

```json
{
  "code": "OMVCS_ARDOUR_SOURCE_CHANGED_DURING_CAPTURE",
  "class": "capture",
  "recoverable": true,
  "context": {
    "source_id": "..."
  }
}
```

---

# 139. Missing Source during restore

If a Resource should exist but Core cannot materialise it:

```text
Core handles storage availability failure
```

before Adapter Restore where possible.

The Adapter must not substitute an arbitrary file with the same filename.

---

# 140. Ardour missing-file mechanisms

Ardour may offer its own missing-source replacement mechanisms.

The Adapter MUST be careful not to silently trigger them during exact OMVCS restore.

User-directed substitution may be allowed as degraded Working State.

---

# 141. User substitutes missing audio

If user intentionally chooses another file:

```text
Working State becomes modified
Reproducibility = partial/modified
```

Publishing creates new history.

The historical Version remains unchanged.

---

# 142. Session validation

After Restore, Adapter SHOULD inspect:

```text
Session UUID/binding
Route count/mappings
Source associations
critical native state
Dependencies
```

and return validation levels.

---

# 143. Full normalized state re-capture

A strong validation method:

```text
restore state
        |
capture normalized state again
        |
compare against expected normalized Adapter State
```

Potentially expensive, but useful for conformance testing.

---

# 144. Golden Ardour Project

The reference Adapter MUST include at least one Golden Project.

It should deliberately exercise:

```text
audio track
MIDI track
multiple Routes
Bus
Plugin
Plugin automation
Track automation
Tempo change
Markers
Alternate Playlist
External Source
Muted Route
Hidden Route
Reference Render
```

---

# 145. Extended Golden Project

A second, more complex fixture SHOULD test:

```text
sidechains
sends
multiple buses
complex routing
several plugin formats
alternate takes
grouped OMVCS Components
Contribution integration
```

---

# 146. Repeat capture test

Critical test:

```text
Open Golden Project
Capture A
Do nothing
Capture B
```

Expected:

```text
same Resource IDs
same normalized Adapter State ID
same Project State ID
```

---

# 147. Rename test

Rename Bass Track.

Expected:

```text
same Creative Component ID
same Source Resource IDs
new Adapter State
semantic change: track/component display name changed
```

---

# 148. Region move test

Move Bass Region.

Expected:

```text
same audio Resource ID
new Adapter State
same Bass Component identity
new Bass Component State if component state captures arrangement state
semantic diff: region moved
```

---

# 149. Plugin parameter test

Change reverb wet amount.

Expected:

```text
audio Resource IDs unchanged
Adapter State changed
Project State changed
semantic diff identifies processor parameter where supported
```

---

# 150. Record new take test

Record new Bass take.

Expected:

```text
new audio Resource Object
new/updated Component State
new Adapter State
```

Old Bass Resource remains reachable through history.

---

# 151. Alternate playlist test

Switch active Playlist.

Expected:

```text
underlying Sources may remain identical
Adapter State changes
Component State changes
inactive playlist state preserved
```

---

# 152. Routing test

Change Bass output from:

```text
Band Bus
```

to:

```text
Master
```

Expected:

```text
no audio Resource change
Adapter State change
semantic routing change
```

---

# 153. Missing plugin test

Restore Golden Project without plugin X.

Expected:

```text
project opens if Ardour permits
plugin state preserved
dependency missing
reproducibility partial
Reference Render playable
```

---

# 154. Full restore test

Capture Project A.

Delete Working Session.

Materialise from OMVCS.

Expected:

```text
Ardour session opens
all required Sources resolve
bindings restore
normalized state matches
exact or defined reproducibility level
```

---

# 155. Path independence test

Capture on:

```text
C:\OM\Song
```

restore to:

```text
/home/test/othername
```

Expected:

```text
same OMVCS Project identity
all Sources restored
Ardour opens correctly
```

---

# 156. Platform-offline test

Disable Platform.

Expected:

```text
Capture
Save Version
Restore history
Component browsing
```

still work with Repository Home/storage available.

---

# 157. Storage migration test

Move Resource storage.

Ardour project should continue to open without Adapter-level creative state changes.

---

# 158. Component replacement test

Current:

```text
Bass B7
Drums D2
Vocal V4
```

Materialise historical B3.

Expected:

```text
Bass changes
Drums/Vocal preserved
required routing handled
Custom Working State
no Revision automatically created
```

---

# 159. Selective Contribution test

Contribution changes:

```text
Cello
Drums
```

User selects:

```text
Cello
```

Expected:

```text
only intended compatible Cello state integrated
Drums preserved
provenance result returned to Core
```

---

# 160. Conflict Contribution test

Contribution requires incompatible Tempo Map.

Expected:

```text
Integration Plan reports conflict
Working State unchanged until user decision
```

---

# 161. Unknown native state test

Inject unsupported Ardour/extension state.

If preserved:

```text
capture allowed with preserved_but_uninterpreted
```

If unpreservable and relevant:

```text
capture degraded/fails
```

Never silently discard.

---

# 162. Autosave recovery test

Simulate Ardour crash and recover autosave.

Expected:

```text
recovered state is Working State
not automatic OMVCS Version
```

---

# 163. Native snapshot test

Create Ardour snapshot.

Expected:

```text
snapshot preserved in Ardour state
no OMVCS Revision automatically created
```

---

# 164. Adapter capability plan

Expected target for reference Adapter:

```text
project_identification          REQUIRED
full_state_capture              REQUIRED
full_state_restore              REQUIRED
resource_enumeration            REQUIRED
change_detection                REQUIRED
restore_validation              REQUIRED

component_mapping               YES
dependency_reporting            YES
reference_render                YES
reproducibility_validation      YES

semantic_diff                   TARGET
component_materialisation       TARGET
selective_integration           TARGET
structured_merge_assistance     TARGET
safe_inspection                 TARGET
```

---

# 165. Implementation phases

The implementation SHOULD deliberately proceed in stages.

### Phase A — reconnaissance and capture

Prove:

```text
bind Ardour Session
capture native state
enumerate Sources
hash Resources
restore exact whole Session
```

No rich UI required.

### Phase B — OMVCS Core integration

Prove:

```text
Save Version
History
Open Version
Lines
Reference Render
```

### Phase C — semantic integration

Add:

```text
Component mappings
semantic diff
dependency reporting
```

### Phase D — advanced musical workflows

Add:

```text
Component history
Component materialisation
Custom Working States
Contribution selective integration
```

### Phase E — polished reference UI

Integrate deeply into Ardour interface and harden cross-platform behaviour.

---

# 166. First vertical slice

The first complete proof should be extremely narrow:

```text
Open Ardour Project
        |
Save Version 1
        |
Change bass Region
        |
Save Version 2
        |
Open Version 1
        |
Ardour reproduces Version 1
        |
Open Version 2
        |
Ardour reproduces Version 2
```

This validates the core boundary before building advanced features.

---

# 167. Second vertical slice

Then:

```text
Version 1
        |
Version 2
        |
new Creative Direction from Version 1
        |
Version A1
```

Proves Line semantics.

---

# 168. Third vertical slice

Then storage:

```text
Resources initially on Storage A
        |
move to Storage B
        |
restore old Version
```

Proves storage independence.

---

# 169. Fourth vertical slice

Then collaboration:

```text
Contributor starts from Version 2
        |
adds Cello
        |
submits Contribution
        |
creator opens contribution
        |
integrates Cello
```

This begins validating the real Open Music value.

---

# 170. Core library integration

The Ardour fork SHOULD consume OMVCS Core through a well-defined library/API.

Conceptually:

```text
libomvcs-core
libomvcs-ardour-adapter
Ardour UI integration
```

This allows:

```text
Core unit testing
Mock Adapter testing
Ardour Adapter testing
other DAWs later
```

---

# 171. Do not place Core in Ardour codebase conceptually

Even if the initial build system vendors OMVCS Core into the Ardour fork, source organization SHOULD retain a clean boundary.

Bad:

```text
random OMVCS logic scattered through Session.cc
```

Preferred:

```text
OMVCS integration hooks in Ardour
        ↕
Ardour Adapter module
        ↕
OMVCS Core
```

---

# 172. Minimum Ardour modifications

Prefer:

```text
small stable hooks
Adapter access to native APIs
UI extension points
```

over:

```text
large invasive rewrites of Ardour internals
```

This reduces long-term maintenance burden.

---

# 173. Upstream independence

The Open Music Ardour fork does not need to be accepted upstream.

However, keeping changes reasonably isolated has major value:

```text
easier rebasing onto newer Ardour
easier code review
clearer adapter architecture
less accidental fork divergence
```

---

# 174. Potential upstreamable hooks

Some generic Ardour changes needed for OMVCS might be useful upstream independently.

Examples could include:

```text
clean state-capture API
stable state comparison helpers
Route export hooks
dependency enumeration
```

But upstream acceptance is not an OMVCS requirement.

---

# 175. Build targets

The reference implementation SHOULD ultimately build for:

```text
Windows
macOS
Linux
```

to demonstrate that the concept is not platform-specific.

Cross-platform packaging is part of reference-implementation quality, not DAW Adapter semantics.

---

# 176. CI

CI SHOULD build:

```text
OMVCS Core
Mock DAW Adapter tests
Mock Storage tests
Ardour Adapter unit tests
Ardour integration test harness
```

on all targeted operating systems where feasible.

---

# 177. Headless tests

As much Adapter functionality as possible SHOULD be testable without manual GUI interaction.

Ardour test harnesses or library-level Session setup should be used where feasible.

---

# 178. GUI tests

Only UI-specific behaviours should require GUI automation/manual testing.

Correctness of:

```text
capture
restore
mapping
hashing
integration planning
```

must not depend solely on clicking through the application.

---

# 179. Fixture versioning

Golden Ardour test Projects SHOULD be stored under test fixture control.

Expected normalized OMVCS outputs SHOULD be versioned.

This allows coding agents to detect unintended semantic drift.

---

# 180. Agent task boundaries

A coding agent implementing a specific Ardour integration feature should receive:

```text
relevant spec sections
exact Adapter operation
affected Ardour subsystem
input/output schema
invariants
conformance tests
accepted files/modules
```

not merely:

> "Add version control to Ardour."

---

# 181. Coding agent rules

Coding agents implementing the Ardour Adapter MUST obey:

> Do not put Ardour-specific semantics into OMVCS Core.

> Do not make Ardour snapshots OMVCS Revisions.

> Do not use Route names as Component identity.

> Do not use absolute paths as historical identity.

> Do not mutate immutable Resource cache objects through Ardour.

> Do not assume all files inside an Ardour session folder are required Resources.

> Do not exclude state merely because it is currently muted, hidden or inaudible.

> Do not claim exact restore because the Session merely opened.

> Do not discard unknown native Ardour state silently.

> Do not run hashing/network operations on Ardour's real-time audio thread.

> Do not create new OMVCS Versions for native Save or Autosave.

> Do not overwrite Current Work when opening historical Versions without the required safety workflow.

> Do not treat plugin availability as equivalent to plugin-state preservation.

> Do not implement creative conflict resolution with timestamp/file precedence.

> Do not make Platform connectivity necessary for local Ardour version history.

---

# 182. Ardour mapping summary

The initial conceptual mapping is:

| OMVCS concept | Ardour reference mapping |
|---|---|
| Project | Bound Ardour Session + independent OMVCS Project ID |
| Adapter State | Canonical OMVCS metadata object containing bindings and references to Ardour native-state Resources |
| Resource | Audio/MIDI/native state/support files required by Session |
| Creative Component | Usually one or more Routes/Tracks |
| Component State | State of bound creative Route/group within a Project State |
| Working State | Current editable Ardour Session |
| Capture | Native Session serialization + Resource enumeration |
| Materialisation | Construct/open Ardour Working Session from OMVCS state |
| Semantic Diff | Structured comparison of Ardour Session entities |
| Dependency | Plugin/hardware/external requirement |
| Reference Render | Ardour native export of captured mix |
| Selective Integration | Ardour-aware import/reconciliation of selected Component state |

---

# 183. Areas requiring exact Ardour source mapping before coding

This design intentionally does not invent exact APIs where we have not verified them.

Before implementation, coding tasks must map and document exact Ardour classes/functions for:

```text
Session state capture
Session restoration
Session UUID
Route stable IDs
Track/Route enumeration
Source enumeration
Playlist enumeration
Region state
Processor/plugin state
Automation
Tempo map
Locations
Route export/import
native session snapshots
dependency/plugin enumeration
export/render engine
dirty-state notifications
recording state
thread/main-loop scheduling
```

This is implementation reconnaissance inside this design, not another specification.

---

# 184. Required source-mapping table

Before coding begins, this document SHOULD gain an implementation appendix/table:

```text
OMVCS requirement
Ardour class
Ardour method/signal
file
threading requirements
notes
```

Example:

| Requirement | Ardour mapping |
|---|---|
| Capture full Session | `ARDOUR::Session` / `save_state(...)` |
| Enumerate possible states | `Session::possible_states(...)` |
| Archive Session | `Session::archive_session(...)` |
| Export Route state | `Session::export_route_state(...)` |
| Session UUID | `Session::_uuid` / public accessor to verify |
| Save synchronization | `Session::save_state_lock` |

Several of these mappings are already visible in current Ardour source and give us concrete starting points.  

---

# 185. Exact method accessibility

A method existing in Ardour source does not necessarily mean it is exposed in the right API layer.

During implementation reconnaissance we must determine:

```text
public API available
protected/private
requires friend/access change
safe for external Adapter
requires small Ardour integration hook
```

The design should favor explicit integration hooks over abusing private internals.

---

# 186. State serialization stability

The Adapter MUST not assume Ardour's XML/native format is eternally stable.

Historical state must record:

```text
Ardour version
native state format/version
Adapter State schema
```

The Adapter is responsible for compatibility/migration behaviour.

---

# 187. Old Ardour Version preservation

If future Ardour cannot load a historical format:

```text
historical Resource remains valid
```

but:

```text
reproducibility = unavailable/partial
```

unless a compatible old Adapter/runtime exists.

OMVCS history remains interpretable regardless.

---

# 188. Optional reference Ardour builds

Long-term archival quality might benefit from retaining:

```text
which Ardour build/version created a Revision
```

and possibly links to known compatible releases.

OMVCS itself should not need to archive executable binaries to preserve history semantics.

---

# 189. Cross-DAW portability

The Ardour Adapter SHOULD expose portable Component forms where possible.

Examples:

```text
raw audio
MIDI
Reference Render
lyrics/text
```

These can be understood by foreign adapters more easily than Ardour-native Route state.

---

# 190. Native fidelity versus portability

A published Revision may therefore associate:

```text
native Ardour state through its referenced Adapter State
+
portable Resources through their owning Component States as applicable
+
Reference Render where the applicable approved historical schema defines that association
```

This illustration does not add top-level fields to the closed Project State body defined by Core Specification section 13. Reference Render policy and association remain subject to the applicable Core decision and versioned schema.

This is valuable because it gives us:

```text
maximum exactness in Ardour
maximum reasonable accessibility elsewhere
```

without pretending all DAWs share one native format.

---

# 191. Optional interchange export

Future Ardour Adapter versions MAY additionally expose:

```text
DAWproject
AAF
MIDI/stems package
```

as explicit interchange representations.

These are supplemental.

They must not replace native Ardour state as the authoritative exact state unless equivalence is proven.

---

# 192. Open Music value demonstrated by Ardour

The reference Adapter should demonstrate that Open Music can provide:

```text
version history deeper than DAW snapshots
storage independent of the DAW
creator-controlled storage
selective musical history
contributions backed by actual state
provenance
Platform independence
```

Ardour itself remains what it already is:

> the workstation where the music is edited.

---

# 193. Reference implementation completion criteria

The Ardour reference implementation is sufficiently complete for the intended "here it is, fork it, develop it" release when it can reliably demonstrate:

```text
Create/adopt Project
Save Versions
Open historical Versions
Create alternate Creative Directions
Restore exact Ardour states where dependencies permit
Preserve Resources independently from storage location
Move storage without history change
Generate Reference Renders
Show meaningful change summaries
Map Creative Components
Create and review Contributions
Integrate at least simple independent Components
Create Releases
Recover without Open Music Platform
Export/import Project
Build and run on intended desktop platforms
Pass documented conformance tests
```

Not every imaginable advanced merge must be solved.

The system must be coherent and extensible.

---

# 194. Features that may remain explicitly experimental

At initial public reference release, it is acceptable to label some capabilities experimental, particularly:

```text
complex selective integration
deep plugin semantic diff
cross-DAW conversion
safe execution sandboxing
complex grouped Component reconstruction
```

What is implemented MUST be truthful and testable.

---

# 195. What may not remain fake

The following MUST actually work before the reference implementation is claimed coherent:

```text
whole-Project capture
whole-Project restore
immutable Version history
storage independence
basic Component identity
Resource integrity
Platform independence
Project recovery
```

These are foundational.

---

# 196. Ardour-specific unresolved decisions

The following questions remain inside this design and must be resolved during implementation reconnaissance:

1. Exact Ardour Session extension location for OMVCS Project binding.
2. Whether native Session XML or an OMVCS sidecar is preferable for binding metadata.
3. Exact Ardour API/accessor for Session UUID.
4. Exact Resource set required for lossless Session reconstruction.
5. Exact handling of alternate Playlists and unused-but-referenced Sources.
6. Exact distinction between creatively relevant and local-only Ardour state.
7. Whether native `save_state` or temporary snapshot creation is the safest Capture primitive.
8. Whether Ardour Route export/import is sufficiently complete for OMVCS Component materialisation.
9. Exact mapping between Route IDs and Creative Components.
10. How grouped multi-Route Components are represented.
11. Exact plugin/dependency enumeration API.
12. Exact normalization rules for deterministic semantic Adapter State.
13. Exact semantic-diff coverage for 0.1.
14. Exact mechanisms for preventing Resources from changing during hashing.
15. Exact threading constraints for state capture and restoration.
16. Exact export/render API for deterministic Reference Render.
17. Exact behaviour when Ardour upgrades historical Session formats.
18. Exact safe-inspection possibilities for third-party Session/plugin state.
19. Exact restore-validation strategy.
20. Exact packaging model for OMVCS Core and Adapter inside the Ardour fork.

These are implementation questions inside the **Ardour Reference Adapter Design**. They do not create another top-level specification.

---

# 197. First implementation reconnaissance checklist

Before the first production code task, we should inspect and map the Ardour source for:

```text
Session lifecycle
Session::save_state
Session::load_state
snapshot handling
SessionDirectory
Source classes and source graph
Track / Route classes and IDs
Playlist
Region
Processor
PluginInsert/plugin identity
AutomationControl / AutomationList
TempoMap
Locations
export system
Session archive logic
Route export/import
signals for state change
recording lifecycle
dirty state
UI integration points
build system
tests
```

That is the correct next step after the design set is stabilized.

---

# 198. Final architecture

The complete reference path is:

```text
                    MUSICIAN
                       |
                       v
                    ARDOUR UI
                       |
                       v
             ARDOUR OMVCS ADAPTER
                       |
           +-----------+-----------+
           |                       |
           v                       v
      ARDOUR STATE             OMVCS CORE
                                  |
                 +----------------+---------------+
                 |                                |
                 v                                v
          STORAGE ADAPTER                  PLATFORM PROTOCOL
                 |                                |
                 v                                v
        CREATOR STORAGE                    OPEN MUSIC SITE
```

Ardour interprets and edits music.

OMVCS remembers creative history.

Storage holds immutable Resources.

The Platform coordinates people.

None of those layers owns the others.

---

# 199. Architectural statement

The reference Adapter can be summarized in six rules:

> **Use Ardour's native state model rather than rebuilding a DAW inside OMVCS.**

> **Preserve Ardour's complete editable state, but expose musically meaningful Components above it.**

> **Treat audio/MIDI/native-state bytes as immutable OMVCS Resources while keeping storage location separate.**

> **Use Ardour-aware semantics for changes and integration rather than file-level merging.**

> **Keep OMVCS Core completely ignorant of Ardour internals.**

> **Prove the generic Adapter contract with Ardour so another DAW can later implement the same contract without using Ardour at all.**

---

## Document status

**Document:** Ardour Reference Adapter Design  
**Version:** 0.1 Draft  
**Normative:** Reference design, subordinate to normative OMVCS specifications  
**Depends on:** all seven preceding OMVCS documents  
**Top-level OMVCS design set:** **Complete at Draft 0.1**
