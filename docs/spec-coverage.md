# OMVCS Specification Coverage

This file maps normative requirements to implementation work and tests.

The Verifier owns completeness of this map. Implementers update entries for their work.

Do not mark a requirement `done` merely because code exists. `done` means implementation and required verification are complete.

| Requirement | Work item | Implementation | Tests | Status |
|---|---|---|---|---|
| Assigned Project, Creative Component, Storage Endpoint, and Contribution IDs are generated as UUIDv7 in canonical lowercase form; Project and Component identity is independent of filenames, paths, storage endpoints, Platform URLs, and credentials (Core §§4, 9; INV-PROJ-001–003) | WORK-0001 | `crates/omvcs-model/` | generated UUIDv7/canonical text and strict UUID parsing; test fixture changes filename/path/storage/Platform/credential values while assigned IDs remain stable | verified |
| ActorId is assigned UUIDv7 in lowercase canonical form and independent of display name, email, username, Platform account, and signing keys, including key rotation (Glossary Actor Identifier; Core §§4.1, 59; INV-HIST-007; ADR-0002) | WORK-0001 | `crates/omvcs-model/` | canonical UUIDv7 parsing/formatting; test fixture changes profile, account, and signing-key values (including rotation) while ActorId remains stable | verified |
| Revision authorship uses ActorId and preserves authorship in immutable Revision history (Core §§14, 59; INV-HIST-007; ADR-0002) | WORK-0008 | `crates/omvcs-model/src/revision.rs` | `tests/revision.rs`: direct ActorId field and identity-change case; stable ActorId/profile/account/key format is additionally covered by WORK-0001 | verified |
| Revision ancestry forms a directed acyclic graph within one Project; direct and transitive parent relationships, including multiple parents, determine ancestry independently of timestamps (Core §§14–15, 55–56; Glossary Revision Graph/Parent Revision; INV-HIST-003–004, INV-HIST-009; ADR-0001, ADR-0014) | WORK-0009 | `crates/omvcs-core/` | `tests/revision_graph.rs`: root, direct/transitive and multiple parents, shared ancestor deduplication, stable order-independent result, timestamp and metadata-only/Resource-byte independence, unresolved parent, cross-Project admission rejection and resolver ID mismatch; synthetic cycle test distinguishes structure failure; shallow-import policy excluded | verified |
| Mutable Line history references and shared Default Line: closed five-member record, assigned UUIDv7 identity, exact Project-scoped name uniqueness, Create/Move/Rename/Delete CAS and validation, JCS-safe generation, Repository Home Default Line preference contract and `SetDefaultLine` CAS, local-selection separation, atomic protection from deleting the designated Line (Core §§3.2, 4.1, 16–17, 26, 56–58, 74, 82–83; Glossary Line/Default Line; INV-HIST-005, INV-HIST-010–011, INV-GC-001, INV-UX-002; ADR-0016, ADR-0020–0022) | WORK-0010 | `crates/omvcs-core/` | `tests/line_operations.rs`: closed record/UUIDv7, exact names, admitted same-Project target checks without Resource bytes, generation domain/CAS/increments, Default Line CAS/local-selection separation, failed-create atomicity, deletion/history preservation, atomic uniqueness and SetDefaultLine/DeleteLine race; `src/line.rs`: operation effects/failure/retry documentation and generation exhaustion tests. In-memory boundary is a contract test double; durable Repository Home adapter persistence belongs to later repository/storage work. Independently verified; durable adapter implementation remains out of scope. | verified |
| Immutable Release references, content-derived identity, closed body, Project-scoped naming, atomic admission, and same-Project Revision association (Core §§4.1–4.2, 18, 56, 62, 74, 82–83; Glossary Release/Release Identifier; INV-HIST-001–005, INV-HIST-009, INV-HIST-012, INV-GC-001, INV-REC-002; ADR-0017, ADR-0023) | WORK-0011 | `crates/omvcs-model/src/release.rs`; `crates/omvcs-core/src/release.rs` | `omvcs-model/tests/release.rs`: seven-field closed-body, omission/null/unknown/duplicate rejection, typed IDs, exact schema authority, name/timestamp/ActorId/description checks, admitted same-Project target without Resource-byte resolver, resolver identifier-mismatch rejection, canonical JCS and independent literal hash golden vector, each-field digest sensitivity, immutable getters and ReleaseId format; `omvcs-core/tests/release_operations.rs`: exact duplicate idempotency, exact Project/name namespace and normalization distinctions, atomic name conflict, missing/cross-Project failure and concurrent uniqueness; Core unit tests: no partial claim, stored ID/body integrity failure, and overlapping integrity/name failure precedence with no mutation. Independently accepted and integrated at `e179d27147ea6efcac3aa46fedcecd34ddd5725f` | verified |
| Mutable local Working State lifecycle, Base Revision, and separation from immutable history (Core §§3, 19–25, 62, 82–83; Glossary Working State/Base Revision/Materialisation/Custom Working State/Local Modification/AdapterWorkingStateRef/Working State recovery condition; INV-PROJ-004–005, INV-WORK-001–006; ADR-0018, ADR-0024–0026) | WORK-0012 | `crates/omvcs-core/` | Human-approved ADRs resolve persistent local operational classification, optional Base Revision and Line association, pre-first-Revision state, component source map, derived Core tri-state, scoped destructive-replacement authorization, opaque AdapterWorkingStateRef ownership/prepare/commit/restart recovery, and operation failure/retry/idempotency behavior. Acceptance tests are specified in WORK-0012. DEC-INTERACTION-004 checkpoints are separate and excluded. Not yet implemented or independently verified | planned |
| Repository metadata reachability from Lines, Releases, and Working State, distinct from Resource-byte availability and deletion (Core §§56, 62; Glossary Reachability; INV-GC-001–003, INV-WORK-002–006; ADR-0017 Release root edge; ADR-0018 Working State distinction; ADR-0025 AdapterWorkingStateRef boundary) | WORK-0013 | `crates/omvcs-core/` | Blocked on WORK-0012 completion and the WORK-0013 reachability contract. Working State itself and AdapterWorkingStateRef are not roots merely by existing; only explicit Working State safety references are roots under Core §62. Release edge is `Release -> revision_id`; Contributions excluded | blocked |
| Repository validation operation, integrity results, and distinction between metadata completeness and Resource availability (Core §§21–22, 29, 47–56, 62–67, 76–77, 82–83; INV-HIST-003, INV-HIST-009, INV-RES-007, INV-WORK-002–004, INV-INT-001–003, INV-GC-001–003) | WORK-0014 | `crates/omvcs-core/` | blocked pending DG-0019 and decisions DEC-CORE-004/009; depends on WORK-0009–0013 | blocked |
| Typed SHA-256 identifier formats and object-type namespaces (Core §§4, 6, 9–14; INV-HIST-002; INV-RES-002–003) | WORK-0001 | `crates/omvcs-model/` | strict prefix/digest parsing and round trips for Resource, Component State, Adapter State, Project State, and Revision IDs; same digest text remains separated by object type | verified |
| Content-derived identity from raw Resource bytes and canonical metadata objects (Core §§4–6, 55; INV-HIST-002; INV-RES-002–003) | WORK-0003 | `crates/omvcs-model/` | exact raw-byte and canonical-object-body SHA-256 vectors; object-type namespace outside digest input; wrapper exclusion; map insertion-order and duplicate-name behavior via WORK-0002; streaming/whole-buffer equivalence if supported | verified |
| Canonical UTF-8 JSON metadata bytes use RFC 8785 semantics; array collection fields declare ordered/set-like semantics; set-like elements sort by canonical serialized bytes and reject duplicates; object maps use RFC 8785 member ordering only and reject duplicate names; wrappers excluded (Core §§5, 5.1; INV-HIST-006; ADR-0001, ADR-0005) | WORK-0002 | `crates/omvcs-model/src/canonical.rs` | `crates/omvcs-model/tests/canonical_serialization.rs`: UTF-8 and escaping/numeric vectors; RFC 8785 UTF-16 member ordering and insertion-order invariance; duplicate raw names (including escaped aliases); map-value recursion without extra entry sorting; ordered/set-like arrays and duplicate canonical-element rejection; nested unclassified schema rejection including absent/empty values; body-only API boundary; deterministic repeated serialization | verified |
| Resource identity hashes complete raw bytes; filename and operational location/Replica data do not alter it (Core §§6–8; INV-RES-001–006; ADR-0007) | WORK-0003, WORK-0004 | `crates/omvcs-model/` | `tests/content_hashing.rs`: SHA-256 vectors, one-byte mutation, and equal-byte filename/location/provider/Replica independence; `tests/resource_model.rs`: immutable Resource Object byte/ID pairing and physical-context exclusion | verified |
| Generic historical Resource References contain required typed `resource_id` and bounded complete-resource `byte_length`, optional `role`, `media_type`, and schema/Adapter-supplied `properties`; present values affect containing-object identity but not Resource identity; generic names and physical data are not represented; Resource Manifest has no separate historical identifier in 0.1 (Glossary; Core §§5.1, 6–8, 56; INV-RES-001–008, INV-PROJ-003; ADR-0001, ADR-0005, ADR-0007, ADR-0008) | WORK-0004 | `crates/omvcs-model/src/resource.rs` | `tests/resource_model.rs`: required typed fields and wrong-ID namespace rejection; accept `0`, `1`, `9007199254740991` and mathematically integral JSON-number forms; reject `-1`, `1.5`, `9007199254740992`, `"1"`, host-u64 maximum, over-precision fractions and alternate field types; exact byte count from Resource Object; present-field/value identity changes without changing same-byte Resource ID; map insertion-order invariance after admission; reject generic name/storage/chunk/Replica/provider/credential/evidence/property-schema-ID fields; raw duplicate names at root, nested and escaped-equivalent rejected before map decoding; no Resource Manifest object/identifier is modeled | verified |
| Property-bearing Resource Reference validation and historical admission use the exact applicable versioned schema/Adapter context; unchecked candidates cannot enter valid history (Core §§5.1, 7, 10, 12–13, 56, 76–77; INV-RES-004, INV-RES-008, INV-DAW-004; ADR-0009) | WORK-0004 | `crates/omvcs-model/src/resource.rs` | `tests/resource_model.rs`: absent properties generic admission; `{}` requires semantic validation; exact-context success and rejection propagation; unknown/unavailable/unrelated/latest-version/non-unique authority fails, with unchecked transport roundtrip; containing schema, Adapter ID and schema version mismatches block historical output; changing properties revokes admission; context supplies recursive shapes and ordered/set-like normalization, unclassified arrays/shape mismatches/duplicate set members fail before semantic callback; callback sees normalized data; no Core key heuristics; operational evidence/context absent from canonical bytes. Four compile-fail doctests prevent candidate historical output, candidate-as-admitted use, unconditional admitted serialization and direct admitted decoding. Real Adapter semantic vocabularies and WORK-0006/0007 history integration remain planned, not implemented here | verified |
| Generic Creative Component object contains only required typed `component_id`; identity is independent of Resource content, parentage, DAW-native identity, Project membership, names, classifications, timestamps, storage, Platform accounts, and locations; Project association is Project State membership (Core §§9–10, 13; Glossary; INV-PROJ-001–003; ADR-0010) | WORK-0005 | `crates/omvcs-model/` | `tests/creative_component.rs`: Resource replacement/re-recording, membership, DAW-native IDs, name/kind/timestamp, filename, storage, Platform, and location changes do not alter Component ID; generic model excludes `project_id`, `name`, `kind`, `created_at`; rename creates no new ID or required state absent schema rule; no clone/fork/copy/import/move semantics; non-object JSON text and values rejected. `tests/creative_component_acceptance.rs`: retained positional-array rejection regression, invalid assigned identity encodings, and 256 exact one-field JSON text roundtrips. Independent gate 2026-10-08 passed at remediation `947fa4e`; original positional-array defect resolved without weakening the regression | verified |
| Component State uses the closed OMVCS 0.1 body (`schema`, `component_id`, `resources`, `metadata` required; `parents` optional), schema-owned metadata semantics, exact property-authority admission, and canonical hash preimage (Glossary; Core §§5.1, 7, 10–11, 56, 76–77; INV-HIST-001–003, INV-PROJ-002, INV-PROV-003, INV-HIST-006, INV-RES-008; ADR-0001, ADR-0003, ADR-0005, ADR-0007–0011) | WORK-0006 | `crates/omvcs-model/` | `tests/component_state.rs`: required/optional members, empty resources/metadata acceptance, omitted-required and unknown-top-level rejection; unknown/unavailable and non-unique schema rejection; typed Component ID; optional-parent empty-vs-omitted semantics; exact contextual property validation, missing/unavailable/non-unique authority and semantic rejection; embedded byte-length vectors; set-like parent/resource permutation invariance and duplicate rejection; schema-owned metadata keys, requirements, shapes, nested array classifications and unclassified-array rejection; RFC 8785 map/property insertion-order invariance and duplicate-name rejection; identical body/hash identity and identity changes for each present body/resource field including schema; no validation evidence or operational/wrapper fields in canonical bytes; typed namespaces and compile-fail immutability/admission boundaries. `tests/component_state_acceptance.rs` and `tests/component_state_remediation_acceptance.rs`: retained W6-V001 regressions, exact JCS/hash bytes, 576 combined set permutations, canonical-equivalent duplicate rejection, expanded parser/history ingress matrix and named-object controls; independent acceptance of remediation `ff3a531` recorded below | verified |
| Project State is a closed OMVCS 0.1 five-member historical object (`schema`, `project_id`, `components`, `adapter_state_id`, `project_metadata`); identity hashes all five canonical members and admission requires exact schema validation, typed/resolvable admitted references, and Component State/map-key identity consistency (Core §§5.1, 12–13, 24, 56, 76–77; Glossary; INV-HIST-003, INV-HIST-006, INV-HIST-008, INV-RES-004, INV-RES-008, INV-PROJ-004, INV-DAW-004; ADR-0001, ADR-0004–0005, ADR-0009–0012) | WORK-0007 | `crates/omvcs-model/src/project_state.rs` | `tests/project_state.rs`: closed five-member body, required/unknown fields, exact unique schema authority, typed Project and Component IDs, empty maps, duplicate raw names, resolvable admitted Component State with matching `component_id`, trusted admitted Adapter State resolution, schema-owned nested metadata shapes/semantics/array classifications, exact five-member hash preimage, identity changes for every canonical member, map-order invariance, storage/operational exclusion, and metadata admission independent of Resource-byte materialization | verified |
| Revision uses the closed OMVCS 0.1 seven-member body, identifies one admitted Project State, preserves same-Project immutable ancestry, and derives identity from all seven canonical members (Glossary; Core §§5.1, 14–15, 23, 42–43, 55–56, 59–60, 68, 76–77; INV-HIST-001–009, INV-COL-003–004, INV-RES-002, INV-PROJ-001; ADR-0001–0002, 0004–0005, 0009–0012, 0014) | WORK-0008 | `crates/omvcs-model/src/revision.rs` | `tests/revision.rs`: closed body/required fields; exact unique schema authority; admitted Project State and same-Project parent resolution; empty/single/multiple set-like parents and permutation invariance; direct ActorId; canonical UTC nanosecond timestamp including announced leap seconds; exact message preservation; schema-directed provenance structure, nested-array normalization, ordering and duplicate rejection using test-only fixtures; identity changes for all seven fields; timestamp/ancestry separation; metadata-only reference resolution. DG-0015 operation-specific provenance remains unimplemented. | verified |

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

