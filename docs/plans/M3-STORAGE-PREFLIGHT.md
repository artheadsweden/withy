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
| Resource Object, ResourceId, Chunk, Chunk Manifest, Resource Replica, Storage Location, Storage Map | Core §§6-8, 29-36, 48-51, 55; Storage §§14-36, 59-75, 157-164; Glossary Resource Object/Identifier, Chunk/Manifest, Resource Replica, Storage Location, Storage Map | INV-RES-001–007; INV-STOR-003–005; INV-INT-001–002 | Resource/Chunk identity, order, and integrity are defined. DG-0029 is resolved by ADR-0032 and provider locator representation by ADR-0033. Storage Map generation/CAS semantics remain open in DG-0032. |
| Endpoint, capabilities, Repository Home, logical keys, operational metadata | Core §§29-34, 57-58; Storage §§3-15, 41-53, 124-129, 221, 235-243; Glossary Storage Endpoint, Project Storage, OMVCS Repository, Repository Home, Operational Metadata | INV-STOR-001–004; INV-PLAT-001–005; INV-SYNC-002–003 | Keep Endpoint, Resource storage, and Repository Home roles distinct. Expose capabilities without claiming the unresolved Repository Home conformance minimum. |
| Verification, availability, corruption, evidence | Core §§48-51, 55; Storage §§30-36, 76-80, 109-111, 183, 192; Glossary Availability State/Resource Replica | INV-INT-001–003; INV-RES-005–007 | Resource SHA-256 covers reconstructed complete bytes; Chunk hashes cover each Chunk; mere existence/length is not integrity. Exact strength taxonomy and post-upload requirement remain open. |
| Replication, migration, failure | Core §§35-36, 57-58, 69; Storage §§54-59, 111-112, 204-205; Interaction §§94-99 | INV-STOR-005; INV-INT-003–004; INV-GC-001–003 | Copy, verify, then register; retain the old valid copy on failure. M3 returns explicit operation outcomes but does not add M4 recovery orchestration. |
| Temporary grants and playback | Storage §§101–106, 176, 190; Platform §§77–80, 144–145, 245 items 8-9 | INV-PLAT-003; INV-STOR-002 | Temporary grants are optional for the general Adapter contract: §§101–102 use MAY, §190 explicitly says optional, and §176's list is not a universal mandate (CopyWithinEndpoint is also optional under §191). DEC-PLATFORM-008/009 and DEC-STORAGE-009 gate grant/public-playback profiles only; no base-contract contradiction or Design Gap was found. |
| Retention and physical deletion | Core §§62-65, 89; Storage §§37-40, 114-123, 194-197, 209-210, 227; Glossary Replica Removal/Garbage Collection | INV-GC-001–003 | No GC, orphan cleanup, automatic Replica deletion, or old-source removal is included in the first M3 packages. |
| Storage administration and DAW boundary | Interaction §§91-107, §308 items 5, 11-12; DAW Adapter §§126-129; Ardour Reference Adapter Design §§14-15, 33-35 | INV-DAW-001; INV-SYNC-002 | UI terminology and warnings are presentation work, not Storage Adapter semantics. DAW Adapters receive materialised paths and do not own storage credentials or provider logic. |

The full fixed set was searched for storage, Resource/Chunk identity,
Replica, Endpoint, Repository Home, verification, grant, encryption,
retention, replication, migration, and decision terms. Subsequent
approved decisions ADR-0032/0033 and WORK-0015 closeout have been reconciled
in the package/dependency status below:

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
ADR-0032/0033 now resolve the bounded Replica identity/representation and
provider-locator-envelope decisions; they do not resolve chunking,
verification-strength, upload-assurance, Repository Home, or shared
namespace policy.

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
packages. Listed DEC entries remain OPEN except for the approved resolution
of DEC-STORAGE-007 recorded by ADR-0033; this preflight itself resolves
none.

