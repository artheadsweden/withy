# OMVCS Storage Adapter Specification
## Open Music Version Control System
### Draft 0.1

This document defines the normative contract between **OMVCS Core** and physical or logical storage systems.

It is subordinate to:

1. **OMVCS Glossary**
2. **OMVCS Core Invariants Specification**
3. **OMVCS Core Specification**

If this specification conflicts with a Core Invariant, the invariant takes precedence.

The governing architectural rule is:

> **OMVCS Core decides what must be stored, verified, retrieved, replicated or removed. The Storage Adapter decides how a particular storage system performs those operations.**

The Storage Adapter MUST NOT become the authority for creative history.

---

# 1. Scope

The Storage Adapter is responsible for translating between:

```text
OMVCS storage operations
```

and:

```text
provider-specific storage behaviour
```

Examples of Storage Endpoints include:

- local filesystem;
- S3-compatible object storage;
- WebDAV;
- OneDrive;
- Google Drive;
- Dropbox-like systems;
- NAS;
- self-hosted object stores;
- future storage services.

The Adapter MAY expose different capabilities depending on the underlying provider.

It MUST NOT determine:

- Project identity;
- Revision identity;
- Line semantics;
- Release semantics;
- Contribution semantics;
- DAW state;
- creative provenance;
- Open Music Platform behaviour.

Those belong elsewhere.

---

# 2. Architectural boundary

Conceptually:

```text
+-----------------------------------+
|            OMVCS Core             |
|                                   |
| objects / replicas / migration    |
| integrity / policy / reachability |
+----------------+------------------+
                 |
          Storage Adapter Contract
                 |
                 v
+-----------------------------------+
|         Storage Adapter           |
|                                   |
| auth / paths / APIs / retries     |
| provider-specific transport       |
+----------------+------------------+
                 |
                 v
+-----------------------------------+
|        Physical Provider          |
|                                   |
| S3 / WebDAV / Drive / NAS / FS    |
+-----------------------------------+
```

Core MUST NOT need to understand provider-specific APIs.

---

# 3. Storage Adapter identity

Every Storage Adapter implementation MUST expose a stable Adapter Identifier.

Recommended format:

```text
reverse-DNS
```

Examples:

```text
org.openmusic.storage.filesystem
org.openmusic.storage.s3
org.openmusic.storage.webdav
com.vendor.storage.onedrive
```

The Adapter Identifier identifies the implementation family, not one configured Storage Endpoint.

---

# 4. Adapter version information

The Adapter MUST expose:

```text
adapter_id
adapter_version
storage_contract_version
provider_family
supported_provider_versions where relevant
```

Example:

```json
{
  "adapter_id": "org.openmusic.storage.s3",
  "adapter_version": "0.2.0",
  "storage_contract_version": "omvcs.storage-adapter/0.1",
  "provider_family": "s3-compatible"
}
```

---

# 5. Storage Endpoint identity

A **Storage Endpoint** is a configured instance of a Storage Adapter.

Examples:

```text
Joakim S3 bucket
Anna OneDrive
Studio NAS
Local archive drive
```

Each Endpoint MUST have a stable OMVCS Endpoint Identifier.

The Endpoint Identifier MUST remain stable if:

- credentials rotate;
- access tokens change;
- display name changes;
- temporary provider session details change.

It MAY change if the user deliberately creates a logically new Endpoint.

---

# 6. Endpoint Descriptor

Core maintains an operational Endpoint Descriptor.

Conceptually:

```json
{
  "endpoint_id": "019ef...",
  "adapter_id": "org.openmusic.storage.s3",
  "display_name": "Main Project Storage",
  "owner_actor_id": "019cc17d-1b22-7a41-9fe9-c345c468f82c",
  "capabilities": [],
  "configuration_reference": "local-secret-store:s3-main",
  "status": "available"
}
```

The Descriptor MUST NOT contain raw credentials in historical metadata.

---

# 7. Secrets boundary

Credentials belong to a secure local or platform-specific secret mechanism.

Examples include:

- OAuth refresh tokens;
- API keys;
- S3 access keys;
- signed session tokens;
- passwords;
- client secrets.

These MUST NOT be:

- embedded in Revision objects;
- embedded in Project State;
- embedded in historical Resource References or Resource Manifest views;
- copied into Platform Mirror metadata.

---

# 8. Endpoint configuration

Provider configuration MAY include operational values such as:

```text
bucket
container
base path
tenant
region
provider URL
drive identifier
account identifier
```

Configuration that does not constitute a secret MAY be stored in operational repository metadata if necessary.

Provider-specific configuration MUST NOT participate in creative object identity.

---

# 9. Capability declaration

Each Adapter MUST explicitly declare supported capabilities.

Conceptual example:

```json
{
  "read": true,
  "write": true,
  "delete": true,
  "range_read": true,
  "conditional_write": true,
  "atomic_create": true,
  "server_side_copy": true,
  "list": true,
  "metadata": true,
  "object_versioning": false,
  "multipart_upload": true,
  "strong_read_after_write": true
}
```

Capabilities MUST be truthful.

Core MUST NOT assume a capability that is undeclared.

---

# 10. Required capabilities

A Storage Adapter capable of acting as a Resource Storage Endpoint MUST support at minimum:

```text
read
write
exists/stat
delete or explicit non-deletable semantics
content verification support
```

A Storage Adapter claiming **Repository Home** conformance MUST additionally
provide all of the following for one OMVCS Project:

```text
durable storage and retrieval of locally admitted immutable historical
  metadata required by the implemented Core repository model
durable storage and retrieval of operational repository metadata required
  by implemented Core contracts
conditional and logically atomic guarded updates for mutable operational
  metadata, including the complete Project Storage Map and its
  StorageMapGeneration under Core §34
the Repository Home marker and discovery information defined in §§124–126
durable-write acknowledgement meeting the Storage Adapter's declared
  durability semantics
```

Historical metadata MAY be locally incomplete only with the applicable
declared-boundary state. Repository Home status MUST NOT be treated as proof
that metadata history is complete.

Resource and Chunk byte storage are not required capabilities of a
Repository Home. A Home MAY also provide them only when it independently
advertises the applicable Resource Storage capabilities. Repository Home
capability and Resource Storage capability are separate.

The Platform MUST NOT serve as the authoritative Repository Home merely
because it stores or mirrors OMVCS metadata.

---

# 11. Read-only endpoints

An Adapter MAY expose an Endpoint as read-only.

Example:

```text
archival drive mounted read-only
public contributor storage
historic repository snapshot
```

Core MAY retrieve from it but MUST NOT attempt write/migration completion through it.

---

# 12. Write-only or restricted endpoints

Providers with unusual permissions MAY expose restricted capability combinations.

Core MUST reject operations whose required capabilities are unavailable.

It MUST NOT silently assume success.

---

# 13. Storage Adapter conformance classes

OMVCS 0.1 defines conceptual classes.

## Class R — Resource Storage

Supports storage and retrieval of immutable Resource Objects or Chunks.

## Class H — Repository Home

Supports the complete minimum Repository Home capability profile in §10
for one Project. Resource and Chunk byte storage are not required.

## Class RH — Combined

Supports both Resource Storage and Repository Home functionality, with
each capability independently declared.

## Class A — Archival

Supports durable export/import but may not support active repository mutation.

These are capability groupings, not user-facing names.

---

# 14. Logical key namespace

Storage Adapters operate on **logical OMVCS keys** supplied by Core.

The canonical logical keys for immutable Resource and Chunk bytes are
defined in Core Specification §8.3. Conforming implementations MUST use
those exact keys for those objects.

Other metadata operations use the logical keys or object identifiers
defined by their applicable Core/Storage operation contracts. This section
does not define a canonical metadata-key path layout.

The Resource and Chunk key forms are:

```text
resources/sha256/<p1>/<p2>/<digest>
chunks/sha256/<p1>/<p2>/<digest>
```

The digest is the complete 64-character lowercase hexadecimal SHA-256
digest of the corresponding ResourceId or ChunkId. `<p1>` is characters
1–2 of that digest and `<p2>` is characters 3–4. The typed identifier is
authoritative; a logical key is an operational address and MUST NOT be
used as identity.

Logical keys use `/` separators regardless of host OS. The canonical
Resource and Chunk keys are relative, consist only of ASCII lowercase
components, and contain no empty, `.` or `..` segments, drive letters,
absolute-path syntax, platform-dependent separators, or user-controlled
filename components. They contain no Resource metadata, Project name,
creator name, timestamps, Endpoint identifier, or credentials.

A provider MAY add its configured physical storage-root prefix or use an
internal physical mapping, provided that it preserves the externally
observable logical key, does not create aliases, and does not change
content or historical identity.

Provider-specific paths MUST NOT become historical identity.

---

# 15. Key mapping

An Adapter MAY transform an OMVCS logical key to satisfy provider restrictions.

Examples:

```text
path length limits
illegal characters
case-insensitive filesystem
object-name restrictions
```

The physical mapping MUST be deterministic and reversible or persistently
recorded. It MUST preserve the logical key presented at the OMVCS boundary.
A mapping MUST NOT truncate a key or cause two distinct logical keys to
alias the same physical object.

A provider-specific mapped key MUST NOT replace the OMVCS identity of the object.

---

# 16. Resource storage model

Core may request storage of:

```text
complete Resource Object
```

or:

```text
Chunks + Chunk Manifest
```

depending on repository policy and Adapter capability.

The Adapter MUST treat supplied immutable content as immutable.

It MUST NOT alter, recompress, normalize or transcode the bytes.

---

# 17. Byte preservation

Storage Adapters MUST preserve exact bytes.

A Resource stored as:

```text
SHA256 = ABC
```

must retrieve byte-for-byte so that:

```text
SHA256(retrieved bytes) = ABC
```

If the provider performs transparent transformation that changes bytes, that provider representation is not valid for OMVCS immutable Resource storage unless the Adapter compensates losslessly.

---

# 18. Provider metadata is non-authoritative

Provider metadata such as:

```text
ETag
Last-Modified
provider checksum
version ID
```

MAY be used operationally.

It MUST NOT replace OMVCS content verification unless the specification explicitly establishes equivalence.

The OMVCS hash remains authoritative.

---

# 19. PutResource

The Adapter MUST implement a logical operation equivalent to:

```text
PutResource
```

Inputs conceptually include:

```text
Endpoint
Resource Identifier
byte length
content stream
logical storage key
expected hash
operation ID
```

The operational byte-length input measures the Resource bytes and is not a historical Resource Reference field or alternate encoding. If the Resource is represented by a historical Resource Reference, its `byte_length` MUST satisfy Core Specification §7.

The Adapter MUST either:

- persist the exact content;
- or fail.

It MUST NOT return success before the provider's durability semantics satisfy the Adapter's declared guarantee.
For a Resource stored under its canonical logical key, the Adapter MUST
validate the key against the supplied ResourceId before native path
resolution. The key MUST be derived from that identifier as specified in
Core §8.3; caller-supplied filename or metadata MUST NOT participate.

---

# 20. Existing object optimization

If the destination already contains the requested object, the Adapter MAY avoid re-uploading it.

Before treating the operation as satisfied, it MUST establish that the stored object corresponds to the requested Resource Identifier.

How strongly this must be verified depends on available provider guarantees and previous verification metadata.
If the bytes at the canonical key do not verify as the requested immutable
object, the Adapter MUST report an integrity/storage conflict and MUST NOT
silently overwrite them.

