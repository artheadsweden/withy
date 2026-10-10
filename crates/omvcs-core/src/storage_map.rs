//! Project-scoped Storage Map snapshots and guarded operational mutations.
//!
//! This module deliberately exposes no Replica-addition operation. Removal
//! affects only Storage Map metadata; it does not invoke a Storage Adapter or
//! authorize physical deletion.

use omvcs_model::replica::{ProviderLocator, Replica, ReplicaAvailability};
use omvcs_model::{ProjectId, ReplicaId, ResourceId};
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::value::RawValue;
use std::collections::BTreeMap;
use std::fmt;

/// Largest portable exact JSON integer for a Storage Map generation.
pub const MAX_STORAGE_MAP_GENERATION: u64 = 9_007_199_254_740_991;

/// The Project-wide exact integer that guards the complete logical Storage Map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StorageMapGeneration(u64);

/// Invalid exact-integer value or representation for a map generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageMapGenerationError {
    /// The input is not the canonical decimal representation of an integer.
    InvalidRepresentation,
    /// The integer lies outside the interoperable generation domain.
    OutOfRange,
}

impl fmt::Display for StorageMapGenerationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRepresentation => {
                formatter.write_str("StorageMapGeneration must be a canonical JSON integer")
            }
            Self::OutOfRange => {
                formatter.write_str("StorageMapGeneration must be in 0..=9007199254740991")
            }
        }
    }
}

impl std::error::Error for StorageMapGenerationError {}

impl StorageMapGeneration {
    /// Generation for a newly initialized empty map.
    pub const ZERO: Self = Self(0);

    /// Largest representable Storage Map generation.
    pub const MAX: Self = Self(MAX_STORAGE_MAP_GENERATION);

    /// Returns the exact generation integer.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    fn next(self) -> Option<Self> {
        (self.0 < MAX_STORAGE_MAP_GENERATION).then(|| Self(self.0 + 1))
    }

    fn parse_canonical_decimal(raw: &str) -> Result<Self, StorageMapGenerationError> {
        let digits = raw.as_bytes();
        let canonical = digits == b"0"
            || matches!(digits.first(), Some(b'1'..=b'9'))
                && digits[1..].iter().all(u8::is_ascii_digit);
        if !canonical {
            return Err(StorageMapGenerationError::InvalidRepresentation);
        }
        let value = raw
            .parse::<u64>()
            .map_err(|_| StorageMapGenerationError::OutOfRange)?;
        Self::try_from(value)
    }
}

impl TryFrom<u64> for StorageMapGeneration {
    type Error = StorageMapGenerationError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value <= MAX_STORAGE_MAP_GENERATION {
            Ok(Self(value))
        } else {
            Err(StorageMapGenerationError::OutOfRange)
        }
    }
}

impl Serialize for StorageMapGeneration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(self.0)
    }
}

impl<'de> Deserialize<'de> for StorageMapGeneration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = Box::<RawValue>::deserialize(deserializer)?;
        Self::parse_canonical_decimal(raw.get()).map_err(D::Error::custom)
    }
}

/// The complete logical map of one Project to its known Replica records.
///
/// The index is keyed by `ReplicaId`. Resource membership is derived from each
/// record's `ResourceId`, which structurally permits zero, one, or many
/// Replicas for a Resource, including at a common Endpoint.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StorageMap {
    replicas: BTreeMap<ReplicaId, Replica>,
}

/// Invalid persisted Storage Map contents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageMapBuildError {
    /// A `ReplicaId` occurs more than once in the persisted map.
    DuplicateReplicaId,
}

impl fmt::Display for StorageMapBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateReplicaId => {
                formatter.write_str("Storage Map contains a duplicate ReplicaId")
            }
        }
    }
}

impl std::error::Error for StorageMapBuildError {}

/// Replica records and generation returned by an authoritative persistence
/// implementation before Core validates and reconstructs a Storage Map.
///
/// This data container is not itself a registered Storage Map. Only Core's
/// persistence-load path reconstructs it into a [`StorageMap`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistedStorageMapData {
    replicas: Vec<Replica>,
    generation: StorageMapGeneration,
}

impl PersistedStorageMapData {
    /// Supplies persisted Replica records and their Project-wide generation.
    ///
    /// Repository Home implementations MUST return only records that were
    /// admitted through applicable registration requirements.
    #[must_use]
    pub fn new(
        replicas: impl IntoIterator<Item = Replica>,
        generation: StorageMapGeneration,
    ) -> Self {
        Self {
            replicas: replicas.into_iter().collect(),
            generation,
        }
    }

    /// Returns the persisted records as supplied by the persistence boundary.
    #[must_use]
    pub fn replicas(&self) -> &[Replica] {
        &self.replicas
    }

