# OMVCS Specification Coverage

This file maps normative requirements to implementation work and tests.

The Verifier owns completeness of this map. Implementers update entries for their work.

Do not mark a requirement `done` merely because code exists. `done` means implementation and required verification are complete.

| Requirement | Work item | Implementation | Tests | Status |
|---|---|---|---|---|
| Assigned Project, Creative Component, Storage Endpoint, and Contribution IDs are generated as UUIDv7 in canonical lowercase form; Project and Component identity is independent of filenames, paths, storage endpoints, Platform URLs, and credentials (Core §§4, 9; INV-PROJ-001–003) | WORK-0001 | `crates/omvcs-model/` | generated UUIDv7/canonical text and strict UUID parsing; test fixture changes filename/path/storage/Platform/credential values while assigned IDs remain stable | verified |
| ActorId is assigned UUIDv7 in lowercase canonical form and independent of display name, email, username, Platform account, and signing keys, including key rotation (Glossary Actor Identifier; Core §§4.1, 59; INV-HIST-007; ADR-0002) | WORK-0001 | `crates/omvcs-model/` | canonical UUIDv7 parsing/formatting; test fixture changes profile, account, and signing-key values (including rotation) while ActorId remains stable | verified |
| Revision authorship uses ActorId and preserves authorship in immutable Revision history (Core §§14, 59; INV-HIST-007; ADR-0002) | WORK-0008 | `crates/omvcs-model/` | Revision author field is ActorId; profile, account, and key changes do not rewrite historical authorship | planned |
| Typed SHA-256 identifier formats and object-type namespaces (Core §§4, 6, 9–14; INV-HIST-002; INV-RES-002–003) | WORK-0001 | `crates/omvcs-model/` | strict prefix/digest parsing and round trips for Resource, Component State, Adapter State, Project State, and Revision IDs; same digest text remains separated by object type | verified |
| Content-derived identity from raw Resource bytes and canonical metadata objects (Core §§4–6, 55; INV-HIST-002; INV-RES-002–003) | WORK-0003 | `crates/omvcs-model/` | exact raw-byte and canonical-object-body SHA-256 vectors; object-type namespace outside digest input; wrapper exclusion; map insertion-order and duplicate-name behavior via WORK-0002; streaming/whole-buffer equivalence if supported | verified |
| Canonical UTF-8 JSON metadata bytes use RFC 8785 semantics; array collection fields declare ordered/set-like semantics; set-like elements sort by canonical serialized bytes and reject duplicates; object maps use RFC 8785 member ordering only and reject duplicate names; wrappers excluded (Core §§5, 5.1; INV-HIST-006; ADR-0001, ADR-0005) | WORK-0002 | `crates/omvcs-model/src/canonical.rs` | `crates/omvcs-model/tests/canonical_serialization.rs`: UTF-8 and escaping/numeric vectors; RFC 8785 UTF-16 member ordering and insertion-order invariance; duplicate raw names (including escaped aliases); map-value recursion without extra entry sorting; ordered/set-like arrays and duplicate canonical-element rejection; nested unclassified schema rejection including absent/empty values; body-only API boundary; deterministic repeated serialization | verified |
| Resource identity hashes complete raw bytes; filename and operational location/Replica data do not alter it (Core §§6–8; INV-RES-001–006; ADR-0007) | WORK-0003, WORK-0004 | `crates/omvcs-model/` | `tests/content_hashing.rs`: SHA-256 vectors, one-byte mutation, and equal-byte filename/location/provider/Replica independence; `tests/resource_model.rs`: immutable Resource Object byte/ID pairing and physical-context exclusion | verified |
| Generic historical Resource References contain required typed `resource_id` and bounded complete-resource `byte_length`, optional `role`, `media_type`, and schema/Adapter-supplied `properties`; present values affect containing-object identity but not Resource identity; generic names and physical data are not represented; Resource Manifest has no separate historical identifier in 0.1 (Glossary; Core §§5.1, 6–8, 56; INV-RES-001–008, INV-PROJ-003; ADR-0001, ADR-0005, ADR-0007, ADR-0008) | WORK-0004 | `crates/omvcs-model/src/resource.rs` | `tests/resource_model.rs`: required typed fields and wrong-ID namespace rejection; accept `0`, `1`, `9007199254740991` and mathematically integral JSON-number forms; reject `-1`, `1.5`, `9007199254740992`, `"1"`, host-u64 maximum, over-precision fractions and alternate field types; exact byte count from Resource Object; present-field/value identity changes without changing same-byte Resource ID; map insertion-order invariance after admission; reject generic name/storage/chunk/Replica/provider/credential/evidence/property-schema-ID fields; raw duplicate names at root, nested and escaped-equivalent rejected before map decoding; no Resource Manifest object/identifier is modeled | verified |
| Property-bearing Resource Reference validation and historical admission use the exact applicable versioned schema/Adapter context; unchecked candidates cannot enter valid history (Core §§5.1, 7, 10, 12–13, 56, 76–77; INV-RES-004, INV-RES-008, INV-DAW-004; ADR-0009) | WORK-0004 | `crates/omvcs-model/src/resource.rs` | `tests/resource_model.rs`: absent properties generic admission; `{}` requires semantic validation; exact-context success and rejection propagation; unknown/unavailable/unrelated/latest-version/non-unique authority fails, with unchecked transport roundtrip; containing schema, Adapter ID and schema version mismatches block historical output; changing properties revokes admission; context supplies recursive shapes and ordered/set-like normalization, unclassified arrays/shape mismatches/duplicate set members fail before semantic callback; callback sees normalized data; no Core key heuristics; operational evidence/context absent from canonical bytes. Four compile-fail doctests prevent candidate historical output, candidate-as-admitted use, unconditional admitted serialization and direct admitted decoding. Real Adapter semantic vocabularies and WORK-0006/0007 history integration remain planned, not implemented here | verified |
| Generic Creative Component object contains only required typed `component_id`; identity is independent of Resource content, parentage, DAW-native identity, Project membership, names, classifications, timestamps, storage, Platform accounts, and locations; Project association is Project State membership (Core §§9–10, 13; Glossary; INV-PROJ-001–003; ADR-0010) | WORK-0005 | `crates/omvcs-model/` | `tests/creative_component.rs`: Resource replacement/re-recording, membership, DAW-native IDs, name/kind/timestamp, filename, storage, Platform, and location changes do not alter Component ID; generic model excludes `project_id`, `name`, `kind`, `created_at`; rename creates no new ID or required state absent schema rule; no clone/fork/copy/import/move semantics; non-object JSON text and values rejected. `tests/creative_component_acceptance.rs`: retained positional-array rejection regression, invalid assigned identity encodings, and 256 exact one-field JSON text roundtrips. Independent gate 2026-10-08 passed at remediation `947fa4e`; original positional-array defect resolved without weakening the regression | verified |
| Component State uses the closed OMVCS 0.1 body (`schema`, `component_id`, `resources`, `metadata` required; `parents` optional), schema-owned metadata semantics, exact property-authority admission, and canonical hash preimage (Glossary; Core §§5.1, 7, 10–11, 56, 76–77; INV-HIST-001–003, INV-PROJ-002, INV-PROV-003, INV-HIST-006, INV-RES-008; ADR-0001, ADR-0003, ADR-0005, ADR-0007–0011) | WORK-0006 | `crates/omvcs-model/` | `tests/component_state.rs`: required/optional members, empty resources/metadata acceptance, omitted-required and unknown-top-level rejection; unknown/unavailable and non-unique schema rejection; typed Component ID; optional-parent empty-vs-omitted semantics; exact contextual property validation, missing/unavailable/non-unique authority and semantic rejection; embedded byte-length vectors; set-like parent/resource permutation invariance and duplicate rejection; schema-owned metadata keys, requirements, shapes, nested array classifications and unclassified-array rejection; RFC 8785 map/property insertion-order invariance and duplicate-name rejection; identical body/hash identity and identity changes for each present body/resource field including schema; no validation evidence or operational/wrapper fields in canonical bytes; typed namespaces and compile-fail immutability/admission boundaries. `tests/component_state_acceptance.rs` and `tests/component_state_remediation_acceptance.rs`: retained W6-V001 regressions, exact JCS/hash bytes, 576 combined set permutations, canonical-equivalent duplicate rejection, expanded parser/history ingress matrix and named-object controls; independent acceptance of remediation `ff3a531` recorded below | verified |
| Project State is a closed OMVCS 0.1 five-member historical object (`schema`, `project_id`, `components`, `adapter_state_id`, `project_metadata`); identity hashes all five canonical members and admission requires exact schema validation, typed/resolvable admitted references, and Component State/map-key identity consistency (Core §§5.1, 12–13, 24, 56, 76–77; Glossary; INV-HIST-003, INV-HIST-006, INV-HIST-008, INV-RES-004, INV-RES-008, INV-PROJ-004, INV-DAW-004; ADR-0001, ADR-0004–0005, ADR-0009–0012) | WORK-0007 | `crates/omvcs-model/src/project_state.rs` | `tests/project_state.rs`: closed five-member body, required/unknown fields, exact unique schema authority, typed Project and Component IDs, empty maps, duplicate raw names, resolvable admitted Component State with matching `component_id`, trusted admitted Adapter State resolution, schema-owned nested metadata shapes/semantics/array classifications, exact five-member hash preimage, identity changes for every canonical member, map-order invariance, storage/operational exclusion, and metadata admission independent of Resource-byte materialization | verified |
| Revision identifies exactly one complete valid/admitted Project State; immutable parentage and identity depend only on immutable history (Glossary; Core §§14–15, 43, 55–56, 59–60, 76–77; INV-HIST-001–007, INV-RES-002, INV-PROJ-001; ADR-0001–0002, 0004–0005, 0009–0012) | WORK-0008 | `crates/omvcs-model/` | Planned: exact body/admission and hash-preimage tests, typed Project State and parent references, set-like parent and schema-defined provenance canonicalization, UTC timestamp and timestamp/ancestry separation, ActorId authorship independence, and operational/platform/storage exclusion. Exact field and provenance-entry tests await DG-0014 resolution; no WORK-0008 tests or implementation are claimed | blocked (DG-0014) |

