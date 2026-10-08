# OMVCS DAW Adapter Specification
## Open Music Version Control System
### Draft 0.1

This document defines the normative contract between **OMVCS Core** and a Digital Audio Workstation integration.

It is subordinate to:

1. **OMVCS Glossary**
2. **OMVCS Core Invariants Specification**
3. **OMVCS Core Specification**

If this specification conflicts with a Core Invariant, the invariant takes precedence.

The purpose of this document is to make it possible for Ardour, Logic, Ableton Live, Cubase, Reaper, Studio One, Bitwig, or another DAW to integrate with the same OMVCS Core without requiring OMVCS to understand each DAW's internal project model.

The governing architectural rule is:

> **OMVCS owns history. The DAW Adapter owns interpretation.**

---

# 1. Scope

The DAW Adapter is responsible for translating between:

```text
DAW-native project/session state
```

and:

```text
OMVCS Project State,
Creative Components,
Resources,
Adapter State,
Change Descriptions,
Dependencies,
Reproducibility information.
```

The Adapter MUST provide enough information for OMVCS Core to:

- identify the current Project;
- capture the DAW's current editable state;
- enumerate Resources;
- associate DAW entities with OMVCS Creative Components where possible;
- detect relevant changes;
- create a publishable Project State;
- materialise a historical Project State;
- validate restoration;
- report missing dependencies;
- report reproducibility limitations;
- optionally provide semantic differences;
- optionally support component-level replacement;
- optionally assist Creative Integration;
- optionally create a Reference Render.

The Adapter MUST NOT implement independent OMVCS history.

It MUST NOT decide:

- Revision ancestry;
- Line semantics;
- Release semantics;
- Contribution history;
- storage replication;
- storage migration;
- repository garbage collection;
- Platform Protocol behaviour.

Those belong to OMVCS Core or other fixed specifications.

---

# 2. Adapter architecture

Conceptually:

```text
+--------------------------------------------------+
|                     DAW                          |
|                                                  |
| tracks / clips / plugins / routing / automation |
+-------------------------+------------------------+
                          |
                          |
                  DAW-native API/state
                          |
                          v
+--------------------------------------------------+
|                OMVCS DAW Adapter                 |
|                                                  |
| capture / restore / map / validate / describe    |
+-------------------------+------------------------+
                          |
                   Adapter Contract
                          |
                          v
+--------------------------------------------------+
|                   OMVCS Core                     |
|                                                  |
| history / resources / revisions / contributions |
+--------------------------------------------------+
```

The Adapter boundary MUST remain explicit.

---

# 3. Adapter identity

Every Adapter implementation MUST expose a globally unique stable Adapter Identifier.

Recommended form:

```text
reverse-DNS
```

Examples:

```text
org.openmusic.ardour
com.apple.logic.omvcs
com.ableton.live.omvcs
com.steinberg.cubase.omvcs
```

The Adapter Identifier MUST identify the logical adapter family, not one installation.

It MUST remain stable across ordinary Adapter updates.

---

# 4. Adapter version

The Adapter MUST expose:

```text
adapter_id
adapter_version
adapter_contract_version
native_format_versions_supported
```

Example:

```json
{
  "adapter_id": "org.openmusic.ardour",
  "adapter_version": "0.3.1",
  "adapter_contract_version": "omvcs.daw-adapter/0.1",
  "native_format_versions_supported": [
    "ardour-session/8",
    "ardour-session/9"
  ]
}
```

OMVCS Core MUST be able to distinguish:

> Adapter software version

from:

> OMVCS Adapter Contract version

from:

> DAW native project/state version.

---

# 5. Adapter capability model

Every Adapter MUST explicitly declare supported capabilities.

Capabilities MUST NOT be inferred from Adapter identity or DAW name.

A possible capability declaration:

```json
{
  "capabilities": {
    "project_identification": true,
    "full_state_capture": true,
    "full_state_restore": true,
    "resource_enumeration": true,
    "change_detection": true,
    "semantic_diff": true,
    "component_mapping": true,
    "component_materialisation": true,
    "selective_integration": false,
    "dependency_reporting": true,
    "reference_render": true,
    "reproducibility_validation": true
  }
}
```

A capability declared `false` MUST NOT be used by OMVCS Core.

An Adapter MUST NOT report a capability as supported if its behaviour is only approximate in a way that could cause silent data loss.

---

# 6. Capability classes

OMVCS 0.1 defines three broad classes.

## 6.1 Required capabilities

Every conforming DAW Adapter MUST support:

```text
project_identification
full_state_capture
full_state_restore
resource_enumeration
change_detection
restore_validation
```

Without these, the Adapter cannot provide minimum OMVCS version-control functionality.

---

## 6.2 Strongly recommended capabilities

Adapters SHOULD support:

```text
component_mapping
dependency_reporting
reference_render
reproducibility_validation
```

---

## 6.3 Optional advanced capabilities

Adapters MAY support:

```text
semantic_diff
component_materialisation
selective_integration
structured_merge_assistance
dependency_substitution
native_preview_render
live_resource_mapping
```

---

# 7. Capability truthfulness

Capability declarations are contractual.

If an Adapter declares:

```text
full_state_restore = true
```

it MUST either:

1. restore all state required by the Adapter's own definition of exact native restoration; or
2. fail and report why restoration is incomplete.

It MUST NOT silently restore a degraded approximation while returning success.

---

# 8. Project Identification

The Adapter MUST implement a logical operation equivalent to:

```text
IdentifyProject
```

Purpose:

> Determine whether the currently open DAW session is already associated with an OMVCS Project.

Possible outcomes:

```text
associated
unassociated
ambiguous
invalid_association
```

---

# 9. Project association metadata

The DAW project/session SHOULD contain a lightweight durable association to:

```text
OMVCS Project Identifier
```

and MAY additionally contain:

```text
last known Base Revision
Creative Component bindings
Adapter metadata version
```

Example conceptually:

```json
{
  "omvcs_project_id": "019cc...",
  "base_revision": "omvcs:revision:sha256:...",
  "adapter_binding_version": "1"
}
```

The exact storage mechanism is DAW-specific.

The Project Identifier MUST NOT be inferred from filename or folder name.

---

# 10. Project association safety

If the Adapter finds contradictory Project associations, it MUST NOT choose silently.

Example:

```text
Native session says Project A
Local OMVCS metadata says Project B
```

The Adapter MUST return an ambiguity or association error.

---

# 11. Capture operation

The Adapter MUST implement:

```text
CaptureState
```

Capture is the process of converting the current DAW Working State into structured OMVCS-compatible information.

Capture MUST NOT itself create a Revision.

Capture produces input for OMVCS Core.

---

# 12. Capture result

A Capture Result MUST include enough information for Core to construct or update:

```text
Creative Component States
Resource Objects
Adapter State
Project State
Dependency records
Change Descriptions
```

Conceptually:

```json
{
  "capture_id": "...",
  "project_id": "...",
  "base_revision": "...",
  "components": [],
  "resources": [],
  "adapter_state": {},
  "dependencies": [],
  "changes": [],
  "warnings": [],
  "reproducibility": {}
}
```

---

# 13. Capture consistency

Capture MUST represent one internally consistent DAW state.

The Adapter MUST prevent or detect state mutation during critical capture.

Acceptable mechanisms include:

- short DAW state lock;
- native snapshot;
- state-copy mechanism;
- transaction boundary;
- save-state hook.

The Adapter MUST NOT knowingly capture:

```text
half of track state before an edit
+
half after the edit
```

and report it as coherent.

---

# 14. Capture while playback is active

Each Adapter MUST define whether capture may occur while:

- playback is active;
- recording is active;
- rendering is active;
- background analysis is active.

At minimum:

> Capture during active recording MUST NOT silently truncate or inconsistently capture recorded media.

An Adapter MAY reject capture during recording.

---

# 15. Resource enumeration

The Adapter MUST implement:

```text
EnumerateResources
```

It MUST identify every Resource required to preserve the captured Project State to the extent that the Adapter claims complete restoration.

Examples include:

- audio files;
- MIDI files;
- native DAW session/project state;
- embedded samples;
- clip data;
- plugin state blobs;
- automation state;
- referenced external files;
- video references where necessary.

---

# 16. Resource classification

Each Resource MUST include an Adapter-level classification.

Example:

```json
{
  "native_resource_id": "ardour-source-4832",
  "logical_name": "Bass.wav",
  "role": "audio-source",
  "required": true,
  "mutable_source": false,
  "external": false,
  "path": "/working/path/Bass.wav"
}
```

These fields describe capture.

The eventual OMVCS Resource Object identity is determined by Core from content.