    /// Returns the persisted Project-wide generation.
    #[must_use]
    pub const fn generation(&self) -> StorageMapGeneration {
        self.generation
    }
}

impl StorageMap {
    /// Creates an empty logical Storage Map.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Reconstitutes records loaded from an authoritative Repository Home.
    ///
    /// This is not a registration or candidate-promotion operation. Callers
    /// MUST NOT use this constructor to admit records from a non-authoritative
    /// source unless they independently satisfy the applicable registration
    /// requirements. Structural validity alone does not establish persistence
    /// authority.
    ///
    /// # Errors
    ///
    /// Returns [`StorageMapBuildError::DuplicateReplicaId`] if the persisted
    /// records do not have unique identifiers.
    fn from_persisted_records(
        replicas: impl IntoIterator<Item = Replica>,
    ) -> Result<Self, StorageMapBuildError> {
        let mut indexed = BTreeMap::new();
        for replica in replicas {
            if indexed.insert(replica.id(), replica).is_some() {
                return Err(StorageMapBuildError::DuplicateReplicaId);
            }
        }
        Ok(Self { replicas: indexed })
    }

    /// Returns all known records in `ReplicaId` order.
    pub fn replicas(&self) -> impl Iterator<Item = &Replica> {
        self.replicas.values()
    }

    /// Returns the record with the requested assigned identifier, if present.
    #[must_use]
    pub fn get(&self, id: ReplicaId) -> Option<&Replica> {
        self.replicas.get(&id)
    }

    /// Returns the known records for one immutable Resource identity.
    pub fn replicas_for_resource(&self, resource_id: ResourceId) -> impl Iterator<Item = &Replica> {
        self.replicas
            .values()
            .filter(move |replica| replica.resource_id() == resource_id)
    }

    /// Returns the number of known Replica records.
    #[must_use]
    pub fn len(&self) -> usize {
        self.replicas.len()
    }

    /// Returns whether the map contains no Replica records.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.replicas.is_empty()
    }
}

/// A coherent Storage Map plus its Project-wide generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageMapSnapshot {
    map: StorageMap,
    generation: StorageMapGeneration,
}

/// Invalid combination of a persisted Storage Map and generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageMapSnapshotError {
    /// Generation zero denotes the initialized empty map, not populated state.
    NonEmptyAtGenerationZero,
}

impl fmt::Display for StorageMapSnapshotError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonEmptyAtGenerationZero => {
                formatter.write_str("generation zero requires an empty Storage Map")
            }
        }
    }
}

impl std::error::Error for StorageMapSnapshotError {}

impl StorageMapSnapshot {
    /// Establishes a newly initialized empty map at generation zero.
    #[must_use]
    pub fn initialized_empty() -> Self {
        Self {
            map: StorageMap::empty(),
            generation: StorageMapGeneration::ZERO,
        }
    }

    /// Reconstitutes an existing persisted map and its generation together.
    ///
    /// Generation zero represents only the initialized empty Storage Map.
    ///
    /// # Errors
    ///
    /// Returns [`StorageMapSnapshotError::NonEmptyAtGenerationZero`] if the
    /// persisted state contains records at generation zero.
    fn from_persisted(
        map: StorageMap,
        generation: StorageMapGeneration,
    ) -> Result<Self, StorageMapSnapshotError> {
        if generation == StorageMapGeneration::ZERO && !map.is_empty() {
            return Err(StorageMapSnapshotError::NonEmptyAtGenerationZero);
        }
        Ok(Self { map, generation })
    }

    /// Returns the complete map snapshot.
    #[must_use]
    pub const fn map(&self) -> &StorageMap {
        &self.map
    }

    /// Returns the map's exact current generation.
    #[must_use]
    pub const fn generation(&self) -> StorageMapGeneration {
        self.generation
    }
}

/// One already-normative change to an existing Replica record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplicaChange {
    /// Remove the record from Storage Map metadata only.
    Remove,
    /// Update only mutable operational facts on the same representation.
    Update {
        /// Replacement opaque locator, if changed by this request.
        locator: Option<ProviderLocator>,
        /// Replacement retrieval state, if changed by this request.
        availability: Option<ReplicaAvailability>,
    },
}

/// A logical map mutation request with at most one change per `ReplicaId`.
///
/// Replica addition is deliberately absent until the approved verification
/// result contract is available. In particular, this API cannot promote an
/// unverified model value:
///
/// ```compile_fail
/// use omvcs_core::storage_map::StorageMapMutation;
/// use omvcs_model::replica::Replica;
///
/// let mut mutation = StorageMapMutation::new();
/// let candidate: Replica = todo!();
/// mutation.add_replica(candidate);
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StorageMapMutation {
    changes: BTreeMap<ReplicaId, ReplicaChange>,
}