---

# 21. Atomic immutable object creation

Where provider semantics support it, immutable objects SHOULD be created with:

```text
create-if-absent
```

semantics.

Concurrent uploads of the same Resource MUST result logically in one Resource Object.

Concurrent attempts to write different bytes to the same content-derived key MUST be treated as integrity failure.

For the filesystem Adapter, immutable Resource and Chunk creation MUST use
a safe temporary-write and atomic-publication process: write complete bytes
to a temporary object inside the controlled filesystem root, perform the
applicable flush/durability handling, verify the bytes as required, then
publish atomically to the canonical destination without replacing an
existing object. A concurrent create MUST be resolved without an
existence-check/creation race that can overwrite the existing object.
Interrupted temporary writes MUST NOT appear at the final canonical key.
If the platform cannot provide safe no-replace publication and the
required containment/durability guarantees, the operation MUST fail
explicitly rather than weaken these requirements.

---

# 22. PutChunk

For Chunk-capable storage, the Adapter MUST support a logical:

```text
PutChunk
```

with the same immutable semantics as Resource storage.

Chunk identity is derived from Chunk bytes.

---

# 23. GetResource

The Adapter MUST implement:

```text
GetResource
```

or equivalent reconstruction from Chunks.

The output MUST be a byte stream representing exactly the requested Resource Object.

Core performs or requests verification.

---

# 24. GetChunk

Chunk-capable Adapters MUST provide:

```text
GetChunk
```

for the requested Chunk Identifier.

Retrieved bytes MUST verify against the Chunk Identifier.

---

# 25. Streaming

Read and write operations MUST support streaming.

The contract MUST NOT require full Resource Objects to fit into memory.

This is mandatory because Resources may be:

```text
hundreds of MB
multiple GB
potentially larger
```

---

# 26. Range reads

Adapters SHOULD support range reads where the provider does.

Range reads are useful for:

- streaming preview;
- partial verification;
- resume;
- chunk reconstruction;
- large-file access.

If unsupported, Core MUST fall back to full-object reads where valid.

---

# 27. Resumable upload

Adapters SHOULD support resumable upload for large objects.

Provider mechanisms may include:

```text
multipart upload
chunked upload session
resumable HTTP transfer
temporary partial object
```

The Adapter MUST expose whether an interrupted upload can be resumed.

---

# 28. Resumable download

Adapters SHOULD support resumable retrieval where possible.

A resumed download MUST still result in final whole-object verification.

---

# 29. Temporary upload state

Incomplete uploads MUST NOT be registered as Resource Replicas or reported
as verified.

They may exist physically as:

```text
temporary multipart state
staging object
partial file
```

but remain operationally:

```text
pending
```

until complete and verified. Such pending upload state is not a registered
Resource Replica.

---

# 30. Replica lifecycle

A storage workflow MAY use operational labels such as:

```text
planned
uploading
stored_unverified
verified
degraded
unavailable
corrupt
removing
removed
```

`planned`, `uploading`, and `stored_unverified` describe candidate
representations before registration, not registered Resource Replicas.
These illustrative workflow/status values do not form one availability or
verification taxonomy. `corrupt` is an integrity finding, not an
availability state; unavailable does not imply corrupt, and existence does
not imply verified. A candidate may be registered as a Resource Replica
only after the applicable Content Verification requirements succeed.
Exact storage-provider internals may differ.

Core-visible state MUST never mark a replica `verified` before verification requirements are satisfied.

New registration requires `resource_identity` assurance under §§32–34.
`chunk_identity` for some or all constituent Chunks alone is insufficient
unless the complete deterministic reconstruction requirements in §33 also
establish `resource_identity`.

These registration requirements apply when admitting a new candidate. A
Repository Home's reconstruction of its authoritative persisted Storage
Map is not a new registration and does not require re-verifying Resource
bytes solely because the map is loaded; §51 defines that persistence
boundary.

---

# 31. VerifyResource

The Adapter MUST support a logical operation equivalent to:

```text
VerifyResource
```

This may operate by:

- downloading and hashing bytes;
- hashing locally immediately after upload;
- using a provider checksum proven equivalent;
- reconstructing Chunks and establishing Resource-level assurance under §33.

The verification method MUST be reported.

The result MUST distinguish outcome, verification strength, and
verification method/evidence. Missing or inaccessible bytes MUST NOT be
reported as an identity mismatch.

---

# 32. Verification strength

Verification strength states what identity proposition was proved.
Verification method/evidence states how it was proved. OMVCS 0.1 defines
these strengths:

```text
chunk_identity
resource_identity
```

`chunk_identity` means the exact bytes of one Chunk were obtained and
cryptographically verified against that ChunkId. It proves only that Chunk.
It does not prove complete Resource reconstruction, completeness, order, or
ResourceId.

`resource_identity` means the complete Resource byte sequence was
cryptographically verified against its ResourceId. It may be established by
complete-object hashing or the deterministic verified-reconstruction path
in §33.

Outcomes are `verified`, `failed`, or `indeterminate`. `failed` means bytes
actually checked against the required identity did not match. Missing or
unavailable bytes produce an indeterminate/unavailable outcome and are not
corruption. An OMVCS verification result identifies its subject, outcome,
the requested/established strength, and method/evidence. Any timing or
source detail is operational. Evidence MUST NOT include credentials or
transient signed URLs. Methods include direct byte-read hashing,
deterministic Chunk reconstruction, and `provider_equivalent_checksum`;
these are methods, not strengths.

OMVCS publication policy may require a particular verification strength.

`chunk_identity` MUST NOT be silently represented as
`resource_identity`.

---

# 33. Full Resource verification

The strongest verification is:

```text
SHA256(reconstructed complete Resource bytes)
=
Resource Identifier
```

This MUST be supported at least when deep verification is explicitly requested.

This is `resource_identity` verification. It may be established by either:

1. obtaining or reconstructing the destination's complete Resource bytes
   and hashing them against ResourceId; or
2. for a chunked representation, establishing deterministic verified
   reconstruction without physically rereading the same bytes into a
   monolithic buffer solely to hash them again, when all of the following
   hold:
   - the Chunk Manifest and reconstruction description are deterministic
     and complete;
   - its ordered Chunk sequence is the one defined for the intended Resource
     representation by the applicable OMVCS Chunking policy;
   - every required destination Chunk has independently verified
     `chunk_identity` evidence for its exact ChunkId;
   - lengths and ordering reconstruct exactly the complete intended byte
     sequence;
   - an existing `resource_identity` result or equivalent approved proof
     binds that exact complete ordered manifest and its reconstruction to
     the expected ResourceId, and the destination's verified ChunkIds and
     order establish that it has the same complete bytes; and
   - no required Chunk or evidence is missing, stale, or indeterminate.

Provider-equivalent checksum evidence may satisfy a strength only if it
proves the exact OMVCS SHA-256 identity over the exact same byte sequence:
one Chunk for `chunk_identity`, or the complete Resource for
`resource_identity`. Generic provider success, existence, length, ETag,
provider version, CRC, multipart ETag, or a checksum without proven
algorithm and byte scope MUST NOT be treated as verification.

---

# 34. Chunk verification

When chunked storage is used:

```text
each Chunk hash
```

MUST verify individually.

Each successful Chunk check yields at most `chunk_identity` for that exact
Chunk. Missing or unavailable Chunk bytes yield an indeterminate/unavailable
result; bytes that were checked and fail the ChunkId are failed/corrupt.
Complete Resource assurance still requires `resource_identity` under §33.
Every OMVCS 0.1 reference chunked representation MUST use the deterministic
fixed-size sequential policy defined by Core §8. Provider-internal
segmentation is not an OMVCS Chunk representation.

---

# 35. StatResource

The Adapter MUST support a lightweight operation equivalent to:

```text
StatResource
```

returning provider-visible information such as:

```text
exists
byte length
last modified
provider checksum
provider version ID
accessibility
```

`StatResource` MUST NOT itself imply cryptographic verification.
Provider checksums and version identifiers returned by `StatResource` are
not verification evidence unless separately proven equivalent under §§32–33.

The operational byte-length result measures the Resource and is not a historical Resource Reference field or alternate encoding. If the Resource is represented by a historical Resource Reference, its `byte_length` MUST satisfy Core Specification §7.

---

# 36. Object existence is not integrity

This distinction is normative:

```text
exists
```

does not mean:

```text
verified
```

Core MUST treat those states separately.

Only bytes actually checked and found not to match the required immutable
identity establish corruption. Corruption is per Replica: it does not change
ResourceId, invalidate another Replica, or corrupt the Resource Object.

---

# 37. DeleteReplica

Where deletion is supported, the Adapter MUST implement:

```text
DeleteReplica
```

Core determines whether deletion is permissible according to reachability, retention and replica policy.

The Adapter performs the physical deletion.

The Adapter MUST NOT independently garbage-collect creative data.

---

# 38. Delete confirmation

Deletion MUST return a defined outcome:

```text
deleted
already_absent
permission_denied
retention_locked
provider_error
unknown
```

`already_absent` MAY be treated as idempotent success where appropriate.

---

# 39. Provider retention/versioning

Some providers may retain deleted versions.

This does not mean OMVCS still considers the Replica active.

The Adapter SHOULD report provider-side retention/versioning capability where known.

OMVCS Resource Replica state concerns retrievable configured storage, not invisible provider backup semantics.

---

# 40. Object immutability enforcement

Adapters SHOULD use provider-side immutability features when practical:

```text
object lock
write-once mode
immutable bucket policies
read-only archival permissions
```

but these features are optional.

OMVCS content-addressed identity remains the primary semantic guarantee.

---

# 41. List operation

Repository Home capable Adapters SHOULD support logical listing by prefix.

Example:

```text
metadata/revisions/
chunks/sha256/
```

Listing is useful for:

- validation;
- recovery;
- garbage collection;
- import;
- orphan discovery.

If a provider cannot efficiently list, Core MAY rely on explicit indexes.

---

# 42. Listing consistency

Adapters MUST declare whether provider listing is:

```text
strongly consistent
eventually consistent
unknown
```

Core MUST NOT assume that a just-written object immediately appears in list results if the provider does not guarantee it.

---

# 43. Read-after-write semantics

Adapters MUST declare whether new writes are immediately readable.

Possible declarations:

```text
strong
eventual
unknown
```

Publication logic may need to verify readback depending on provider guarantees.

---

# 44. Conditional writes

Repository Home capable Adapters SHOULD support conditional updates.

Examples:

```text
write if generation = N
write if ETag = X
create if absent
```

This is critical for safe concurrent updates to:

- Lines;
- Storage Map generations;
- operational metadata.

An Adapter/Repository Home that accepts Core Storage Map mutations MUST
provide conditional persistence sufficient to atomically guard the complete
Project Storage Map and its `StorageMapGeneration`, including a mutation
that changes multiple entries. If that guarantee is unavailable, the
operation MUST be reported as unsupported or as an applicable provider
failure; the Adapter MUST NOT weaken the Core CAS contract or report
success.

---

# 45. Compare-and-swap abstraction

The Adapter SHOULD expose a provider-independent operation equivalent to:

```text
ConditionalPut
```

Inputs:

```text
logical key
new bytes
expected current version/generation/token
```

Possible outcomes:

```text
success
conflict
unsupported
not_found
permission_denied
provider_error
```