## WORK-0006 independent gate — 2026-10-08

**Rejected** implementation `298e5d0198e9b125d8f9b258800ed4f2620ec196` against
base `0ff182ab005fff7d58cebdea73dcb36e82c33b42`; the WORK-0006 row remains
`implemented`. `crates/omvcs-model/tests/component_state_acceptance.rs` links
Core §§5.1, 7, 10–11, 76–77, INV-HIST-006, INV-RES-004/008 and
ADR-0001/0005/0007/0008/0009/0011 to 12 independent acceptance tests.

W6-V001: both generic and property-bearing positional Resource Reference arrays
decode through JSON text/value APIs and admit into historical Component State.
Two retained rejection regressions fail; the other ten pass, including 576
combined parent/resource permutations, canonical-equivalent duplicate rejection,
explicit JCS/preimage assertions, embedded number/type boundaries, duplicate raw
names, strict field shapes and exact schema/binding checks. The original 85 model
tests still pass; final full gate totals **95 passed, 2 failed** including six
passing compile-fail doctests. No production changes or test weakening. See the
WORK-0006 plan and handover for root-cause evidence and remediation boundary.
No new Design Gap; WORK-0007/0008 remain unstarted.

## WORK-0006 acceptance after remediation — 2026-10-08

**Verified** implementation `298e5d0198e9b125d8f9b258800ed4f2620ec196` with
remediation `ff3a531a7b3b106cdbd5918f0e4804058a788022` against base
`0ff182ab005fff7d58cebdea73dcb36e82c33b42`. This supersedes the historical
rejection above. W6-V001 is resolved; the original 12 verifier tests from
`b3f20468012a0baf04e378b4d8e5eec46ed68880` remain unchanged and all pass.

