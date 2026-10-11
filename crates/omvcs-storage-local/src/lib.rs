//! Local filesystem implementation of the approved Resource/Chunk byte seam.
//!
//! The configured root is held as a directory capability. Caller keys are
//! checked against ADR-0039 before they are resolved. On Unix, writes use a
//! synced temporary file and an atomic same-filesystem hard-link publication,
//! which cannot replace an existing name. Filesystems/platforms without the
//! needed primitives report the affected operation as unsupported.
//!
//! Repository Home metadata operations are exposed separately from the
//! Resource/Chunk byte provider and Storage Map persistence boundary. Only an
//! opened, initialized Home declares the ADR-0040 Repository Home profile.

#![forbid(unsafe_code)]

#[cfg(target_os = "linux")]
use std::io::Write;
#[cfg(target_os = "linux")]
use std::sync::atomic::{AtomicU64, Ordering};
use std::{
    collections::BTreeSet,
    io::{self, Read},
    path::{Path, PathBuf},
};

#[cfg(target_os = "linux")]
use cap_std::fs::File;
#[cfg(target_os = "linux")]
use cap_std::fs::OpenOptions;
use cap_std::{ambient_authority, fs::Dir};
use omvcs_core::storage_map::{
    ConditionalWriteOutcome, PersistedStorageMapData, StorageMapGeneration, StorageMapPersistence,
    StorageMapPersistenceFailure, StorageMapSnapshot,
};
use omvcs_model::{ProjectId, StorageEndpointId};
use omvcs_storage::{
    AdapterInfo, ByteStorage, Capabilities, ErrorClass, LogicalKey, ObjectId, ObjectRequest,
    Operation, ProviderDownload, ProviderMetadata, PutRequest, StatOutcome, StorageError,
};
#[cfg(target_os = "linux")]
use rustix::fs::{Mode, OFlags, ResolveFlags};
use serde::{Deserialize, Serialize};
#[cfg(target_os = "linux")]
use sha2::{Digest, Sha256};

#[cfg(target_os = "linux")]
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// A root-confined filesystem Resource/Chunk byte provider.
pub struct FilesystemStorage {
    root_path: PathBuf,
    #[cfg(target_os = "linux")]
    root: Dir,
    endpoint: StorageEndpointId,
}

/// Independently declared Repository Home capabilities from Storage §10.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)] // Independent normative capabilities, not a state machine.
pub struct FilesystemHomeCapabilities {
    pub immutable_historical_metadata: bool,
    pub operational_metadata: bool,
    pub guarded_operational_updates: bool,
    pub project_storage_map_cas: bool,
    pub marker_and_discovery: bool,
    pub durable_write_acknowledgement: bool,
}

impl FilesystemHomeCapabilities {
    const SUPPORTED: Self = Self {
        immutable_historical_metadata: true,
        operational_metadata: true,
        guarded_operational_updates: true,
        project_storage_map_cas: true,
        marker_and_discovery: true,
        durable_write_acknowledgement: true,
    };
}

impl FilesystemStorage {
    /// Opens the explicitly selected existing storage root.
    ///
    /// This does not discover an ancestor or create/adopt a repository.
    ///
    /// # Errors
    ///
    /// Returns a typed configuration error if the root cannot be opened.
    pub fn open(root: impl AsRef<Path>, endpoint: StorageEndpointId) -> Result<Self, StorageError> {
        let root_path = root.as_ref().to_path_buf();
        #[cfg(target_os = "linux")]
        let dir = Dir::open_ambient_dir(&root_path, ambient_authority())
            .map_err(|error| storage_error(endpoint, Operation::GetResource, error))?;
        #[cfg(not(target_os = "linux"))]
        Dir::open_ambient_dir(&root_path, ambient_authority())
            .map_err(|error| storage_error(endpoint, Operation::GetResource, error))?;
        Ok(Self {
            root_path,
            #[cfg(target_os = "linux")]
            root: dir,
            endpoint,
        })
    }

    /// Returns the explicitly configured root path for diagnostics only.
    #[must_use]
    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    #[cfg(target_os = "linux")]
    fn canonical_key(id: ObjectId) -> String {
        let digest = id.digest();
        let digest_hex = hex_digest(digest);
        match id {
            ObjectId::Resource(_) => format!(
                "resources/sha256/{}/{}/{}",
                &digest_hex[0..2],
                &digest_hex[2..4],
                digest_hex
            ),
            ObjectId::Chunk(_) => format!(
                "chunks/sha256/{}/{}/{}",
                &digest_hex[0..2],
                &digest_hex[2..4],
                digest_hex
            ),
        }
    }

    #[cfg(target_os = "linux")]
    fn physical_key(id: ObjectId) -> String {
        // Numeric-only components cannot acquire case-folded name aliases.
        let kind = match id {
            ObjectId::Resource(_) => '0',
            ObjectId::Chunk(_) => '1',
        };
        let mut digits = String::with_capacity(128);
        for byte in id.digest() {
            use std::fmt::Write as _;
            let _ = write!(digits, "{:02}{:02}", byte >> 4, byte & 0x0f);
        }
        format!("{kind}/{}/{digits}", &digits[..4])
    }

    #[cfg(target_os = "linux")]
    fn validate_key(
        request: &ObjectRequest<ObjectId>,
        operation: Operation,
    ) -> Result<String, StorageError> {
        let canonical = Self::canonical_key(request.id);
        if request.logical_key.0 != canonical {
            return Err(error(
                request.endpoint,
                operation,
                ErrorClass::InvalidRequest,
                "NON_CANONICAL_KEY",
                "Logical key does not exactly match the canonical key for its identifier",
            ));
        }
        Ok(Self::physical_key(request.id))
    }

    #[cfg(not(target_os = "linux"))]
    fn unsupported(&self, operation: Operation, summary: &str) -> StorageError {
        error(
            self.endpoint,
            operation,
            ErrorClass::Unsupported,
            "FILESYSTEM_PRIMITIVE_UNAVAILABLE",
            summary,
        )
    }

    #[cfg(target_os = "linux")]
    fn sync_directory(&self, dir: &Dir, operation: Operation) -> Result<(), StorageError> {
        rustix::fs::fsync(dir).map_err(|failure| {
            error(
                self.endpoint,
                operation,
                ErrorClass::ProviderUnavailable,
                "DIRECTORY_SYNC_FAILED",
                &failure.to_string(),
            )
        })
    }

    #[cfg(target_os = "linux")]
    fn sync_key_ancestors(&self, key: &str, operation: Operation) -> Result<(), StorageError> {
        let (parent, _) = key.rsplit_once('/').ok_or_else(|| {
            error(
                self.endpoint,
                operation,
                ErrorClass::Internal,
                "INVALID_INTERNAL_KEY",
                "Canonical key has no parent",
            )
        })?;
        let mut path = parent.to_owned();
        loop {
            let dir = open_directory(&self.root, &path.split('/').collect::<Vec<_>>())
                .map_err(|failure| storage_error(self.endpoint, operation, failure))?;
            self.sync_directory(&dir, operation)?;
            match path.rsplit_once('/') {
                Some((ancestor, _)) => path = ancestor.to_owned(),
                None => break,
            }
        }
        self.sync_directory(&self.root, operation)
    }

    #[cfg(target_os = "linux")]
    #[allow(clippy::too_many_arguments)]
    fn publish_staged(
        &self,
        parent: &Dir,
        stage_name: &str,
        filename: &str,
        key: &str,
        id: ObjectId,
        byte_length: u64,
        operation: Operation,
    ) -> Result<ProviderMetadata, StorageError> {
        match rustix::fs::renameat_with(
            parent,
            stage_name,
            parent,
            filename,
            rustix::fs::RenameFlags::NOREPLACE,
        )
        .map_err(rustix_io_error)
        {
            Ok(()) => {}
            Err(failure) if failure.kind() == io::ErrorKind::AlreadyExists => {
                let _ = parent.remove_file(stage_name);
                return self.verify_existing(key, id, Some(byte_length), operation);
            }
            Err(failure) => {
                let _ = parent.remove_file(stage_name);
                return Err(storage_error(self.endpoint, operation, failure));
            }
        }

        self.sync_key_ancestors(key, operation)?;
        Ok(ProviderMetadata {
            byte_length: Some(byte_length),
            ..ProviderMetadata::default()
        })
    }

