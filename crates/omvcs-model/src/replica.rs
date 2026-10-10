//! Core-owned operational Resource Replica representations.
//!
//! These values describe complete physical representations and their
//! operational locations. They do not represent verification evidence or
//! provide a Replica-registration operation.

use std::fmt;

use crate::canonical::UniqueJsonValue;
use crate::resource::ResourceByteLength;
use crate::verification::PromotionEligibility;
use crate::{ChunkId, ReplicaId, ResourceId, StorageEndpointId};
use serde::de::{Error as _, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

/// The opaque, versioned provider-owned locator envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderLocator {
    schema: String,
    value: Value,
}

/// An invalid generic `ProviderLocator` envelope or canonical JSON value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderLocatorError {
    /// A locator schema identifier must not be empty.
    EmptySchema,
    /// The value cannot be represented as RFC 8785 canonical JSON.
    InvalidCanonicalJson,
}

impl fmt::Display for ProviderLocatorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySchema => formatter.write_str("ProviderLocator schema must not be empty"),
            Self::InvalidCanonicalJson => {
                formatter.write_str("ProviderLocator value is not valid canonical JSON data")
            }
        }
    }
}

impl std::error::Error for ProviderLocatorError {}

impl ProviderLocator {
    /// Creates an opaque locator envelope after checking that the schema
    /// identifier is non-empty and the value is JCS-representable.
    ///
    /// The schema identifier is preserved exactly; its versioned lexical
    /// grammar is not interpreted here.
    ///
    /// The named provider schema owns the meaning and semantic validation of
    /// `value`; this method does not interpret provider-specific fields.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderLocatorError::EmptySchema`] for an empty schema or
    /// [`ProviderLocatorError::InvalidCanonicalJson`] when JCS serialization
    /// fails.
    pub fn new(schema: impl Into<String>, value: Value) -> Result<Self, ProviderLocatorError> {
        let schema = schema.into();
        if schema.is_empty() {
            return Err(ProviderLocatorError::EmptySchema);
        }
        serde_jcs::to_vec(&value).map_err(|_| ProviderLocatorError::InvalidCanonicalJson)?;
        Ok(Self { schema, value })
    }

    /// Returns the exact versioned schema identifier.
    #[must_use]
    pub fn schema(&self) -> &str {
        &self.schema
    }

    /// Returns the provider-owned value without interpreting it.
    #[must_use]
    pub const fn value(&self) -> &Value {
        &self.value
    }

    /// Serializes the complete envelope in RFC 8785 canonical JSON form.
    ///
    /// # Errors
    ///
    /// Returns an error only if the value cannot be serialized under JCS.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, ProviderLocatorError> {
        serde_jcs::to_vec(self).map_err(|_| ProviderLocatorError::InvalidCanonicalJson)
    }
}

impl Serialize for ProviderLocator {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Wire<'a> {
            schema: &'a str,
            value: &'a Value,
        }

        Wire {
            schema: &self.schema,
            value: &self.value,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ProviderLocator {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            schema: String,
            value: UniqueJsonValue,
        }

        let wire = Wire::deserialize(deserializer)?;
        Self::new(wire.schema, wire.value.0).map_err(D::Error::custom)
    }
}

/// A Chunk's exact position and size in a reconstruction manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkManifestEntry {
    chunk_id: ChunkId,
    offset: ResourceByteLength,
    length: ResourceByteLength,
}

impl ChunkManifestEntry {
    /// Creates one manifest element.
    #[must_use]
    pub const fn new(
        chunk_id: ChunkId,
        offset: ResourceByteLength,
        length: ResourceByteLength,
    ) -> Self {
        Self {
            chunk_id,
            offset,
            length,
        }
    }

    /// Returns the content identity of this Chunk.
    #[must_use]
    pub const fn chunk_id(self) -> ChunkId {
        self.chunk_id
    }

    /// Returns the byte offset in the complete Resource.
    #[must_use]
    pub const fn offset(self) -> ResourceByteLength {
        self.offset
    }

    /// Returns the exact number of bytes in this Chunk.
    #[must_use]
    pub const fn length(self) -> ResourceByteLength {
        self.length
    }
}