This Adapter-level capture classification is not the generic historical Resource Reference. Logical names and paths MUST NOT be copied into a generic Resource Reference. If a particular name is required for exact native reconstruction, that naming state belongs in Adapter State.

---

# 17. Required versus optional Resources

The Adapter MUST distinguish:

```text
required Resource
```

from:

```text
optional Resource
```

A required Resource is necessary for claimed restoration behaviour.

An optional Resource improves experience but is not necessary for exact restoration as defined by the Adapter.

The Adapter MUST NOT label a Resource optional merely because it is inconvenient to capture.

---

# 18. External Resources

A DAW may reference files outside its project directory.

The Adapter MUST report these explicitly.

Example:

```text
/home/user/Samples/choir.wav
```

The Adapter MUST NOT assume that a future machine will have the same path.

For a durable OMVCS Revision, required external files MUST either:

- become Resource Objects;
- be represented as declared Dependencies under explicitly supported semantics;
- or cause publication to warn/fail according to policy.

---

# 19. Native DAW state

The Adapter MUST preserve enough native state to reconstruct the project according to its declared restoration level.

For some DAWs this may mean one native project file.

For others it may mean:

```text
project package
session XML
sidecar data
plugin state files
auxiliary indexes
```

OMVCS Core does not need to understand those resources.

---

# 20. Adapter State

The Adapter MUST produce an immutable **Adapter State** representation.

Every Adapter State MUST be a canonical OMVCS metadata object with a content-derived Adapter State Identifier. The Adapter State MAY reference one or more opaque Resource Objects containing native DAW state. A native Resource Object MUST NOT serve directly as the complete Adapter State.

The Adapter State describes how OMVCS Resources and Creative Components relate to the DAW-native project model.

Any array-valued collection field in Adapter State that participates in its content-derived identity MUST declare whether it is ordered or set-like in the Adapter State schema and MUST follow the canonical collection rules in Core Specification section 5.1. This applies to adapter-specific opaque metadata and extension data as well as the generic fields below. The generic `resources` collection is set-like; `component_bindings` is a JSON object map keyed by Creative Component Identifier and follows RFC 8785 object-member ordering solely. Each map value MUST be an adapter-specific binding record and MAY contain fields such as `binding_kind`, `native_ids`, and other metadata permitted by that Adapter State schema. The Creative Component Identifier MUST NOT be duplicated inside a value merely to repeat its map key unless a future schema has a separate justified need. Duplicate member names in Adapter State maps MUST be rejected before hashing or canonical serialization; no additional sorting transformation applies to map entries.

It MUST contain:

```text
adapter_id
adapter_state_schema
native_format information
native project resources
component bindings where available
adapter-specific opaque metadata
```

The `adapter_id` and exact `adapter_state_schema` version identify the Adapter context governing Adapter-owned Resource Reference `properties` in Adapter State. An Adapter MUST define the immutable interpretation-property meanings and structures valid for each supported schema version and MUST validate them before producing or admitting a property-bearing Resource Reference into valid historical state. The Adapter's schema MUST NOT admit presentation or logical names, filenames, Chunk or Chunk Manifest information, storage locations, endpoints, Replicas, credentials, provider metadata, or other physical reconstruction/storage information prohibited by Core Specification section 7.

For Adapter-supplied Resource Reference properties in any containing historical object, the Adapter contract MUST bind them to one exact versioned validation authority determinable from that object's schema/Adapter context. If the authority cannot be uniquely determined, the candidate remains unchecked and cannot be admitted as valid history. The Adapter MUST NOT add an independent Resource Reference property-schema identifier unless a future explicit decision establishes that the containing context cannot identify the validator.

Core owns generic Resource Reference structure, duplicate JSON member rejection, canonical validation, declared value shapes, nested collection classification, and historical admission mechanics. The applicable schema/Adapter authority owns semantic admissibility. Core MUST NOT infer it from key spellings, deny lists, heuristics, or DAW-specific knowledge.

If an Adapter schema context is unknown, unavailable, or non-unique, an implementation MAY preserve or transport candidate data as unchecked, but MUST NOT admit it as a valid historical Resource Reference or Adapter State, use it to create a valid historical identity, or commit it into valid OMVCS history. Validation status, callbacks, timestamps, signatures, and other validation evidence are operational and MUST NOT be added to historical Resource Reference fields or canonical identity.

Adapter conformance tests for each supported Adapter State schema version MUST include schema-approved property cases and semantic rejection cases for ADR-0007-excluded content, including content represented under alternate keys or nested values. They MUST establish that Core's admission result follows the applicable schema/Adapter validation result and that validation evidence does not enter canonical Adapter State bytes.

---

# 21. Adapter State identity

Adapter State MUST be content-addressed through the OMVCS canonical metadata mechanism as a canonical metadata object.

Changing Adapter State creates a new Adapter State object.

It MUST NOT be modified in place once referenced by a published Project State.

---

# 22. Opaque Adapter metadata

OMVCS Core MUST preserve Adapter-specific opaque metadata and MUST NOT semantically interpret its Adapter-owned values. Generic structure, canonicalization, duplicate-member rejection, and Resource Reference admission requirements remain governed by Core Specification §§5.1 and 7.

The Adapter MUST version its own opaque structure.

The Adapter State schema MUST declare the ordering semantics of every collection within opaque metadata that participates in Adapter State identity. An Adapter MUST reject duplicate elements in any set-like collection before producing the immutable Adapter State.

When a Resource Reference in Adapter State has `properties`, the Adapter MUST validate the property map and all nested values under the exact `adapter_state_schema` version identified by that state before treating the state as valid historical metadata. The applicable schema MUST declare all required value shapes and nested array classifications; unclassified arrays are invalid. A reference with no `properties` remains subject to generic Core validation only.

Example:

```json
{
  "adapter_id": "org.openmusic.ardour",
  "adapter_state_schema": "ardour-omvcs-state/1",
  "opaque": {
    "...": "..."
  }
}
```

---

# 23. Creative Component mapping

If capability:

```text
component_mapping
```

is supported, the Adapter MUST map stable DAW-native entities to OMVCS Creative Components.

Example:

```text
OMVCS Bass Component
    ↕
Ardour Route UUID 7c18...
```

The Adapter MUST NOT bind components using display names alone.

---

# 24. Stable DAW-native entity identity

For component mapping, the Adapter SHOULD use DAW-native stable identifiers whenever available.

Examples might include:

```text
track UUID
route ID
clip ID
region ID
instrument lane ID
```

If the DAW has no stable identifier, the Adapter MUST document and implement a deterministic fallback.

Name-only matching SHOULD NOT be considered sufficient.

---

# 25. Component creation

When a new musically meaningful DAW entity appears and the Adapter supports component mapping, it MAY propose creation of a new OMVCS Creative Component.

Example:

```text
New track: Cello
```

The Adapter MUST NOT create Project history itself.

It reports:

```text
new_component_candidate
```

and Core creates the Creative Component during publication.

---

# 26. Component deletion

Deleting a DAW track does not delete the Creative Component from OMVCS history.

The Adapter reports that the Component is absent from the current captured state.

Historical Component States remain intact.

---

# 27. Component rename

Renaming:

```text
Bass
```

to:

```text
Electric Bass
```

MUST NOT automatically create a new Creative Component if the underlying stable DAW entity remains the same.

The Adapter SHOULD treat this as metadata evolution.

---

# 28. Component split

If one DAW entity becomes several musically meaningful entities:

```text
Drums
```

becomes:

```text
Kick
Snare
Overheads
```

the Adapter MUST NOT invent lineage silently.

It MAY report:

```text
component_structure_changed
```

and provide candidate derivation relationships.

The user or higher-level workflow may confirm the intended Creative Component mapping.

This rule does not require parentage where the derivation is unknown. Confirmed, known lineage SHOULD be recorded in parent Component State references; an operation or provenance rule that explicitly requires preserving the derivation MUST do so.

---

# 29. Component merge

If several logical DAW entities become one:

```text
Backing Vocal Left
Backing Vocal Right
```

becomes:

```text
Backing Vocals Stem
```

the Adapter SHOULD report candidate provenance relationships where possible.

It MUST NOT erase earlier Component history.

---

# 30. Change detection

The Adapter MUST implement:

```text
DetectChanges
```

at least sufficiently to determine whether the current DAW state differs from the Base Revision.

Minimum allowed result:

```text
unchanged
changed
unknown
```

An Adapter MUST NOT report `unchanged` unless it has sufficient evidence.

---

# 31. Change detection layers

OMVCS defines three Adapter-facing change levels.