    #[cfg(target_os = "linux")]
    fn verify_existing(
        &self,
        key: &str,
        id: ObjectId,
        expected_length: Option<u64>,
        operation: Operation,
    ) -> Result<ProviderMetadata, StorageError> {
        let mut file = open_file(&self.root, key)
            .map_err(|failure| storage_error(self.endpoint, operation, failure))?;
        let (digest, length) = hash_reader(&mut file)
            .map_err(|failure| storage_error(self.endpoint, operation, failure))?;
        if digest != *id.digest() || expected_length.is_some_and(|expected| expected != length) {
            return Err(error(
                self.endpoint,
                operation,
                ErrorClass::Integrity,
                "IMMUTABLE_CONTENT_CONFLICT",
                "Existing bytes at the canonical key do not match the requested identity",
            ));
        }
        self.sync_key_ancestors(key, operation)?;
        Ok(ProviderMetadata {
            byte_length: Some(length),
            ..ProviderMetadata::default()
        })
    }

    #[cfg(target_os = "linux")]
    fn put_immutable(
        &self,
        request: &PutRequest<ObjectId>,
        content: &mut dyn Read,
        operation: Operation,
    ) -> Result<ProviderMetadata, StorageError> {
        let key = Self::validate_key(&request.object, operation)?;
        if request.expected_hash != *request.object.id.digest() {
            return Err(error(
                self.endpoint,
                operation,
                ErrorClass::InvalidRequest,
                "IDENTITY_HASH_MISMATCH",
                "Expected digest does not match the supplied content identifier",
            ));
        }

        let (parent_path, filename) = key.rsplit_once('/').ok_or_else(|| {
            error(
                self.endpoint,
                operation,
                ErrorClass::Internal,
                "INVALID_INTERNAL_KEY",
                "Canonical key has no parent",
            )
        })?;
        create_directories(&self.root, parent_path)
            .map_err(|failure| storage_error(self.endpoint, operation, failure))?;
        if self
            .root
            .try_exists(&key)
            .map_err(|failure| storage_error(self.endpoint, operation, failure))?
        {
            verify_reader(
                content,
                request.object.id,
                request.expected_hash,
                request.byte_length,
            )
            .map_err(|failure| input_verification_error(self.endpoint, operation, failure))?;
            return self.verify_existing(
                &key,
                request.object.id,
                Some(request.byte_length),
                operation,
            );
        }

        let parent = open_directory(&self.root, &parent_path.split('/').collect::<Vec<_>>())
            .map_err(|failure| storage_error(self.endpoint, operation, failure))?;
        let (stage_name, mut stage_file) = create_staging_file(&parent).map_err(|failure| {
            if failure.kind() == io::ErrorKind::AlreadyExists {
                error(
                    self.endpoint,
                    operation,
                    ErrorClass::ProviderUnavailable,
                    "STAGING_NAME_EXHAUSTED",
                    "Could not allocate a unique private staging file",
                )
            } else {
                storage_error(self.endpoint, operation, failure)
            }
        })?;

        let staged = stage_and_verify(
            &mut stage_file,
            content,
            request.object.id,
            request.expected_hash,
            request.byte_length,
        );
        if let Err(failure) = staged {
            drop(stage_file);
            let _ = parent.remove_file(&stage_name);
            return Err(input_verification_error(self.endpoint, operation, failure));
        }

        if let Err(failure) = stage_file.sync_all() {
            drop(stage_file);
            let _ = parent.remove_file(&stage_name);
            return Err(storage_error(self.endpoint, operation, failure));
        }
        drop(stage_file);

        self.publish_staged(
            &parent,
            &stage_name,
            filename,
            &key,
            request.object.id,
            request.byte_length,
            operation,
        )
    }

    #[cfg(not(target_os = "linux"))]
    fn put_immutable(
        &self,
        _request: &PutRequest<ObjectId>,
        _content: &mut dyn Read,
        operation: Operation,
    ) -> Result<ProviderMetadata, StorageError> {
        Err(self.unsupported(
            operation,
            "Atomic no-replace publication with durable parent-directory flush is not established for this target",
        ))
    }
}

impl ByteStorage for FilesystemStorage {
    fn adapter_info(&self) -> AdapterInfo {
        AdapterInfo {
            adapter_id: "org.openmusic.omvcs.storage.local-filesystem".into(),
            adapter_version: env!("CARGO_PKG_VERSION").into(),
            storage_contract_version: "0.1".into(),
            provider_family: "local-filesystem".into(),
            supported_provider_versions: Vec::new(),
        }
    }

    fn capabilities(
        &self,
        endpoint: StorageEndpointId,
        operation: Operation,
    ) -> Result<Capabilities, StorageError> {
        if endpoint != self.endpoint {
            return Err(error(
                endpoint,
                operation,
                ErrorClass::Configuration,
                "ENDPOINT_MISMATCH",
                "Operation Endpoint differs from the configured filesystem Endpoint",
            ));
        }
        #[cfg(target_os = "linux")]
        let supported = true;
        #[cfg(not(target_os = "linux"))]
        let supported = false;
        Ok(Capabilities {
            put_resource: supported,
            get_resource: supported,
            stat_resource: supported,
            put_chunk: supported,
            get_chunk: supported,
            stat_chunk: supported,
        })
    }

    fn put(
        &mut self,
        request: &PutRequest<ObjectId>,
        content: &mut dyn Read,
    ) -> Result<ProviderMetadata, StorageError> {
        let operation = match request.object.id {
            ObjectId::Resource(_) => Operation::PutResource,
            ObjectId::Chunk(_) => Operation::PutChunk,
        };
        if request.object.endpoint != self.endpoint {
            return Err(error(
                request.object.endpoint,
                operation,
                ErrorClass::Configuration,
                "ENDPOINT_MISMATCH",
                "Operation Endpoint differs from the configured filesystem Endpoint",
            ));
        }
        self.put_immutable(request, content, operation)
    }

    fn get(
        &mut self,
        request: &ObjectRequest<ObjectId>,
    ) -> Result<ProviderDownload<'_>, StorageError> {
        let operation = match request.id {
            ObjectId::Resource(_) => Operation::GetResource,
            ObjectId::Chunk(_) => Operation::GetChunk,
        };
        #[cfg(not(target_os = "linux"))]
        {
            let _ = request;
            Err(self.unsupported(
                operation,
                "Safe exact-spelling filesystem reads are not established for this target",
            ))
        }
        #[cfg(target_os = "linux")]
        {
            if request.endpoint != self.endpoint {
                return Err(error(
                    request.endpoint,
                    operation,
                    ErrorClass::Configuration,
                    "ENDPOINT_MISMATCH",
                    "Operation Endpoint differs from the configured filesystem Endpoint",
                ));
            }
            let key = Self::validate_key(request, operation)?;
            let file = open_file(&self.root, &key)
                .map_err(|failure| storage_error(self.endpoint, operation, failure))?;
            let length = file
                .metadata()
                .map_err(|failure| storage_error(self.endpoint, operation, failure))?
                .len();
            Ok(ProviderDownload {
                stream: Box::new(file),
                metadata: ProviderMetadata {
                    byte_length: Some(length),
                    ..ProviderMetadata::default()
                },
            })
        }
    }

    fn stat(&mut self, request: &ObjectRequest<ObjectId>) -> Result<StatOutcome, StorageError> {
        let operation = match request.id {
            ObjectId::Resource(_) => Operation::StatResource,
            ObjectId::Chunk(_) => Operation::StatChunk,
        };
        #[cfg(not(target_os = "linux"))]
        {
            let _ = request;
            Err(self.unsupported(
                operation,
                "Safe exact-spelling filesystem reads are not established for this target",
            ))
        }
        #[cfg(target_os = "linux")]
        {
            if request.endpoint != self.endpoint {
                return Err(error(
                    request.endpoint,
                    operation,
                    ErrorClass::Configuration,
                    "ENDPOINT_MISMATCH",
                    "Operation Endpoint differs from the configured filesystem Endpoint",
                ));
            }
            let key = Self::validate_key(request, operation)?;
            match open_file(&self.root, &key).and_then(|file| file.metadata()) {
                Ok(metadata) if metadata.file_type().is_file() => {
                    Ok(StatOutcome::Present(ProviderMetadata {
                        byte_length: Some(metadata.len()),
                        ..ProviderMetadata::default()
                    }))
                }
                Ok(_) => Err(error(
                    self.endpoint,
                    operation,
                    ErrorClass::Integrity,
                    "NON_REGULAR_OBJECT",
                    "Canonical key resolves to a non-regular filesystem object",
                )),
                Err(failure) if failure.kind() == io::ErrorKind::NotFound => {
                    Ok(StatOutcome::Absent)
                }
                Err(failure) => Err(storage_error(self.endpoint, operation, failure)),
            }
        }
    }
}

/// Filesystem persistence for Core's complete Storage Map compare-and-swap.
pub struct FilesystemStorageMapPersistence {
    root: Dir,
}

/// Discovery classification for the marker and initial operational state.
///
/// This classification covers initialization state. Use
/// [`FilesystemRepositoryHome`] for the complete Repository Home metadata
/// profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilesystemHomeDiscovery {
    /// No marker or bootstrap evidence exists at the selected root.
    NotRepository,
    /// A supported marker and valid empty generation-zero Storage Map agree.
    Initialized,
    /// Some bootstrap evidence exists, but the complete initial state does not.
    IncompleteInitialization,
}