| Open decision(s) | Actual dependency and package gate |
|---|---|
| DEC-CORE-001 + DEC-STORAGE-002 | One shared human decision, recorded once and reflected in both Specs/decision entries. It selects whether fixed 8 MiB is the reference policy or another deterministic policy. It gates choosing a default Chunking implementation and chunk-aware filesystem/conformance claims, but not the Resource/Chunk put/get byte contract in WORK-0015. Complete Resource SHA-256 identity remains independent. |
| DEC-CORE-005, DEC-CORE-008 | Their register earliest-blocker fields are M2, now complete; their downstream M3 impact is physical deletion/retention. They gate eligibility/removal of the last valid Replica and GC, not additive storage, registration after verification, replication, or source-preserving migration. |
| DEC-PLATFORM-008 | Exact Platform temporary-access request/response schema. Gates Platform-integrated grant support only; it does not block the base Endpoint contract. |
| DEC-PLATFORM-009 + DEC-STORAGE-009 | Whether a public-playback Endpoint must support direct grants or whether limited proxy fallback is allowed. Gates public-playback capability/conformance only, not generic storage. Storage §§101–102/190 already establish that general grant support is optional. |
| DEC-STORAGE-001 | Exact canonical logical-key layout. Gates choosing persistent key generation/layout in the filesystem Adapter; the generic API may accept opaque Core-supplied logical keys. |
| DEC-STORAGE-003 | Exact Repository Home minimum capabilities. Gates claiming Class H/RH conformance and a complete filesystem Repository Home, not capability reporting or the Resource-only interface. |
| DEC-STORAGE-004 | Verification-strength taxonomy. Gates typed strength classification and evidence mapping in WORK-0017 and verified-Replica promotion; WORK-0016 may model records but incomplete/unverified candidates stay outside the registered map, and no evidence or strength may be invented. |
| DEC-STORAGE-005 | Whether upload-time assurance must hash the full reconstructed Resource or may use previously verified deterministic Chunk reconstruction. Gates upload promotion and publication-strength claims; WORK-0015 keeps Put and explicit Verify separate. |
| DEC-STORAGE-006 | Repository Operation Log format and compaction. M4 only; no initial M3 package persists this log. |
| DEC-STORAGE-007 | Resolved by ADR-0033: durable Replica location uses the typed `{schema, value}` ProviderLocator envelope; provider-specific values remain opaque to Core. |
| DEC-STORAGE-008 | Client-side encryption treatment. Gates encrypted Endpoint conformance only. Plain, exact-byte storage remains in scope; no encryption/key management is inferred. |
| DEC-STORAGE-010 | Shared-namespace GC requirements. Gates GC only; GC is excluded from M3's first package set. |
| DEC-STORAGE-011 | Whether stable Endpoint namespace identity is mandatory. Remains open and gates shared-namespace identity, cross-Project physical-object/deduplication guarantees, and shared-namespace GC only; WORK-0016 excludes these claims. |
| DEC-STORAGE-012 | Abandoned multipart/unreferenced-object retention. Gates cleanup and interrupted-transfer retention behavior; M3 leaves incomplete/unregistered objects untouched and defers recovery/cleanup policy. |
| DEC-STORAGE-013 | Official reference-conformance status of filesystem/S3 Adapters. Gates the official designation and scope only. A local filesystem implementation may be planned from §64/§201 without claiming official status; S3 is not in the bounded package list. |
| DEC-STORAGE-014 | Storage-layout marker and discovery format. Gates Repository Home bootstrap/discovery and layout migration; not the low-level provider-neutral Resource interface. |
| DEC-INTERACTION-011 | Repository Home musician-facing terminology. Presentation-only; deferred beyond the M3 technical packages. |
| DEC-INTERACTION-012 | Release replication warning wording/defaults. M3 supplies machine-readable status; exact warning presentation is deferred beyond the technical packages. |
| DEC-CORE-003, DEC-PLATFORM-002/015, DEC-INTERACTION-004/006/017 | M4 publication durability, authorization, Platform mirror recovery, and recovery UX. M3 exposes storage capabilities/results only and does not implement M4 transaction or publication policy. |

Other open entries were checked against the complete decision register.
No additional OPEN decision whose actual earliest technical dependency is
the provider-neutral Resource/Chunk contract was found. Open M2-classified
retention decisions are retained above for their M3 deletion implications.
Open M6, M5, M8, LATER and NONE decisions do not gate these storage packages.

## Unresolved specification gaps