## 31.1 Binary/resource change

Example:

```text
Bass.wav content changed
```

## 31.2 Structural/component change

Example:

```text
Bass Component now references another Resource
Track added
Track removed
```

## 31.3 Semantic DAW change

Example:

```text
Vocal automation changed
Reverb plugin parameter changed
Tempo map changed
Track routing changed
```

The Adapter MAY support different levels for different entity types.

---

# 32. Semantic Diff capability

If:

```text
semantic_diff = true
```

the Adapter MUST be able to compare two supported Adapter States and return structured semantic differences.

Conceptual result:

```json
{
  "changes": [
    {
      "kind": "automation-change",
      "component_id": "019-vocal",
      "parameter": "volume",
      "scope": "00:31.400-00:44.120"
    },
    {
      "kind": "plugin-parameter-change",
      "component_id": "019-vocal",
      "plugin": "reverb",
      "parameter": "wet",
      "old": 0.18,
      "new": 0.27
    }
  ]
}
```

---

# 33. Semantic Diff is descriptive, not identity-defining

Semantic Diff MUST NOT determine Resource or Revision identity.

It explains changes.

The immutable Project States remain authoritative.

If semantic analysis is incorrect or incomplete, historical identity must still remain correct.

---

# 34. Semantic Diff confidence

Adapters SHOULD assign confidence where semantic interpretation is not exact.

Example:

```json
{
  "confidence": "exact"
}
```

or:

```json
{
  "confidence": "inferred"
}
```

An inferred semantic difference MUST NOT be presented as guaranteed fact.

---

# 35. Materialisation operation

The Adapter MUST implement the DAW-specific part of:

```text
RestoreState
```

or equivalent.

Core first resolves and retrieves required Resources.

The Adapter then reconstructs the DAW-native editable state.

---

# 36. Restore inputs

The Adapter receives conceptually:

```text
Project Identifier
Project State
Adapter State
materialised Resource paths
Creative Component bindings
requested restoration mode
```

The Adapter MUST NOT retrieve arbitrary historical data independently from OMVCS without an explicitly defined mechanism.

Core owns Resource resolution.

---

# 37. Restore result

The Adapter MUST return a structured result.

Example:

```json
{
  "status": "success",
  "native_project_location": "...",
  "restored_components": [],
  "missing_dependencies": [],
  "warnings": [],
  "reproducibility": "exact"
}
```

Possible statuses MUST include at least:

```text
success
success_with_warnings
partial
failed
```

---

# 38. Exact restore

`success` with exact reproducibility means:

> the Adapter has restored the Project State to the degree required by the Adapter's exact-restoration contract.

It MUST NOT use `success` merely because the DAW opened.

---

# 39. Partial restore

If some state cannot be restored but a usable DAW session can still be created, the Adapter MAY return:

```text
partial
```

It MUST identify what could not be restored.

Example:

```text
Plugin X unavailable.
Automation associated with Plugin X preserved but inactive.
```

---

# 40. Restore failure

The Adapter MUST return failure if proceeding would materially misrepresent the requested Project State.

Examples:

- required native project state corrupt;
- essential Resource missing;
- unsupported native format;
- structural mapping impossible;
- state parser failure.

---

# 41. Restore into existing working state

The Adapter MUST distinguish:

```text
replace current Working State
```

from:

```text
merge/materialise into current Working State
```

Core MUST NOT assume they are interchangeable.

The minimum required Adapter capability is full replacement restoration.

---

# 42. Component Materialisation capability

If:

```text
component_materialisation = true
```

the Adapter MAY restore or replace a specific Creative Component within an existing Working State.

Example:

```text
Replace current Bass with Bass State from Revision R12
```

without restoring the entire Project.

---

# 43. Component Materialisation safety

Before component substitution, the Adapter MUST determine whether the operation is structurally valid.

Examples of possible incompatibility:

- component assumes unavailable bus routing;
- component requires tempo context not present;
- component's native DAW entity no longer exists;
- plugin dependencies unavailable.

If safe substitution cannot be guaranteed, the Adapter MUST refuse or require explicit degraded behaviour.

---

# 44. Custom Working State support

Adapters supporting component materialisation SHOULD support constructing Custom Working States.

Example:

```text
Bass       Revision 4
Drums      Revision 9
Vocals     Revision 12
Current FX local edits
```

The Adapter MUST report resulting unresolved incompatibilities.

---

# 45. DAW structural dependencies

A Creative Component may not be self-contained.

For example:

```text
Vocal track
   -> Vocal Bus
       -> Master Reverb Bus
```

The Adapter MUST be capable of declaring structural dependencies when component-level operations are supported.

Conceptually:

```json
{
  "component_id": "vocal",
  "requires": [
    "bus:vocal",
    "bus:master-reverb"
  ]
}
```

---

# 46. Dependency reporting

If:

```text
dependency_reporting = true
```

the Adapter MUST enumerate known external dependencies required to reproduce the current state.

Examples:

- audio plugins;
- virtual instruments;
- sample libraries;
- external codecs;
- hardware processors;
- external applications;
- fonts used in score views if materially relevant;
- scripts/extensions.

---

# 47. Dependency identity

Dependencies SHOULD use stable vendor-defined identity where possible.

Example:

```json
{
  "kind": "audio-plugin",
  "format": "VST3",
  "identifier": "com.vendor.plugin.superverb",
  "name": "SuperVerb",
  "version": "4.2.1",
  "required": true
}
```

A display name alone SHOULD NOT be considered sufficient if a stable identifier is available.

---

# 48. Plugin state capture

Where a DAW exposes plugin-state serialization, the Adapter SHOULD preserve that state as part of DAW State.

If plugin state is stored as opaque binary data, OMVCS MAY version it as an opaque Resource Object.

Core need not understand it.

---

# 49. Plugin availability versus plugin state

The Adapter MUST distinguish:

```text
plugin state preserved
```

from:

```text
plugin implementation available
```

A Revision may preserve exact plugin-state bytes while being unreproducible because the plugin software is absent.

---

# 50. Dependency severity

The Adapter SHOULD classify Dependencies at least as:

```text
required
optional
substitutable
informational
```

This classification MUST reflect reconstruction impact, not user convenience.

---

# 51. Reproducibility assessment

If:

```text
reproducibility_validation = true
```

the Adapter MUST assess the current environment against a requested Project State.

Conceptual result:

```json
{
  "state": "partial",
  "issues": [
    {
      "kind": "missing-plugin",
      "dependency": "com.vendor.superverb",
      "affects": ["019-vocal"]
    }
  ]
}
```

---

# 52. Reproducibility states

OMVCS DAW Adapters use at minimum:

```text
exact
exact_with_external_dependencies
partial
render_only
unavailable
unknown
```

### exact

All required DAW state and Resources can be restored with no known loss.

### exact_with_external_dependencies

Exact restoration is possible and required dependencies are known and available, but those dependencies are not themselves stored by OMVCS.

### partial

A usable editable project can be reconstructed, but some original state cannot be reproduced.

### render_only

The editable state cannot be reconstructed faithfully, but a valid Reference Render is available.

### unavailable

The Adapter cannot reconstruct or meaningfully open the Project State.

### unknown

The Adapter cannot assess reproducibility reliably.

---

# 53. Reference Render capability

If:

```text
reference_render = true
```

the Adapter MUST be able to produce an audio render representing the captured Project State.

The Adapter SHOULD render through the DAW's native mix engine.

---

# 54. Reference Render consistency

A Reference Render MUST correspond to the same captured state being published.

The Adapter MUST NOT:

1. capture state;
2. allow substantial DAW changes;
3. render later;
4. claim the render represents the captured state.

The Adapter SHOULD use snapshot/frozen-state semantics where available.

---

# 55. Reference Render metadata

The Adapter SHOULD report:

```text
sample rate
bit depth
channel count
duration
normalisation behaviour
render range
render preset
```

The render becomes an ordinary immutable OMVCS Resource Object.

---

# 56. Render failure

Failure to produce a Reference Render MUST NOT silently substitute another audio file.

Whether publication may continue without a render is determined by OMVCS Core policy and the type of publication.

---

# 57. Preview Render

An Adapter MAY support lightweight preview rendering distinct from Reference Render.

Preview render:

- may be lossy;
- may be lower resolution;
- may be temporary;
- need not form part of historical Project State.

Reference Render is historical.

Preview Render is operational.

---

# 58. Selective Integration capability

If:

```text
selective_integration = true
```

the Adapter MUST assist in combining selected Component States from a Contribution into the target Working State.

Example:

```text
Contribution changes:
Cello
Drums
Reverb

User selects:
Cello only
```

