# M3 Storage Abstraction — bounded preflight

Status: PREFLIGHT COMPLETE; work packages PLANNED; production implementation is not authorized by this preflight
Date: 2026-10-10
Owner: OMVCS Lead
Integration branch: `spec/0003-canonical-collection-order`

## Scope and authority

M2 is complete at `31f114cfffe5984fa6e2d24d73184824eef67544`.
This preflight covers only the next milestone, M3 Storage Abstraction. The
fixed eight documents under `Specs/` remain authoritative. No Spec or ADR was
changed, and no semantic decision was made.

The governing rule is:

> History identifies content. Storage metadata locates content.

Historical Resource References identify immutable Resource Objects.
Chunking, Replica locations, provider locators, Endpoint configuration,
verification evidence, and Storage Map mutations remain operational. Moving
or re-representing storage MUST NOT change a ResourceId, ChunkId,
ComponentStateId, ProjectStateId, RevisionId, or historical bytes.

## Specification impact analysis

| Normative area / terms | Requirements read and cross-searched | Direct invariants | M3 impact |
|---|---|---|---|
| Resource Object, ResourceId, Chunk, Chunk Manifest, Resource Replica, Storage Location, Storage Map | Core §§6-8, 29-36, 48-51, 55; Storage §§14-36, 59-75, 157-164; Glossary Resource Object/Identifier, Chunk/Manifest, Resource Replica, Storage Location, Storage Map, Storage Map Generation | INV-RES-001–007; INV-STOR-003–005; INV-INT-001–003 | Resource/Chunk identity, order, and integrity are defined. DG-0029 is resolved by ADR-0032, provider locator representation by ADR-0033, and Storage Map generation/CAS by ADR-0034 resolving DG-0032. |
| Endpoint, capabilities, Repository Home, logical keys, operational metadata | Core §§29-34, 57-58; Storage §§3-15, 41-53, 124-129, 221, 235-243; Glossary Storage Endpoint, Project Storage, OMVCS Repository, Repository Home, Operational Metadata | INV-STOR-001–004; INV-PLAT-001–005; INV-SYNC-002–003 | Keep Endpoint, Resource storage, and Repository Home roles distinct. Expose capabilities without claiming the unresolved Repository Home conformance minimum. |
| Verification, availability, corruption, evidence | Core §§48-51, 55; Storage §§30-36, 76-80, 109-112, 183, 192; Glossary Availability State/Resource Replica/Content Verification | INV-INT-001–003; INV-RES-005–007 | Resource SHA-256 covers complete Resource bytes; each Chunk identity covers that Chunk only; existence/length is not integrity. ADR-0037 defines strengths/methods and ADR-0038 defines destination Resource-level assurance. |
| Replication, migration, failure | Core §§35-36, 57-58, 69; Storage §§54-59, 111-112, 204-205; Interaction §§94-99 | INV-STOR-005; INV-INT-003–004; INV-GC-001–003 | Copy, verify, then register; retain the old valid copy on failure. M3 returns explicit operation outcomes but does not add M4 recovery orchestration. |
| Temporary grants and playback | Storage §§101–106, 176, 190; Platform §§77–80, 144–145, 245 items 8-9 | INV-PLAT-003; INV-STOR-002 | Temporary grants are optional for the general Adapter contract: §§101–102 use MAY, §190 explicitly says optional, and §176's list is not a universal mandate (CopyWithinEndpoint is also optional under §191). DEC-PLATFORM-008/009 and DEC-STORAGE-009 gate grant/public-playback profiles only; no base-contract contradiction or Design Gap was found. |
| Retention and physical deletion | Core §§62-65, 89; Storage §§37-40, 114-123, 194-197, 209-210, 227; Glossary Replica Removal/Garbage Collection | INV-GC-001–003 | No GC, orphan cleanup, automatic Replica deletion, or old-source removal is included in the first M3 packages. |
| Storage administration and DAW boundary | Interaction §§91-107, §308 items 5, 11-12; DAW Adapter §§126-129; Ardour Reference Adapter Design §§14-15, 33-35 | INV-DAW-001; INV-SYNC-002 | UI terminology and warnings are presentation work, not Storage Adapter semantics. DAW Adapters receive materialised paths and do not own storage credentials or provider logic. |

The full fixed set was searched for storage, Resource/Chunk identity,
Replica, Endpoint, Repository Home, verification, grant, encryption,
retention, replication, migration, and decision terms. Subsequent
approved decisions ADR-0032/0033, WORK-0015 closeout, and the subsequent
ADR-0036/0037/0038 decisions have been reconciled in the package/dependency
status below:

- Core Specification
- Storage Adapter Specification
- Platform Protocol
- Interaction Specification
- Glossary
- Core Invariants Specification
- DAW Adapter Specification
- Ardour Reference Adapter Design

The DAW and Ardour documents do not add M3 provider semantics. They reinforce
the adapter boundary and the rule that location changes are not creative
changes. Existing relevant ADRs include ADR-0001/0005 (historical
canonicalization only), ADR-0007/0008/0009 (Resource Reference identity and
admission), ADR-0027/0028/0029 (Repository Home completeness and validation),
and ADR-0030/0031 (Working State roots and unresolved classification).
ADR-0032/0033 resolve the bounded Replica identity/representation and
provider-locator-envelope decisions. ADR-0036/0037/0038 subsequently resolve
the M3 chunking, verification-strength/evidence, and destination-assurance
decisions. Repository Home and shared namespace policy remain open.

## Repository Home boundary

Current Core §29 requires a Repository Home for actively published Projects
to store available durable historical metadata and required operational
metadata, including local completeness/boundary information and storage
resolution. It is not necessarily the Resource store. Storage §10 and §13
describe capability groupings; §§44-53 define guarded operational metadata
updates and Storage Map persistence. DEC-STORAGE-003 still owns the exact
minimum conformance profile.

M3 must keep these roles separate:

1. Historical metadata: immutable object bodies and identifiers are retained
   as authored; the Home does not rewrite history.
2. Operational Repository metadata: completeness/boundary records,
   Endpoint descriptors, mutable Storage Map, and guarded generations.
3. Resource/Chunk storage: a Resource Endpoint can exist without being a
   Repository Home; one Project can use multiple Endpoints.
4. Repository Home migration: copy/verify operational and historical
   metadata without upgrading incomplete history to complete.
5. M4-owned state: publication transactions, cross-step recovery and the
   Repository Operation Log's exact layout/compaction are not part of the
   initial M3 API or provider.
6. Platform mirrors: Platform coordination is not Repository Home and
   Platform storage is not authoritative Resource storage.

## Decision dependency audit

The decision register's `Blocks` value is not treated as a blanket gate on
every M3 package. These are the actual dependencies for the proposed
packages. The table records current status; the original preflight itself
resolved no decisions.

| Decision(s) and current status | Actual dependency and package gate |
|---|---|
| DEC-CORE-001 + DEC-STORAGE-002 | RESOLVED jointly by ADR-0036: OMVCS 0.1 uses fixed-size sequential Chunking with an exact 8,388,608-byte target. ResourceId remains the complete Resource SHA-256; the policy fixes Chunk boundaries and resulting ChunkIds. |
| DEC-CORE-005, DEC-CORE-008 | Their register earliest-blocker fields are M2, now complete; their downstream M3 impact is physical deletion/retention. They gate eligibility/removal of the last valid Replica and GC, not additive storage, registration after verification, replication, or source-preserving migration. |
| DEC-PLATFORM-008 | Exact Platform temporary-access request/response schema. Gates Platform-integrated grant support only; it does not block the base Endpoint contract. |
| DEC-PLATFORM-009 + DEC-STORAGE-009 | Whether a public-playback Endpoint must support direct grants or whether limited proxy fallback is allowed. Gates public-playback capability/conformance only, not generic storage. Storage §§101–102/190 already establish that general grant support is optional. |
| DEC-STORAGE-001 | RESOLVED by ADR-0039: canonical Resource/Chunk logical keys. No canonical metadata path layout is defined. |
| DEC-STORAGE-003 | RESOLVED by ADR-0040: minimum Home capabilities; Resource/Chunk byte storage remains independently declared. |
| DEC-STORAGE-004 | RESOLVED by ADR-0037: `chunk_identity` and `resource_identity` are verification strengths, separate from outcome and method/evidence. Provider evidence counts only when equivalent to the exact OMVCS SHA-256 proposition and byte scope. |
| DEC-STORAGE-005 | RESOLVED by ADR-0038: destination `resource_identity` assurance may come from direct complete Resource verification or the specified deterministic verified-reconstruction path. Source verification or provider operation success alone is insufficient. |
| DEC-STORAGE-006 | Repository Operation Log format and compaction. M4 only; no initial M3 package persists this log. |
| DEC-STORAGE-007 | Resolved by ADR-0033: durable Replica location uses the typed `{schema, value}` ProviderLocator envelope; provider-specific values remain opaque to Core. |
| DEC-STORAGE-008 | Client-side encryption treatment. Gates encrypted Endpoint conformance only. Plain, exact-byte storage remains in scope; no encryption/key management is inferred. |
| DEC-STORAGE-010 | Shared-namespace GC requirements. Gates GC only; GC is excluded from M3's first package set. |
| DEC-STORAGE-011 | Whether stable Endpoint namespace identity is mandatory. Remains open and gates shared-namespace identity, cross-Project physical-object/deduplication guarantees, and shared-namespace GC only; WORK-0016 excludes these claims. |
| DEC-STORAGE-012 | Abandoned multipart/unreferenced-object retention. Gates cleanup and interrupted-transfer retention behavior; M3 leaves incomplete/unregistered objects untouched and defers recovery/cleanup policy. |
| DEC-STORAGE-013 | Official reference-conformance status of filesystem/S3 Adapters. Gates the official designation and scope only. A local filesystem implementation may be planned from §64/§201 without claiming official status; S3 is not in the bounded package list. |
| DEC-STORAGE-014 | RESOLVED by ADR-0041: filesystem marker and explicit-root discovery. ADR-0042 resolves filesystem bootstrap atomicity/retry. DG-0036 remains open for non-filesystem Home discovery. |
| DEC-INTERACTION-011 | Repository Home musician-facing terminology. Presentation-only; deferred beyond the M3 technical packages. |
| DEC-INTERACTION-012 | Release replication warning wording/defaults. M3 supplies machine-readable status; exact warning presentation is deferred beyond the technical packages. |
| DEC-CORE-003, DEC-PLATFORM-002/015, DEC-INTERACTION-004/006/017 | M4 publication durability, authorization, Platform mirror recovery, and recovery UX. M3 exposes storage capabilities/results only and does not implement M4 transaction or publication policy. |