/// Invalid construction of a single logical mutation request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageMapMutationBuildError {
    /// One `ReplicaId` is assigned more than one operation in a request.
    DuplicateReplicaChange,
    /// A removal and an operational update were requested for one record.
    ConflictingReplicaChange,
}

impl fmt::Display for StorageMapMutationBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateReplicaChange => {
                formatter.write_str("mutation repeats a change to one ReplicaId")
            }
            Self::ConflictingReplicaChange => {
                formatter.write_str("mutation cannot update and remove one ReplicaId")
            }
        }
    }
}

impl std::error::Error for StorageMapMutationBuildError {}

impl StorageMapMutation {
    /// Creates an empty logical mutation, which is a valid no-op.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Requests metadata-only removal of an existing Replica record.
    ///
    /// This does not call a provider or delete physical bytes.
    ///
    /// # Errors
    ///
    /// Returns [`StorageMapMutationBuildError::ConflictingReplicaChange`] if
    /// this request already contains any change for `id`.
    pub fn remove_replica(&mut self, id: ReplicaId) -> Result<(), StorageMapMutationBuildError> {
        if self.changes.contains_key(&id) {
            return Err(StorageMapMutationBuildError::ConflictingReplicaChange);
        }
        self.changes.insert(id, ReplicaChange::Remove);
        Ok(())
    }

    /// Requests a mutable locator and/or retrieval-state update for a record.
    ///
    /// An update with both options absent is a valid no-op request.
    ///
    /// # Errors
    ///
    /// Returns [`StorageMapMutationBuildError::DuplicateReplicaChange`] if
    /// this request already contains a change for `id`.
    pub fn update_replica(
        &mut self,
        id: ReplicaId,
        locator: Option<ProviderLocator>,
        availability: Option<ReplicaAvailability>,
    ) -> Result<(), StorageMapMutationBuildError> {
        if self.changes.contains_key(&id) {
            return Err(StorageMapMutationBuildError::DuplicateReplicaChange);
        }
        self.changes.insert(
            id,
            ReplicaChange::Update {
                locator,
                availability,
            },
        );
        Ok(())
    }

    /// Returns whether the logical request contains no changes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }
}

/// Failure to read or atomically persist a complete Project map snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageMapPersistenceFailure {
    /// No initialized or complete map-generation pair exists for this Project.
    MissingOperationalMetadata,
    /// The authoritative persisted records or map/generation pair is invalid.
    InvalidPersistedState,
    /// The backend cannot guarantee the required conditional atomic write.
    UnsupportedConditionalAtomicity,
    /// The provider failed to read or persist the guarded map state.
    ProviderFailure,
    /// The in-memory synchronization primitive is unavailable.
    RepositoryUnavailable,
}

impl fmt::Display for StorageMapPersistenceFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::MissingOperationalMetadata => {
                "Storage Map operational metadata is absent or incomplete"
            }
            Self::InvalidPersistedState => {
                "authoritative Storage Map persistence contains invalid state"
            }
            Self::UnsupportedConditionalAtomicity => {
                "Storage Map persistence cannot provide conditional atomicity"
            }
            Self::ProviderFailure => "Storage Map persistence provider failed",
            Self::RepositoryUnavailable => "Storage Map repository is unavailable",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for StorageMapPersistenceFailure {}

/// The result of one conditional persistence attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionalWriteOutcome {
    /// The complete replacement map and generation were persisted atomically.
    Committed,
    /// Another state now has a different Core generation.
    Conflict {
        /// The observed Core generation, if available.
        observed_generation: Option<StorageMapGeneration>,
    },
}

/// Provider-neutral persistence boundary for complete map snapshots.
///
/// Implementations must return coherent map/generation snapshots and perform
/// `compare_and_swap` as one conditional, atomic replacement. Successful
/// outcomes MUST represent state that is durably persisted or reconstructible
/// under Storage Adapter §51. Provider tokens remain implementation details
/// and are never exposed as Core generations. Replica records MUST be
/// persisted using [`Replica::canonical_bytes`] so `ProviderLocator` values use
/// RFC 8785/JCS canonical JSON.
pub trait StorageMapPersistence {
    /// Reads persisted Replica records and generation from the authoritative
    /// Repository Home persistence boundary.
    ///
    /// Implementations MUST uphold ADR-0035: loaded Replica records represent
    /// prior successful registration. Core validates the returned records
    /// and constructs the map/generation snapshot; loading does not re-verify
    /// Resource bytes solely because the map is reconstructed.
    ///
    /// # Errors
    ///
    /// Returns an applicable [`StorageMapPersistenceFailure`] when metadata is
    /// absent/incomplete or the persistence boundary cannot read it.
    fn load(
        &self,
        project_id: ProjectId,
    ) -> Result<PersistedStorageMapData, StorageMapPersistenceFailure>;

