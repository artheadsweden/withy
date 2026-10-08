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
| Content-derived identity from raw Resource bytes and canonical metadata objects (Core §§4–6, 55; INV-HIST-002; INV-RES-002–003) | WORK-0003 | `crates/omvcs-model/` | exact raw-byte and canonical-object-body SHA-256 vectors; object-type namespace outside digest input; wrapper exclusion; map insertion-order and duplicate-name behavior via WORK-0002; streaming/whole-buffer equivalence if supported | planned |
| Canonical UTF-8 JSON metadata bytes use RFC 8785 semantics; array collection fields declare ordered/set-like semantics; set-like elements sort by canonical serialized bytes and reject duplicates; object maps use RFC 8785 member ordering only and reject duplicate names; wrappers excluded (Core §§5, 5.1; INV-HIST-006; ADR-0001, ADR-0005) | WORK-0002 | `crates/omvcs-model/src/canonical.rs` | `crates/omvcs-model/tests/canonical_serialization.rs`: UTF-8 and escaping/numeric vectors; RFC 8785 UTF-16 member ordering and insertion-order invariance; duplicate raw names (including escaped aliases); map-value recursion without extra entry sorting; ordered/set-like arrays and duplicate canonical-element rejection; nested unclassified schema rejection including absent/empty values; body-only API boundary; deterministic repeated serialization | verified |
| Resource identity hashes complete raw bytes; chunking/location do not alter it (Core §§6–8; INV-RES-001–006) | WORK-0003, WORK-0004 | `crates/omvcs-model/` | arbitrary-byte properties; identical bytes and one-byte mutation; rename/location/replica independence; no chunk-hash or storage-layout semantics | planned |
| Resource Object, Resource Reference, Resource Manifest, and Friendly Name remain distinct; historic refs use identities, not locations (Glossary; Core §§6–8; INV-RES-004, INV-PROJ-003) | WORK-0004 | `crates/omvcs-model/` | references contain Resource ID; same content across replicas; no physical locator in identity | planned |
| Creative Component has stable semantic identity distinct from files and names (Glossary; Core §9; INV-PROJ-001–003) | WORK-0005 | `crates/omvcs-model/` | Resource replacement preserves Component ID; filename/name is not authoritative identity | planned |
| Component State is immutable, references a Component and its Resources, and has optional non-fabricated parentage (Glossary; Core §§10–11, 23; INV-HIST-001–003, INV-PROJ-002, INV-PROV-003, INV-HIST-006; ADR-0001, ADR-0003, ADR-0005) | WORK-0006 | `crates/omvcs-model/` | immutable state; changed refs change identity; initial state has zero parents; omitted parentage means unknown/unasserted; known derived states SHOULD record parents; explicitly required derivation is enforced; parent/resource set-like permutation invariance and duplicate rejection; metadata-map insertion-order invariance, RFC 8785 ordering only, duplicate-name rejection, and map-value schema normalization | planned |
| Project State identifies one complete logical state through immutable references and not storage locations; it references exactly one canonical Adapter State object, which may reference opaque native-state Resources (Glossary; Core §§12–13, 24; INV-HIST-003, INV-RES-004, INV-PROJ-004, INV-HIST-006; ADR-0001, ADR-0004, ADR-0005, ADR-0006) | WORK-0007 | `crates/omvcs-model/` | complete-state vs delta; exactly one typed Adapter State reference; native Resource cannot stand in for Adapter State; opaque state Resources are referenced through Adapter State; `component_bindings` is keyed by Creative Component Identifier with adapter-specific record values and no redundant key repetition; different creative refs differ; location independence; Component, Adapter binding and project-metadata map insertion-order invariance; RFC 8785 ordering only; duplicate-name rejection; map-value schema normalization | planned |
| Revision identifies exactly one complete Project State; immutable parentage and identity depend only on immutable history (Glossary; Core §§14–15, 59; INV-HIST-001–004, INV-HIST-006–007; ADR-0001, ADR-0002, ADR-0004, ADR-0005) | WORK-0008 | `crates/omvcs-model/` | state required; parent immutability; no timestamp ancestry; operational changes do not alter ID; parent/provenance permutation invariance and duplicate rejection; ActorId form and stability | planned (after WORK-0007) |

## M1 planning status

DG-0001 through DG-0006 are resolved by ADR-0004, ADR-0003, ADR-0001, ADR-0002, ADR-0005, and ADR-0006 respectively, and their requirements are reflected in the applicable Specs, plans, and conformance criteria. No remaining open decision blocks the generic M1 model or the Ardour component-binding schema. WORK-0001 identifier code and model tests are independently verified and handed over. Other M1 implementation and verification remain pending; WORK-0008 follows WORK-0007. DEC-PLATFORM-016 does not block the current M1 scope because WORK-0007 excludes licensing fields pending that separate decision.

### Recommended M1 implementation sequence

1. WORK-0001 identifiers.
2. WORK-0002 canonical serialization, followed by WORK-0003 content hashes. The isolated RFC 8785 foundation in WORK-0002 and raw Resource hashing in WORK-0003 may proceed in parallel with WORK-0001 only when file/module ownership is explicitly separated. WORK-0003 metadata hashing waits for WORK-0002.
3. WORK-0004 Resource model.
4. WORK-0005 Creative Component identity.
5. WORK-0006 Component State.
6. WORK-0007 Project State.
7. WORK-0008 Revision.

The model work packages share the same crate scope, so do not parallelize whole packages without first confirming non-overlapping module/file ownership. WORK-0008 remains sequential after WORK-0007.

M0 created work-package acceptance criteria and conformance vectors. WORK-0001 now contains identifier implementation and focused model unit tests. No other M1 implementation or repository-level executable conformance suite has started.

Suggested status values:

```text
unplanned
planned
in-progress
blocked
implemented
verified
```