/// Typed filesystem Home marker/discovery/bootstrap failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilesystemHomeError {
    /// The selected root has no initialized Repository Home.
    NotInitialized,
    /// Marker bytes or required operational state are malformed or inconsistent.
    InvalidRepositoryMetadata,
    /// Bootstrap evidence is incomplete and cannot be resumed safely.
    IncompleteInitialization,
    /// The marker uses a schema or layout version this implementation does not support.
    UnsupportedVersion,
    /// A valid marker names a different Project than the caller requested.
    ProjectIdentityMismatch,
    /// The selected root cannot be safely read or durably updated.
    ProviderFailure,
    /// Required no-replace publication or directory durability is unavailable.
    UnsupportedFilesystemGuarantee,
}

/// Result of an explicit-root bootstrap call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilesystemHomeBootstrapOutcome {
    /// This call established and verified the initialized root.
    Initialized,
    /// A complete Home for the same Project was already present.
    AlreadyInitialized,
}

/// Outcome of writing an immutable historical metadata object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImmutableMetadataWriteOutcome {
    /// A new immutable object was durably stored.
    Created,
    /// Identical bytes were already stored under this logical key.
    AlreadyPresent,
}

/// Outcome of one guarded mutable operational-metadata update.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationalMetadataWriteOutcome {
    /// The replacement was durably committed.
    Updated,
    /// The current bytes did not match the expected prior value.
    Conflict,
}

/// Typed error from filesystem Repository Home metadata operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilesystemHomeMetadataError {
    /// The logical metadata key is empty.
    InvalidKey,
    /// No immutable metadata object exists for the requested key.
    NotFound,
    /// The key already identifies different immutable bytes or stored metadata is corrupt.
    Integrity,
    /// The required safe filesystem operation is unavailable on this target.
    Unsupported,
    /// The filesystem operation failed.
    ProviderFailure,
}

/// Filesystem implementation of the generic Repository Home metadata boundary.
///
/// Metadata logical keys are opaque to this type. Their filesystem mapping is
/// private implementation detail; Core remains responsible for validating
/// metadata schemas and content-derived identities.
pub struct FilesystemRepositoryHome {
    #[cfg_attr(not(target_os = "linux"), allow(dead_code))]
    root: Dir,
    project_id: ProjectId,
}

