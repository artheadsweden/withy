# OMVCS Specification Coverage

This file maps normative requirements to implementation work and tests.

The Verifier owns completeness of this map. Implementers update entries for their work.

Do not mark a requirement `done` merely because code exists. `done` means implementation and required verification are complete.

| Requirement | Work item | Implementation | Tests | Status |
|---|---|---|---|---|
| Assigned IDs use UUIDv7; canonical lowercase text; identity independent of names, paths, storage, and platform (Core §4; INV-PROJ-001–003) | WORK-0001 | `crates/omvcs-model/` | UUIDv7/canonical-text round trip; invalid form; no filename/location identity | planned |
| Typed content-derived identifiers and object namespaces (Core §§4, 6; INV-HIST-002; INV-RES-002–003) | WORK-0001, WORK-0003 | `crates/omvcs-model/` | same bytes/same Resource ID; changed byte/different ID; object-type namespace distinction | planned |
| Canonical UTF-8 JSON metadata bytes use RFC 8785 semantics; wrappers excluded (Core §5) | WORK-0002 | `crates/omvcs-model/` | member-order and whitespace invariance; RFC 8785 vectors; wrapper exclusion | blocked: DG-0003 for collection ordering |
| Resource identity hashes complete raw bytes; chunking/location do not alter it (Core §§6–8; INV-RES-001–006) | WORK-0003, WORK-0004 | `crates/omvcs-model/` | arbitrary-byte property; one-byte mutation; rename/location independence | planned |
| Resource Object, Resource Reference, Resource Manifest, and Friendly Name remain distinct; historic refs use identities, not locations (Glossary; Core §§6–8; INV-RES-004, INV-PROJ-003) | WORK-0004 | `crates/omvcs-model/` | references contain Resource ID; same content across replicas; no physical locator in identity | planned |
| Creative Component has stable semantic identity distinct from files and names (Glossary; Core §9; INV-PROJ-001–003) | WORK-0005 | `crates/omvcs-model/` | Resource replacement preserves Component ID; filename/name is not authoritative identity | planned |
| Component State is immutable, references a Component and its Resources, and expresses lineage per the resolved rules (Glossary; Core §§10–11; INV-HIST-001–003, INV-PROJ-002, INV-PROV-003) | WORK-0006 | `crates/omvcs-model/` | immutable state; changed refs change identity; parentage and collection-order tests after resolution | blocked: DG-0002, DG-0003 |
| Project State identifies one complete logical state through immutable references and not storage locations (Glossary; Core §§13, 24; INV-HIST-003, INV-RES-004, INV-PROJ-004) | WORK-0007 | `crates/omvcs-model/` | complete-state vs delta; different creative refs differ; location independence | blocked: DG-0001, DG-0003 |
| Revision identifies exactly one complete Project State; immutable parentage and identity depend only on immutable history (Glossary; Core §§14–15; INV-HIST-001–004) | WORK-0008 | `crates/omvcs-model/` | state required; parent immutability; no timestamp ancestry; operational changes do not alter ID | blocked: DG-0003, DG-0004 |

## M1 planning status

M0 has created work-package acceptance criteria only. No implementation or conformance tests have been started. The blocked rows must not be resolved in code; first resolve the referenced Design Gaps through the approved ADR/Spec-update workflow.

Suggested status values:

```text
unplanned
planned
in-progress
blocked
implemented
verified
```
