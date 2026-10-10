//! Generic OMVCS Storage Adapter contract.
//!
//! Provider-specific implementations must remain outside this crate.
//!
//! WORK-0015 implements only whole-Resource and caller-selected Chunk byte I/O.
//! This is not a Resource Storage conformance-class or Repository Home claim.
//! No chunk boundaries, manifests, Replica records, verification-strength
//! taxonomy, key layout, overwrite/versioning policy, or history are defined.
//!
//! Downloads expose provisional bytes: callers must obtain a successful
//! [`Download::complete`] before accepting them as the requested content.
//! A failed/abandoned transfer may have delivered a prefix; it is not success.

#![forbid(unsafe_code)]

mod identity;
mod stream;

pub use identity::ChunkId;
pub use omvcs_model::{ResourceId, StorageEndpointId};
pub use stream::Download;

use std::fmt;
use std::io::Read;

/// Stable implementation-family identity and version information (Storage §§3–4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterInfo {
    pub adapter_id: String,
    pub adapter_version: String,
    pub storage_contract_version: String,
    pub provider_family: String,
    pub supported_provider_versions: Vec<String>,
}

/// Opaque caller-supplied logical key, not a provider path or content identity.
/// No canonical layout or key-mapping policy is selected here (Storage §§14–15).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LogicalKey(pub String);

/// Opaque caller-supplied operation correlation token, not a historical identity.
/// This seam does not prescribe its format or persistent retry semantics.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OperationId(pub String);

/// The six independently declared operations in this restricted seam.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    PutResource,
    GetResource,
    StatResource,
    PutChunk,
    GetChunk,
    StatChunk,
}

/// Endpoint-specific support. Undeclared operations are never attempted.
///
/// These flags do not imply deletion, resumability, range reads, verification
/// operations, Repository Home support, or any particular durability guarantee.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)] // Six independent normative capabilities, not a state machine.
pub struct Capabilities {
    pub put_resource: bool,
    pub get_resource: bool,
    pub stat_resource: bool,
    pub put_chunk: bool,
    pub get_chunk: bool,
    pub stat_chunk: bool,
}

impl Capabilities {
    #[must_use]
    pub const fn supports(self, operation: Operation) -> bool {
        match operation {
            Operation::PutResource => self.put_resource,
            Operation::GetResource => self.get_resource,
            Operation::StatResource => self.stat_resource,
            Operation::PutChunk => self.put_chunk,
            Operation::GetChunk => self.get_chunk,
            Operation::StatChunk => self.stat_chunk,
        }
    }
}

/// Minimum Storage §86 classes, with explicit invalid-input/precondition classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorClass {
    Configuration,
    Authentication,
    Authorization,
    Network,
    ProviderUnavailable,
    RateLimit,
    NotFound,
    Conflict,
    Integrity,
    Quota,
    Retention,
    Unsupported,
    Cancelled,
    Internal,
    InvalidRequest,
    Precondition,
}

/// Storage §§85–86 error shape. Provider context must be sanitized by the
/// implementation: never include credentials, tokens or secret-bearing URLs.
///
/// Authentication/authorization/network failures are not evidence of absence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageError {
    pub code: String,
    pub class: ErrorClass,
    pub operation: Operation,
    pub endpoint: StorageEndpointId,
    pub recoverable: bool,
    pub provider_context: Option<String>,
    pub summary: String,
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.summary)
    }
}

impl std::error::Error for StorageError {}

/// Caller-supplied address. Neither the Endpoint nor the logical key is identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectRequest<I> {
    pub endpoint: StorageEndpointId,
    pub id: I,
    pub logical_key: LogicalKey,
    pub operation_id: OperationId,
}

/// Put inputs (Storage §19). The expected SHA-256 must equal the supplied ID's
/// digest. Resource lengths obey Core §7; no Chunk size policy is selected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PutRequest<I> {
    pub object: ObjectRequest<I>,
    pub byte_length: u64,
    pub expected_hash: [u8; 32],
}

/// Typed content namespace used only at the provider byte-I/O boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObjectId {
    Resource(ResourceId),
    Chunk(ChunkId),
}

impl ObjectId {
    #[must_use]
    pub const fn digest(&self) -> &[u8; 32] {
        match self {
            Self::Resource(id) => id.digest(),
            Self::Chunk(id) => id.digest(),
        }
    }
}