#[cfg(target_os = "linux")]
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MetadataRecordWire {
    logical_key: String,
    bytes: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RepositoryHomeMarker {
    schema: String,
    project_id: String,
    layout: String,
}

const HOME_MARKER_SCHEMA: &str = "omvcs.repository-home/0.1";
const HOME_LAYOUT: &str = "omvcs.storage-layout/0.1";

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedMapWire {
    project_id: String,
    generation: StorageMapGeneration,
    replicas: Vec<omvcs_model::replica::Replica>,
}

impl FilesystemStorageMapPersistence {
    /// Opens an existing explicit Home candidate root; does not discover
    /// ancestors or create a marker.
    ///
    /// # Errors
    ///
    /// Returns a typed configuration error when the root cannot be opened.
    pub fn open(root: impl AsRef<Path>) -> Result<Self, StorageMapPersistenceFailure> {
        let dir = Dir::open_ambient_dir(root, ambient_authority())
            .map_err(|_| StorageMapPersistenceFailure::ProviderFailure)?;
        Ok(Self { root: dir })
    }

    /// Establishes a generation-zero empty Storage Map record if absent.
    ///
    /// This low-level operational-state initializer does not create a Home
    /// marker and MUST NOT be interpreted as Repository Home bootstrap.
    ///
    /// # Errors
    ///
    /// Returns an applicable persistence failure if initialization conflicts
    /// or the filesystem cannot durably publish the state.
    #[cfg_attr(not(target_os = "linux"), allow(clippy::missing_const_for_fn))]
    pub fn initialize_empty_map(
        &self,
        project_id: ProjectId,
    ) -> Result<(), StorageMapPersistenceFailure> {
        #[cfg(not(target_os = "linux"))]
        {
            let _ = project_id;
            Err(StorageMapPersistenceFailure::UnsupportedConditionalAtomicity)
        }
        #[cfg(target_os = "linux")]
        {
            let directory = self.map_dir()?;
            let _lock = Self::lock_map(&directory)?;
            match open_file(&directory, "storage-map.json") {
                Ok(mut file) => {
                    let mut bytes = Vec::new();
                    file.read_to_end(&mut bytes)
                        .map_err(|_| StorageMapPersistenceFailure::ProviderFailure)?;
                    let wire: PersistedMapWire = serde_json::from_slice(&bytes)
                        .map_err(|_| StorageMapPersistenceFailure::InvalidPersistedState)?;
                    let canonical = serde_jcs::to_vec(&wire)
                        .map_err(|_| StorageMapPersistenceFailure::InvalidPersistedState)?;
                    if canonical == bytes
                        && wire.project_id == project_id.to_string()
                        && wire.generation == StorageMapGeneration::ZERO
                        && wire.replicas.is_empty()
                    {
                        file.sync_all()
                            .map_err(|_| StorageMapPersistenceFailure::ProviderFailure)?;
                        rustix::fs::fsync(&directory)
                            .map_err(|_| StorageMapPersistenceFailure::ProviderFailure)?;
                        rustix::fs::fsync(&self.root)
                            .map_err(|_| StorageMapPersistenceFailure::ProviderFailure)?;
                        return Ok(());
                    }
                    return Err(StorageMapPersistenceFailure::InvalidPersistedState);
                }
                Err(failure) if failure.kind() == io::ErrorKind::NotFound => {}
                Err(_) => return Err(StorageMapPersistenceFailure::ProviderFailure),
            }
            let wire = PersistedMapWire {
                project_id: project_id.to_string(),
                generation: StorageMapGeneration::ZERO,
                replicas: Vec::new(),
            };
            let bytes = serde_jcs::to_vec(&wire)
                .map_err(|_| StorageMapPersistenceFailure::InvalidPersistedState)?;
            publish_create_bytes(&directory, "storage-map.json", &bytes)
                .map_err(|_| StorageMapPersistenceFailure::ProviderFailure)?;
            rustix::fs::fsync(&self.root).map_err(|_| StorageMapPersistenceFailure::ProviderFailure)
        }
    }

    fn map_dir(&self) -> Result<Dir, StorageMapPersistenceFailure> {
        open_directory(&self.root, &[".omvcs"]).map_err(|failure| {
            if failure.kind() == io::ErrorKind::NotFound {
                StorageMapPersistenceFailure::MissingOperationalMetadata
            } else {
                StorageMapPersistenceFailure::ProviderFailure
            }
        })
    }

    #[cfg(target_os = "linux")]
    fn lock_map(directory: &Dir) -> Result<FileLock, StorageMapPersistenceFailure> {
        let file =
            open_lock_file(directory).map_err(|_| StorageMapPersistenceFailure::ProviderFailure)?;
        fs4::FileExt::lock(&file)
            .map_err(|_| StorageMapPersistenceFailure::RepositoryUnavailable)?;
        Ok(FileLock { file })
    }
}

impl FilesystemRepositoryHome {
    /// Opens a complete initialized Home at exactly the selected root.
    ///
    /// This is available only where the implementation can provide
    /// root-relative no-follow access, durable writes, and atomic publication.
    ///
    /// # Errors
    ///
    /// Returns an explicit initialization, identity, or unsupported error.
    pub fn open(
        root: impl AsRef<Path>,
        project_id: ProjectId,
    ) -> Result<Self, FilesystemHomeError> {
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (root, project_id);
            Err(FilesystemHomeError::UnsupportedFilesystemGuarantee)
        }
        #[cfg(target_os = "linux")]
        {
            let root = Dir::open_ambient_dir(root, ambient_authority())
                .map_err(|_| FilesystemHomeError::ProviderFailure)?;
            match discover_at_root(&root, Some(project_id))? {
                FilesystemHomeDiscovery::Initialized => {
                    sync_initialized_home(&root)?;
                    Ok(Self { root, project_id })
                }
                FilesystemHomeDiscovery::NotRepository => Err(FilesystemHomeError::NotInitialized),
                FilesystemHomeDiscovery::IncompleteInitialization => {
                    Err(FilesystemHomeError::IncompleteInitialization)
                }
            }
        }
    }

    /// Returns the Project identity owned by this Home.
    #[must_use]
    pub const fn project_id(&self) -> ProjectId {
        self.project_id
    }

    /// Declares the complete minimum Repository Home profile implemented by
    /// this opened Home. Resource/Chunk byte capabilities are separate.
    #[must_use]
    pub const fn capabilities(&self) -> FilesystemHomeCapabilities {
        FilesystemHomeCapabilities::SUPPORTED
    }

    /// Stores an immutable metadata object without interpreting its schema.
    ///
    /// `logical_key` is opaque here. Core validates that it identifies the
    /// canonical metadata bytes before calling the adapter.
    ///
    /// # Errors
    ///
    /// Returns an explicit key, integrity, durability, or provider failure.
    #[cfg_attr(not(target_os = "linux"), allow(clippy::missing_const_for_fn))]
    pub fn write_immutable_metadata_object(
        &self,
        logical_key: &LogicalKey,
        bytes: &[u8],
    ) -> Result<ImmutableMetadataWriteOutcome, FilesystemHomeMetadataError> {
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (logical_key, bytes);
            Err(FilesystemHomeMetadataError::Unsupported)
        }
        #[cfg(target_os = "linux")]
        {
            validate_metadata_key(logical_key)?;
            let key = logical_key.0.as_str();
            let path = metadata_record_path("objects", key);
            let (parent, filename) = open_metadata_parent(&self.root, &path)?;
            let record = MetadataRecordWire {
                logical_key: key.to_owned(),
                bytes: bytes.to_vec(),
            };
            let encoded =
                serde_jcs::to_vec(&record).map_err(|_| FilesystemHomeMetadataError::Integrity)?;

            match read_metadata_record(&self.root, &path, key)? {
                Some(existing) if existing == bytes => {
                    sync_metadata_record_and_ancestors(&self.root, &path)?;
                    return Ok(ImmutableMetadataWriteOutcome::AlreadyPresent);
                }
                Some(_) => return Err(FilesystemHomeMetadataError::Integrity),
                None => {}
            }

            match publish_create_bytes(&parent, filename, &encoded) {
                Ok(()) => {}
                Err(failure) if failure.kind() == io::ErrorKind::AlreadyExists => {
                    if read_metadata_record(&self.root, &path, key)?.as_deref() == Some(bytes) {
                        sync_metadata_record_and_ancestors(&self.root, &path)?;
                        return Ok(ImmutableMetadataWriteOutcome::AlreadyPresent);
                    }
                    return Err(FilesystemHomeMetadataError::Integrity);
                }
                Err(failure) => return Err(metadata_io_error(failure.kind())),
            }
            sync_metadata_ancestors(&self.root, &path)?;
            Ok(ImmutableMetadataWriteOutcome::Created)
        }
    }

    /// Retrieves exact immutable metadata bytes for Core-side validation.
    ///
    /// # Errors
    ///
    /// Returns not-found, integrity, unsupported, or provider failure.
    #[cfg_attr(not(target_os = "linux"), allow(clippy::missing_const_for_fn))]
    pub fn read_metadata_object(
        &self,
        logical_key: &LogicalKey,
    ) -> Result<Vec<u8>, FilesystemHomeMetadataError> {
        #[cfg(not(target_os = "linux"))]
        {
            let _ = logical_key;
            Err(FilesystemHomeMetadataError::Unsupported)
        }
        #[cfg(target_os = "linux")]
        {
            validate_metadata_key(logical_key)?;
            read_metadata_record(
                &self.root,
                &metadata_record_path("objects", &logical_key.0),
                &logical_key.0,
            )?
            .ok_or(FilesystemHomeMetadataError::NotFound)
        }
    }

    /// Reads an operational metadata value, returning `None` only when absent.
    ///
    /// # Errors
    ///
    /// Returns integrity, unsupported, or provider failure.
    #[cfg_attr(not(target_os = "linux"), allow(clippy::missing_const_for_fn))]
    pub fn read_operational_metadata(
        &self,
        logical_key: &LogicalKey,
    ) -> Result<Option<Vec<u8>>, FilesystemHomeMetadataError> {
        #[cfg(not(target_os = "linux"))]
        {
            let _ = logical_key;
            Err(FilesystemHomeMetadataError::Unsupported)
        }
        #[cfg(target_os = "linux")]
        {
            validate_metadata_key(logical_key)?;
            read_metadata_record(
                &self.root,
                &metadata_record_path("operational", &logical_key.0),
                &logical_key.0,
            )
        }
    }

    /// Replaces one operational value only if its exact previous bytes match.
    ///
    /// The expected value is a guard, not a Core generation. Storage Map
    /// changes must use [`StorageMapPersistence::compare_and_swap`].
    ///
    /// # Errors
    ///
    /// Returns integrity, unsupported, or provider failure.
    #[cfg_attr(not(target_os = "linux"), allow(clippy::missing_const_for_fn))]
    pub fn compare_and_swap_operational_metadata(
        &self,
        logical_key: &LogicalKey,
        expected: Option<&[u8]>,
        replacement: &[u8],
    ) -> Result<OperationalMetadataWriteOutcome, FilesystemHomeMetadataError> {
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (logical_key, expected, replacement);
            Err(FilesystemHomeMetadataError::Unsupported)
        }
        #[cfg(target_os = "linux")]
        {
            validate_metadata_key(logical_key)?;
            let home_metadata = open_directory(&self.root, &[".omvcs"])
                .map_err(|failure| metadata_io_error(failure.kind()))?;
            let _lock = lock_named(&home_metadata, "operational.lock")?;
            let path = metadata_record_path("operational", &logical_key.0);
            let current = read_metadata_record(&self.root, &path, &logical_key.0)?;
            if current.as_deref() != expected {
                return Ok(OperationalMetadataWriteOutcome::Conflict);
            }

            let record = MetadataRecordWire {
                logical_key: logical_key.0.clone(),
                bytes: replacement.to_vec(),
            };
            let encoded =
                serde_jcs::to_vec(&record).map_err(|_| FilesystemHomeMetadataError::Integrity)?;
            let (parent, filename) = open_metadata_parent(&self.root, &path)?;
            let result = if current.is_some() {
                publish_replacement_bytes(&parent, filename, &encoded)
            } else {
                publish_create_bytes(&parent, filename, &encoded)
            };
            if let Err(failure) = result {
                if current.is_none() && failure.kind() == io::ErrorKind::AlreadyExists {
                    return Ok(OperationalMetadataWriteOutcome::Conflict);
                }
                return Err(metadata_io_error(failure.kind()));
            }
            sync_metadata_ancestors(&self.root, &path)?;
            Ok(OperationalMetadataWriteOutcome::Updated)
        }
    }
}