The Adapter attempts to construct a valid Working State containing the selected Cello state.

---

# 59. Integration Plan

Before modifying the Working State, an Adapter supporting selective integration SHOULD produce an **Integration Plan**.

Conceptually:

```json
{
  "source_revision": "...",
  "target_revision": "...",
  "selected_components": ["cello"],
  "required_supporting_state": ["cello-bus"],
  "conflicts": [],
  "warnings": []
}
```

The Integration Plan is not itself a Revision.

---

# 60. Integration conflict

The Adapter MUST report conflict where selected states cannot be combined safely.

Examples:

- both sides changed tempo map incompatibly;
- both replace same track with structurally different versions;
- one contribution depends on a deleted bus;
- plugin routing conflicts;
- region timings target incompatible arrangement structure.

The Adapter MUST NOT silently choose one side.

---

# 61. Conflict classes

Adapters SHOULD report conflict categories such as:

```text
component_conflict
routing_conflict
timeline_conflict
tempo_conflict
plugin_conflict
automation_conflict
resource_conflict
structural_conflict
unknown_conflict
```

These categories are descriptive.

The user-facing terminology belongs to the Interaction Specification.

---

# 62. Merge assistance versus merge authority

The Adapter MAY propose a result.

The Adapter MUST NOT create OMVCS Revision history directly.

Workflow:

```text
Adapter proposes merged Working State
        |
User/Core accept result
        |
Core publishes Revision
```

---

# 63. Automatic integration

An Adapter MAY automatically combine changes only where the operation is deterministic and declared safe.

Example:

```text
Contributor adds cello track
Target independently changes lead vocal
```

An Adapter might determine that both can coexist without conflict.

It MUST NOT generalise this to arbitrary state.

---

# 64. Destructive integration safeguards

An integration operation that would remove or overwrite local unpublished work MUST require explicit acknowledgement from the caller.

Adapters MUST expose whether an operation is destructive.

---

# 65. Dirty state

The Adapter MUST expose whether the DAW has unsaved or otherwise uncommitted native changes.

Core MUST be able to distinguish:

```text
DAW says clean
```

from:

```text
OMVCS says unchanged
```

These are not necessarily identical.

---

# 66. Native save versus OMVCS publish

Native DAW save and OMVCS Publish Revision are separate concepts.

A user MAY save the DAW project repeatedly without creating OMVCS history.

OMVCS publication creates history.

The Adapter MUST NOT equate every native save with a Revision unless a future explicit workflow chooses that behaviour.

---

# 67. Autosave

DAW autosave state MUST NOT automatically become published OMVCS history.

Autosave MAY assist crash recovery.

It remains Working State unless explicitly published.

---

# 68. Crash recovery

The Adapter SHOULD integrate with native DAW crash-recovery mechanisms where possible.

Recovered local DAW work MUST remain Working State.

It MUST NOT be inserted retroactively into an existing Revision.

---

# 69. Native snapshot systems

If the DAW supports native snapshots, they MAY be used internally by the Adapter.

Native snapshots MUST NOT be treated as OMVCS Revisions unless explicitly imported/published.

The Adapter MUST preserve the distinction.

---

# 70. Adapter lifecycle

An Adapter instance conceptually progresses through:

```text
UNINITIALIZED
INITIALIZED
PROJECT_UNBOUND
PROJECT_BOUND
CAPTURE_READY
RESTORE_READY
ACTIVE
ERROR
SHUTDOWN
```

Exact internal state machines may differ.

The Adapter MUST reject operations that are invalid in its current lifecycle state.

---

# 71. Initialisation

On initialisation the Adapter MUST validate:

- compatible DAW version;
- compatible OMVCS Adapter Contract version;
- Adapter configuration;
- required native API availability.

Failure MUST be explicit.

---

# 72. Binding

Before project-specific operations, the Adapter binds to a particular open DAW session/project.

A bound Adapter MUST NOT accidentally capture another concurrently open project.

If the DAW supports multiple open projects, each binding MUST be unambiguous.

---

# 73. Rebinding

When the user switches projects inside the DAW, the Adapter MUST detect or be notified of the change.

It MUST NOT continue using stale bindings.

---

# 74. Adapter operation identifiers

Long-running Adapter operations SHOULD use unique operation identifiers.

Example:

```text
capture-operation-UUID
restore-operation-UUID
render-operation-UUID
```

This enables cancellation, progress reporting and failure recovery.

---

# 75. Progress reporting

Long-running operations SHOULD report structured progress.

Example:

```json
{
  "operation_id": "...",
  "phase": "resource-enumeration",
  "completed": 47,
  "total": 120
}
```

The UI mapping belongs to the Interaction Specification.

---

# 76. Cancellation

Adapters SHOULD support safe cancellation for long-running operations where possible.

Cancellation MUST leave the DAW in a defined state.

An Adapter MUST document operations that cannot be safely cancelled.

---

# 77. Error model

Every Adapter error MUST include:

```text
error code
error class
human-readable summary
machine-readable context
recoverability
affected operation
```

Conceptually:

```json
{
  "code": "OMVCS_DAW_MISSING_DEPENDENCY",
  "class": "dependency",
  "recoverable": true,
  "summary": "Required plugin is unavailable",
  "context": {
    "plugin_id": "com.vendor.superverb"
  }
}
```

---

# 78. Error classes

At minimum, the Adapter specification recognises:

```text
configuration
compatibility
association
resource
dependency
capture
restore
validation
render
integration
conflict
native-daw
permission
cancelled
internal
```

---

# 79. Warnings versus errors

Warnings describe successful but noteworthy behaviour.

Errors describe a requested guarantee that was not achieved.

An Adapter MUST NOT downgrade a failure into a warning merely to continue.

---

# 80. Validation operation

The Adapter MUST implement:

```text
ValidateRestoredState
```

or equivalent.

After restoration, it verifies as much as possible that the DAW-native state corresponds to the requested Adapter State.

---

# 81. Validation levels

Validation MAY occur at several levels.

## Structural

Expected tracks/routes/entities exist.

## Resource

Expected Resources are attached.

## Native state

Expected session/project state loaded.

## Dependency

Required external dependencies available.

## Semantic

Known parameter/routing/automation state matches.

The Adapter MUST report which validation levels were actually performed.

---

# 82. Validation honesty

If the Adapter can validate only structural state, it MUST NOT claim semantic validation.

Example:

```json
{
  "validated": [
    "structural",
    "resource"
  ],
  "not_validated": [
    "plugin-runtime-behaviour"
  ]
}
```

---

# 83. Native project path independence

A captured Adapter State MUST NOT depend permanently on the original local filesystem path.

Example:

```text
C:\Users\Joakim\Music\Project
```

must not be necessary for successful restoration.

The Adapter MUST translate local paths into portable Resource relationships where required.

---

# 84. Temporary files

Temporary DAW files MUST be classified.

The Adapter SHOULD exclude ephemeral files that are not needed for reconstruction.

Examples may include:

- waveform cache;
- temporary peak analysis;
- disposable render cache;
- transient lock files.

The Adapter MUST document its classification rules.

---

# 85. Derived Resources

The Adapter MUST distinguish derived Resources that can be recreated from authoritative state.

Examples:

```text
waveform caches
analysis files
temporary freeze cache
```

Such files SHOULD NOT be treated as required historical Resources unless regeneration would materially alter the project.

---

# 86. Frozen/bounced tracks

A DAW may contain both:

```text
editable source state
```

and:

```text
frozen/bounced audio
```

The Adapter MUST preserve enough information to restore the user's actual DAW state.

It SHOULD identify the relationship where possible.

---

# 87. Alternate takes and playlists

If a DAW project contains non-active takes/playlists that remain part of editable project state, the Adapter MUST preserve them if `full_state_capture = true`.

It MUST NOT capture only currently audible media and silently discard alternatives.

---

# 88. Muted and hidden entities

Muted or hidden tracks may still be part of creative state.

The Adapter MUST preserve them where they form part of native project state.

Visibility does not imply historical irrelevance.

---

# 89. Unused but referenced media

The Adapter MUST distinguish:

```text
media referenced by current native project state
```

from:

```text
media merely present in project folder
```

Only folder presence is insufficient for determining historical relevance.

The Adapter SHOULD use the DAW's own source/reference graph where available.

---

# 90. Media consolidation

An Adapter MAY use DAW-native consolidation or archival mechanisms to gather external required media.

If used, it MUST preserve content identity correctly.

Consolidation MUST NOT silently alter audio content unless such transformation is explicitly part of the captured creative state.

---

# 91. Lossless transformation