/// One ordered, complete physical reconstruction description for a Resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkManifest {
    resource_id: ResourceId,
    total_length: ResourceByteLength,
    chunks: Vec<ChunkManifestEntry>,
}

/// A Chunk Manifest whose ordering, offsets, or total length is inconsistent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkManifestError {
    /// An entry does not start at the cumulative length of preceding entries.
    InvalidOffset,
    /// The manifest's total length does not equal the sum of its entries.
    TotalLengthMismatch,
    /// The sum exceeds the exact Resource byte-length domain.
    LengthOutOfRange,
    /// The ordered Chunk lengths do not obey the OMVCS 0.1 fixed-size policy.
    InvalidChunkingPolicy,
}

impl fmt::Display for ChunkManifestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOffset => formatter
                .write_str("Chunk Manifest offsets must match the ordered reconstruction sequence"),
            Self::TotalLengthMismatch => {
                formatter.write_str("Chunk Manifest total length does not match its entries")
            }
            Self::LengthOutOfRange => {
                formatter.write_str("Chunk Manifest length exceeds the exact integer domain")
            }
            Self::InvalidChunkingPolicy => formatter.write_str(
                "Chunk Manifest does not obey the OMVCS 0.1 fixed-size sequential policy",
            ),
        }
    }
}

impl std::error::Error for ChunkManifestError {}

impl ChunkManifest {
    /// Creates and validates an ordered Chunk Manifest.
    ///
    /// Offsets must be contiguous in sequence order and the sum of the
    /// element lengths must equal `total_length`. Chunk lengths also conform
    /// to the exact OMVCS 0.1 fixed-size sequential policy.
    ///
    /// # Errors
    ///
    /// Returns [`ChunkManifestError`] if offsets are not contiguous, the
    /// total length differs from the sum, or the sum exceeds the exact
    /// Resource byte-length domain, or
    /// [`ChunkManifestError::InvalidChunkingPolicy`] when the ordered lengths
    /// do not conform to the OMVCS 0.1 policy.
    pub fn new(
        resource_id: ResourceId,
        total_length: ResourceByteLength,
        chunks: Vec<ChunkManifestEntry>,
    ) -> Result<Self, ChunkManifestError> {
        let mut next_offset = 0_u64;
        for entry in &chunks {
            if entry.offset.get() != next_offset {
                return Err(ChunkManifestError::InvalidOffset);
            }
            next_offset = next_offset
                .checked_add(entry.length.get())
                .ok_or(ChunkManifestError::LengthOutOfRange)?;
            if ResourceByteLength::new(next_offset).is_err() {
                return Err(ChunkManifestError::LengthOutOfRange);
            }
        }
        if next_offset != total_length.get() {
            return Err(ChunkManifestError::TotalLengthMismatch);
        }
        if chunks.is_empty() {
            return Err(ChunkManifestError::InvalidChunkingPolicy);
        }
        if total_length.get() == 0 {
            if chunks.len() != 1 || chunks[0].length.get() != 0 {
                return Err(ChunkManifestError::InvalidChunkingPolicy);
            }
        } else if chunks.iter().enumerate().any(|(index, entry)| {
            if index + 1 < chunks.len() {
                entry.length.get() != 8_388_608
            } else {
                entry.length.get() == 0 || entry.length.get() > 8_388_608
            }
        }) {
            return Err(ChunkManifestError::InvalidChunkingPolicy);
        }
        Ok(Self {
            resource_id,
            total_length,
            chunks,
        })
    }

    /// Returns the Resource reconstructed by this manifest.
    #[must_use]
    pub const fn resource_id(&self) -> ResourceId {
        self.resource_id
    }

    /// Returns the complete Resource length.
    #[must_use]
    pub const fn total_length(&self) -> ResourceByteLength {
        self.total_length
    }

    /// Returns the Chunk entries in reconstruction order.
    #[must_use]
    pub fn chunks(&self) -> &[ChunkManifestEntry] {
        &self.chunks
    }