DG-0001 through DG-0012 are resolved by ADR-0004, ADR-0003, ADR-0001, ADR-0002, ADR-0005, ADR-0006, ADR-0007, ADR-0008, ADR-0009, ADR-0010, ADR-0011, and ADR-0012 respectively; DG-0014 is resolved by ADR-0014. DG-0015 remains open for M6 operation-specific provenance mapping and does not block WORK-0008's generic schema-directed model. WORK-0001 through WORK-0008 are independently verified. DEC-PLATFORM-016 does not authorize licensing fields in the current M1 model scope.

### Recommended M1 implementation sequence

1. WORK-0001 identifiers.
2. WORK-0002 canonical serialization, followed by WORK-0003 content hashes. The isolated RFC 8785 foundation in WORK-0002 and raw Resource hashing in WORK-0003 may proceed in parallel with WORK-0001 only when file/module ownership is explicitly separated. WORK-0003 metadata hashing waits for WORK-0002.
3. WORK-0004 Resource model.
4. WORK-0005 Creative Component identity.
5. WORK-0006 Component State.
6. WORK-0007 Project State.
7. WORK-0008 Revision.

The model work packages share the same crate scope, so do not parallelize whole packages without first confirming non-overlapping module/file ownership. WORK-0008 remains sequential after WORK-0007.

WORK-0001 through WORK-0008 contain independently verified implementations and focused model tests. DG-0015 continues to block operation-specific provenance mapping for M6, not the generic Revision model. WORK-0009 and M6 work remain unstarted.

Suggested status values:

```text
unplanned
planned
in-progress
blocked
implemented
verified
```