Other open entries were checked against the complete decision register.
No additional OPEN decision whose actual earliest technical dependency is
the provider-neutral Resource/Chunk contract was found. Open M2-classified
retention decisions are retained above for their M3 deletion implications.
Open M6, M5, M8, LATER and NONE decisions do not gate these storage packages.

## Storage Map gap resolution

DG-0032 is resolved by human-approved
[ADR-0034](../decisions/ADR-0034-storage-map-generation-and-cas.md).
WORK-0016 may implement the Project-scoped generation and guarded mutation
contract. No remaining open Design Gap blocks that bounded WORK-0016 scope.

The existing open DG-0015 is M6 Contribution provenance; it does not block
M3. DG-0027/0028 and other M2 gaps are resolved. The independent Spec
Guardian review confirmed that Resource/Chunk identity, reconstruction
order, and integrity rules are already specified, and that temporary-grant
optionality is not a contradiction. DG-0029 and DEC-STORAGE-007 were
resolved by ADR-0032 and ADR-0033 respectively. No additional gap was found
for cross-Replica Chunk-location ownership because WORK-0016 stores
provider-specific Chunk locations within each Resource Replica and makes no
shared-object or deletion claim.

## Bounded M3 work packages and dependency graph

The staged prerequisite flow is:

```text
WORK-0015 -> WORK-0016 Replica/locator/Storage Map/generation/CAS
                                  |
                                  +-> WORK-0017 verification result
                                           |
                                           +-> verified-promotion integration
```

WORK-0016's model, locator, map structure, and guarded/CAS mechanics are
integrated under ADR-0032/0033/0034. Its verified-promotion integration is
not part of that accepted implementation; it may be delivered as a small
WORK-0016 follow-up or as the integration portion of WORK-0017 after the
verification-result API exists. WORK-0017's semantic gates are resolved by
ADR-0036/0037/0038.
DEC-STORAGE-011 shared namespace claims remain excluded. WORK-0019 waits
for its concrete key/layout/Home decisions. WORK-0016 was authorized by the
original preflight and is now integrated at its bounded scope. WORK-0017 is
verified and integrated at its bounded scope; WORK-0018–0021 remain
unstarted and retain their own implementation gates and authorization
process.