impl From<ResourceId> for ObjectId {
    fn from(id: ResourceId) -> Self {
        Self::Resource(id)
    }
}

impl From<ChunkId> for ObjectId {
    fn from(id: ChunkId) -> Self {
        Self::Chunk(id)
    }
}

/// Provider-visible hints only (Storage §§18, 35–36), never integrity evidence.
/// No provider locator or historical/Replica identity is created here.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProviderMetadata {
    pub byte_length: Option<u64>,
    pub last_modified: Option<String>,
    pub checksum: Option<String>,
    pub version_id: Option<String>,
}

/// Absence is an ordinary stat outcome, distinct from any failed stat operation.
/// Presence, even with matching length/checksum, does not mean verified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatOutcome {
    Absent,
    Present(ProviderMetadata),
}

/// Successful byte-I/O completion, not a Replica or durability/strength record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferOutcome<I> {
    pub id: I,
    pub byte_length: u64,
    pub metadata: ProviderMetadata,
}

/// Provider stream. Opening it is not successful retrieval of the content.
pub struct ProviderDownload<'a> {
    pub stream: Box<dyn Read + 'a>,
    pub metadata: ProviderMetadata,
}

/// Provider-neutral implementation boundary, not a concrete Adapter.
///
/// Implementations MUST report capabilities truthfully per Endpoint and return
/// structured errors with the requested operation and Endpoint. `get` must
/// distinguish not-found, inaccessible, unavailable and known integrity issues.
/// Stream I/O failures are subsequently contextualized by the checked facade.
///
/// `put` MUST consume the input to EOF and persist exact immutable bytes, or
/// fail. It MUST NOT acknowledge before its declared provider durability
/// semantics are satisfied (Storage §19). EOF/input validation alone is not
/// proof that a provider persisted the bytes; that obligation belongs here.
/// An existing-object optimization MUST establish stored correspondence to the
/// supplied ID (§20), not merely existence/length/provider metadata. Different
/// bytes under an immutable content key MUST fail with Integrity (§21).
/// Failed/partial puts MUST NOT register a valid Replica (§29, §181).
///
/// No operation may create or rewrite history, assert global reachability, or
/// register a Replica. Key mapping and safe provider metadata stay behind this
/// boundary. This contract does not authorize deletion or overwrite.
pub trait ByteStorage {
    fn adapter_info(&self) -> AdapterInfo;

    /// # Errors
    /// Returns a structured failure when capability discovery cannot complete.
    fn capabilities(
        &self,
        endpoint: StorageEndpointId,
        operation: Operation,
    ) -> Result<Capabilities, StorageError>;

    /// # Errors
    /// Returns a structured provider/input failure; never partial success.
    fn put(
        &mut self,
        request: &PutRequest<ObjectId>,
        content: &mut dyn Read,
    ) -> Result<ProviderMetadata, StorageError>;

    /// # Errors
    /// Returns a structured failure, including `NotFound` when content is absent.
    fn get(
        &mut self,
        request: &ObjectRequest<ObjectId>,
    ) -> Result<ProviderDownload<'_>, StorageError>;

    /// # Errors
    /// A failed stat is not an Absent result.
    fn stat(&mut self, request: &ObjectRequest<ObjectId>) -> Result<StatOutcome, StorageError>;
}

/// Checked byte-I/O facade. Memory use does not scale with Resource size.
/// Integrity checks here apply to these transfers only, not persistent Replica
/// verification strength or future upload promotion.
pub struct Storage<P> {
    provider: P,
}

impl<P: ByteStorage> Storage<P> {
    #[must_use]
    pub const fn new(provider: P) -> Self {
        Self { provider }
    }

    pub fn adapter_info(&self) -> AdapterInfo {
        self.provider.adapter_info()
    }

    /// # Errors
    /// Propagates capability discovery failures, never assumes support.
    pub fn capabilities(
        &self,
        endpoint: StorageEndpointId,
        operation: Operation,
    ) -> Result<Capabilities, StorageError> {
        self.provider.capabilities(endpoint, operation)
    }

    fn require(
        &self,
        endpoint: StorageEndpointId,
        operation: Operation,
    ) -> Result<(), StorageError> {
        if self.capabilities(endpoint, operation)?.supports(operation) {
            Ok(())
        } else {
            Err(error(
                endpoint,
                operation,
                ErrorClass::Unsupported,
                "UNSUPPORTED",
                "Operation is not declared for this Endpoint",
            ))
        }
    }