Core handles the semantic consequence.
The `expected current version/generation/token` may be a provider token for
this conditional write. It is not thereby the portable Core
`StorageMapGeneration`; the Adapter maps the Core generation comparison
onto provider facilities.

---

# 46. Repository metadata write

Repository Home metadata MUST be written in a manner that prevents torn/corrupt state.

Valid implementation techniques include:

```text
immutable object + atomic pointer update
temporary file + atomic rename
versioned object + compare-and-swap head
transactional provider write
```

The exact method is provider-specific.

Filesystem Repository Home bootstrap is governed by §§124–126 as one
logical atomic initialization operation. Individual metadata writes MAY be
staged separately, but the Home MUST NOT be exposed as initialized until
its marker and complete required initial operational state are durable and
mutually consistent. This bootstrap rule does not generalize to
non-filesystem Homes.

---

# 47. No assumed POSIX semantics

Core MUST NOT assume Storage Endpoints provide:

```text
atomic rename
directories
file locking
hard links
filesystem timestamps
POSIX permissions
```

Those concepts may not exist on object storage.

The Storage Adapter abstracts them.

---

# 48. No assumed object-store semantics

Likewise Core MUST NOT assume:

```text
flat namespace
ETag
bucket
multipart upload
server-side copy
```

because a local filesystem or Drive-like service may behave differently.

---

# 49. Metadata object persistence

Historical metadata objects are immutable.

Repository Home adapters SHOULD store them under content-derived keys.

Repeated writes of the same immutable metadata object MUST be idempotent.

---

# 50. Mutable operational pointers

Mutable records such as:

```text
Line targets
Storage Map current generation
Repository Home state
sync markers
```

MUST use guarded update semantics.

They MUST NOT rely on blind overwrite under concurrency.

For Storage Map updates, the guarded state is the entire Project Storage
Map together with its Core `StorageMapGeneration`. The conditional write
MUST provide the logical atomicity defined by Core §34, regardless of
provider-internal write count.

---

# 51. Storage Map persistence

Repository Home MUST durably persist the Storage Map or enough operational log/state to reconstruct it.

The logical Storage Map and its `StorageMapGeneration` MUST be persisted or
reconstructed together as one guarded state. A persisted map with an old
generation, a new generation with an old map, or a partial multi-entry
mutation MUST NOT be exposed as a successful state. Repository Home
initialization MUST establish the empty Storage Map at generation `0`.

For a filesystem Repository Home, the empty map and generation `0` become
authoritative only at the logical bootstrap commit point defined in §124.
They MUST be established together with the valid marker and other required
initial operational state. Bootstrap initialization is not a Storage Map
mutation and MUST NOT advance generation to `1`. Discovery and retry MUST
classify interrupted or inconsistent initialization as specified in
§§124–125; ordinary map reconstruction and later mutations retain their
existing contracts.

The persisted Storage Map at a Repository Home is authoritative for its
prior successful Replica registrations. Repository Home MUST persist only
records admitted through the applicable registration requirements and
MUST uphold that authority when reconstructing the map. Reconstruction
MUST validate the map's structure and consistency with its generation, but
MUST NOT re-verify Resource bytes solely because the map is being loaded.
Deserializing or decoding a Replica record alone does not establish that
the record came from an authoritative Repository Home and does not register
it. A record from another source MUST satisfy the applicable new-registration
requirements before it is admitted as a registered Resource Replica.

The Platform Mirror MUST NOT be the sole copy.

---

# 52. Repository Operation Log persistence

If the Core implementation uses the recommended append-only Repository Operation Log, Repository Home adapters MUST be capable of storing it durably.

The log MAY be stored as:

```text
one immutable object per operation
segmented append-only files
journal objects
provider-native append structure
```

provided ordering and integrity semantics are preserved.

---

# 53. Operation ordering

Where operational log sequence matters, Core assigns or validates logical generation/order.

The Adapter MUST NOT infer repository semantics merely from provider timestamps.

---

# 54. Server-side copy

Adapters MAY support provider-native:

```text
ServerSideCopy
```

This can optimize replication within the same provider.

However, after copy the destination MUST still satisfy OMVCS verification requirements.

Provider copy success alone is not automatically Resource identity verification unless equivalent guarantees are established.

---

# 55. Cross-endpoint copy

Core orchestrates cross-endpoint migration.

Conceptually:

```text
Source Adapter -> byte stream -> Destination Adapter
```

or, where compatible:

```text
provider-native transfer
```

The Storage Adapter Contract MUST permit both.

---

# 56. Migration is Core-owned

Storage Adapters perform physical transfers.

Core owns:

```text
migration intent
replica registration
verification requirement
source-retention decision
Storage Map mutation
```

An Adapter MUST NOT remove the source merely because a copy completed.

---

# 57. Migration transaction

A Core migration operation SHOULD follow:

```text
PLANNED
SOURCE_VALIDATED
COPYING
DESTINATION_STORED
DESTINATION_VERIFIED
REPLICA_REGISTERED
SOURCE_REMOVAL_PENDING
SOURCE_REMOVED
COMPLETE
```

Adapter results feed these states.

The Adapter itself does not create creative history.

---

# 58. Failed migration

If destination verification fails:

```text
new replica MUST NOT be registered as valid
```

and an existing source replica MUST remain untouched unless independently requested.

---

# 59. Replication

Replication is the creation of another valid Resource Replica without removing the source.

Storage Adapters treat replication physically like copy/upload.

Core distinguishes replication from migration semantically.

---

# 60. Deduplication

Adapters MAY exploit provider-level deduplication.

OMVCS Core itself gains natural deduplication from content-addressed identity.

If two Projects reference the same Resource Identifier in a shared storage namespace, only one physical copy MAY exist.

---

# 61. Shared object namespaces

Storage Endpoints MAY serve multiple Projects.

Resource garbage collection MUST therefore consider all Projects sharing that namespace.

The Adapter SHOULD expose namespace identity where needed.

---

# 62. Endpoint namespace identifier

A Storage Endpoint SHOULD expose a stable namespace identifier for the logical object store it represents.

Example:

```text
S3 bucket + configured root prefix
NAS repository root
Drive folder acting as OMVCS root
```

This assists:

- deduplication;
- garbage collection;
- migration;
- conflict avoidance.

---

# 63. Key collision safety

Because immutable objects use content-derived keys, a destination containing different bytes at the same logical content key indicates severe corruption or provider misbehaviour.

The Adapter MUST return an integrity error.

It MUST NOT overwrite such content silently.

---

# 64. Local filesystem adapter

The local filesystem Adapter is a first-class Storage Adapter.

It MUST validate a canonical Resource or Chunk logical key before resolving
it to a native path. Validation MUST reject any key that does not exactly
match the applicable Core §8.3 form, including alternate-case spellings,
absolute paths, drive letters, backslashes, empty or dot segments, and
extra path components.

Resolution MUST remain contained beneath the explicitly configured
filesystem storage root. The Adapter MUST NOT follow a symlink or
reparse-point component in a way that permits access outside that root.
It MUST use safe no-follow/open or equivalent containment semantics across
path resolution and the operation; if those guarantees cannot be provided
on a supported platform, the operation MUST fail explicitly.

The Adapter MUST prevent a case-folded or alternate-case physical alias
from satisfying or replacing a canonical logical key. It MUST NOT silently
truncate or normalize a key to satisfy native path restrictions. It MAY use
the deterministic physical mapping permitted by §15, or fail explicitly
when the native path cannot be represented safely.

Filesystem writes MUST satisfy the atomic immutable creation requirements
in §21. Temporary objects MUST remain inside the controlled root and MUST
NOT be exposed at a canonical key before complete write, required
durability handling, and verification.

Filesystem Repository Home bootstrap staging and commit machinery MUST
also remain beneath the explicitly configured repository root. It MUST
follow the containment and no-follow requirements above, MUST NOT follow
symlink/reparse components outside the root, and MUST NOT resolve through
caller-controlled arbitrary paths. Private staging paths are not
interoperable layout and MUST NOT be exposed as authoritative repository
metadata before bootstrap commit. If the platform cannot provide the
required containment and logical publication guarantees, Home capability
MUST be reported as unsupported.

It SHOULD also use:

- deterministic directory layout;
- file permissions appropriate to the host OS.

It MUST remain portable across supported operating systems.

---

# 65. Filesystem case sensitivity

Canonical Resource and Chunk logical keys use lowercase hexadecimal and
lowercase path components. The filesystem Adapter MUST NOT depend on a
host filesystem's case-sensitive path distinction and MUST preserve the
exact canonical logical-key namespace on case-insensitive filesystems.

---

# 66. Filesystem path length

The filesystem Adapter MUST account for host path-length and native-name
limitations without truncation or aliasing. It MAY use the deterministic
physical mapping allowed by §15 while preserving the canonical logical
key. If a path cannot be represented safely under the selected mapping,
the Adapter MUST fail explicitly.

---

# 67. Filesystem links

Hard links or reflinks MAY be used as optimization only where they preserve
the immutable-byte, containment, and no-replace requirements.

They MUST NOT be required for correctness.

---

# 68. S3-compatible adapter

An S3-compatible Adapter MAY use:

```text
bucket
prefix
multipart upload
range GET
conditional writes
object metadata
server-side copy
```

but Core must remain unaware of these details.

---

# 69. WebDAV adapter

A WebDAV Adapter MAY have weaker guarantees.

It MUST declare whether it supports:

```text
conditional writes
locking
atomic replacement
range reads
reliable listing
```

Core may restrict Repository Home use if guarantees are insufficient.

---

# 70. Drive-like adapters

Drive-like systems may expose:

```text
file IDs
folder IDs
revision IDs
rename/move APIs
eventual indexing
```

The Adapter MUST hide these behind OMVCS logical key semantics.

Human-facing folder names MUST NOT become object identity.

---

# 71. Provider-generated file identifiers

Provider-specific stable file IDs MAY be stored as operational replica metadata.

They MUST NOT replace Resource Identifiers.

---

# 72. Provider rename

If a provider changes visible file/path naming while the physical content remains the same, the Adapter may update the Storage Location.

Creative history remains unaffected.

---

# 73. Storage Location representation

A Resource Replica record MUST contain enough operational information to
retrieve or reconstruct exactly one complete Resource representation at
its associated Storage Endpoint. The Resource's physical representation
MUST be bound as exactly one representation. OMVCS 0.1 implementations
MUST support at least the complete-object and chunked representation kinds
defined in Core §32; a chunked representation is bound to an ordered Chunk
Manifest under Core §8.2.

Conceptually:

```json
{
  "replica_id": "019f1234-5678-7abc-8def-0123456789ab",
  "resource_id": "omvcs:resource:sha256:...",
  "endpoint_id": "019ef123-4567-7abc-8def-0123456789ab",
  "representation": {
    "kind": "chunked",
    "manifest": {
      "resource_id": "omvcs:resource:sha256:...",
      "total_length": 1234,
      "chunks": [
        {
          "chunk_id": "omvcs:chunk:sha256:...",
          "offset": 0,
          "length": 1234
        }
      ]
    }
  },
  "locator": {
    "schema": "example.storage.chunk-locator/1",
    "value": {
      "opaque": "provider-specific data"
    }
  },
  "availability": "available"
}
```

Provider locator data MUST use a generic typed envelope equivalent to:

```text
ProviderLocator {
    schema: string identifying the exact versioned locator schema,
    value: canonical JSON data governed by that schema
}
```