#[cfg(target_os = "linux")]
fn validate_metadata_key(key: &LogicalKey) -> Result<(), FilesystemHomeMetadataError> {
    if key.0.is_empty() {
        return Err(FilesystemHomeMetadataError::InvalidKey);
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn metadata_record_path(namespace: &str, logical_key: &str) -> String {
    let digest: [u8; 32] = Sha256::digest(logical_key.as_bytes()).into();
    let hex = hex_digest(&digest);
    format!(".omvcs/metadata/{namespace}/{}/{}", &hex[..2], &hex[2..])
}

#[cfg(target_os = "linux")]
fn open_metadata_parent<'a>(
    root: &Dir,
    path: &'a str,
) -> Result<(Dir, &'a str), FilesystemHomeMetadataError> {
    let (parent, filename) = path
        .rsplit_once('/')
        .ok_or(FilesystemHomeMetadataError::ProviderFailure)?;
    create_directories(root, parent).map_err(|failure| metadata_io_error(failure.kind()))?;
    let parent = open_directory(root, &parent.split('/').collect::<Vec<_>>())
        .map_err(|failure| metadata_io_error(failure.kind()))?;
    Ok((parent, filename))
}

#[cfg(target_os = "linux")]
fn read_metadata_record(
    root: &Dir,
    path: &str,
    expected_key: &str,
) -> Result<Option<Vec<u8>>, FilesystemHomeMetadataError> {
    let mut file = match open_file(root, path) {
        Ok(file) => file,
        Err(failure) if failure.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(failure) => return Err(metadata_io_error(failure.kind())),
    };
    let mut encoded = Vec::new();
    file.read_to_end(&mut encoded)
        .map_err(|failure| metadata_io_error(failure.kind()))?;
    let record: MetadataRecordWire =
        serde_json::from_slice(&encoded).map_err(|_| FilesystemHomeMetadataError::Integrity)?;
    let canonical =
        serde_jcs::to_vec(&record).map_err(|_| FilesystemHomeMetadataError::Integrity)?;
    if canonical != encoded || record.logical_key != expected_key {
        return Err(FilesystemHomeMetadataError::Integrity);
    }
    Ok(Some(record.bytes))
}

#[cfg(target_os = "linux")]
fn sync_metadata_ancestors(root: &Dir, path: &str) -> Result<(), FilesystemHomeMetadataError> {
    let (mut parent, _) = path
        .rsplit_once('/')
        .ok_or(FilesystemHomeMetadataError::ProviderFailure)?;
    loop {
        let directory = open_directory(root, &parent.split('/').collect::<Vec<_>>())
            .map_err(|failure| metadata_io_error(failure.kind()))?;
        rustix::fs::fsync(&directory)
            .map_err(|failure| metadata_io_error(rustix_io_error(failure).kind()))?;
        match parent.rsplit_once('/') {
            Some((ancestor, _)) => parent = ancestor,
            None => break,
        }
    }
    rustix::fs::fsync(root).map_err(|failure| metadata_io_error(rustix_io_error(failure).kind()))
}

#[cfg(target_os = "linux")]
fn sync_metadata_record_and_ancestors(
    root: &Dir,
    path: &str,
) -> Result<(), FilesystemHomeMetadataError> {
    let file = open_file(root, path).map_err(|failure| metadata_io_error(failure.kind()))?;
    file.sync_all()
        .map_err(|failure| metadata_io_error(failure.kind()))?;
    sync_metadata_ancestors(root, path)
}

#[cfg(target_os = "linux")]
fn lock_named(directory: &Dir, name: &str) -> Result<FileLock, FilesystemHomeMetadataError> {
    let file = open_named_lock_file(directory, name)
        .map_err(|failure| metadata_io_error(failure.kind()))?;
    fs4::FileExt::lock(&file).map_err(|_| FilesystemHomeMetadataError::ProviderFailure)?;
    Ok(FileLock { file })
}

#[cfg(target_os = "linux")]
const fn metadata_io_error(kind: io::ErrorKind) -> FilesystemHomeMetadataError {
    match kind {
        io::ErrorKind::NotFound => FilesystemHomeMetadataError::NotFound,
        io::ErrorKind::Unsupported => FilesystemHomeMetadataError::Unsupported,
        io::ErrorKind::InvalidData => FilesystemHomeMetadataError::Integrity,
        _ => FilesystemHomeMetadataError::ProviderFailure,
    }
}

#[cfg(target_os = "linux")]
fn open_file(root: &Dir, path: &str) -> io::Result<std::fs::File> {
    let descriptor = rustix::fs::openat2(
        root,
        path,
        OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NONBLOCK,
        Mode::empty(),
        ResolveFlags::BENEATH | ResolveFlags::NO_SYMLINKS,
    )
    .map_err(rustix_io_error)?;
    let file = std::fs::File::from(descriptor);
    if !file.metadata()?.file_type().is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "canonical object is not a regular file",
        ));
    }
    Ok(file)
}

#[cfg(target_os = "linux")]
fn open_lock_file(directory: &Dir) -> io::Result<std::fs::File> {
    open_named_lock_file(directory, "storage-map.lock")
}

#[cfg(target_os = "linux")]
fn open_named_lock_file(directory: &Dir, name: &str) -> io::Result<std::fs::File> {
    let descriptor = rustix::fs::openat2(
        directory,
        name,
        OFlags::RDWR | OFlags::CREATE | OFlags::CLOEXEC,
        Mode::from_raw_mode(0o600),
        ResolveFlags::BENEATH | ResolveFlags::NO_SYMLINKS,
    )
    .map_err(rustix_io_error)?;
    Ok(std::fs::File::from(descriptor))
}

#[cfg(target_os = "linux")]
fn open_directory(root: &Dir, components: &[&str]) -> io::Result<Dir> {
    validate_internal_components(components)?;
    let path = components.join("/");
    let descriptor = rustix::fs::openat2(
        root,
        path,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
        Mode::empty(),
        ResolveFlags::BENEATH | ResolveFlags::NO_SYMLINKS,
    )
    .map_err(rustix_io_error)?;
    Ok(Dir::from_std_file(std::fs::File::from(descriptor)))
}

#[cfg(not(target_os = "linux"))]
fn open_file(root: &Dir, path: &str) -> io::Result<std::fs::File> {
    root.open(path).map(cap_std::fs::File::into_std)
}

#[cfg(not(target_os = "linux"))]
fn open_directory(root: &Dir, components: &[&str]) -> io::Result<Dir> {
    validate_internal_components(components)?;
    let mut directory = root.try_clone()?;
    for component in components {
        let metadata = directory.symlink_metadata(component);
        if metadata
            .as_ref()
            .is_ok_and(|entry| entry.file_type().is_symlink())
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "symbolic link in filesystem namespace",
            ));
        }
        directory = directory.open_dir(component)?;
    }
    Ok(directory)
}

fn validate_internal_components(components: &[&str]) -> io::Result<()> {
    if components.is_empty()
        || components.iter().any(|component| {
            component.is_empty()
                || *component == "."
                || *component == ".."
                || component.contains(['/', '\\'])
        })
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid internal path component",
        ));
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn create_directories(root: &Dir, path: &str) -> io::Result<()> {
    let components = path.split('/').collect::<Vec<_>>();
    validate_internal_components(&components)?;
    let mut directory = root.try_clone()?;
    for component in components {
        match rustix::fs::mkdirat(&directory, component, Mode::from_raw_mode(0o700)) {
            Ok(()) => {}
            Err(failure) if failure == rustix::io::Errno::EXIST => {}
            Err(failure) => return Err(rustix_io_error(failure)),
        }
        directory = open_directory(&directory, &[component])?;
    }
    Ok(())
}

#[cfg(not(target_os = "linux"))]
#[allow(dead_code)]
fn create_directories(root: &Dir, path: &str) -> io::Result<()> {
    root.create_dir_all(path)
}

#[cfg(target_os = "linux")]
fn rustix_io_error(failure: rustix::io::Errno) -> io::Error {
    if failure == rustix::io::Errno::LOOP {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "symbolic link in filesystem namespace",
        )
    } else {
        failure.into()
    }
}

#[cfg(target_os = "linux")]
struct FileLock {
    file: std::fs::File,
}

#[cfg(target_os = "linux")]
impl Drop for FileLock {
    fn drop(&mut self) {
        let _ = fs4::FileExt::unlock(&self.file);
    }
}

impl StorageMapPersistence for FilesystemStorageMapPersistence {
    fn load(
        &self,
        project_id: ProjectId,
    ) -> Result<PersistedStorageMapData, StorageMapPersistenceFailure> {
        let directory = self.map_dir()?;
        let bytes = open_file(&directory, "storage-map.json")
            .and_then(|mut file| {
                let mut bytes = Vec::new();
                file.read_to_end(&mut bytes)?;
                Ok(bytes)
            })
            .map_err(|failure| {
                if failure.kind() == io::ErrorKind::NotFound {
                    StorageMapPersistenceFailure::MissingOperationalMetadata
                } else {
                    StorageMapPersistenceFailure::ProviderFailure
                }
            })?;
        let wire: PersistedMapWire = serde_json::from_slice(&bytes)
            .map_err(|_| StorageMapPersistenceFailure::InvalidPersistedState)?;
        let canonical = serde_jcs::to_vec(&wire)
            .map_err(|_| StorageMapPersistenceFailure::InvalidPersistedState)?;
        if canonical != bytes || wire.project_id != project_id.to_string() {
            return Err(StorageMapPersistenceFailure::InvalidPersistedState);
        }
        if wire.generation == StorageMapGeneration::ZERO && !wire.replicas.is_empty() {
            return Err(StorageMapPersistenceFailure::InvalidPersistedState);
        }
        Ok(PersistedStorageMapData::new(wire.replicas, wire.generation))
    }

    fn compare_and_swap(
        &self,
        project_id: ProjectId,
        expected_generation: StorageMapGeneration,
        replacement: StorageMapSnapshot,
    ) -> Result<ConditionalWriteOutcome, StorageMapPersistenceFailure> {
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (project_id, expected_generation, replacement);
            Err(StorageMapPersistenceFailure::UnsupportedConditionalAtomicity)
        }
        #[cfg(target_os = "linux")]
        {
            let directory = self.map_dir()?;
            let _lock = Self::lock_map(&directory)?;
            let current = self.load(project_id)?;
            if current.generation() != expected_generation {
                return Ok(ConditionalWriteOutcome::Conflict {
                    observed_generation: Some(current.generation()),
                });
            }
            if expected_generation == StorageMapGeneration::MAX
                || replacement.generation().get() != expected_generation.get() + 1
            {
                return Err(StorageMapPersistenceFailure::InvalidPersistedState);
            }
            let mut replicas = replacement.map().replicas().cloned().collect::<Vec<_>>();
            replicas.sort_by_key(omvcs_model::replica::Replica::id);
            let wire = PersistedMapWire {
                project_id: project_id.to_string(),
                generation: replacement.generation(),
                replicas,
            };
            publish_replacement(&directory, "storage-map.json", &wire)?;
            Ok(ConditionalWriteOutcome::Committed)
        }
    }
}