    fn put<I: Copy + Into<ObjectId>>(
        &mut self,
        request: &PutRequest<I>,
        content: &mut dyn Read,
        operation: Operation,
    ) -> Result<TransferOutcome<I>, StorageError> {
        let object = erase(&request.object);
        self.require(object.endpoint, operation)?;
        if request.expected_hash != *object.id.digest()
            || (matches!(object.id, ObjectId::Resource(_))
                && request.byte_length > omvcs_model::resource::MAX_RESOURCE_BYTE_LENGTH)
        {
            return Err(error(
                object.endpoint,
                operation,
                ErrorClass::InvalidRequest,
                "INVALID_REQUEST",
                "Expected hash or Resource length contradicts supplied identity/domain",
            ));
        }
        let provider_request = PutRequest {
            object,
            byte_length: request.byte_length,
            expected_hash: request.expected_hash,
        };
        let mut input = stream::CheckedStream::new(
            content,
            &provider_request.object,
            operation,
            Some(request.byte_length),
        );
        let result = self.provider.put(&provider_request, &mut input);
        // Even a provider that swallows a read error or stops early cannot
        // manufacture a successful transfer completion.
        let metadata = result?;
        let byte_length = input.complete()?;
        Ok(TransferOutcome {
            id: request.object.id,
            byte_length,
            metadata,
        })
    }

    fn get<I: Copy + Into<ObjectId>>(
        &mut self,
        request: &ObjectRequest<I>,
        operation: Operation,
    ) -> Result<Download<'_, I>, StorageError> {
        let object = erase(request);
        self.require(object.endpoint, operation)?;
        let result = self.provider.get(&object)?;
        Ok(Download::new(result, &object, request.id, operation))
    }

    fn stat<I: Copy + Into<ObjectId>>(
        &mut self,
        request: &ObjectRequest<I>,
        operation: Operation,
    ) -> Result<StatOutcome, StorageError> {
        let object = erase(request);
        self.require(object.endpoint, operation)?;
        self.provider.stat(&object)
    }
}

macro_rules! operations {
    ($id:ty, $put:ident, $get:ident, $stat:ident, $put_op:ident, $get_op:ident, $stat_op:ident) => {
        impl<P: ByteStorage> Storage<P> {
            /// Persists caller-supplied immutable content without selecting chunks.
            /// # Errors
            /// Unsupported, invalid inputs, stream/integrity, or provider failure.
            pub fn $put(
                &mut self,
                request: &PutRequest<$id>,
                content: &mut dyn Read,
            ) -> Result<TransferOutcome<$id>, StorageError> {
                self.put(request, content, Operation::$put_op)
            }

            /// Opens a provisional checked stream. Require `complete()` after EOF.
            /// # Errors
            /// Unsupported or provider failure; later stream errors prevent completion.
            pub fn $get(
                &mut self,
                request: &ObjectRequest<$id>,
            ) -> Result<Download<'_, $id>, StorageError> {
                self.get(request, Operation::$get_op)
            }

            /// Returns presence hints only, never an integrity claim.
            /// # Errors
            /// Unsupported or provider failure, distinct from absence.
            pub fn $stat(
                &mut self,
                request: &ObjectRequest<$id>,
            ) -> Result<StatOutcome, StorageError> {
                self.stat(request, Operation::$stat_op)
            }
        }
    };
}

operations!(
    ResourceId,
    put_resource,
    get_resource,
    stat_resource,
    PutResource,
    GetResource,
    StatResource
);
operations!(
    ChunkId, put_chunk, get_chunk, stat_chunk, PutChunk, GetChunk, StatChunk
);

fn erase<I: Copy + Into<ObjectId>>(request: &ObjectRequest<I>) -> ObjectRequest<ObjectId> {
    ObjectRequest {
        endpoint: request.endpoint,
        id: request.id.into(),
        logical_key: request.logical_key.clone(),
        operation_id: request.operation_id.clone(),
    }
}

fn error(
    endpoint: StorageEndpointId,
    operation: Operation,
    class: ErrorClass,
    code: &str,
    summary: &str,
) -> StorageError {
    StorageError {
        code: format!("OMVCS_STORAGE_{code}"),
        class,
        operation,
        endpoint,
        recoverable: false,
        provider_context: None,
        summary: summary.into(),
    }
}