The exact field names MAY follow the applicable schema conventions, but the
semantics are normative. The schema identifier MUST use the existing
versioned schema-identifier convention and MUST identify the exact provider
locator schema and version. Generic Core MUST be able to persist,
reproduce, compare, and pass the envelope unchanged to the applicable
Storage Adapter without understanding the provider-specific meaning of
`value`; canonical JSON serialization is the only representation step.
The envelope and `value` MUST use RFC 8785/JCS canonical JSON
serialization, without transformations beyond those canonical JSON rules.
The Storage Adapter/provider owns validation under the named schema and
translation to provider operations. The named schema and its provider
validator MUST reject credentials, access tokens, expiring signed URLs,
other secrets, and process-local handles, and MUST ensure the locator can
rediscover the physical representation after restart.

The Endpoint Identifier and ProviderLocator together identify where the
Replica representation can be accessed. A locator is interpreted only in
the context of its Endpoint and Storage Adapter. The same locator value at
two Endpoints MUST NOT imply the same physical object or Replica.

An `availability` value describes retrieval only. It MUST NOT be treated
as evidence of integrity or verification.

ProviderLocator is mutable operational metadata. It MUST NOT be Resource
identity, Replica identity, Chunk identity, or historical metadata. It MUST
NOT contain credentials, access tokens, expiring signed URLs, or other
secrets; rely on process-local handles; or prevent rediscovery of the
physical representation after restart. Temporary access grants remain
separate from ProviderLocator.

For `complete-object`, the locator MUST permit access to the provider
object whose bytes are the complete Resource. For `chunked`, it MUST permit
the Adapter to retrieve every Chunk identified by the Replica's manifest.
Provider-specific Chunk locations are reconstruction details of that
Resource Replica; an individual Chunk copy is not a Resource Replica.
The locator's availability or provider-object existence does not prove
content verification or registration eligibility.

---

# 74. Locator mutability

The physical locator MAY change.

Example:

```text
folder moved
object renamed internally
provider returns replacement file ID
```

Updating locator metadata MUST NOT change Resource identity.

Updating the locator for the same logical physical representation within
the same Endpoint/provider context MUST retain the ReplicaId. Creating a
distinct independently addressable copy MUST create a new Replica record
with a new ReplicaId.

---

# 75. Preferred replica

Core MAY mark a Replica or Endpoint as preferred.

Preference may reflect:

```text
speed
cost
locality
ownership
availability
user choice
```

The Adapter simply performs the requested operation.

Preference does not imply authority over history.

---

# 76. Availability probe

Adapters SHOULD expose:

```text
ProbeAvailability
```

that determines whether a known Replica appears retrievable.

This SHOULD avoid full Resource download.

Possible results:

```text
available
temporarily_unavailable
permission_denied
not_found
unknown
```

Availability probe is not content verification.

---

# 77. Provider outage

Temporary provider failure MUST NOT be translated into:

```text
Resource missing forever
```

unless sufficient evidence exists.

The Adapter SHOULD distinguish:

```text
network failure
authentication failure
provider outage
object not found
```

---

# 78. Authentication failure

An authentication error means:

```text
current client cannot access endpoint
```

not necessarily:

```text
resource does not exist
```

This distinction MUST be preserved.

---

# 79. Authorization failure

Similarly:

```text
permission denied
```

MUST remain distinct from:

```text
not found
```

where the provider permits the distinction.

---

# 80. Provider ambiguity

Some providers intentionally obscure unauthorized objects as `not found`.

Adapters SHOULD report such cases as:

```text
not_found_or_inaccessible
```

rather than invent certainty.

---

# 81. Rate limiting

Adapters MUST identify provider rate-limit errors where possible.

They SHOULD return retry metadata such as:

```text
retry_after
backoff_hint
```

Core may schedule retries.

---

# 82. Retry policy

Adapters SHOULD perform only transport-level retries that are unquestionably safe.

Repository-semantic retries belong to Core.

The Adapter MUST NOT repeat non-idempotent provider actions blindly if duplicate effects are possible.

---

# 83. Exponential backoff

Adapters MAY use exponential backoff and jitter for transient provider errors.

This is operational behaviour.

It must not affect repository identity or semantics.

---

# 84. Retry exhaustion

When retries are exhausted, the Adapter MUST return a structured transient/permanent failure classification.

It MUST NOT block indefinitely.

---

# 85. Error model

Every Storage Adapter error MUST provide:

```text
error code
error class
operation
endpoint
recoverability
provider context where safe
human-readable summary
```

Example:

```json
{
  "code": "OMVCS_STORAGE_AUTH_EXPIRED",
  "class": "authentication",
  "recoverable": true,
  "operation": "GetResource",
  "endpoint_id": "019...",
  "summary": "Storage authorization has expired"
}
```

---

# 86. Error classes

At minimum:

```text
configuration
authentication
authorization
network
provider_unavailable
rate_limit
not_found
conflict
integrity
quota
retention
unsupported
cancelled
internal
```

---

# 87. Quota exhaustion

Quota exhaustion MUST be explicit.

Publication MUST NOT claim durability when required uploads fail due to quota.

Core/UI may then offer another Storage Endpoint.

---

# 88. Provider capacity unknown

If provider capacity cannot be queried, the Adapter SHOULD report:

```text
capacity_unknown
```

rather than assuming sufficient storage.

---

# 89. Estimate required storage

Adapters MAY expose provider free-space/capacity where possible.

Core MAY use this to estimate whether a migration or publication can succeed.

The value is advisory.

---

# 90. Storage cost metadata

Adapters MAY expose optional cost information.

Examples:

```text
egress class
storage tier
estimated per-GB cost
```

OMVCS correctness MUST NOT depend on cost metadata.

---

# 91. Cold/archive storage

Adapters MAY expose storage classes requiring delayed retrieval.

Example:

```text
archive
deep archive
offline media
```

A Resource Replica in such storage may be valid yet not immediately materialisable.

---

# 92. Retrieval latency class

Adapters MAY classify Replicas as:

```text
online
nearline
archive
offline
```

Core can use this for replica selection.

---

# 93. Archival endpoint

An archival Endpoint may:

- accept immutable Resources;
- reject ordinary deletion;
- have slow retrieval;
- not support active Repository Home mutation.

This is valid if its capability declaration is accurate.

---

# 94. Encryption at rest

Provider-side encryption MAY be used.

It is operational and MUST be transparent to OMVCS byte identity after retrieval.

---

# 95. Client-side encryption

OMVCS MAY later support client-side encrypted Resource storage.

For 0.1, if implemented, the Adapter must distinguish:

```text
logical Resource identity
stored encrypted representation
```

The encryption layer MUST allow exact reconstruction and verification of the original Resource.

The cryptographic design itself is not being invented inside this document.

---

# 96. Compression

Transparent storage compression MAY be used only if retrieval reconstructs exact original bytes.

Provider representation is not Resource identity.

---

# 97. Encryption keys are secrets

Client-side encryption keys MUST NOT be stored in historical creative metadata.

Loss of such keys may affect availability and MUST be reported honestly.

---

# 98. Data residency

Adapters MAY report provider region/residency information.

This is operational policy information and does not affect Resource identity.

---

# 99. Privacy classification

Adapters SHOULD permit Core/UI to know whether an Endpoint is:

```text
private
shared
publicly readable
publicly writeable
```

Public write access SHOULD be treated as unsafe for authoritative immutable storage unless additional verification and policy protect integrity.

---

# 100. Public read access

A Resource Replica MAY be publicly readable.

Its content hash still determines authenticity.

Public availability does not alter authorship or ownership.

---

# 101. Temporary access

Adapters MAY generate temporary access mechanisms where provider APIs support them.

Examples:

```text
presigned URL
temporary OAuth-scoped token
short-lived share link
```

Such credentials are operational.

They MUST NOT be stored in historical objects.

---

# 102. Direct playback access

For Open Music Platform playback, a Storage Adapter MAY support:

```text
CreateTemporaryReadGrant
```

allowing media to flow:

```text
creator-controlled storage -> listener
```

without passing through central platform storage.

This directly supports the Core invariant that the platform does not hold Resource bytes.

---

# 103. Temporary read grant contract

A temporary read grant SHOULD specify:

```text
Resource or Replica
scope
expiry
optional byte-range permission
audience where supported
```

The grant MUST be short-lived enough for the intended use.

---

# 104. Grant revocation

Adapters SHOULD expose whether temporary grants can be revoked before expiry.

If unsupported, Core/UI must know that.

---

# 105. Public release storage

A Project MAY choose to make certain Release Resources publicly accessible.

This is an access policy decision.

It does not move ownership or Resource identity to the Platform.

---

# 106. Access-control separation

Storage access control and OMVCS collaboration permissions are related but distinct.

For example:

```text
user may see Contribution metadata
```

does not automatically mean:

```text
user has raw storage credentials
```

The Platform Protocol may coordinate authorization without exposing permanent credentials.

---

# 107. Contributor-owned storage

A contributor may host Resource Replicas on an Endpoint they control.

The Adapter model MUST support this.

Core may know:

```text
resource D
 -> endpoint Anna-Storage
```

without holding Anna's permanent credentials globally.

---

# 108. Capability-scoped access

A contributor MAY grant only:

```text
read specific Resource
```

rather than full Endpoint access.

Storage Adapter implementations SHOULD support least-privilege access where providers permit.

---

# 109. Replica durability metadata

Replica records MAY include durability-relevant information such as:

```text
verified_at
provider storage class
retention lock
known redundancy class
last successful probe
```

These fields are operational.
Verification evidence MUST be scoped to the exact immutable Resource or
Chunk bytes and strength it proves. Evidence for a source Replica does not
automatically establish assurance for a copied destination representation.
It MUST NOT include credentials or transient signed URLs.

---

# 110. Replica freshness

Because Resource Objects are immutable, a verified Replica does not become creatively stale.

However, its **verification freshness** may age.

Core MAY request periodic re-verification.
Age does not change immutable identity. Evidence may be reused only while it
remains applicable to the exact immutable bytes/Chunk identity and
representation; stale or non-applicable evidence cannot establish
registration assurance.

---

# 111. Verification policy

Projects MAY define policy such as:

```text
verify on upload
verify annually
verify before Release
verify before deleting another replica
```

The Adapter performs requested verification.
Such policy MUST use the normative strengths in §32 and MUST NOT redefine
provider operation success, object existence, or matching length as content
verification.

---

# 112. Migration safety rule

Before removing a source Replica during migration, Core SHOULD ensure:

```text
destination replica verified
```

For a new destination registration, "verified" here requires
`resource_identity` assurance under §§32–33. Source verification or provider
copy-success alone does not establish destination assurance.

and:

```text
minimum replica policy still satisfied after removal
```

The Adapter MUST not bypass this on its own.

---

# 113. Bulk migration

Adapters MUST support operations suitable for bulk migration of many Resources.

Core SHOULD orchestrate:

```text
bounded concurrency
progress reporting
retry
resume
verification
```

rather than treating each Resource as an isolated UI action.

---

# 114. Progress reporting

Long-running operations SHOULD report:

```text
operation ID
phase
bytes transferred
resources completed
resources total
current item
transfer rate where available
```

---

# 115. Cancellation

Adapters SHOULD support safe cancellation.

Cancellation of transfer MUST NOT register an incomplete Replica as valid.

---

# 116. Multipart cleanup

Adapters using provider multipart uploads SHOULD clean abandoned multipart state eventually.