`tests/component_state_remediation_acceptance.rs` adds four tests under Core
§§7/10 and ADR-0007/0011. Positional prefixes (lengths 0–8), partial optional
arrays, null holes, wrappers, tagged enum forms and nonobjects fail through
JSON text, owned and borrowed value ingress both directly and embedded, before
history admission. All eight valid named optional-member combinations preserve
exact fields and admit identical bodies/IDs across ingress paths. Duplicate raw
names and trailing documents remain rejected.

Targeted tests: **58 passed**. Full locked model suite: **104 passed**, including
six compile-fail doctests. Fmt, locked warnings-denied Clippy and diff checks
passed. Only WORK-0006 status changes to `verified`; no other coverage status
changes. No new gap or semantic vocabulary, no implementation changes in this
gate, no push/integration. WORK-0007/0008 remain unstarted.

## WORK-0006 integration and initial WORK-0007 revalidation — 2026-10-08

Verified WORK-0006 commit `8971c6ee43ee66030ef6351ad8a7be0a6bef1a49` was pushed
and integrated into `spec/0003-canonical-collection-order` by no-fast-forward
merge `139fdf2e2f144618a671d5ae45856b72c48d2ba0`; the base was pushed and
verified clean and synchronized with `origin`. WORK-0001 through WORK-0006
remain verified. DG-0012 was open during this initial revalidation, so WORK-0007
was blocked at that point; WORK-0008 remained unstarted.

## M1 planning status

DG-0001 through DG-0012 are resolved by ADR-0004, ADR-0003, ADR-0001, ADR-0002, ADR-0005, ADR-0006, ADR-0007, ADR-0008, ADR-0009, ADR-0010, ADR-0011, and ADR-0012 respectively. WORK-0001 through WORK-0007 are independently verified; WORK-0008 is blocked by DG-0014. DEC-PLATFORM-016 does not authorize licensing fields in the current M1 model scope.

### Recommended M1 implementation sequence

1. WORK-0001 identifiers.
2. WORK-0002 canonical serialization, followed by WORK-0003 content hashes. The isolated RFC 8785 foundation in WORK-0002 and raw Resource hashing in WORK-0003 may proceed in parallel with WORK-0001 only when file/module ownership is explicitly separated. WORK-0003 metadata hashing waits for WORK-0002.
3. WORK-0004 Resource model.
4. WORK-0005 Creative Component identity.
5. WORK-0006 Component State.
6. WORK-0007 Project State.
7. WORK-0008 Revision.

The model work packages share the same crate scope, so do not parallelize whole packages without first confirming non-overlapping module/file ownership. WORK-0008 remains sequential after WORK-0007.

WORK-0001 through WORK-0007 contain independently verified implementations and focused model tests. WORK-0008 remains unstarted and blocked by DG-0014.

Suggested status values:

```text
unplanned
planned
in-progress
blocked
implemented
verified
```