- [DG-0032 — Storage Map generation domain and CAS contract](../gaps/DG-0032-storage-map-generation-and-cas.md):
  blocks guarded Storage Map mutations and WORK-0016 completion, but not
  Replica/locator model work or WORK-0015.

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
WORK-0015 -> WORK-0016 model/locator subdeliverable -> WORK-0017
DG-0032 ---------------------------------------------> WORK-0016 guarded-map completion
WORK-0017 approved result ---------------------------> WORK-0016 promotion integration
```

WORK-0017 may consume the independently reviewed WORK-0016 model/locator
subdeliverable without waiting for WORK-0016's full completion. WORK-0016
promotion integration follows WORK-0017. WORK-0018 through WORK-0020 use
their specified package prerequisites and own gates; WORK-0021 depends on
replication, not on a filesystem or mock provider.
The arrows indicate package prerequisites, not permission to bypass the
semantic gates below. WORK-0016 may develop its Replica/locator model under
ADR-0032/0033, but guarded map updates and completion wait for DG-0032;
verified promotion waits for WORK-0017's approved verification result.
DEC-STORAGE-011 shared namespace claims remain excluded. WORK-0017
additionally waits for the shared chunking and verification decisions.
WORK-0019 waits for the concrete key/layout/Home decisions. Every package
requires its own implementation authorization.

| Work | Title | Dependency / gate |
|---|---|---|
| WORK-0015 | Storage primitives and provider-neutral contract | No new human decision for its restricted Resource/Chunk byte-I/O and capability seam. This is the first package that can be implemented from current Specs, after separate authorization. It makes no Repository Home conformance claim. |
| WORK-0016 | Replica model and Storage Map | WORK-0015; ADR-0032/0033 resolve Replica identity/representation and locator fields. DG-0032 blocks guarded map updates/completion. DEC-STORAGE-004/005 gate verified promotion and upload assurance. DEC-STORAGE-011 shared namespace claims are excluded. |
| WORK-0017 | Resource and Chunk verification model | WORK-0015 and independently reviewed WORK-0016 model/locator subdeliverable; ADR-0032/0033; human decisions DEC-CORE-001 + DEC-STORAGE-002 together, DEC-STORAGE-004/005. DG-0032 gates map-mutating integration only. |
| WORK-0018 | Mock Storage Adapter | WORK-0015/0016/0017; approved identity/locator contracts and deterministic failure injection; DG-0032 before guarded-map fixtures. It is not an official reference conformance decision. |
| WORK-0019 | Local filesystem Storage Adapter | WORK-0015/0016/0017; DEC-STORAGE-001, 003, 014 and shared chunk policy for selected layout. DEC-STORAGE-013 is needed only before claiming official reference status. |
| WORK-0020 | Resource replication | WORK-0015/0016/0017; source-preserving copy, verify, register under ADR-0032/0033. DG-0032 before map registration; DEC-CORE-001/DEC-STORAGE-002 gate only chunk-policy-dependent transfer/rechunking. No deletion, GC, retention policy, or M4 recovery state. |
| WORK-0021 | Storage migration | WORK-0015/0016/0017/0020; copy, verify, register new Replica under ADR-0032/0033 and retain source. DG-0032 before map update. Removing the old source is outside this package until deletion/retention rules are approved. |

The work package files define exact sections, acceptance criteria, non-goals,
Verifier scope, owners, and Design Gap gates. M3's first package is executable
in its documented restricted scope but no M3 production implementation is
authorized by this task.

## Explicit boundaries

- No Revision or other historical object contains a physical Storage
  Endpoint, provider locator, Chunk Manifest, Replica, credential, or
  verification-evidence field.
- Chunking never changes Resource identity. Each Chunk is identified from
  its bytes; reconstruction order is significant; no algorithm is chosen
  here.
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
- WORK-0016 is PLANNED. Its model/locator subdeliverable may proceed under
  ADR-0032/0033. DG-0032 blocks guarded Storage Map mutations and package
  completion; DEC-STORAGE-004/005 gate verified promotion. DEC-STORAGE-011
  remains open but does not block the explicitly excluded shared-namespace
  claims.
- WORK-0017 is not ready for verification implementation until its listed
  chunking and verification decisions are resolved. It may use the reviewed
  WORK-0016 model/locator contract without waiting for WORK-0016 closeout.
- No M3 implementation beyond the already integrated WORK-0015 scope is
  authorized by this preflight or the decision reconciliation.