Cleanup is operational garbage collection, not creative history garbage collection.

---

# 117. Provider-side orphan objects

Failed uploads may leave temporary or complete unreferenced objects.

Adapters MAY expose cleanup support.

Core determines whether permanent immutable content is safe to delete.

---

# 118. Unknown remote content

A Storage Endpoint may contain content not known to the current Repository.

The Adapter MUST NOT delete unknown content by default.

This is especially important for shared namespaces.

---

# 119. Namespace ownership

An Endpoint Descriptor SHOULD indicate whether OMVCS has:

```text
exclusive namespace control
shared namespace control
unknown namespace control
```

Garbage collection policies may depend on this.

---

# 120. Exclusive namespace

If OMVCS owns an exclusive namespace, Core may safely reason about unreferenced OMVCS objects within that namespace.

It still MUST obey retention rules.

---

# 121. Shared namespace

If the namespace is shared by multiple repositories/projects, deletion requires cross-project reachability knowledge or explicit ownership metadata.

When uncertain, retain.

---

# 122. Garbage collection listing

For Resource garbage collection, the Adapter may need to enumerate stored OMVCS objects.

It SHOULD return:

```text
logical key
provider locator
size
provider metadata
```

Core determines reachability.

---

# 123. Tombstones

Adapters MAY support operational tombstones to record removal.

A tombstone MUST NOT pretend the historical Resource never existed.

Tombstone use is implementation-specific.

---

# 124. Repository Home bootstrap

A filesystem-backed Repository Home MUST use the explicit
caller-selected filesystem repository root. An initial bootstrap MUST
require a new or empty root; a populated directory without a valid marker
MUST NOT be silently reinterpreted as a repository. A retry after
interrupted bootstrap MAY encounter only validated artifacts of that
bootstrap and MUST follow the safe-retry rules below; this does not
authorize adoption of unrelated populated directories.

Bootstrap is one logical atomic initialization operation. A Home is
successfully initialized only when the valid marker defined in §126 and
all required initial operational state under §10 are durable according to
the declared filesystem/storage durability semantics and mutually
consistent. The initial set includes the empty Storage Map at Core
`StorageMapGeneration` `0`. Bootstrap MUST return success only after the
same root can be rediscovered as that initialized Home. The marker's
Project identity, schema, and layout MUST be supported and consistent with
the identities and schemas of required initial operational records.

The externally meaningful bootstrap states are:

```text
uninitialized
initialized
incomplete_initialization
```

`uninitialized` means no valid OMVCS Home has been established at the
selected root. `initialized` means the supported valid marker and complete
required initial operational state are present, durable, and mutually
consistent. `incomplete_initialization` means bootstrap artifacts exist
but the complete required initialization set is absent, invalid, or
inconsistent. It is not a usable Repository Home and MUST be reported
explicitly, not as `not_repository`, initialized Home, or ordinary missing
metadata.

The marker MUST be stored at:

```text
.omvcs/repository.json
```

The `.omvcs` directory is reserved for OMVCS repository metadata at that
root. Bootstrap MUST NOT replace an existing incompatible marker. It MUST
establish the marker, empty Storage Map, Core `StorageMapGeneration` `0`,
and other required initial operational state as one logical atomic
operation. The marker and generation are distinct records; the marker
alone MUST NOT be treated as proof that the Home is initialized. Absence
of a persisted Storage Map record after successful initialization is not
equivalent to generation `0`.

The logical commit point is the durable transition at which the valid
marker and complete required initial operational state become authoritative
together. An implementation MAY prepare these records through a private
staging/bootstrap area under the selected root, persist them according to
the declared durability semantics, and then publish the logical commit
point using supported atomic/publish primitives. Physical multi-file
atomicity is not required if the protocol provides the same all-or-nothing
logical outcome. Exact staging names, structure, and the physical commit
mechanism are implementation details and MUST NOT become normative layout
unless required for interoperability. The marker itself is not the commit
point while required operational state is absent.

If failure or interruption occurs before logical commit, bootstrap MUST
report failure and the root MUST NOT be treated as initialized. Staging
artifacts remain non-authoritative; automatic cleanup is not required.
After logical commit, discovery or retry MUST recognize the initialized
Home even if the caller did not receive the original success response.

Retry at the same root MUST be idempotent for the same Project and
supported layout. A fully valid initialized Home MUST return a typed
`already_initialized` or equivalent idempotent-success outcome and MUST
NOT rewrite generation-zero metadata unnecessarily. If no authoritative
initialized state exists, retry MAY resume or restart only when any
existing marker matches the requested Project and supported schema/layout,
and validated state establishes that completing initialization is safe.
Incomplete artifacts MUST NOT be trusted merely because they exist.
Conflicting/inconsistent marker or operational state MUST fail explicitly
as `incomplete_initialization` or an equivalent typed inconsistency; no
automatic repair is defined. A Home initialized for a different Project
MUST fail with identity mismatch and MUST NOT be overwritten.

Bootstrap MUST fail explicitly if it cannot establish the safe root,
establish the logical commit point, or meet the required durability and
containment guarantees. If the platform cannot provide the required
publish/durability primitives, filesystem Home capability MUST be
reported as unsupported rather than weakening this contract. No creative
history is changed merely by bootstrap.

---

# 125. Repository Home discovery

A filesystem Repository Home discovery operation MUST use only the
explicitly supplied candidate root. It MUST NOT walk parent directories
looking for `.omvcs/repository.json`.

Discovery MUST classify the selected root deterministically:

- no marker and no evidence of bootstrap artifacts: `not_repository` or
  equivalent `uninitialized`;
- complete valid supported marker and complete consistent required
  initial operational state: initialized Repository Home;
- malformed marker with no bootstrap-state inconsistency: invalid
  repository metadata;
- valid but unsupported marker schema/layout: explicit unsupported
  version;
- valid marker with a different requested Project identity: identity
  mismatch;
- partial bootstrap artifacts, a valid marker with missing/invalid
  required initial operational state, or initial operational state without
  a valid marker: explicit `incomplete_initialization`.

`incomplete_initialization` MUST NOT be reported as `not_repository`,
initialized Home, or ordinary missing metadata. Where a more specific
invalid-marker, unsupported-version, or identity-mismatch cause applies,
discovery MUST NOT convert it to success. Discovery MUST NOT infer a Home
from operational files alone. If the complete initialized state is valid,
abandoned non-authoritative staging artifacts that do not conflict with
that state MUST NOT downgrade discovery from initialized; automatic
cleanup remains unnecessary.

Discovery MUST NOT infer Project identity from the folder name or path.

---

# 126. Repository Home root record

A filesystem Repository Home marker is a closed canonical JSON object
containing exactly these fields:

```json
{
  "schema": "omvcs.repository-home/0.1",
  "project_id": "<canonical ProjectId>",
  "layout": "omvcs.storage-layout/0.1"
}
```

The JSON bytes MUST use the canonical serialization rules in Core §5.
`project_id` MUST use the existing canonical ProjectId serialization.
Unknown, missing, duplicate, or incorrectly typed members, invalid
canonical JSON, and invalid ProjectId values MUST be reported as invalid
repository metadata.

If the marker schema or `layout` value is valid but unsupported, discovery
MUST report an explicit unsupported-version result. If a requested Project
identity differs from `project_id`, discovery MUST report an explicit
Project-identity-mismatch result. Only a valid marker with supported schema
and layout and a matching requested Project identity permits
Repository Home interpretation.

The marker establishes intentional repository-root status, marker schema,
Project identity, and storage-layout version only. It is not proof of
history completeness, Resource verification, Platform registration,
publication state, or an operation log. Repository completeness and
operational records MUST be validated independently. A valid marker with
missing, invalid, or inconsistent required initial operational state is
`incomplete_initialization`, not an initialized Home.

The marker MUST NOT contain a separate RepositoryId, Repository format
version, namespace identifier, generation, credentials, or other fields.
The Storage Map and `StorageMapGeneration` are persisted separately under
§51 and Core §34.

---

# 127. Repository Home migration

The destination Adapter MUST support receiving:

```text
available historical metadata
local completeness and declared-boundary information
operational metadata
storage map
operation log or compacted equivalent
```

Migration MUST verify metadata integrity and preserve the source's local
completeness/boundary state before Core designates the destination
authoritative. It MUST NOT claim complete history when the source is
incomplete or not assessed.

---

# 128. Dual-home transition

During migration, Core MAY temporarily treat both old and new Repository Homes as valid.

The Adapter must support operations without confusing repository authority.

Authority choice belongs to Core.

---

# 129. Repository Home failure

If Repository Home becomes unavailable:

- existing local Working State may continue;
- published history remains locally meaningful;
- durable publication may be blocked;
- another Home may later be established.

The Storage Adapter MUST report unavailability, not creative corruption.

---

# 130. Storage Adapter must not invent history

A Storage Adapter MUST NOT:

```text
create Revisions
move Lines
create Releases
create Contributions
alter provenance
reinterpret Project State
```

even if the provider has its own version-history features.

---

# 131. Provider-native version history

A provider may maintain file versions.

OMVCS MAY use this for recovery or operational safety.

Provider versions MUST NOT become OMVCS Revisions automatically.

---

# 132. Provider snapshots

Similarly, filesystem snapshots or object-store versioning may assist recovery.

They are infrastructure mechanisms, not creative history.

---

# 133. Conflict handling

Provider-level conflicts MUST be surfaced accurately.

Examples:

```text
conditional write failed
object modified externally
expected generation mismatch
```

Core determines repository-level response.

---

# 134. External mutation

Someone may alter or delete OMVCS objects directly through the provider.

The Adapter MUST not assume storage is pristine.

Verification and validation must detect such cases where possible.

---

# 135. Mutable object corruption

Historical immutable metadata stored under a content-derived key that no longer hashes correctly is corrupt.

The Adapter MUST return integrity failure.

Core may recover from another copy.

---

# 136. Replica repair

Core MAY repair a corrupt/missing Replica from another verified Replica.

The destination Adapter stores the replacement.

Repair does not create creative history.

---

# 137. Self-healing replication

An implementation MAY automatically restore replica policy after detecting loss.

Example:

```text
policy requires 2 replicas
one endpoint lost
```

Core may create another Replica on an available configured Endpoint.

This is operational behaviour.

---

# 138. No silent provider substitution

If the configured Endpoint is unavailable, the Adapter MUST NOT secretly store data on another provider.

Core must explicitly select the destination.

---

# 139. Cross-project Resource reuse

If the same immutable Resource exists across Projects, shared physical storage MAY reuse it.

This MUST NOT merge Project histories.

Only the Resource bytes are shared.

---

# 140. Cross-project privacy

Deduplication MUST NOT leak whether another Project possesses identical content.

Implementations must avoid exposing shared-object existence across access boundaries where that would reveal private information.

---

# 141. Existence probing

A user without access rights MUST NOT be able to use the Storage Adapter to probe arbitrary Resource hashes in another user's private namespace unless the provider/access policy explicitly allows this.

---

# 142. Storage-side metadata privacy

Provider object metadata SHOULD avoid storing sensitive creative descriptions when unnecessary.

Content-addressed identifiers are preferable to human-readable song/track names in physical keys.

---

# 143. Logical names and provider keys

A Resource called:

```text
Vocals about confidential project.wav
```

need not be stored under a key containing that name.

The Adapter SHOULD favor content-derived opaque keys.

