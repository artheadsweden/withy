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

A Storage Adapter capable of acting as **Repository Home** MUST additionally provide enough functionality to safely store and update repository metadata.

This normally requires:

```text
atomic or guarded metadata updates
listing or known-key retrieval
durable write acknowledgement
```

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

Supports durable OMVCS metadata plus operational metadata.

## Class RH — Combined

Supports both Resource Storage and Repository Home functionality.

## Class A — Archival

Supports durable export/import but may not support active repository mutation.

These are capability groupings, not user-facing names.

---

# 14. Logical key namespace

Storage Adapters operate on **logical OMVCS keys** supplied by Core.

Core SHOULD use deterministic provider-independent key conventions.

Example:

```text
objects/resources/96/d2/<resource-digest>
objects/chunks/aa/13/<chunk-digest>
metadata/revisions/0c/a7/<revision-digest>.json
metadata/project-states/84/aa/<digest>.json
```

A provider MAY internally map these keys differently.

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

The mapping MUST be deterministic and reversible or persistently recorded.

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

---

# 20. Existing object optimization

If the destination already contains the requested object, the Adapter MAY avoid re-uploading it.

Before treating the operation as satisfied, it MUST establish that the stored object corresponds to the requested Resource Identifier.

How strongly this must be verified depends on available provider guarantees and previous verification metadata.

---

# 21. Atomic immutable object creation

Where provider semantics support it, immutable objects SHOULD be created with:

```text
create-if-absent
```

semantics.

Concurrent uploads of the same Resource MUST result logically in one Resource Object.

Concurrent attempts to write different bytes to the same content-derived key MUST be treated as integrity failure.

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

Incomplete uploads MUST NOT be registered as verified Resource Replicas.

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

until complete and verified.

---

# 30. Replica lifecycle

A Resource Replica SHOULD progress through states such as:

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

Exact storage-provider internals may differ.

Core-visible state MUST never mark a replica `verified` before verification requirements are satisfied.

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
- reconstructing chunks and verifying final Resource hash.

The verification method MUST be reported.

---

# 32. Verification strength

Verification SHOULD expose a strength/classification.

Conceptually:

```text
full_content
trusted_provider_checksum
chunk_verified_plus_manifest
metadata_only
unknown
```

OMVCS publication policy may require a particular verification strength.

A weak verification MUST NOT be silently represented as strong verification.

---

# 33. Full Resource verification

The strongest verification is:

```text
SHA256(reconstructed complete Resource bytes)
=
Resource Identifier
```

This MUST be supported at least when deep verification is explicitly requested.

---

# 34. Chunk verification

When chunked storage is used:

```text
each Chunk hash
```

MUST verify individually.

The reconstructed full Resource SHOULD also be verifiable against its Resource Identifier.

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
objects/chunks/
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
not_found
permission_denied
provider_error
```

Core handles the semantic consequence.

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

---

# 51. Storage Map persistence

Repository Home MUST durably persist the Storage Map or enough operational log/state to reconstruct it.

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

It SHOULD use:

- deterministic directory layout;
- temporary writes;
- fsync/durability where available;
- atomic rename where supported;
- file permissions appropriate to the host OS.

It MUST remain portable across supported operating systems.

---

# 65. Filesystem case sensitivity

The filesystem Adapter MUST not depend on case-sensitive path distinction.

OMVCS canonical keys SHOULD use lowercase-safe naming to avoid Windows/macOS/Linux inconsistencies.

---

# 66. Filesystem path length

The filesystem Adapter MUST account for host path-length limitations.

It MAY use shortened deterministic internal layout.

Path-shortening MUST NOT alter object identity.

---

# 67. Filesystem links

Hard links or reflinks MAY be used as optimization.

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

A Resource Replica record SHOULD contain enough provider-independent operational information to identify the physical copy.

Conceptually:

```json
{
  "replica_id": "019...",
  "resource_id": "omvcs:resource:sha256:...",
  "endpoint_id": "019...",
  "locator": {
    "provider_key": "..."
  },
  "state": "verified",
  "verification": {
    "kind": "full_content",
    "verified_at": "..."
  }
}
```

`locator` is Adapter-specific operational information.

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

---

# 110. Replica freshness

Because Resource Objects are immutable, a verified Replica does not become creatively stale.

However, its **verification freshness** may age.

Core MAY request periodic re-verification.

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

---

# 112. Migration safety rule

Before removing a source Replica during migration, Core SHOULD ensure:

```text
destination replica verified
```

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

A new Repository Home MUST support creation of a minimal repository root record identifying:

```text
Project ID
Repository format version
current metadata generation
configured namespace
```

No creative history is changed merely by bootstrap.

---

# 125. Repository Home discovery

A client connecting to a Storage Endpoint MAY attempt to discover an OMVCS Repository Home at a known configured root.

Discovery MUST not infer Project identity solely from folder name.

---

# 126. Repository Home root record

Conceptually:

```json
{
  "schema": "omvcs.repository-home/0.1",
  "project_id": "019...",
  "repository_format": "omvcs/0.1",
  "current_generation": 188
}
```

This is operational metadata.

---

# 127. Repository Home migration

The destination Adapter MUST support receiving:

```text
complete historical metadata
operational metadata
storage map
operation log or compacted equivalent
```

Migration MUST verify completeness before Core designates the destination authoritative.

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

# 160. Chunk layout independence

Different Endpoints MAY physically chunk the same Resource differently.

Creative identity remains the Resource Identifier.

A migration MAY therefore:

```text
read source representation
reconstruct Resource bytes
re-chunk for destination
```

without creating creative history.

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

---

# 162. Chunk manifest storage

Chunk Manifests are operational physical-reconstruction information and MAY live in Repository metadata or Storage Endpoint representation. They MUST NOT be included in a historical Resource Reference or affect historical object identity. Their `chunks` collection is an ordered sequence in reconstruction order, as defined in the Core Specification and Glossary; this ordering requirement is for reconstruction and does not make the Chunk Manifest part of historical identity.

Regardless, a Replica using chunked representation MUST expose enough information to reconstruct the complete Resource.

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
verified
failed
indeterminate
verification method
```