A storage or project packaging transformation MAY change container representation while preserving audible content.

However, OMVCS Resource identity is byte-based.

Therefore:

> byte-changing transformations create new Resource Objects.

The Adapter MUST NOT pretend transformed bytes are the original Resource Object.

---

# 92. Format conversion

If the Adapter converts:

```text
WAV -> FLAC
```

or:

```text
AIFF -> WAV
```

the resulting file is a new Resource Object.

The Adapter MAY record derivation metadata.

It MUST NOT substitute it transparently for the original unless the applicable operation explicitly allows such behaviour.

---

# 93. DAW-native IDs and OMVCS IDs

OMVCS Component IDs and DAW-native IDs are separate namespaces.

A binding MAY look like:

```text
OMVCS Component:
019c...

DAW Native Entity:
route-7ff0...
```

The Adapter owns the mapping.

Core MUST NOT interpret native IDs.

---

# 94. Binding persistence

Component bindings SHOULD survive:

- save/reopen;
- ordinary track rename;
- ordinary DAW restart;
- OMVCS Revision switching where corresponding state exists.

If a binding becomes invalid, the Adapter MUST report it.

---

# 95. Binding reconstruction

If binding metadata is lost but stable DAW-native identities remain, the Adapter MAY reconstruct bindings.

Reconstruction confidence MUST be reported.

It MUST NOT silently guess ambiguous mappings.

---

# 96. DAW project created outside OMVCS

The Adapter MUST support initial onboarding of an existing DAW project.

Conceptual operation:

```text
AdoptProject
```

It performs:

```text
identify Resources
establish Creative Components
create Adapter State
associate with new/existing OMVCS Project
```

The resulting first Revision is created by Core.

---

# 97. Existing Project opened without local OMVCS metadata

If a DAW-native project contains a valid OMVCS Project association but local OMVCS metadata is missing, the Adapter SHOULD report:

```text
known Project Identifier
local repository unavailable
```

Core may then recover metadata from Repository Home.

The Adapter MUST NOT create a new Project automatically.

---

# 98. Foreign Adapter State

A DAW may encounter a Project State created by another Adapter.

Example:

```text
Project State includes:
Ableton Adapter State

Current environment:
Ardour Adapter
```

The Ardour Adapter MUST NOT claim it can restore Ableton state unless a defined conversion capability exists.

---

# 99. Cross-DAW transfer

Cross-DAW transfer is not automatically guaranteed by OMVCS.

There are two distinct cases.

## Native historical preservation

The original Adapter State remains preserved forever.

## Cross-DAW interpretation

Another Adapter MAY import common Components/Resources into its own Working State.

That produces a new Adapter State when published.

The original history remains unchanged.

---

# 100. Adapter conversion

An Adapter MAY support:

```text
ImportForeignState
```

or equivalent.

Example:

```text
DAWproject interchange
AAF
stems + MIDI
```

Such conversion MUST be explicit.

It MUST report loss or unsupported state.

---

# 101. Cross-DAW provenance

If a Project is converted from Adapter A to Adapter B and published, provenance SHOULD preserve:

```text
source Revision
source Adapter State
conversion relationship
known losses
```

---

# 102. Adapter State compatibility

An Adapter MUST specify which of its older Adapter State schema versions it can read.

Example:

```text
writes:
ardour-omvcs-state/3

reads:
1
2
3
```

It MUST NOT reinterpret unsupported schema versions.

---

# 103. Adapter migrations

An Adapter MAY internally migrate old Adapter State into a newer native representation during restoration.

Such migration MUST NOT rewrite the historical Adapter State.

If the migrated result is later published, it becomes a new Adapter State in a new Revision.

---

# 104. Native DAW format upgrades

Suppose:

```text
Revision created in Ardour 8
opened in Ardour 11
```

If Ardour upgrades its native session representation, the historical Resource Objects remain unchanged.

Opening them may create upgraded Working State.

Publishing that upgraded state creates new history.

---

# 105. Read-only historical restoration

An Adapter SHOULD permit restoration for audition or inspection even when exact editable publication is not possible.

Example:

```text
old project opens read-only
```

It MUST report the mode accurately.

---

# 106. Audition-only mode

An Adapter MAY provide:

```text
audition_only
```

where it cannot produce a fully editable DAW project but can reproduce sound sufficiently for listening.

This is distinct from Reference Render playback.

---

# 107. Project lock state

If the DAW project is locked by another process/user, the Adapter MUST report that condition.

It MUST NOT bypass native safety mechanisms without explicit defined behaviour.

---

# 108. Concurrent editing

The DAW Adapter Specification does not require real-time collaborative editing.

Two users may independently edit the same Base Revision.

Their work becomes divergent OMVCS history.

Adapters MUST NOT assume shared mutable project state.

---

# 109. External file mutation

If a Resource file referenced by the current Working State changes outside the DAW, the Adapter SHOULD detect this where technically possible.

At minimum, Capture MUST re-read/re-hash relevant Resources rather than relying solely on stale DAW dirty flags.

---

# 110. File timestamp distrust

Adapters MUST NOT use modification timestamps as Resource identity.

Timestamp MAY be used as an optimisation hint.

Content identity is authoritative.

---

# 111. Performance caches

Adapters MAY maintain local caches such as:

```text
last known resource hash
native entity fingerprint
semantic diff cache
dependency scan cache
```

Caches MUST be invalidatable.

They MUST NOT become historical truth.

---

# 112. Incremental capture

Adapters SHOULD support incremental capture where possible.

Rather than hashing all 80 GB every time, an Adapter may use:

```text
native change notifications
file identity
size
mtime as hint
cached hash
```

to avoid unnecessary reads.

But before creating a new Resource Object identity, content verification MUST ultimately be correct.

---

# 113. Large resource handling

The Adapter SHOULD stream large Resources to OMVCS Core rather than require loading them fully into memory.

Resource API design MUST support large multi-gigabyte files.

---

# 114. Streaming contract

The Adapter should expose Resources through:

```text
stream/read handle
known length if available
seek/range support if available
```

rather than only:

```text
byte[]
```

The exact programming-language API is implementation-specific.

---

# 115. Resource mutation during hashing

If a Resource changes while being captured, the Adapter/Core pipeline MUST detect or guard against inconsistent reads.

Possible strategies:

- file lock;
- native snapshot;
- size/mtime verification before and after;
- copy-to-staging;
- DAW-provided immutable source handle.

A knowingly inconsistent Resource MUST NOT be published.

---

# 116. Staging

Adapters MAY use a temporary staging area for capture or restore.

Staging files are operational and MUST NOT become historical identity unless intentionally captured as Resources.

---

# 117. Restore atomicity

Where practical, restoration SHOULD create or prepare the new native project state separately before replacing the user's current Working State.

This reduces risk of destructive partial restore.

---

# 118. Restore rollback

If restoration fails after changing native state, the Adapter SHOULD either:

- restore the previous Working State;
- leave a recoverable backup;
- or clearly report that rollback was incomplete.

Silent destructive failure is prohibited.

---

# 119. Working-state safety checkpoint

Before destructive operations such as:

```text
full restore
component replacement
selective integration
```

the Adapter SHOULD permit Core/UI to request a temporary Working State safety checkpoint.

This checkpoint is operational and need not become published history.

---

# 120. User intent boundary

The Adapter MUST NOT publish or create creative history merely because a technically significant DAW event occurred.

Examples:

```text
project saved
project closed
plugin scanned
waveform cache rebuilt
```

These events do not inherently mean:

```text
publish Revision
```

---

# 121. Adapter notifications

The Adapter MAY emit events such as:

```text
project_opened
project_closed
state_changed
resource_added
resource_removed
component_changed
dependency_changed
recording_started
recording_stopped
```

Events are advisory.

Core MUST NOT use them as immutable history without Capture.

---

# 122. Event coalescing

Adapters MAY coalesce high-frequency DAW events.

Example:

```text
500 automation point updates
```

may become:

```text
automation changed
```

until a semantic diff is requested.

---

# 123. Threading

The Adapter MUST respect DAW threading constraints.

If the DAW requires some operations on a UI or audio-engine thread, the Adapter is responsible for scheduling accordingly.

OMVCS Core MUST NOT assume direct arbitrary-thread access to DAW internals.

---

# 124. Audio-engine safety

Adapter operations MUST NOT block the real-time audio thread with:

- network I/O;
- hashing large files;
- JSON serialization;
- storage access;
- long database operations.

Real-time safety is the Adapter's responsibility.

---

# 125. Recording safety

Resource hashing/upload MUST NOT interfere with active recording performance.