| Work | Title | Dependency / gate |
|---|---|---|
| WORK-0015 | Storage primitives and provider-neutral contract | No new human decision for its restricted Resource/Chunk byte-I/O and capability seam. This is the first package that can be implemented from current Specs, after separate authorization. It makes no Repository Home conformance claim. |
| WORK-0016 | Replica model and Storage Map | VERIFIED / INTEGRATED at `ee8d77ddfc9859ee8c7bcae13018332632f6efa4`. Its previously gated promotion integration is now semantically defined by ADR-0037/0038 and can accompany WORK-0017's result API or land as a bounded WORK-0016 follow-up. DEC-STORAGE-011 shared namespace claims remain excluded. |
| WORK-0017 | Resource and Chunk verification model | VERIFIED / INTEGRATED at `60f71c9c624eb7b08bb8fd18ac7abb7a7a7ce250`. Uses WORK-0015/0016 and ADR-0032/0033/0034/0035/0036/0037/0038. |
| WORK-0018 | Mock Storage Adapter | WORK-0015/0016/0017; approved identity/locator/CAS contracts and deterministic failure injection. It is not an official reference conformance decision. |
| WORK-0019 | Local filesystem Storage Adapter | WORK-0015/0016/0017; ADR-0039/0040/0041/0042 define filesystem keys, Home profile, marker/discovery, and logical bootstrap atomicity/retry. Chunked behavior follows ADR-0036. DG-0036 (non-filesystem discovery) is outside scope. DEC-STORAGE-013 remains open only for official-reference designation. |
| WORK-0020 | Resource replication | WORK-0015/0016/0017; source-preserving copy, verify, register under ADR-0032/0033/0034 and ADR-0037/0038. ADR-0036 supplies Chunk boundaries; no automatic rechunking to satisfy Endpoint preference. No deletion, GC, retention policy, or M4 recovery state. |
| WORK-0021 | Storage migration | WORK-0015/0016/0017/0020; copy, verify, register new Replica under ADR-0032/0033/0034 and ADR-0037/0038 and retain source. ADR-0036 governs OMVCS Chunk boundaries; no automatic Endpoint-preference rechunking. Removing the old source remains outside this package. |

The work package files define exact sections, acceptance criteria, non-goals,
Verifier scope, owners, and Design Gap gates. WORK-0016's Replica/locator/
Storage Map/generation/CAS scope is authorized to proceed by the approved
decisions. No production implementation was performed in this documentation
task. Other packages retain their own gates and authorization process.

## Explicit boundaries

- No Revision or other historical object contains a physical Storage
  Endpoint, provider locator, Chunk Manifest, Replica, credential, or
  verification-evidence field.
- ResourceId remains SHA-256 of complete Resource bytes; ChunkIds derive
  from their respective bytes. ADR-0036 defines the deterministic OMVCS
  0.1 Chunking policy as fixed-size sequential chunks with an exact
  8,388,608-byte target. A zero-byte chunked representation has one
  zero-length Chunk; complete-object representation remains independent.
  Endpoint preference does not automatically rechunk an existing
  representation.
- Repository Home remains operational metadata authority, not a synonym for
  one Resource provider or the Platform.
- Public playback grants remain optional for the general Adapter contract;
  DEC-PLATFORM-008/009 and DEC-STORAGE-009 gate grant/public-playback
  profiles only.
- Client-side encryption/key management is excluded.
- GC, retention, physical deletion, abandoned-upload cleanup, and old-source
  removal are excluded. When uncertain, retain.
- M3 does not persist an M4 operation log or publication/recovery state
  machine. Storage calls return explicit local outcomes and never report
  success for unverified/incomplete content.

## Current M3 readiness status

- M3 is IN PROGRESS. WORK-0015 is VERIFIED/integrated; its restricted
  provider-neutral Resource/Chunk byte-I/O scope is complete.
- WORK-0016's bounded Replica/representation/locator model, Storage Map
  structure, generation type, guarded/CAS mutation, and authoritative
  persisted-map reconstruction are VERIFIED / INTEGRATED at
  `ee8d77ddfc9859ee8c7bcae13018332632f6efa4`. DG-0034 remains open and blocks
  syntax-specific ProviderLocator schema-identifier validation only.
  Promotion semantics are now defined by ADR-0037/0038 and may be integrated
  with WORK-0017's result API or delivered as a bounded WORK-0016 follow-up.
  DEC-STORAGE-011 remains open but does not block the explicitly excluded
  shared-namespace claims.
- WORK-0017's typed outcomes/strength/evidence, deterministic Chunk and
  Resource verification, corruption/unavailability distinction, provider
  checksum-equivalence rules, and promotion eligibility are VERIFIED /
  INTEGRATED at `60f71c9c624eb7b08bb8fd18ac7abb7a7a7ce250`. It consumes the
  WORK-0016 registration boundary without reopening its identity, Storage Map,
  or CAS semantics.
- At initial M3 preflight, no production implementation beyond WORK-0015 had
  started. WORK-0016 was subsequently completed and integrated at its bounded
  approved scope.