    /// Atomically replaces one complete map only when its Core generation
    /// equals `expected_generation`.
    ///
    /// # Errors
    ///
    /// Returns an applicable [`StorageMapPersistenceFailure`] when the
    /// guarded persistence operation is unsupported or fails.
    fn compare_and_swap(
        &self,
        project_id: ProjectId,
        expected_generation: StorageMapGeneration,
        replacement: StorageMapSnapshot,
    ) -> Result<ConditionalWriteOutcome, StorageMapPersistenceFailure>;
}

/// Typed result of applying one guarded Project Storage Map mutation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageMapMutationOutcome {
    /// The full state change committed and advanced the generation once.
    Applied { generation: StorageMapGeneration },
    /// The expected generation was stale; nothing was applied or retried.
    Conflict {
        observed_generation: Option<StorageMapGeneration>,
    },
    /// A requested `ReplicaId` is not present in the pre-mutation map.
    Invalid { replica_id: ReplicaId },
    /// The matching request would change state at the maximum generation.
    GenerationExhausted { generation: StorageMapGeneration },
    /// The validated request leaves the logical map unchanged.
    Unchanged { generation: StorageMapGeneration },
    /// The map could not be read or the atomic persistence failed.
    PersistenceFailure(StorageMapPersistenceFailure),
}

/// Applies a batch against one coherent pre-mutation snapshot.
///
/// This function never retries a conflict and never offers Replica addition.
/// All requested IDs are validated before any staged edit is committed.
#[must_use]
pub fn apply_storage_map_mutation(
    persistence: &impl StorageMapPersistence,
    project_id: ProjectId,
    expected_generation: StorageMapGeneration,
    mutation: &StorageMapMutation,
) -> StorageMapMutationOutcome {
    let persisted = match persistence.load(project_id) {
        Ok(persisted) => persisted,
        Err(failure) => return StorageMapMutationOutcome::PersistenceFailure(failure),
    };
    if persisted.generation != expected_generation {
        return StorageMapMutationOutcome::Conflict {
            observed_generation: Some(persisted.generation),
        };
    }

    let Ok(map) = StorageMap::from_persisted_records(persisted.replicas) else {
        return StorageMapMutationOutcome::PersistenceFailure(
            StorageMapPersistenceFailure::InvalidPersistedState,
        );
    };
    let Ok(snapshot) = StorageMapSnapshot::from_persisted(map, persisted.generation) else {
        return StorageMapMutationOutcome::PersistenceFailure(
            StorageMapPersistenceFailure::InvalidPersistedState,
        );
    };

    for replica_id in mutation.changes.keys() {
        if !snapshot.map.replicas.contains_key(replica_id) {
            return StorageMapMutationOutcome::Invalid {
                replica_id: *replica_id,
            };
        }
    }

    let mut proposed = snapshot.map.clone();
    for (replica_id, change) in &mutation.changes {
        match change {
            ReplicaChange::Remove => {
                proposed.replicas.remove(replica_id);
            }
            ReplicaChange::Update {
                locator,
                availability,
            } => {
                let Some(current) = proposed.replicas.get(replica_id) else {
                    return StorageMapMutationOutcome::Invalid {
                        replica_id: *replica_id,
                    };
                };
                let mut updated = current.clone();
                if let Some(locator) = locator {
                    updated = updated.with_locator(locator.clone());
                }
                if let Some(availability) = availability {
                    updated = updated.with_availability(*availability);
                }
                proposed.replicas.insert(*replica_id, updated);
            }
        }
    }

    if proposed == snapshot.map {
        return StorageMapMutationOutcome::Unchanged {
            generation: snapshot.generation,
        };
    }
    let Some(next_generation) = snapshot.generation.next() else {
        return StorageMapMutationOutcome::GenerationExhausted {
            generation: snapshot.generation,
        };
    };
    let replacement = StorageMapSnapshot {
        map: proposed,
        generation: next_generation,
    };
    match persistence.compare_and_swap(project_id, expected_generation, replacement) {
        Ok(ConditionalWriteOutcome::Committed) => StorageMapMutationOutcome::Applied {
            generation: next_generation,
        },
        Ok(ConditionalWriteOutcome::Conflict {
            observed_generation,
        }) => StorageMapMutationOutcome::Conflict {
            observed_generation,
        },
        Err(failure) => StorageMapMutationOutcome::PersistenceFailure(failure),
    }
}