---

# 144. Path sanitization

Filesystem/WebDAV-like Adapters MUST sanitize provider paths.

Untrusted logical names MUST NOT allow:

```text
../ traversal
absolute path injection
reserved device paths
separator abuse
```

Logical Resource names are metadata, not storage paths.

---

# 145. Symlink safety

Filesystem Adapters MUST handle symbolic links safely.

They SHOULD avoid following untrusted links outside the configured Endpoint root unless explicitly permitted.

---

# 146. Resource streaming security

Adapters must treat remote Resource bytes as untrusted input.

They MUST not execute or interpret content merely to store/retrieve it.

Interpretation belongs to DAW or other appropriate layers.

---

# 147. Integrity before use

Core SHOULD verify newly retrieved Resource bytes before passing them to DAW Adapter restoration where feasible.

A known corrupt Resource MUST NOT be presented as valid project content.

---

# 148. Concurrent readers

Adapters MUST permit safe concurrent reads where the provider allows.

Immutable Resource Objects naturally support concurrent access.

---

# 149. Concurrent writers

Multiple writers may upload the same immutable object.

The final logical state MUST remain one valid object.

Conflicting bytes under the same identity MUST cause integrity failure.

---

# 150. Concurrent operational metadata writers

Repository Home metadata mutation requires guarded semantics.

Blind last-write-wins is prohibited for records whose conflicts matter.

---

# 151. Locking

Adapters MAY expose provider locking.

Core MUST NOT require global locks for immutable content operations.

Locks MAY be useful for maintenance or weak-provider operational metadata updates.

---

# 152. Lease-based locks

If provider locks are lease-based, the Adapter MUST report expiry semantics.

A lost lease MUST not be treated as permanent ownership.

---

# 153. Clock independence

Storage correctness MUST NOT depend on synchronized provider/client clocks for creative identity.

Timestamps may aid diagnostics.

They MUST NOT establish object equality or Revision ancestry.

---

# 154. Provider clock timestamps

Provider `Last-Modified` values are operational hints only.

They may be inaccurate, rounded or provider-generated.

---

# 155. Temporary filesystem staging

A Storage Adapter may stage incoming content locally before upload.

Staging locations MUST be operational.

They MUST not accidentally become the only durable Resource copy after a successful publication claim.

---

# 156. Disk-full failure

Local or remote disk-full conditions MUST be explicit.

Partial content MUST not be registered as valid.

---

# 157. Hashing responsibility

Core owns OMVCS content identity semantics.

The Adapter MAY assist with hashing or provider checksums.

It MUST NOT substitute a different hash algorithm as the Resource Identifier.

---

# 158. Hash algorithm evolution

If future OMVCS versions support other hash algorithms, Adapter APIs MUST treat object identifier algorithm as explicit data rather than assuming SHA-256 forever.

For OMVCS 0.1, SHA-256 is the Core-defined default.

---

# 159. Algorithm agility

Provider storage layout SHOULD include enough object-type and algorithm information to avoid future ambiguity.

Example:

```text
objects/resource/sha256/96/d2/...
```

---

# 160. Chunk representation and Endpoint layout

An OMVCS 0.1 Chunk Manifest uses the fixed-size sequential policy in Core
§§8–8.2. A different Endpoint preference MUST NOT automatically rechunk an
existing OMVCS representation. An explicit operation may create a distinct
supported representation only under an applicable, explicitly identified
policy; OMVCS 0.1 defines no negotiated alternative Chunking policy.

Provider-internal segmentation below the OMVCS representation boundary is
permitted, but MUST NOT be exposed as different OMVCS Chunk identities.
Complete-object representation remains independent of Chunking. ResourceId
and creative history remain unchanged by storage representation.

---

# 161. Chunk deduplication scope

Chunk deduplication MAY occur within:

```text
one Project
many Projects
one Endpoint namespace
```

according to implementation.

It MUST preserve privacy and access-control requirements.

This permission does not define cross-Project physical-object identity,
shared-namespace identity, shared Chunk-location records, or deletion and
ownership semantics. DEC-STORAGE-011 remains open.

---

# 162. Chunk manifest storage

Chunk Manifests are operational physical-reconstruction information and MAY live in Repository metadata or Storage Endpoint representation. They MUST NOT be included in a historical Resource Reference or affect historical object identity. Their `chunks` collection is an ordered sequence in reconstruction order, as defined in the Core Specification and Glossary; this ordering requirement is for reconstruction and does not make the Chunk Manifest part of historical identity.

An OMVCS 0.1 chunked representation MUST conform to the fixed-size
sequential policy in Core §8. Its representation retains that policy;
provider-internal segmentation does not alter its OMVCS Chunk Manifest.

Regardless, a Replica using chunked representation MUST expose enough information to reconstruct the complete Resource.

The ordered Chunk Manifest is bound to one Resource Replica representation.
Provider-specific Chunk locations required by that representation are
operational details of that Replica's locator. A Storage Map MUST NOT
register individual Chunk copies as Resource Replicas. This section does
not define shared Chunk-location records or shared-object ownership.

---

# 163. Partial Chunk loss

If one required Chunk is missing:

```text
the Resource Replica is incomplete
```

even if all other Chunks exist.

The Replica MUST NOT be reported as fully available.

---

# 164. Chunk repair

A missing/corrupt Chunk MAY be repaired from:

- another Replica's corresponding Chunk;
- reconstruction from another complete Resource Replica.

Repair does not alter Resource identity.

Repairing a Chunk from another Resource Replica does not make the Chunk a
separate Resource Replica and does not change the identity or representation
binding of either Resource.

---

# 165. Audition streaming

Adapters MAY support efficient audition access.

This may use:

```text
range requests
provider streaming
temporary direct-access URL
```

Audition must not require permanent local materialisation unless provider limitations force it.

---

# 166. Audition verification

Streaming audition MAY begin before full Resource hash verification finishes, provided the UI/security policy makes the distinction acceptable.

However, a Replica MUST NOT be promoted to `verified` solely because playback succeeded.

---

# 167. Local cache interaction

Core may cache retrieved immutable Resource Objects locally.

Storage Adapters need not know whether retrieval is for:

```text
Working State
audition
cache
verification
migration
```

unless operation-specific provider behaviour requires it.

---

# 168. Pinning

Core MAY mark Resources as pinned to local or remote storage.

The Adapter performs requested retention where its provider supports it.

Pinning is operational.

---

# 169. Offline removable media

A Storage Endpoint MAY represent removable/offline media.

Example:

```text
external SSD
archival HDD
optical archive
```

The Adapter may report:

```text
endpoint_offline
```

when physically disconnected.

History remains valid.

---

# 170. Removable media identity

The Adapter SHOULD identify the logical Endpoint independently from mount path or drive letter.

Example:

```text
E:\ today
F:\ tomorrow
```

should still refer to the same configured Endpoint where detectable.

---

# 171. Endpoint reassociation

If a removable or provider-backed Endpoint appears at a new location, the Adapter MAY reassociate it through stable Endpoint metadata.

Ambiguous reassociation MUST require confirmation.

---

# 172. Multi-account providers

One Adapter implementation may service multiple accounts.

Each configured account/root combination MUST be a separate Storage Endpoint identity where operationally distinct.

---

# 173. Endpoint ownership change

Changing who controls an Endpoint does not change historical Resource identity.

It may change custody and access policy.

Core/Platform may record this operationally.

---

# 174. Shared storage grants

An Endpoint may be shared between collaborators.

The Adapter MAY expose provider permission-management capability, but permission semantics themselves belong primarily to the Platform/interaction layer.

---

# 175. Permission capability

Where supported, an Adapter may declare:

```text
create_read_grant
create_write_grant
revoke_grant
list_grants
```

These are optional capabilities.

---

# 176. Storage Adapter operation set

The normative logical operations for OMVCS 0.1 are:

```text
GetAdapterInfo
GetCapabilities

OpenEndpoint
ProbeEndpoint

PutResource
GetResource
StatResource
VerifyResource
DeleteReplica

PutChunk
GetChunk
StatChunk
VerifyChunk

ListPrefix

ConditionalPut
ReadMetadataObject
WriteImmutableMetadataObject

CreateTemporaryReadGrant

CopyWithinEndpoint
ProbeAvailability

GetCapacityInfo
```

Optional operations include:

```text
ServerSideCopy
CreateReadGrant
CreateWriteGrant
RevokeGrant
ListGrants
ProviderVersionRestore
ProviderSnapshot
```

Exact language-specific function signatures are intentionally not fixed here.

---

# 177. GetAdapterInfo

Returns:

```text
adapter ID
adapter version
contract version
provider family
```

No Endpoint access is required unless implementation requires provider discovery.

---

# 178. GetCapabilities

Returns capability information.

Capabilities may differ per Endpoint configuration.

Example:

```text
same S3 Adapter
bucket A allows delete
bucket B is object-locked
```

---

# 179. OpenEndpoint

Inputs:

```text
Endpoint Descriptor
configuration reference
```

Result:

```text
ready
authentication_required
unavailable
invalid_configuration
```

No creative history effects.

---

# 180. ProbeEndpoint

Checks basic endpoint reachability/capabilities.

It SHOULD avoid destructive writes.

---

# 181. PutResource operation contract

Preconditions:

```text
Endpoint writable
Resource ID known
input stream readable
```

Effects:

```text
physical immutable content persisted
```

Historical effects:

```text
none
```

Operational effects:

```text
candidate/verified Replica may later be registered by Core
```

Failure MUST leave no falsely registered valid Replica.

---

# 182. GetResource operation contract

Preconditions:

```text
Replica locator known
Endpoint readable
```

Result:

```text
byte stream
provider metadata
```

Failure must distinguish:

```text
not found
inaccessible
temporarily unavailable
integrity-related provider issue where known
```

---

# 183. VerifyResource operation contract

Inputs:

```text
expected Resource Identifier
Replica
verification mode
```

Output:

```text
outcome: verified | failed | indeterminate
strength: chunk_identity | resource_identity, when established
method/evidence
```

It MUST NOT return `verified` if verification was not actually completed to requested strength.
`failed` means checked bytes did not match the expected identity.
Unavailable or missing bytes MUST produce an indeterminate/unavailable
result, not corruption.

---

# 184. DeleteReplica operation contract

Inputs:

```text
Replica locator
```

Precondition:

```text
Core has already authorized deletion
```

Adapter MUST NOT decide reachability.

---

# 185. PutChunk/GetChunk

Same semantics as Resource storage, scoped to Chunk identities.

---

# 186. ListPrefix

Returns matching logical/provider keys under the configured Endpoint namespace.

The Adapter MUST document pagination and consistency behaviour.

---

# 187. ConditionalPut

Used for mutable operational metadata.

Must support conflict detection according to provider capabilities.

If the provider cannot offer safe guarded mutation, the Adapter MUST say so.

Such an Endpoint may be unsuitable as Repository Home.

---

# 188. WriteImmutableMetadataObject

Stores canonical immutable metadata under its content-derived key.

Repeated identical writes are idempotent.

Different content under the same immutable key is integrity failure.

---

# 189. ReadMetadataObject

Retrieves metadata bytes exactly as stored.

Core validates schema and content hash.

---

# 190. CreateTemporaryReadGrant

Optional but strategically important.

Inputs:

```text
Replica
expiry
scope
```

Returns temporary retrieval authorization or direct URL.

It MUST NOT expose permanent Endpoint credentials.