/// Explicit-root marker and initial-state operations from ADR-0041/0042.
///
/// Repository Home metadata operations are exposed by
/// [`FilesystemRepositoryHome`].
pub struct FilesystemHomeInitialization;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InitialMapEvidence {
    Absent,
    Valid(ProjectId),
    Invalid,
}

impl FilesystemHomeInitialization {
    /// Discovers Home initialization state at exactly the caller-selected root.
    ///
    /// # Errors
    ///
    /// Returns a specific marker-version, identity, or filesystem failure.
    pub fn discover(
        root: impl AsRef<Path>,
        requested_project: Option<ProjectId>,
    ) -> Result<FilesystemHomeDiscovery, FilesystemHomeError> {
        let root = Dir::open_ambient_dir(root, ambient_authority())
            .map_err(|_| FilesystemHomeError::ProviderFailure)?;
        let discovery = discover_at_root(&root, requested_project)?;
        if discovery == FilesystemHomeDiscovery::Initialized {
            #[cfg(target_os = "linux")]
            sync_initialized_home(&root)?;
            #[cfg(not(target_os = "linux"))]
            return Err(FilesystemHomeError::UnsupportedFilesystemGuarantee);
        }
        Ok(discovery)
    }

    /// Initializes an explicitly selected empty root, or safely resumes
    /// initialization from one validated same-Project marker or empty map.
    ///
    /// This operation establishes the ADR-0041 marker and initial empty
    /// generation-zero Storage Map. Open the result as
    /// [`FilesystemRepositoryHome`] for Repository Home metadata operations.
    ///
    /// # Errors
    ///
    /// Returns an explicit failure for conflicting state, unsupported
    /// filesystem primitives, or failed durable publication.
    pub fn bootstrap(
        root: impl AsRef<Path>,
        project_id: ProjectId,
    ) -> Result<FilesystemHomeBootstrapOutcome, FilesystemHomeError> {
        Self::bootstrap_with_hook(root, project_id, |_| Ok(()))
    }