Adapters SHOULD defer expensive operations or use background-safe mechanisms.

---

# 126. Offline mode

The Adapter MUST be usable for local Working State operations when the Open Music Platform is unavailable.

Capture MAY proceed offline.

Publication durability may be deferred if required creator-controlled storage is unavailable.

The Adapter itself MUST NOT depend on Platform availability for ordinary DAW editing.

---

# 127. Storage independence

The Adapter SHOULD generally receive already materialised local Resource paths from Core.

It MUST NOT need to know whether the Resource originally came from:

```text
S3
WebDAV
OneDrive
NAS
another contributor
```

That belongs to Storage/Core.

---

# 128. Storage credentials

A DAW Adapter MUST NOT require direct access to arbitrary OMVCS storage credentials unless explicitly necessary for a defined advanced capability.

The normal architecture is:

```text
Core / Storage Adapter -> materialised resource
DAW Adapter -> local resource
```

---

# 129. Platform independence

A DAW Adapter MUST NOT require an Open Music Platform account merely to interpret a local OMVCS repository.

Platform authentication belongs to the Platform Protocol.

---

# 130. Adapter signing

A DAW Adapter MAY attach attestations about capture/restore operations.

Such attestations MUST NOT modify historical object identities after creation.

---

# 131. Extension namespace

Adapters MAY include vendor-specific metadata under a namespaced extension area.

Example:

```json
{
  "extensions": {
    "org.openmusic.ardour": {
      "..."
    }
  }
}
```

Extension data that forms part of immutable Adapter State participates in its content hash.

---

# 132. Extension preservation

Conforming OMVCS implementations MUST preserve unknown Adapter extension data when performing lossless repository transfer.

They MUST NOT strip it merely because Core cannot interpret it.

---

# 133. Privacy

The Adapter MUST avoid capturing irrelevant private machine information into historical state.

Examples that SHOULD NOT be captured unless required:

```text
absolute username paths
computer hostname
local account email
temporary cache paths
license tokens
plugin activation secrets
```

---

# 134. Secret exclusion

Adapters MUST NOT include:

- licence keys;
- OAuth tokens;
- passwords;
- private cloud credentials;
- plugin activation secrets;

inside historical Adapter State.

If a native DAW file contains secret material, the Adapter MUST treat that as a security issue and define an appropriate mitigation.

---

# 135. Machine-specific state

Machine-specific data required only for local operation SHOULD be separated from portable Adapter State.

Example:

```text
audio device selection
ASIO/CoreAudio device ID
screen layout
local MIDI controller path
```

The Adapter SHOULD classify whether such state is:

```text
creative
portable operational
local-only
```

---

# 136. Creative versus local preference state

The Adapter MUST make a principled distinction between:

```text
state that changes the music
```

and:

```text
state that changes the local editing environment only
```

Example:

```text
plugin wet/dry value -> creative
window position -> local-only
```

Not every DAW preference belongs in Project history.

---

# 137. Ambiguous state classification

Where classification is uncertain, the Adapter SHOULD err on the side of preserving state if omission could change reproduction.

But it SHOULD avoid capturing irrelevant machine-private metadata.

This tension MUST be documented per Adapter.

---

# 138. Adapter conformance modes

OMVCS 0.1 defines three conceptual conformance levels.

## Level A — Core DAW Adapter

Supports:

```text
project identification
complete state capture
Resource enumeration
complete state restore
basic change detection
restore validation
```

## Level B — Collaborative DAW Adapter

Additionally supports:

```text
Creative Component mapping
dependency reporting
Reference Render
component-level materialisation
```

## Level C — Native OMVCS DAW Adapter

Additionally supports rich features such as:

```text
semantic diff
selective integration
deep provenance mapping
native OMVCS UI
rich dependency analysis
structured conflict reporting
```

These names may change before 1.0, but the capability distinction is useful.

---

# 139. Minimum OMVCS compatibility claim

A DAW integration MUST NOT call itself an **OMVCS DAW Adapter** unless it satisfies Level A.

A lighter importer/exporter may call itself:

```text
OMVCS interchange support
```

but not full Adapter support.

---

# 140. Conformance test requirements

Every DAW Adapter MUST pass a shared Adapter conformance test suite.

The suite must include at minimum:

```text
project association round-trip
capture deterministic state
resource enumeration completeness
save/reopen association
full restore
resource change detection
external resource handling
missing dependency reporting
missing Resource failure
state-validation reporting
schema-version handling
unknown extension preservation
```

---

# 141. Golden Project tests

Each Adapter SHOULD provide one or more **Golden Projects**.

A Golden Project is a small native DAW project deliberately containing representative features.

Example:

```text
audio track
MIDI track
plugin
automation
bus routing
alternate take
external sample
tempo change
```

The test suite captures it, restores it and verifies expected semantics.

---

# 142. Round-trip test

A required test pattern is:

```text
Native Project A
      |
    Capture
      |
 OMVCS State
      |
    Restore
      |
Native Project B
```

The Adapter MUST verify that B satisfies its declared restoration guarantees relative to A.

---

# 143. Repeat-capture stability

If the DAW state has not changed, repeated Capture SHOULD produce the same immutable Adapter State and Resource identities.

Non-deterministic noise SHOULD NOT cause meaningless historical churn.

---

# 144. Native volatile data

If the DAW embeds volatile values such as:

```text
last-opened timestamp
window coordinates
random session nonce
```

the Adapter SHOULD exclude or normalise them where they are not creatively relevant.

Otherwise every capture could appear different despite no musical change.

---

# 145. Determinism

Adapter-generated immutable metadata MUST be deterministic for equivalent captured state.

Ordering of semantically unordered collections MUST be canonicalised before hashing according to Core Specification section 5.1.

---

# 146. Resource ordering

Where ordering is meaningful, it MUST be preserved.

Where ordering is not meaningful in a collection included in hashed Adapter State, the Adapter MUST apply Core Specification section 5.1. Other Adapter outputs MUST use deterministic ordering where the applicable contract requires it.

---

# 147. Capture provenance

The Adapter MAY report technical capture provenance such as:

```text
adapter version
DAW version
native format version
operating system family
```

Only fields explicitly defined as historical participate in Adapter State identity.

---

# 148. Environment provenance

Machine environment details SHOULD be limited to those needed to understand reproducibility.

For example:

```text
DAW version
plugin format
plugin version
```

may matter.

```text
computer serial number
```

does not.

---

# 149. Unknown native state

If the Adapter encounters native DAW state it cannot interpret but can preserve losslessly as opaque state, it MAY capture it.

It MUST report:

```text
preserved_but_uninterpreted
```

where relevant.

---

# 150. Unpreservable native state

If state cannot be interpreted **or preserved**, and its loss could affect reconstruction, Capture MUST report failure or degraded capture.

It MUST NOT claim complete state capture.

---

# 151. DAW-specific optimisations

Adapters MAY optimise heavily for their DAW.

Examples:

- reuse native snapshot system;
- use DAW source graph;
- use stable route IDs;
- exploit native project archives;
- read dirty-state flags;
- invoke native bounce engine.

Such optimisations are encouraged provided they remain behind the Adapter Contract.

---

# 152. Core must remain ignorant

OMVCS Core MUST NOT require knowledge such as:

```text
Ardour Route
Logic Channel Strip
Ableton Device Chain
Cubase Track Version
```

If Core needs DAW-specific vocabulary to operate correctly, the boundary has been violated.

---

# 153. Adapter operation set

The normative logical Adapter operations for OMVCS 0.1 are:

```text
GetAdapterInfo
GetCapabilities

BindProject
UnbindProject
IdentifyProjectAssociation
WriteProjectAssociation

CaptureState
EnumerateResources
DetectChanges

RestoreState
ValidateRestoredState

ReportDependencies
AssessReproducibility
```

Optional operations include:

```text
DescribeSemanticChanges
RenderReference
MaterialiseComponent
BuildCustomWorkingState
CreateIntegrationPlan
ApplyIntegrationPlan
ImportForeignState
CreatePreviewRender
```

Exact language-specific function signatures are intentionally not fixed here.

The semantic contracts are normative.

---

# 154. GetAdapterInfo

Returns at least:

```text
adapter_id
adapter_version
adapter_contract_version
DAW name
DAW version
supported native formats
```

Failure:

```text
incompatible environment
adapter initialization failure
```

---

# 155. GetCapabilities

Returns explicit capability declaration.

The result MUST reflect the current Adapter/DAW environment.

A capability MAY depend on DAW version or platform.

---

# 156. BindProject

Inputs:

```text
native DAW project/session
optional OMVCS Project context
```

Effects:

```text
establish Adapter binding
```

No historical effects.

Failure includes:

```text
unsupported project
ambiguous project
incompatible native format
```

---

# 157. IdentifyProjectAssociation

Returns:

```text
associated Project ID
or unassociated
or ambiguous/invalid
```

MUST NOT guess from filename.

---

# 158. WriteProjectAssociation

Stores the OMVCS Project association into the DAW-native project or supported sidecar mechanism.

This operation MUST NOT by itself create a Revision.

---

# 159. CaptureState

Inputs:

```text
bound DAW state
Base Revision context
capture options
```

Output:

```text
Capture Result
```

Historical effects:

```text
none directly
```

Core decides publication.

---

# 160. EnumerateResources

Returns every required/optional native Resource relevant to current Capture.

Must be safe for large Resources.

---

# 161. DetectChanges

Inputs:

```text
current Working State
Base Project/Adapter State
```

Outputs:

```text
changed status
component/resource changes
optional semantic changes
```

No historical effects.

---

# 162. RestoreState

Inputs:

```text
target Project State
Adapter State
materialised Resources
restore mode
```

Outputs:

```text
restored Working State
restore result
```

Must report loss explicitly.

---

# 163. ValidateRestoredState

Inputs:

```text
expected Adapter State
current native DAW state
```

Outputs:

```text
validation levels
issues
reproducibility assessment
```

---

# 164. ReportDependencies

Returns structured Dependency records.

If capability is unsupported, the Adapter MUST say unsupported rather than return an empty dependency list that could be misread as “none”.

---

# 165. AssessReproducibility

Evaluates whether the requested Project State can be reconstructed in the current environment.

It MUST not modify history.

---

# 166. DescribeSemanticChanges

Optional.

Compares states and returns structured DAW-specific semantic changes translated into OMVCS-compatible categories.

---

# 167. RenderReference

Optional but strongly recommended.

Creates a render corresponding to a stable captured Project State.

It MUST report render settings.

---

# 168. MaterialiseComponent

Optional.

Replaces/inserts a selected Component State into current Working State.

Must validate structural dependencies.

---

# 169. BuildCustomWorkingState

Optional.

Combines specified Component States into a valid local DAW state where possible.

Must report conflicts.

---

# 170. CreateIntegrationPlan

Optional.

Analyses a Contribution and target Working State without changing the project.

Returns proposed actions/conflicts.

---

# 171. ApplyIntegrationPlan

Optional.

Applies a previously validated Integration Plan.

The result remains Working State until Core publishes it.

---

# 172. ImportForeignState

Optional.

Attempts explicit cross-DAW/import interchange.

Must report loss and provenance.

---

# 173. Operation preconditions

Every Adapter operation MUST define preconditions.

Example:

```text
CaptureState
requires:
    Adapter initialized
    project bound
    native state readable
```

Calling an operation with unmet preconditions MUST produce a defined error.

---

# 174. Idempotency

Read-only operations SHOULD be idempotent.

Examples:

```text
GetCapabilities
IdentifyProjectAssociation
ReportDependencies
AssessReproducibility
```

State-changing operations must document idempotency.

---

# 175. Operation timeouts

Adapters SHOULD expose timeout/cancellation behaviour for operations that may block on DAW-native mechanisms.

Core MUST NOT assume all Adapter calls return immediately.

---

# 176. UI independence

The Adapter Contract MUST be usable by:

```text
DAW-integrated GUI
standalone GUI
CLI test harness
coding-agent test harness
headless conformance suite
```

The contract therefore MUST NOT require human interaction inside the low-level operation itself unless explicitly defined.

---

# 177. User-choice requests

If an Adapter needs a user decision, it SHOULD return a structured choice request rather than opening arbitrary modal UI from deep inside the Adapter.

Example:

```json
{
  "needs_user_decision": {
    "kind": "ambiguous-component-binding",
    "options": [...]
  }
}
```

This allows different UI implementations.

---

# 178. Non-interactive mode

Adapters SHOULD support a non-interactive mode for automated testing.

In this mode, unresolved ambiguity MUST fail rather than prompt.

This is particularly important for coding-agent-driven development and CI.

---

# 179. Testability

Every mandatory Adapter operation MUST be invocable independently enough for automated conformance testing.

A design that can only be tested by manually clicking through the DAW is insufficient.

---

# 180. Mock Adapter

The OMVCS project SHOULD provide a **Reference Mock DAW Adapter** implementing the contract without a real DAW.

It may operate on a simple folder/JSON model.

Purpose:

```text
test OMVCS Core
test Adapter Contract
test failure semantics
test contribution workflows
test custom Working States
```

This Mock Adapter is not another top-level specification. It is a test/reference implementation of this specification.

---

# 181. Why the Mock Adapter matters

It ensures OMVCS Core can be developed independently of Ardour.

For example:

```text
Mock Project
  Bass.txt
  Drums.txt
  State.json
```

can simulate:

```text
capture
change
publish
restore
component substitution
integration conflict
dependency failure
```

before Ardour integration is complete.

---

# 182. Reference Adapter

The first full reference implementation will be:

```text
Ardour OMVCS Adapter
```

Its design belongs to the already agreed:

> **Ardour Reference Adapter Design**

The Ardour implementation MUST conform to this generic contract.

This specification MUST NOT be changed merely to expose an Ardour-specific convenience unless the concept generalises to other DAWs.

---

# 183. Compatibility principle

A future proprietary DAW can implement OMVCS without exposing its internal source code.

It needs only to implement the Adapter Contract correctly.

This means:

```text
OMVCS standard = open
OMVCS Core = reusable
DAW Adapter = implementation-specific
DAW itself = may be open or proprietary
```

---

# 184. Interoperability principle

A Project's history MUST remain understandable even when a particular DAW Adapter is unavailable.

At minimum OMVCS must still be able to understand:

```text
Project identity
Revision graph
Creative Components
Resource identities
provenance
Releases
Contributions
Reference Renders
Adapter identity
```

It may be unable to reconstruct the DAW-native editable state without the relevant Adapter.

---

# 185. Adapter absence

If a required Adapter is unavailable, OMVCS MUST report:

```text
Adapter unavailable
```

rather than:

```text
Project corrupt
```

The project may still be:

```text
historically valid
resources available
render playable
editable state unavailable
```

---

# 186. Adapter replacement

A different implementation of the same Adapter Contract for the same DAW MAY restore the same Adapter State if it understands the relevant Adapter State schema.

Adapter software identity does not need to be the exact same executable that created the history.

---

# 187. Independent implementations

Two independent Ardour OMVCS Adapters MAY exist.

If both claim support for:

```text
org.openmusic.ardour Adapter State schema X
```

they must interpret it consistently enough to satisfy the same conformance rules.

This is why Adapter State schemas must be documented.

---

# 188. Security boundary

Adapters process potentially untrusted project files and plugin metadata.

Implementations SHOULD treat Adapter State and Resources obtained from others as untrusted input.

Parsing MUST be robust against malformed data.

The exact secure coding rules belong to implementation guidance, not a new specification.

---

# 189. Resource execution

Opening a DAW project may cause plugins or scripts to execute.

The Adapter SHOULD expose this risk where detectable.

OMVCS MUST NOT assume that materialising arbitrary third-party DAW state is equivalent to opening passive media.

---

# 190. Safe inspection mode

Adapters MAY support a safe inspection mode that avoids activating:

```text
plugins
scripts
external devices
```

until explicitly permitted.

This is especially valuable for Contributions from unknown users.

---

# 191. Contribution safety

An Adapter SHOULD permit reviewing Contribution metadata and Reference Render without immediately loading executable DAW/plugin state.

The Interaction Specification will determine how this appears to users.

---

# 192. No arbitrary silent dependency installation

The Adapter MUST NOT install missing plugins, scripts, packages or other executable Dependencies without explicit authorization.

---

# 193. Licensing neutrality

The Adapter may report Dependency licences or availability where known.

OMVCS itself does not require that all DAW Dependencies be open source.

A proprietary plugin may remain a valid Dependency.

---

# 194. Reference Render as interoperability fallback

Reference Render provides a shared auditory reference even when:

```text
DAW differs
plugin missing
Adapter unavailable
```

but MUST NOT be used to pretend editable cross-DAW interoperability exists when it does not.

---

# 195. Semantic component portability

Creative Components are OMVCS concepts.

A future foreign DAW Adapter may be able to import a Component independently of the original DAW.

For example:

```text
audio Bass Component
```

may be portable.

A complex:

```text
instrument track + proprietary synth + automation
```

may not be.

Adapters MUST report the distinction.

---

# 196. Portability classification