It MUST NOT return `verified` if verification was not actually completed to requested strength.

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
verify B
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

An Endpoint MAY have a storage-layout version distinct from the Adapter version.

Example:

```text
omvcs-storage-layout/1
```

This allows Adapter software to evolve while preserving stored object layout.

---

# 241. Layout migration

If storage layout changes, migration MUST preserve Resource and historical object identities.

Layout migration is infrastructure change only.

---

# 242. Mixed layout support

An Adapter MAY temporarily read multiple layout versions.

It MUST clearly distinguish them.

---

# 243. Repository discovery marker

A Storage Endpoint root MAY contain a small OMVCS marker identifying:

```text
storage layout version
namespace ID
Repository Home information where applicable
```

This marker is operational metadata.

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
main -> R10
generation 20
```

Client B believes the same.

A updates to:

```text
main -> R11
generation 21
```

B attempts:

```text
main -> R12
expected generation 20
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

# 263. Example: re-chunking

Endpoint A stores Resource `ABC` as:

```text
8 MiB chunks
```

Endpoint B prefers:

```text
4 MiB chunks
```

Migration reconstructs `ABC`, stores destination representation, and verifies:

```text
SHA256(full bytes) = ABC
```

The Resource remains the same Resource Object.

Chunk identities/layout may differ operationally.

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

Storage Map reconstructed.

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

# 270. Unresolved Storage Adapter decisions for 0.1

The following questions remain inside this specification and should be settled before 0.1 is frozen:

1. Exact canonical logical-key layout for Resources, Chunks and metadata.
2. Whether fixed 8 MiB chunking is mandated for the reference implementation or only recommended.
3. Exact minimum capability set required for Repository Home conformance.
4. Exact verification-strength taxonomy.
5. Whether full Resource hash verification is mandatory after every upload or may rely on previously verified deterministic chunk reconstruction.
6. Exact layout and compaction semantics for the Repository Operation Log.
7. Exact representation of provider locator data inside Replica records.
8. Exact treatment of client-side encryption in OMVCS 0.1.
9. Whether temporary read-grant support is mandatory for an Endpoint used for public Open Music playback.
10. Exact requirements for shared-namespace garbage collection.
11. Whether Endpoint namespace identity is mandatory.
12. Exact retention policy for abandoned multipart uploads and unreferenced complete objects.
13. Whether local filesystem and S3-compatible Adapters become official reference conformance implementations.
14. Exact storage-layout version marker and repository discovery format.

These are finite decisions within the Storage Adapter Specification, not additional top-level specifications.

---

## Document status

**Document:** OMVCS Storage Adapter Specification  
**Version:** 0.1 Draft  
**Normative:** Draft normative  
**Depends on:** OMVCS Glossary, Core Invariants Specification, OMVCS Core Specification  
**Next fixed document:** **OMVCS Platform Protocol**