    fn bootstrap_with_hook(
        root: impl AsRef<Path>,
        project_id: ProjectId,
        mut hook: impl FnMut(BootstrapCheckpoint) -> Result<(), FilesystemHomeError>,
    ) -> Result<FilesystemHomeBootstrapOutcome, FilesystemHomeError> {
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (root, project_id, &mut hook);
            Err(FilesystemHomeError::UnsupportedFilesystemGuarantee)
        }
        #[cfg(target_os = "linux")]
        {
            let root = Dir::open_ambient_dir(root, ambient_authority())
                .map_err(|_| FilesystemHomeError::ProviderFailure)?;
            match discover_at_root(&root, Some(project_id))? {
                FilesystemHomeDiscovery::Initialized => {
                    hook(BootstrapCheckpoint::BeforeDurability)?;
                    sync_initialized_home(&root)?;
                    return Ok(FilesystemHomeBootstrapOutcome::AlreadyInitialized);
                }
                FilesystemHomeDiscovery::NotRepository => {
                    ensure_new_root_is_empty(&root)?;
                }
                FilesystemHomeDiscovery::IncompleteInitialization => {}
            }

            let metadata_directory = match open_directory(&root, &[".omvcs"]) {
                Ok(directory) => directory,
                Err(failure) if failure.kind() == io::ErrorKind::NotFound => {
                    create_directories(&root, ".omvcs")
                        .map_err(|_| FilesystemHomeError::ProviderFailure)?;
                    open_directory(&root, &[".omvcs"])
                        .map_err(|_| FilesystemHomeError::ProviderFailure)?
                }
                Err(_) => return Err(FilesystemHomeError::ProviderFailure),
            };
            rustix::fs::fsync(&root)
                .map_err(|_| FilesystemHomeError::UnsupportedFilesystemGuarantee)?;

            let map_evidence = read_initial_map(&metadata_directory)?;
            let marker_bytes = read_optional_file(&metadata_directory, "repository.json")?;
            let marker = marker_bytes.as_deref().map(decode_marker).transpose()?;
            ensure_bootstrap_metadata_entries(&root, &metadata_directory, marker.is_some())?;
            if let Some(marker) = &marker {
                if parse_project_id(&marker.project_id)? != project_id {
                    return Err(FilesystemHomeError::ProjectIdentityMismatch);
                }
            }
            match map_evidence {
                InitialMapEvidence::Invalid => {
                    return Err(FilesystemHomeError::IncompleteInitialization);
                }
                InitialMapEvidence::Valid(found) if found != project_id => {
                    return Err(FilesystemHomeError::ProjectIdentityMismatch);
                }
                InitialMapEvidence::Absent | InitialMapEvidence::Valid(_) => {}
            }

            let persistence = FilesystemStorageMapPersistence {
                root: root
                    .try_clone()
                    .map_err(|_| FilesystemHomeError::ProviderFailure)?,
            };
            if marker.is_some() && map_evidence == InitialMapEvidence::Absent {
                hook(BootstrapCheckpoint::BeforeCommit)?;
            }
            if map_evidence == InitialMapEvidence::Absent {
                create_directories(&root, ".omvcs")
                    .map_err(|_| FilesystemHomeError::ProviderFailure)?;
                persistence
                    .initialize_empty_map(project_id)
                    .map_err(|_| FilesystemHomeError::ProviderFailure)?;
            }

            if marker.is_none() {
                hook(BootstrapCheckpoint::BeforeCommit)?;
                let marker = RepositoryHomeMarker {
                    schema: HOME_MARKER_SCHEMA.to_owned(),
                    project_id: project_id.to_string(),
                    layout: HOME_LAYOUT.to_owned(),
                };
                let bytes = serde_jcs::to_vec(&marker)
                    .map_err(|_| FilesystemHomeError::InvalidRepositoryMetadata)?;
                if let Err(failure) =
                    publish_create_bytes(&metadata_directory, "repository.json", &bytes)
                {
                    if failure.kind() != io::ErrorKind::AlreadyExists
                        || discover_at_root(&root, Some(project_id))?
                            != FilesystemHomeDiscovery::Initialized
                    {
                        return Err(FilesystemHomeError::ProviderFailure);
                    }
                }
            }

            if discover_at_root(&root, Some(project_id))? != FilesystemHomeDiscovery::Initialized {
                return Err(FilesystemHomeError::IncompleteInitialization);
            }
            hook(BootstrapCheckpoint::BeforeDurability)?;
            sync_initialized_home(&root)?;
            hook(BootstrapCheckpoint::AfterCommit)?;
            Ok(FilesystemHomeBootstrapOutcome::Initialized)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
enum BootstrapCheckpoint {
    BeforeCommit,
    BeforeDurability,
    AfterCommit,
}

#[cfg(target_os = "linux")]
fn sync_initialized_home(root: &Dir) -> Result<(), FilesystemHomeError> {
    let metadata =
        open_directory(root, &[".omvcs"]).map_err(|_| FilesystemHomeError::ProviderFailure)?;
    for name in ["repository.json", "storage-map.json"] {
        let file = open_file(&metadata, name).map_err(|_| FilesystemHomeError::ProviderFailure)?;
        file.sync_all()
            .map_err(|_| FilesystemHomeError::ProviderFailure)?;
    }
    rustix::fs::fsync(&metadata).map_err(|_| FilesystemHomeError::ProviderFailure)?;
    rustix::fs::fsync(root).map_err(|_| FilesystemHomeError::ProviderFailure)
}

fn discover_at_root(
    root: &Dir,
    requested_project: Option<ProjectId>,
) -> Result<FilesystemHomeDiscovery, FilesystemHomeError> {
    let metadata_directory = match open_directory(root, &[".omvcs"]) {
        Ok(directory) => directory,
        Err(failure) if failure.kind() == io::ErrorKind::NotFound => {
            return Ok(FilesystemHomeDiscovery::NotRepository);
        }
        Err(_) => return Err(FilesystemHomeError::ProviderFailure),
    };
    let marker_bytes = read_optional_file(&metadata_directory, "repository.json")?;
    let map_bytes = read_optional_file(&metadata_directory, "storage-map.json")?;
    let map_evidence = map_bytes
        .as_deref()
        .map_or(InitialMapEvidence::Absent, |bytes| {
            decode_persisted_map(bytes)
                .map_or(InitialMapEvidence::Invalid, InitialMapEvidence::Valid)
        });
    let has_extra_entries = metadata_directory
        .entries()
        .map_err(|_| FilesystemHomeError::ProviderFailure)?
        .try_fold(false, |found, entry| {
            let entry = entry.map_err(|_| FilesystemHomeError::ProviderFailure)?;
            let name = entry.file_name();
            Ok::<_, FilesystemHomeError>(
                found
                    || (name != ".omvcs"
                        && name != "repository.json"
                        && name != "storage-map.json"),
            )
        })?;

    let Some(marker_bytes) = marker_bytes else {
        return Ok(
            if map_evidence != InitialMapEvidence::Absent || has_extra_entries {
                FilesystemHomeDiscovery::IncompleteInitialization
            } else {
                FilesystemHomeDiscovery::NotRepository
            },
        );
    };

    let marker = match decode_marker(&marker_bytes) {
        Ok(marker) => marker,
        Err(FilesystemHomeError::InvalidRepositoryMetadata)
            if map_evidence != InitialMapEvidence::Absent || has_extra_entries =>
        {
            return Ok(FilesystemHomeDiscovery::IncompleteInitialization);
        }
        Err(failure) => return Err(failure),
    };
    let marker_project = parse_project_id(&marker.project_id)?;
    if requested_project.is_some_and(|requested| requested != marker_project) {
        return Err(FilesystemHomeError::ProjectIdentityMismatch);
    }
    Ok(match map_evidence {
        InitialMapEvidence::Valid(map_project) if map_project == marker_project => {
            FilesystemHomeDiscovery::Initialized
        }
        InitialMapEvidence::Absent | InitialMapEvidence::Valid(_) | InitialMapEvidence::Invalid => {
            FilesystemHomeDiscovery::IncompleteInitialization
        }
    })
}

fn decode_marker(bytes: &[u8]) -> Result<RepositoryHomeMarker, FilesystemHomeError> {
    let marker: RepositoryHomeMarker = serde_json::from_slice(bytes)
        .map_err(|_| FilesystemHomeError::InvalidRepositoryMetadata)?;
    let canonical =
        serde_jcs::to_vec(&marker).map_err(|_| FilesystemHomeError::InvalidRepositoryMetadata)?;
    if canonical != bytes {
        return Err(FilesystemHomeError::InvalidRepositoryMetadata);
    }
    parse_project_id(&marker.project_id)?;
    if marker.schema != HOME_MARKER_SCHEMA || marker.layout != HOME_LAYOUT {
        return Err(FilesystemHomeError::UnsupportedVersion);
    }
    Ok(marker)
}

fn parse_project_id(value: &str) -> Result<ProjectId, FilesystemHomeError> {
    let project_id = value
        .parse::<ProjectId>()
        .map_err(|_| FilesystemHomeError::InvalidRepositoryMetadata)?;
    if project_id.to_string() != value {
        return Err(FilesystemHomeError::InvalidRepositoryMetadata);
    }
    Ok(project_id)
}

#[cfg(target_os = "linux")]
fn decode_initial_map(bytes: &[u8]) -> Result<InitialMapEvidence, FilesystemHomeError> {
    let wire: PersistedMapWire = serde_json::from_slice(bytes)
        .map_err(|_| FilesystemHomeError::InvalidRepositoryMetadata)?;
    let canonical =
        serde_jcs::to_vec(&wire).map_err(|_| FilesystemHomeError::InvalidRepositoryMetadata)?;
    if canonical != bytes
        || wire.generation != StorageMapGeneration::ZERO
        || !wire.replicas.is_empty()
    {
        return Err(FilesystemHomeError::InvalidRepositoryMetadata);
    }
    Ok(InitialMapEvidence::Valid(parse_project_id(
        &wire.project_id,
    )?))
}

fn decode_persisted_map(bytes: &[u8]) -> Result<ProjectId, FilesystemHomeError> {
    let wire: PersistedMapWire = serde_json::from_slice(bytes)
        .map_err(|_| FilesystemHomeError::InvalidRepositoryMetadata)?;
    let canonical =
        serde_jcs::to_vec(&wire).map_err(|_| FilesystemHomeError::InvalidRepositoryMetadata)?;
    if canonical != bytes
        || (wire.generation == StorageMapGeneration::ZERO && !wire.replicas.is_empty())
    {
        return Err(FilesystemHomeError::InvalidRepositoryMetadata);
    }
    let mut replica_ids = BTreeSet::new();
    if wire
        .replicas
        .iter()
        .any(|replica| !replica_ids.insert(replica.id()))
    {
        return Err(FilesystemHomeError::InvalidRepositoryMetadata);
    }
    parse_project_id(&wire.project_id)
}

#[cfg(target_os = "linux")]
fn ensure_bootstrap_metadata_entries(
    root: &Dir,
    metadata_directory: &Dir,
    marker_present: bool,
) -> Result<(), FilesystemHomeError> {
    if !marker_present {
        for entry in root
            .entries()
            .map_err(|_| FilesystemHomeError::ProviderFailure)?
        {
            let entry = entry.map_err(|_| FilesystemHomeError::ProviderFailure)?;
            if entry.file_name() != ".omvcs" {
                return Err(FilesystemHomeError::IncompleteInitialization);
            }
        }
    }

    for entry in metadata_directory
        .entries()
        .map_err(|_| FilesystemHomeError::ProviderFailure)?
    {
        let entry = entry.map_err(|_| FilesystemHomeError::ProviderFailure)?;
        let name = entry.file_name();
        let allowed = name == "storage-map.json"
            || name == "storage-map.lock"
            || (marker_present && name == "repository.json");
        if !allowed {
            return Err(FilesystemHomeError::IncompleteInitialization);
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn read_initial_map(directory: &Dir) -> Result<InitialMapEvidence, FilesystemHomeError> {
    match read_optional_file(directory, "storage-map.json") {
        Ok(Some(bytes)) => Ok(decode_initial_map(&bytes).unwrap_or(InitialMapEvidence::Invalid)),
        Ok(None) => Ok(InitialMapEvidence::Absent),
        Err(failure) => Err(failure),
    }
}

fn read_optional_file(directory: &Dir, name: &str) -> Result<Option<Vec<u8>>, FilesystemHomeError> {
    match open_file(directory, name) {
        Ok(mut file) => {
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)
                .map_err(|_| FilesystemHomeError::ProviderFailure)?;
            Ok(Some(bytes))
        }
        Err(failure) if failure.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err(FilesystemHomeError::ProviderFailure),
    }
}

#[cfg(target_os = "linux")]
fn ensure_new_root_is_empty(root: &Dir) -> Result<(), FilesystemHomeError> {
    for entry in root
        .entries()
        .map_err(|_| FilesystemHomeError::ProviderFailure)?
    {
        let entry = entry.map_err(|_| FilesystemHomeError::ProviderFailure)?;
        if entry.file_name() != ".omvcs" {
            return Err(FilesystemHomeError::IncompleteInitialization);
        }
        let metadata_directory = open_directory(root, &[".omvcs"])
            .map_err(|_| FilesystemHomeError::IncompleteInitialization)?;
        if metadata_directory
            .entries()
            .map_err(|_| FilesystemHomeError::ProviderFailure)?
            .next()
            .is_some()
        {
            return Err(FilesystemHomeError::IncompleteInitialization);
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn publish_create_bytes(directory: &Dir, destination: &str, bytes: &[u8]) -> io::Result<()> {
    let (stage, mut file) = create_staging_file(directory)?;
    if let Err(failure) = file.write_all(bytes).and_then(|()| file.sync_all()) {
        drop(file);
        let _ = directory.remove_file(&stage);
        return Err(failure);
    }
    drop(file);
    if let Err(failure) = rustix::fs::renameat_with(
        directory,
        &stage,
        directory,
        destination,
        rustix::fs::RenameFlags::NOREPLACE,
    )
    .map_err(rustix_io_error)
    {
        let _ = directory.remove_file(&stage);
        return Err(failure);
    }
    rustix::fs::fsync(directory).map_err(rustix_io_error)?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn publish_replacement<T: Serialize>(
    directory: &Dir,
    destination: &str,
    value: &T,
) -> Result<(), StorageMapPersistenceFailure> {
    let bytes = serde_jcs::to_vec(value)
        .map_err(|_| StorageMapPersistenceFailure::InvalidPersistedState)?;
    publish_replacement_bytes(directory, destination, &bytes)
        .map_err(|_| StorageMapPersistenceFailure::ProviderFailure)
}

#[cfg(target_os = "linux")]
fn publish_replacement_bytes(directory: &Dir, destination: &str, bytes: &[u8]) -> io::Result<()> {
    let (stage, mut file) = create_staging_file(directory)?;
    if let Err(failure) = file.write_all(bytes).and_then(|()| file.sync_all()) {
        drop(file);
        let _ = directory.remove_file(&stage);
        return Err(failure);
    }
    drop(file);
    if let Err(failure) = directory.rename(&stage, directory, destination) {
        let _ = directory.remove_file(&stage);
        return Err(failure);
    }
    rustix::fs::fsync(directory).map_err(rustix_io_error)
}

#[cfg(target_os = "linux")]
fn create_staging_file(parent: &Dir) -> io::Result<(String, File)> {
    for _ in 0..32 {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let stage_name = format!(".omvcs-stage-{}-{sequence:016x}", std::process::id());
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        match parent.open_with(&stage_name, &options) {
            Ok(file) => return Ok((stage_name, file)),
            Err(failure) if failure.kind() == io::ErrorKind::AlreadyExists => {}
            Err(failure) => return Err(failure),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "private staging names exhausted",
    ))
}

#[cfg(target_os = "linux")]
#[derive(Debug)]
enum StageFailure {
    InputRead(io::Error),
    OutputWrite(io::Error),
    ContentMismatch,
}

#[cfg(target_os = "linux")]
fn stage_and_verify(
    destination: &mut File,
    source: &mut dyn Read,
    id: ObjectId,
    expected_digest: [u8; 32],
    expected_length: u64,
) -> Result<(), StageFailure> {
    let mut hasher = Sha256::new();
    let mut length = 0_u64;
    let mut buffer = vec![0_u8; 64 * 1024].into_boxed_slice();
    loop {
        let count = source.read(&mut buffer).map_err(StageFailure::InputRead)?;
        if count == 0 {
            break;
        }
        destination
            .write_all(&buffer[..count])
            .map_err(StageFailure::OutputWrite)?;
        hasher.update(&buffer[..count]);
        length = length
            .checked_add(count as u64)
            .ok_or(StageFailure::ContentMismatch)?;
    }
    let digest: [u8; 32] = hasher.finalize().into();
    if length != expected_length || digest != expected_digest || digest != *id.digest() {
        return Err(StageFailure::ContentMismatch);
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn hash_reader(reader: &mut dyn Read) -> io::Result<([u8; 32], u64)> {
    let mut hasher = Sha256::new();
    let mut length = 0_u64;
    let mut buffer = vec![0_u8; 64 * 1024].into_boxed_slice();
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
        length = length
            .checked_add(count as u64)
            .ok_or_else(|| io::Error::other("content length overflow"))?;
    }
    Ok((hasher.finalize().into(), length))
}

#[cfg(target_os = "linux")]
fn verify_reader(
    reader: &mut dyn Read,
    id: ObjectId,
    expected_digest: [u8; 32],
    expected_length: u64,
) -> Result<(), StageFailure> {
    let (digest, length) = hash_reader(reader).map_err(StageFailure::InputRead)?;
    if length != expected_length || digest != expected_digest || digest != *id.digest() {
        return Err(StageFailure::ContentMismatch);
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn input_verification_error(
    endpoint: StorageEndpointId,
    operation: Operation,
    failure: StageFailure,
) -> StorageError {
    match failure {
        StageFailure::InputRead(failure) => {
            if let Some(contextual) = failure
                .get_ref()
                .and_then(|source| source.downcast_ref::<StorageError>())
            {
                return contextual.clone();
            }
            error(
                endpoint,
                operation,
                ErrorClass::Precondition,
                "INPUT_STREAM_FAILURE",
                &format!("Input content stream failed: {failure}"),
            )
        }
        StageFailure::OutputWrite(failure) => storage_error(endpoint, operation, failure),
        StageFailure::ContentMismatch => error(
            endpoint,
            operation,
            ErrorClass::Integrity,
            "CONTENT_MISMATCH",
            "Input stream bytes or length do not match the requested immutable identity",
        ),
    }
}

#[cfg(target_os = "linux")]
fn hex_digest(digest: &[u8; 32]) -> String {
    let mut result = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(result, "{byte:02x}");
    }
    result
}

fn error(
    endpoint: StorageEndpointId,
    operation: Operation,
    class: ErrorClass,
    suffix: &str,
    summary: &str,
) -> StorageError {
    StorageError {
        code: format!("OMVCS_STORAGE_{suffix}"),
        class,
        operation,
        endpoint,
        recoverable: false,
        provider_context: None,
        summary: summary.to_owned(),
    }
}

fn storage_error(
    endpoint: StorageEndpointId,
    operation: Operation,
    failure: io::Error,
) -> StorageError {
    if let Some(contextual) = failure
        .get_ref()
        .and_then(|source| source.downcast_ref::<StorageError>())
    {
        return contextual.clone();
    }
    let (class, suffix) = match failure.kind() {
        io::ErrorKind::NotFound => (ErrorClass::NotFound, "NOT_FOUND"),
        io::ErrorKind::PermissionDenied => (ErrorClass::Authorization, "PERMISSION_DENIED"),
        io::ErrorKind::AlreadyExists => (ErrorClass::Conflict, "CONFLICT"),
        io::ErrorKind::InvalidInput => (ErrorClass::InvalidRequest, "INVALID_PATH"),
        io::ErrorKind::Unsupported => (ErrorClass::Unsupported, "UNSUPPORTED"),
        io::ErrorKind::InvalidData => (ErrorClass::Integrity, "LINKED_PATH_COMPONENT"),
        _ => (ErrorClass::ProviderUnavailable, "FILESYSTEM_FAILURE"),
    };
    let summary = failure.to_string();
    drop(failure);
    error(endpoint, operation, class, suffix, &summary)
}

#[cfg(all(test, target_os = "linux"))]
mod bootstrap_tests {
    #![allow(clippy::unwrap_used)]

    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::*;

    static ROOT_ID: AtomicU64 = AtomicU64::new(0);

    struct TestRoot(PathBuf);

    impl TestRoot {
        fn new() -> Self {
            let unique = ROOT_ID.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "omvcs-bootstrap-unit-test-{}-{unique}",
                std::process::id()
            ));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TestRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn project_id() -> ProjectId {
        "019cc17d-1b22-7a41-9fe9-c345c468f82e".parse().unwrap()
    }

    #[test]
    fn physical_key_names_cannot_case_fold_into_aliases() {
        let resource_id =
            omvcs_model::hashing::hash_resource_bytes(b"same content identifier bytes");
        let chunk_id = omvcs_storage::ChunkId::from_digest(*resource_id.digest());
        let resource_key = FilesystemStorage::physical_key(ObjectId::Resource(resource_id));
        let chunk_key = FilesystemStorage::physical_key(ObjectId::Chunk(chunk_id));

        assert_ne!(resource_key, chunk_key);
        assert!(
            resource_key
                .chars()
                .all(|character| character == '/' || character.is_ascii_digit())
        );
        assert!(
            chunk_key
                .chars()
                .all(|character| character == '/' || character.is_ascii_digit())
        );
    }

    #[test]
    fn interrupted_before_commit_is_discovered_as_incomplete() {
        let root = TestRoot::new();
        let result = FilesystemHomeInitialization::bootstrap_with_hook(
            &root.0,
            project_id(),
            |checkpoint| {
                assert_eq!(checkpoint, BootstrapCheckpoint::BeforeCommit);
                Err(FilesystemHomeError::ProviderFailure)
            },
        );
        assert_eq!(result, Err(FilesystemHomeError::ProviderFailure));
        assert_eq!(
            FilesystemHomeInitialization::discover(&root.0, Some(project_id())).unwrap(),
            FilesystemHomeDiscovery::IncompleteInitialization
        );
    }

    #[test]
    fn interrupted_after_commit_is_rediscovered_and_retry_is_idempotent() {
        let root = TestRoot::new();
        let result = FilesystemHomeInitialization::bootstrap_with_hook(
            &root.0,
            project_id(),
            |checkpoint| match checkpoint {
                BootstrapCheckpoint::BeforeCommit | BootstrapCheckpoint::BeforeDurability => Ok(()),
                BootstrapCheckpoint::AfterCommit => Err(FilesystemHomeError::ProviderFailure),
            },
        );
        assert_eq!(result, Err(FilesystemHomeError::ProviderFailure));
        assert_eq!(
            FilesystemHomeInitialization::discover(&root.0, Some(project_id())).unwrap(),
            FilesystemHomeDiscovery::Initialized
        );
        assert_eq!(
            FilesystemHomeInitialization::bootstrap(&root.0, project_id()).unwrap(),
            FilesystemHomeBootstrapOutcome::AlreadyInitialized
        );
    }

    #[test]
    fn initialized_retry_must_reestablish_durability_before_success() {
        let root = TestRoot::new();
        assert_eq!(
            FilesystemHomeInitialization::bootstrap(&root.0, project_id()).unwrap(),
            FilesystemHomeBootstrapOutcome::Initialized
        );

        let retry = FilesystemHomeInitialization::bootstrap_with_hook(
            &root.0,
            project_id(),
            |checkpoint| {
                assert_eq!(checkpoint, BootstrapCheckpoint::BeforeDurability);
                Err(FilesystemHomeError::ProviderFailure)
            },
        );
        assert_eq!(retry, Err(FilesystemHomeError::ProviderFailure));
        assert_eq!(
            FilesystemHomeInitialization::bootstrap(&root.0, project_id()).unwrap(),
            FilesystemHomeBootstrapOutcome::AlreadyInitialized
        );
    }
}