    /// Returns whether each manifest entry's corresponding representation is
    /// currently available.
    ///
    /// `chunk_availability` MUST follow manifest order and contain exactly
    /// one state per entry. Missing, extra, unknown, or unavailable states do
    /// not establish full availability.
    #[must_use]
    pub fn is_fully_available(&self, chunk_availability: &[ReplicaAvailability]) -> bool {
        chunk_availability.len() == self.chunks.len()
            && chunk_availability
                .iter()
                .all(|state| *state == ReplicaAvailability::Available)
    }
}

impl Serialize for ChunkManifestEntry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Wire {
            chunk_id: String,
            offset: ResourceByteLength,
            length: ResourceByteLength,
        }

        Wire {
            chunk_id: self.chunk_id.to_string(),
            offset: self.offset,
            length: self.length,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ChunkManifestEntry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            chunk_id: String,
            offset: ResourceByteLength,
            length: ResourceByteLength,
        }

        let wire = Wire::deserialize(deserializer)?;
        Ok(Self {
            chunk_id: wire.chunk_id.parse().map_err(D::Error::custom)?,
            offset: wire.offset,
            length: wire.length,
        })
    }
}

impl Serialize for ChunkManifest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Wire<'a> {
            resource_id: String,
            total_length: ResourceByteLength,
            chunks: &'a [ChunkManifestEntry],
        }

        Wire {
            resource_id: self.resource_id.to_string(),
            total_length: self.total_length,
            chunks: &self.chunks,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ChunkManifest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            resource_id: String,
            total_length: ResourceByteLength,
            chunks: Vec<ChunkManifestEntry>,
        }

        let wire = Wire::deserialize(deserializer)?;
        let resource_id = wire.resource_id.parse().map_err(D::Error::custom)?;
        Self::new(resource_id, wire.total_length, wire.chunks).map_err(D::Error::custom)
    }
}

/// A Resource's complete-object or ordered chunked physical representation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ReplicaRepresentation {
    /// The provider object contains the complete Resource bytes.
    CompleteObject,
    /// The ordered Chunk Manifest reconstructs the complete Resource.
    Chunked {
        /// The ordered operational reconstruction manifest.
        manifest: ChunkManifest,
    },
}

impl<'de> Deserialize<'de> for ReplicaRepresentation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RepresentationVisitor;

        impl<'de> Visitor<'de> for RepresentationVisitor {
            type Value = ReplicaRepresentation;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a closed Resource representation object")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut kind: Option<String> = None;
                let mut manifest: Option<Option<ChunkManifest>> = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "kind" => {
                            if kind.is_some() {
                                return Err(M::Error::duplicate_field("kind"));
                            }
                            kind = Some(map.next_value()?);
                        }
                        "manifest" => {
                            if manifest.is_some() {
                                return Err(M::Error::duplicate_field("manifest"));
                            }
                            manifest = Some(map.next_value()?);
                        }
                        _ => return Err(M::Error::unknown_field(&key, &["kind", "manifest"])),
                    }
                }
                match kind.as_deref() {
                    Some("complete-object") if manifest.is_none() => {
                        Ok(ReplicaRepresentation::CompleteObject)
                    }
                    Some("chunked") => manifest
                        .flatten()
                        .map(|manifest| ReplicaRepresentation::Chunked { manifest })
                        .ok_or_else(|| M::Error::missing_field("manifest")),
                    Some("complete-object") => Err(M::Error::custom(
                        "complete-object representation has no manifest",
                    )),
                    Some(_) => Err(M::Error::custom("unknown Resource representation kind")),
                    None => Err(M::Error::missing_field("kind")),
                }
            }
        }

        deserializer.deserialize_map(RepresentationVisitor)
    }
}

/// Retrieval state of one known Replica, separate from integrity verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplicaAvailability {
    /// Retrieval of the complete representation has been established.
    ///
    /// For a chunked representation, every required Chunk representation
    /// must be available.
    Available,
    /// The complete representation cannot currently be retrieved.
    TemporarilyUnavailable,
    /// Availability has not been established.
    Unknown,
}