Adapters MAY classify Component States as:

```text
portable
adapter-dependent
dependency-dependent
render-only
unknown
```

This classification can help later UI and contribution workflows.

---

# 197. Adapter-specific metadata must not infect Core semantics

If Ardour needs:

```text
route-group IDs
playlist IDs
processor chains
```

those belong in Ardour Adapter State.

OMVCS Core structures MUST not acquire Ardour-specific fields.

---

# 198. New generic concepts

If implementation reveals a DAW concept that appears universally necessary, the correct process is:

```text
1. identify the generic requirement;
2. determine whether it applies beyond one DAW;
3. amend this Adapter Specification or Core explicitly;
4. update conformance tests;
5. only then implement as normative behaviour.
```

Coding agents MUST NOT silently extend Core.

---

# 199. Adapter conformance statement

An Adapter claiming compliance MUST publish at minimum:

```text
Adapter Identifier
Adapter version
supported Adapter Contract version
supported DAW versions
supported native format versions
capability declaration
known limitations
conformance test result
```

---

# 200. Agent implementation rules

Coding agents implementing a DAW Adapter MUST obey the following rules:

> Do not create a second version-control system inside the Adapter.

> Do not use DAW filenames or track names as stable OMVCS identities.

> Do not discard unsupported native state silently.

> Do not claim exact restoration merely because the DAW project opens.

> Do not store storage-provider logic inside the Adapter.

> Do not make Platform access necessary for Capture or Restore.

> Do not create Revisions from native DAW Save events automatically.

> Do not publish history directly from Adapter code.

> Do not resolve creative conflicts by arbitrary last-write-wins behaviour.

> Do not alter historical Adapter State during native-format upgrades.

> Do not hide missing Dependencies.

> Do not infer component identity from name alone when stable native IDs exist.

> Do not put secrets or irrelevant machine-private information into historical state.

> Do not bypass the Adapter Contract to modify OMVCS Core directly for DAW-specific convenience.

---

# 201. Example: simple capture

Current Ardour-like Working State:

```text
Bass track
  Bass.wav

Vocal track
  Vocal.wav

Room reverb plugin
```

Adapter Capture returns conceptually:

```text
Component:
Bass
    Resource: Bass.wav

Component:
Vocal
    Resource: Vocal.wav

Adapter State:
    native session state
    track/component bindings
    reverb configuration

Dependencies:
    RoomReverb plugin
```

Core then creates:

```text
Resource Objects
Component States
Adapter State Object
Project State
Revision
```

The Adapter creates none of those historical relationships itself.

---

# 202. Example: semantic change

Base Revision:

```text
Vocal reverb wet = 18%
```

Current state:

```text
Vocal reverb wet = 27%
```

Adapter may report:

```text
semantic change:
Vocal / Room Reverb / Wet
18% -> 27%
```

Core may present or record the Change Description.

The resulting history is still identified by the captured Adapter State and Project State, not by the text description.

---

# 203. Example: component replacement

Current Working State:

```text
Bass B7
Drums D4
Vocal V8
```

User asks for:

```text
Bass B3
```

If component materialisation is supported:

```text
Adapter analyses dependencies
Adapter restores B3 into current state
Adapter preserves D4 and V8
Adapter validates resulting project
```

Result:

```text
Custom Working State
```

No Revision exists until the user publishes it.

---

# 204. Example: missing plugin

Historical Revision requires:

```text
SuperVerb 4.2
```

Current machine lacks it.

Resources all exist.

Adapter reports:

```text
Resource availability: available
Reproducibility: partial
Issue: missing required plugin
Reference Render: available
```

The Revision remains historically valid.

---

# 205. Example: foreign DAW

Revision contains:

```text
Adapter State:
com.ableton.live.omvcs
```

User opens with Ardour Adapter.

Ardour Adapter can access:

```text
audio Resources
MIDI Resources
Reference Render
Creative Components
```

but cannot interpret the Ableton native Adapter State.

It may offer explicit import/conversion if supported.

It MUST NOT claim exact restore.

---

# 206. Example: Contribution integration

Anna contributes:

```text
new cello
new drum arrangement
```

Joakim selects:

```text
Cello only
```

Adapter:

```text
creates Integration Plan
checks cello timeline/routing dependencies
detects no conflict
materialises cello into current Working State
validates result
```

Core later publishes resulting Revision with provenance to Anna's Contribution.

---

# 207. Example: unsafe selective integration

Contribution contains:

```text
new cello
tempo map changed 120 -> 95 BPM
cello regions aligned against new tempo
```

User requests cello only.

Adapter determines:

```text
Cello State structurally depends on changed tempo map
```

It MUST NOT silently place the cello against the old tempo and call the operation successful.

It reports conflict/dependency.

---

# 208. Example: native format upgrade

Revision R12:

```text
Ardour 8 state
```

User opens it using Ardour 11.

Adapter restores the old state and Ardour internally upgrades it.

R12 remains unchanged.

If the user publishes:

```text
R13
```

R13 contains newly captured Ardour 11 Adapter State.

---

# 209. Example: rename

Revision R4:

```text
Component ID 123
display name: Bass
native route ID: ABC
```

User renames track:

```text
Fretless Bass
```

native route ID remains ABC.

Adapter maps it to Component ID 123.

No new Creative Component is created.

---

# 210. Example: local path change

Original capture:

```text
C:\Users\Joakim\Music\Song\Bass.wav
```

Restore on another machine:

```text
/Users/Anna/Music/Song/Bass.wav
```

Resource identity remains identical.

Adapter reconstructs native state using the new materialised path.

Original absolute path is not historical identity.

---

# 211. Required conformance scenario set

Before an Adapter may be considered reference-quality, automated tests SHOULD cover at least:

```text
new Project adoption
existing Project reopen
project rename
track rename
track add
track delete
audio Resource change
MIDI change
plugin parameter change
automation change
routing change
external Resource
missing Resource
corrupt Resource
missing plugin
old native format
Reference Render
full restore
component restore
Custom Working State
Contribution integration
integration conflict
DAW restart
machine path change
Platform unavailable
storage provider change
unknown Adapter extension
capture during playback
capture during recording
crash during restore
```

---

# 212. DAW Adapter boundary summary

The contract can be reduced to this:

```text
                    OMVCS CORE
                        |
                        |
      ---------------------------------------
      DAW ADAPTER CONTRACT
      ---------------------------------------
                        |
                        |
                        v
                       DAW
```

Core asks:

> What is the current creative state?

Adapter answers.

Core asks:

> What Resources does that state depend upon?

Adapter answers.

Core asks:

> What changed?

Adapter answers to the depth it supports.

Core asks:

> Reconstruct this historical state.

Adapter reconstructs or explains why it cannot.

Core asks:

> Can these selected creative states coexist?

A capable Adapter analyses and proposes a result.

But:

> **Core decides what becomes history.**

---

# 213. Architectural statement

The most important requirements of this specification are:

> **A DAW Adapter is not a version-control system.**

> **A DAW Adapter is a deterministic translator between DAW-native state and OMVCS state.**

> **Adapters must preserve what they do not understand whenever possible and must report what they cannot preserve.**

> **Capabilities must be explicit.**

> **Exactness must never be faked.**

> **Ardour is the reference implementation of the Adapter Contract, not the definition of the contract.**

---

# 214. Unresolved DAW Adapter decisions for 0.1

Questions 1–9 remain intentionally open inside this specification and should be settled before freezing version 0.1:

1. Exact wire/schema format for Adapter operation request and response objects.
2. Whether `full_state_capture` requires preservation of all inactive/unused-but-referenced DAW state or permits adapter-defined exclusions.
3. Exact minimum required Dependency reporting for Level A versus Level B conformance.
4. Whether Reference Render becomes mandatory for Level B.
5. Exact semantic categories shared across all DAWs versus namespaced Adapter-specific categories.
6. Whether Creative Component mapping is mandatory for Collaborative conformance.
7. Exact requirements for machine-specific state classification.
8. Whether the Mock Adapter becomes an official conformance reference implementation.
9. Exact safe-inspection requirements for untrusted Contributions.
Question 10 is resolved by ADR-0004: every Adapter State is a canonical OMVCS metadata object, and Project State references that object rather than a native Resource Object directly.

These are finite decisions within this document, not invitations to create additional specifications.

---

## Document status

**Document:** OMVCS DAW Adapter Specification  
**Version:** 0.1 Draft  
**Normative:** Draft normative  
**Depends on:** OMVCS Glossary, Core Invariants Specification, OMVCS Core Specification  
**Next fixed document:** **OMVCS Storage Adapter Specification**