---

# 191. CopyWithinEndpoint

Optional optimization.

Core may use it for:

```text
namespace migration
replication
repair
```

Destination verification still applies.

---

# 192. ProbeAvailability

Returns current operational accessibility without full download.

It MUST not claim content integrity.
It MUST distinguish unavailability from a failed identity check; availability
probe results alone do not establish either verification strength.

---

# 193. GetCapacityInfo

Optional.

May return:

```text
free bytes
quota total
quota used
unknown
```

Values are advisory.

---

# 194. Adapter thread safety

The Adapter MUST document whether one instance may be called concurrently.

Core SHOULD be able to use concurrent transfers where safe.

---

# 195. Connection pooling

Adapters MAY maintain provider connection pools.

This is internal optimization.

---

# 196. Concurrency limits

Adapters SHOULD expose recommended maximum concurrency if providers impose practical limits.

Core may use this for scheduling.

---

# 197. Bandwidth throttling

Adapters MAY support transfer rate limits.

This is operational/UI policy and does not affect correctness.

---

# 198. Background operation

Adapters SHOULD support non-blocking or asynchronous execution for large transfers.

The programming-language mechanism is implementation-specific.

---

# 199. Deterministic test backend

OMVCS SHOULD provide a **Mock Storage Adapter**.

It should support deterministic simulation of:

```text
successful upload
corruption
network failure
auth expiry
quota failure
eventual consistency
conditional-write conflict
partial upload
slow storage
missing object
```

This allows Core to be tested independently from real cloud providers.

---

# 200. Mock Adapter purpose

The Mock Storage Adapter is not another top-level specification.

It is a test implementation of this contract.

It should enable coding agents to test failure semantics exhaustively.

---

# 201. Local filesystem reference adapter

The first practical Storage Adapter SHOULD be local filesystem.

Reasons:

```text
simple
deterministic
offline
easy to test
easy to inspect
works in CI
```

It can validate most Storage Adapter semantics before cloud integration.

---

# 202. S3 reference cloud adapter

A strong candidate for the first cloud reference Adapter is S3-compatible storage because it naturally supports:

```text
immutable object patterns
multipart upload
range reads
conditional operations
large objects
server-side copy
```

This does not make S3 semantics normative.

---

# 203. Provider portability test

A critical conformance scenario is:

```text
Resource stored via Adapter A
retrieved
stored via Adapter B
retrieved
```

The final Resource Identifier MUST remain identical.

---

# 204. Migration conformance test

Test:

```text
Resource ABC on Endpoint A
copy to Endpoint B
obtain destination-applicable `resource_identity` assurance for B
register B
remove A
```

Expected:

```text
Resource ID unchanged
Revision IDs unchanged
Project State unchanged
Storage Map changed only
```

Destination assurance may use direct complete Resource verification or the
deterministic verified-reconstruction path in §33. Source verification,
provider copy-success, existence, and length alone are insufficient.

---

# 205. Corrupt destination test

Test:

```text
copy reports success
destination bytes corrupted
verification fails
```

Expected:

```text
destination not registered valid
source retained
creative history unchanged
```

An unavailable destination is indeterminate/unavailable, not corrupt. A
destination is corrupt only if bytes actually checked against the required
identity fail.

---

# 206. Authentication expiry test

During retrieval:

```text
token expires
```

Expected:

```text
authentication failure
Replica not marked missing
operation retryable after re-auth
```

---

# 207. Platform independence test

A Storage Adapter MUST be usable without an Open Music Platform session where local/storage credentials are otherwise available.

---

# 208. Repository Home concurrency test

Two clients attempt:

```text
move Line main
```

from the same old generation.

Adapter must allow Core to detect one guarded-write conflict.

Blind overwrite is non-conforming.

---

# 209. Orphan upload test

Upload Resource succeeds.

Metadata publication fails.

Expected:

```text
physical object may remain
no published Revision
object reusable on retry
eventually eligible for orphan cleanup
```

---

# 210. Shared namespace GC test

Resource belongs to Project A and Project B.

Project A stops referencing it.

Expected:

```text
physical Resource MUST NOT be deleted
```

while Project B still references it.

---

# 211. Temporary grant test

Platform requests playback authorization.

Adapter creates short-lived grant.

Expected:

```text
listener retrieves directly from storage
permanent storage credentials remain secret
platform never stores Resource bytes
```

---

# 212. Archive endpoint test

Resource stored on slow archival Endpoint.

Expected:

```text
Replica valid
availability may be archive/nearline
immediate materialisation may be delayed
history remains valid
```

---

# 213. Exact-byte round trip test

For arbitrary binary Resource `R`:

```text
PutResource(R)
GetResource(R)
```

MUST satisfy:

```text
retrieved bytes == original bytes
```

byte-for-byte.

This is one of the most fundamental Storage Adapter conformance tests.

---

# 214. No-content-transformation rule

Adapters MUST NOT perform:

```text
audio normalization
sample-rate conversion
compression format conversion
metadata rewriting inside media files
newline conversion
image optimization
```

on immutable Resource bytes.

Such transformations belong to creative/tool workflows and produce new Resource Objects.

---

# 215. Provider antivirus/quarantine

A provider may reject or quarantine some content.

The Adapter MUST report the condition.

It MUST NOT claim the Resource is available if the provider blocks retrieval.

---

# 216. Provider scanning side effects

Provider scanning that does not alter bytes is operationally acceptable.

If provider processing modifies bytes, the Replica is invalid unless the Adapter restores exact original content on retrieval.

---

# 217. Resource MIME type

Adapters MAY send provider MIME type metadata.

MIME type MUST NOT influence Resource identity.

Incorrect MIME metadata does not change content identity, although it may affect playback behaviour.

---

# 218. Resource length

Expected byte length SHOULD be recorded.

Length mismatch is an immediate integrity warning.

This operational measurement is not a historical Resource Reference field or alternate encoding. If the Resource is represented by a historical Resource Reference, its `byte_length` MUST satisfy Core Specification §7.

Matching length alone is not verification.

---

# 219. Multipart verification

For multipart providers, provider ETags MUST NOT be assumed to equal MD5 or OMVCS Resource hash unless explicitly proven for the upload mode.

The Adapter must understand provider semantics.

---

# 220. Eventual consistency strategy

For eventually consistent providers, the Adapter SHOULD support retry/readback strategies.

Core must not treat temporary absence immediately after write as permanent loss.

---

# 221. Repository Home suitability

An Adapter MUST expose whether the configured Endpoint is suitable for Repository Home use.

An Endpoint MAY be suitable for Resource storage but not metadata authority.

Example:

```text
write/read large files okay
no safe conditional metadata updates
```

Then:

```text
resource_storage = supported
repository_home = unsupported
```

---

# 222. Storage Endpoint health

Adapters MAY provide an aggregate health status:

```text
healthy
degraded
authentication_required
quota_warning
offline
unknown
```

This is operational information.

---

# 223. Health does not rewrite history

Endpoint health changes MUST never alter Revision or Project State objects.

---

# 224. Storage policy is Core-owned

Policies such as:

```text
minimum replicas
required geographic diversity
preferred provider
archive-after-N-days
```

belong to Core/project operational policy.

Adapters expose capabilities and execute instructions.

---

# 225. Provider-specific policy extensions

Adapters MAY expose optional provider-specific policy controls.

Example:

```text
S3 storage class
object lock retention
Drive folder sharing
```

These MUST remain operational extensions.

---

# 226. Default safe behaviour

When provider semantics are ambiguous, the Adapter SHOULD prefer:

```text
retain data
fail explicitly
avoid destructive mutation
```

over guessing.

The storage layer should be conservative with creative content.

---

# 227. Destructive operation confirmation boundary

The low-level Adapter itself need not ask humans for confirmation.

It must expose destructive nature accurately so Core/UI can obtain confirmation according to the Interaction Specification.

---

# 228. Non-interactive operation

All required Storage Adapter operations MUST support non-interactive execution for:

```text
automated tests
coding agents
CI
background sync
headless server processes
```

If re-authentication requires human intervention, the Adapter must return that condition structurally.

---

# 229. User prompts

A Storage Adapter SHOULD NOT unexpectedly open UI prompts.

It should return structured requirements such as:

```text
authentication_required
permission_upgrade_required
```

Higher layers control interaction.

---

# 230. Adapter sandboxing

Where feasible, provider-specific Adapters SHOULD have limited access to:

```text
their own credentials
their configured Endpoint
requested byte streams
```

They SHOULD NOT require unrelated Project secrets.

---

# 231. Logging

Adapters SHOULD emit structured diagnostic logs.

Logs MUST avoid leaking:

```text
passwords
access tokens
presigned URLs
private encryption keys
```

---

# 232. Telemetry

Storage Adapters MUST NOT require external telemetry for correctness.

Any telemetry must respect application privacy policy.

It is not part of OMVCS history.

---

# 233. Adapter extensions

Adapters MAY expose namespaced provider-specific features.

Example:

```json
{
  "extensions": {
    "org.openmusic.storage.s3": {
      "storage_classes": ["STANDARD", "GLACIER"]
    }
  }
}
```

Core implementations that do not understand an extension MUST be able to ignore it safely.

---

# 234. Unknown capabilities

Unknown optional capabilities MUST be ignored unless required by the requested operation.

An implementation MUST NOT assume unknown means supported.

---

# 235. Contract version compatibility

Adapters MUST declare which Storage Adapter Contract versions they support.

Core MUST reject incompatible contract versions rather than guessing semantics.

---

# 236. Schema versioning

Structured Adapter request/response data MUST identify schema/contract version where persistence or network transport makes ambiguity possible.

---

# 237. Backward compatibility

A newer Adapter SHOULD remain capable of reading operational metadata it previously wrote where practical.

Historical Resource identity is unaffected by Adapter version.

---

# 238. Endpoint migration between Adapter implementations

Two different Adapter implementations MAY access the same provider namespace.

If both implement the same logical object layout and Endpoint semantics, migration need not change Resource identity.

Operational locator details may differ.

---

# 239. Adapter replacement

Replacing:

```text
S3 Adapter implementation A
```

with:

```text
S3 Adapter implementation B
```

MUST NOT rewrite creative history.

At most Endpoint operational configuration changes.

---

# 240. Storage layout version

An Adapter's physical storage layout MAY have a version distinct from its
Adapter version. For a filesystem Repository Home, the marker's `layout`
field MUST be `omvcs.storage-layout/0.1`.

Example:

```text
omvcs.storage-layout/0.1
```

This allows Adapter software to evolve while preserving stored object
layout. The marker value does not select or negotiate an OMVCS Chunking
policy. An OMVCS 0.1 chunked representation continues to use the Core §8
policy regardless of provider-internal layout.

---

# 241. Layout migration

If storage layout changes, migration MUST preserve Resource and historical object identities.

Layout migration is infrastructure change only.
It MUST NOT automatically rechunk an existing OMVCS representation solely
because the Endpoint layout or preference changed.

---

# 242. Mixed layout support

An Adapter MAY temporarily read multiple layout versions.

It MUST clearly distinguish them.
This does not authorize multiple OMVCS 0.1 Chunking policies; provider
layout versions may describe provider-internal representation only.

---

# 243. Repository discovery marker

A filesystem Repository Home uses the marker at `.omvcs/repository.json`
with the exact schema and layout defined in §126. It MUST NOT include a
namespace identifier; DEC-STORAGE-011 remains unresolved. A Resource-only
Endpoint is not required to have a Repository Home marker. The marker is
operational metadata and does not select or negotiate a Chunking policy.