/// An operational record for one complete Resource representation.
///
/// A `Replica` represents an already-registered record reconstituted from
/// authoritative operational persistence. Decoding or constructing this
/// value alone does not register it or establish its persistence authority.
/// New candidates can become registered records only through [`Self`]'s
/// promotion gate with destination-applicable `resource_identity` assurance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Replica {
    id: ReplicaId,
    resource_id: ResourceId,
    endpoint_id: StorageEndpointId,
    representation: ReplicaRepresentation,
    locator: ProviderLocator,
    availability: ReplicaAvailability,
}

/// An incomplete or not-yet-verified representation candidate.
///
/// Candidates have no `ReplicaId` and cannot be inserted into a Storage Map.
///
/// ```compile_fail
/// use omvcs_model::replica::{ProviderLocator, ReplicaCandidate};
/// use omvcs_model::{ChunkId, ProjectId, ResourceId, ReplicaId, StorageEndpointId};
/// use omvcs_core::storage_map::StorageMap;
/// use serde_json::json;
///
/// let candidate = ReplicaCandidate::new(
///     ResourceId::from_digest([0; 32]),
///     StorageEndpointId::new(),
///     omvcs_model::replica::ReplicaRepresentation::CompleteObject,
///     ProviderLocator::new("provider.locator/1", json!({})).unwrap(),
/// ).unwrap();
/// let _ = StorageMap::from_persisted_records([candidate]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplicaCandidate {
    resource_id: ResourceId,
    endpoint_id: StorageEndpointId,
    representation: ReplicaRepresentation,
    locator: ProviderLocator,
}

/// Invalid binding between a Replica and its physical representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplicaError {
    /// A chunked manifest describes a different Resource than the Replica.
    ManifestResourceMismatch,
    /// The destination Resource assurance does not apply to this candidate.
    PromotionAssuranceMismatch,
}

impl fmt::Display for ReplicaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ManifestResourceMismatch => {
                formatter.write_str("Chunk Manifest ResourceId differs from Replica ResourceId")
            }
            Self::PromotionAssuranceMismatch => {
                formatter.write_str("Resource assurance does not apply to this Replica candidate")
            }
        }
    }
}

impl std::error::Error for ReplicaError {}

impl ReplicaCandidate {
    /// Creates a candidate without assigning a registered `ReplicaId`.
    ///
    /// No verification or registration is implied.
    ///
    /// # Errors
    ///
    /// Returns [`ReplicaError::ManifestResourceMismatch`] if a chunked
    /// representation is bound to a different `ResourceId`.
    pub fn new(
        resource_id: ResourceId,
        endpoint_id: StorageEndpointId,
        representation: ReplicaRepresentation,
        locator: ProviderLocator,
    ) -> Result<Self, ReplicaError> {
        validate_representation(resource_id, &representation)?;
        Ok(Self {
            resource_id,
            endpoint_id,
            representation,
            locator,
        })
    }

    /// Returns the Resource identity described by this candidate.
    #[must_use]
    pub const fn resource_id(&self) -> ResourceId {
        self.resource_id
    }

    /// Returns the target Storage Endpoint.
    #[must_use]
    pub const fn endpoint_id(&self) -> StorageEndpointId {
        self.endpoint_id
    }

    /// Returns the candidate's complete representation description.
    #[must_use]
    pub const fn representation(&self) -> &ReplicaRepresentation {
        &self.representation
    }

    /// Returns the opaque provider locator.
    #[must_use]
    pub const fn locator(&self) -> &ProviderLocator {
        &self.locator
    }

    /// Promotes this candidate only with destination-applicable Resource
    /// identity assurance. The caller supplies observed retrieval state;
    /// verification itself does not define availability policy.
    ///
    /// # Errors
    ///
    /// Returns [`ReplicaError::PromotionAssuranceMismatch`] unless the opaque
    /// eligibility result applies to this exact candidate.
    pub fn promote(
        self,
        eligibility: &PromotionEligibility,
        availability: ReplicaAvailability,
    ) -> Result<Replica, ReplicaError> {
        if !eligibility.applies_to(&self) {
            return Err(ReplicaError::PromotionAssuranceMismatch);
        }
        Ok(Replica {
            id: ReplicaId::new(),
            resource_id: self.resource_id,
            endpoint_id: self.endpoint_id,
            representation: self.representation,
            locator: self.locator,
            availability,
        })
    }
}