---

# 244. Human-readable directories are optional

An implementation MAY provide a friendly browsing layout.

But correctness MUST NOT depend on a human manually navigating folders.

Content-addressed identifiers remain authoritative.

---

# 245. Manual storage tampering

Users may manually edit or delete provider files.

OMVCS cannot prevent all tampering.

The Adapter/Core combination MUST detect resulting integrity or availability problems where possible.

---

# 246. Repair after manual move

If a user manually moves provider content but bytes remain intact, an Adapter MAY rediscover and reassociate the Replica.

Such recovery MUST verify Resource identity.

---

# 247. Recovery scan

Adapters with list capability MAY support scanning an Endpoint for OMVCS objects and rebuilding operational locator information.

This can aid disaster recovery.

---

# 248. Recovery scan safety

Discovery of a content-addressed object does not automatically establish access rights, provenance or Project membership.

Core reconnects it only where Resource identity/history already supports the relationship.

---

# 249. Unknown Project recovery

An archival export may contain enough metadata to reconstruct Projects without a current Storage Map.

Storage scanning MAY assist locating Resource Objects by hash.

---

# 250. Checksum sidecars

Filesystem-like Adapters MAY maintain checksum sidecars as optimization.

They MUST not replace authoritative content verification unless safely validated.

---

# 251. File locking during verification

Adapters MAY lock local files during read/verification where necessary to detect mutation.

Immutable object namespaces SHOULD ordinarily prevent mutation anyway.

---

# 252. Storage Adapter conformance suite

Every Adapter claiming conformance SHOULD pass tests covering at least:

```text
exact byte round trip
large streaming upload
large streaming download
existing object reuse
corrupt-object detection
missing-object handling
authentication failure
authorization failure
quota failure
rate limiting
interrupted upload
resume where supported
conditional-write conflict
concurrent same-object upload
delete idempotency
range read where supported
listing semantics
temporary access grant where supported
Repository Home suitability declaration
migration source retention on failure
provider path/key mapping
unknown provider metadata
provider locator schema/version validation and exact opaque pass-through
locator durability across process restart
provider locator contains no credentials, expiring grants, or process-local handles
```

---

# 253. Reference failure injection

The Mock Storage Adapter SHOULD allow deterministic injection such as:

```text
fail after N bytes
corrupt byte at offset N
delay visibility by N seconds
return conflict on generation N
expire auth during transfer
report quota exceeded
drop network on final chunk
```

This will be particularly valuable for coding agents.

---

# 254. Agent implementation rules

Coding agents implementing a Storage Adapter MUST obey:

> Do not create or alter OMVCS creative history.

> Do not use provider paths as Resource identity.

> Do not mutate Resource bytes.

> Do not return `verified` unless the required verification actually occurred.

> Do not register partial uploads as valid Replicas.

> Do not delete source data automatically after copy.

> Do not expose permanent credentials through metadata.

> Do not assume provider timestamps establish correctness.

> Do not use provider version history as OMVCS Revision history.

> Do not use blind last-write-wins for guarded Repository Home metadata.

> Do not treat authentication failure as proof that content is missing.

> Do not delete unknown content in shared namespaces.

> Do not assume filesystem semantics on object stores.

> Do not assume object-store semantics on filesystems.

> Do not transform media content.

> Do not make Open Music Platform availability necessary for storage operations.

---

# 255. Example: simple Resource upload

Core has:

```text
Resource ABC
Bass.wav
```

Core calls:

```text
PutResource(
    endpoint = Joakim-S3,
    resource = ABC
)
```

Adapter uploads bytes.

Adapter reports:

```text
stored_unverified
```

Core requests:

```text
VerifyResource(ABC)
```

Adapter reports:

```text
verified
```

Core then registers:

```text
Replica R1
ABC -> Joakim-S3
```

Creative history has not changed.

---

# 256. Example: migration

Current:

```text
ABC -> Server A
```

Core requests copy to Server B.

Adapter B stores Resource.

Verification succeeds.

Core updates:

```text
ABC -> Server A
ABC -> Server B
```

Then Core authorizes deletion on A.

Adapter A deletes.

Final:

```text
ABC -> Server B
```

Revision history is byte-for-byte unchanged.

---

# 257. Example: contributor storage

Anna contributes Cello Resource `D`.

Storage Map:

```text
D -> Anna-OneDrive
```

Joakim integrates the contribution.

OMVCS may retain:

```text
D -> Anna-OneDrive
```

or create:

```text
D -> Anna-OneDrive
D -> Joakim-S3
```

No content identity changes.

---

# 258. Example: endpoint credentials expire

Resource `ABC` exists on Server A.

OAuth token expires.

Adapter returns:

```text
authentication_required
```

OMVCS reports:

```text
Replica accessibility unknown/inaccessible with current credentials
```

It MUST NOT say:

```text
Resource deleted
```

---

# 259. Example: corrupt replica

Expected:

```text
ABC
```

Retrieved bytes hash to:

```text
XYZ
```

Adapter returns integrity failure.

Core marks that Replica corrupt.

If Server B contains another valid `ABC`, Core can retrieve from B and optionally repair A.

Creative history remains valid.

---

# 260. Example: platform playback

Listener clicks Play on Open Music Platform.

Platform knows:

```text
Reference Render Resource = R
```

Storage Map identifies a public/authorized Replica.

Through the applicable storage-access mechanism a temporary grant is created.

Listener receives:

```text
R bytes directly from creator-controlled storage
```

The Platform coordinates but does not become the media store.

---

# 261. Example: Repository Home conflict

Client A believes:

```text
Line main -> R10
Line generation 20
```

Client B believes the same.

A updates to:

```text
Line main -> R11
Line generation 21
```

B attempts:

```text
Line main -> R12
expected Line generation 20
```

Conditional update fails.

Storage Adapter returns conflict.

Core reports divergent work.

No history is silently lost.

---

# 262. Example: local filesystem to S3 migration

Source Replica:

```text
Local archive drive
```

Destination:

```text
S3
```

The two Adapters have completely different physical models.

Core still sees:

```text
Resource ABC
```

Source Adapter provides bytes.

Destination Adapter persists them.

Hash verifies.

Storage Map changes.

Nothing above the storage layer needs to know the implementation difference.

---

# 263. Example: preserving a Chunk representation across Endpoints

Endpoint A stores Resource `ABC` as:

```text
the OMVCS 0.1 fixed-size sequential Chunk representation
```

Endpoint B prefers:

```text
provider-internal 4 MiB segments
```

The Adapter may store those provider-internal segments while preserving the
same OMVCS Chunk boundaries and identities. The Endpoint preference alone
does not cause OMVCS rechunking. After transfer, destination verification
establishes:

```text
SHA256(full bytes) = ABC
```

The Resource remains the same Resource Object.

If a future protocol version supports another OMVCS Chunking policy, it must
identify/version that policy explicitly; this example does not define one.

---

# 264. Example: cold archive

Release Resource `MIX1` has:

```text
Replica A: online S3
Replica B: cold archive
```

A is accidentally deleted.

B still preserves the Resource.

OMVCS reports:

```text
available through archive
retrieval delayed
durability degraded relative to policy if required
```

History remains unchanged.

---

# 265. Example: shared object

Project X and Project Y both reference identical drum sample `D`.

Endpoint stores one physical `D`.

Project X is deleted.

Garbage collection examines reachability and finds Y still references `D`.

The object remains.

---

# 266. Example: provider-side rename

OneDrive provider item moves from:

```text
/OpenMusic/OldFolder/object
```

to:

```text
/OpenMusic/NewFolder/object
```

Adapter locates same provider item/content.

Storage Location changes.

Resource Identifier does not.

No Revision occurs.

---

# 267. Example: failed Repository Home

Repository Home Server A disappears.

Resource Replicas remain across Servers B and C.

Local machine still has complete metadata.

Core establishes Server B as new Repository Home.

Historical metadata is copied and verified.

The Storage Map is reconstructed from authoritative Repository Home state
under §51; reconstruction continues prior Replica registration and does not
re-verify Resource bytes solely because the map is loaded.

Project identity and all Revision IDs remain unchanged.

---

# 268. Storage architecture summary

The Storage Adapter boundary can be reduced to:

```text
                 OMVCS CORE

       What object do I need?
       Where should it go?
       How many copies?
       Is it verified?
       May the old copy be removed?

                     |
                     v

            STORAGE ADAPTER

       How do I authenticate?
       What API do I call?
       How do I upload?
       How do I resume?
       How do I read?
       What does this provider guarantee?

                     |
                     v

              STORAGE PROVIDER
```

Core owns repository meaning.

The Adapter owns provider mechanics.

---

# 269. Architectural statement

The most important requirements of this specification are:

> **Resource identity is independent of storage representation.**

> **A Storage Adapter transports and preserves immutable bytes; it does not interpret creative meaning.**

> **Storage migration changes operational metadata, not creative history.**

> **Provider-specific semantics must never leak upward as assumptions in OMVCS Core.**

> **A Replica is not valid merely because an object exists; integrity and availability are separate properties.**

> **Credentials and provider locations are replaceable infrastructure.**

> **No Storage Endpoint may become inseparable from the Project's creative identity.**

---

# 270. Storage Adapter decisions for 0.1

The following finite decisions are recorded here. Entries marked resolved
have an accepted ADR; the remaining questions should be settled before 0.1
is frozen:

1. RESOLVED by ADR-0039: canonical provider-neutral logical keys for
   immutable Resource and Chunk bytes; metadata uses its existing logical
   operation contract without a canonical metadata path layout.
2. RESOLVED by ADR-0036: OMVCS 0.1 mandates fixed-size sequential chunking
   with an exact 8,388,608-byte target.
3. RESOLVED by ADR-0040: Repository Home minimum durable historical and
   operational metadata, guarded-update, discovery, and durability
   capabilities; Resource/Chunk byte storage is not required.
4. RESOLVED by ADR-0037: verification strength is `chunk_identity` or
   `resource_identity`, separate from method/evidence.
5. RESOLVED by ADR-0038: destination assurance may be established by direct
   full Resource verification or by the specified deterministic verified
   Chunk-reconstruction path.
6. Exact layout and compaction semantics for the Repository Operation Log.
7. RESOLVED by ADR-0033: provider locator data uses the generic typed
   `{schema, value}` envelope defined in §73.
8. Exact treatment of client-side encryption in OMVCS 0.1.
9. Whether temporary read-grant support is mandatory for an Endpoint used for public Open Music playback.
10. Exact requirements for shared-namespace garbage collection.
11. Whether Endpoint namespace identity is mandatory.
12. Exact retention policy for abandoned multipart uploads and unreferenced complete objects.
13. Whether local filesystem and S3-compatible Adapters become official reference conformance implementations.
14. RESOLVED by ADR-0041/0042: filesystem Repository Home marker,
   explicit-root discovery, logically atomic bootstrap, and retry format.

These are finite decisions within the Storage Adapter Specification, not
additional top-level specifications.

---

## Document status

**Document:** OMVCS Storage Adapter Specification  
**Version:** 0.1 Draft  
**Normative:** Draft normative  
**Depends on:** OMVCS Glossary, Core Invariants Specification, OMVCS Core Specification  
**Next fixed document:** **OMVCS Platform Protocol**