fn validate_representation(
    resource_id: ResourceId,
    representation: &ReplicaRepresentation,
) -> Result<(), ReplicaError> {
    if let ReplicaRepresentation::Chunked { manifest } = representation
        && manifest.resource_id() != resource_id
    {
        return Err(ReplicaError::ManifestResourceMismatch);
    }
    Ok(())
}

impl Replica {
    /// Reconstitutes one already-registered persisted record.
    ///
    /// # Errors
    ///
    /// Returns [`ReplicaError::ManifestResourceMismatch`] if a chunked
    /// representation is bound to a different `ResourceId`.
    fn with_id(
        id: ReplicaId,
        resource_id: ResourceId,
        endpoint_id: StorageEndpointId,
        representation: ReplicaRepresentation,
        locator: ProviderLocator,
        availability: ReplicaAvailability,
    ) -> Result<Self, ReplicaError> {
        validate_representation(resource_id, &representation)?;
        Ok(Self {
            id,
            resource_id,
            endpoint_id,
            representation,
            locator,
            availability,
        })
    }

    /// Returns the stable assigned identifier of this operational record.
    #[must_use]
    pub const fn id(&self) -> ReplicaId {
        self.id
    }

    /// Returns the immutable Resource identity represented by this record.
    #[must_use]
    pub const fn resource_id(&self) -> ResourceId {
        self.resource_id
    }

    /// Returns the associated Storage Endpoint.
    #[must_use]
    pub const fn endpoint_id(&self) -> StorageEndpointId {
        self.endpoint_id
    }

    /// Returns the complete physical representation binding.
    #[must_use]
    pub const fn representation(&self) -> &ReplicaRepresentation {
        &self.representation
    }

    /// Returns the opaque provider locator.
    #[must_use]
    pub const fn locator(&self) -> &ProviderLocator {
        &self.locator
    }

    /// Returns retrieval state, not verification or integrity evidence.
    #[must_use]
    pub const fn availability(&self) -> ReplicaAvailability {
        self.availability
    }

    /// Returns this record with an updated locator and unchanged identity.
    #[must_use]
    pub fn with_locator(&self, locator: ProviderLocator) -> Self {
        let mut updated = self.clone();
        updated.locator = locator;
        updated
    }

    /// Returns this record with updated retrieval state and unchanged identity.
    #[must_use]
    pub fn with_availability(&self, availability: ReplicaAvailability) -> Self {
        let mut updated = self.clone();
        updated.availability = availability;
        updated
    }

    /// Serializes this persisted record as RFC 8785/JCS canonical JSON.
    ///
    /// Persistence implementations MUST use this representation instead of
    /// relying on the ordering or number formatting of a generic serializer.
    ///
    /// # Errors
    ///
    /// Returns the JCS serializer error if the record cannot be represented
    /// as canonical JSON.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_jcs::to_vec(self)
    }
}

impl Serialize for Replica {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Wire<'a> {
            replica_id: String,
            resource_id: String,
            endpoint_id: String,
            representation: &'a ReplicaRepresentation,
            locator: &'a ProviderLocator,
            availability: ReplicaAvailability,
        }

        Wire {
            replica_id: self.id.to_string(),
            resource_id: self.resource_id.to_string(),
            endpoint_id: self.endpoint_id.to_string(),
            representation: &self.representation,
            locator: &self.locator,
            availability: self.availability,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Replica {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            replica_id: String,
            resource_id: String,
            endpoint_id: String,
            representation: ReplicaRepresentation,
            locator: ProviderLocator,
            availability: ReplicaAvailability,
        }

        let wire = Wire::deserialize(deserializer)?;
        Self::with_id(
            wire.replica_id.parse().map_err(D::Error::custom)?,
            wire.resource_id.parse().map_err(D::Error::custom)?,
            wire.endpoint_id.parse().map_err(D::Error::custom)?,
            wire.representation,
            wire.locator,
            wire.availability,
        )
        .map_err(D::Error::custom)
    }
}
