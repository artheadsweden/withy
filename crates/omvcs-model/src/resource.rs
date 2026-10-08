//! Logical Resource references and immutable Resource Object bytes.
//!
//! Resource identity is always derived from complete raw bytes. Descriptive
//! reference fields belong to a containing historical object's canonical
//! body, not to the Resource Identifier. Physical storage and reconstruction
//! information is intentionally outside these types.

use std::collections::BTreeMap;
use std::fmt;

use serde::de::{Error as _, MapAccess, Visitor, value::MapAccessDeserializer};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use serde_json::value::RawValue;

use crate::ResourceId;
use crate::canonical::{CanonicalMetadataError, MetadataSchema, canonicalize_metadata_body};
use crate::hashing::hash_resource_bytes;

const MAX_BYTE_LENGTH_DECIMAL: &str = "9007199254740991";

/// The largest `ResourceReference.byte_length` allowed by OMVCS 0.1.
pub const MAX_RESOURCE_BYTE_LENGTH: u64 = 9_007_199_254_740_991;

/// An invalid value for an OMVCS 0.1 Resource byte length.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceByteLengthError {
    /// The value is negative.
    Negative,
    /// The value is not mathematically an integer or is not a JSON number.
    NotAnInteger,
    /// The value exceeds the OMVCS 0.1 maximum.
    OutOfRange,
}

impl fmt::Display for ResourceByteLengthError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Negative => formatter.write_str("Resource byte length cannot be negative"),
            Self::NotAnInteger => formatter
                .write_str("Resource byte length must be a JSON number with an integer value"),
            Self::OutOfRange => {
                formatter.write_str("Resource byte length must be in 0..=9007199254740991")
            }
        }
    }
}

impl std::error::Error for ResourceByteLengthError {}

/// A Resource byte count constrained to the OMVCS 0.1 interoperable range.
///
/// The private representation prevents callers from constructing a value
/// outside `0..=9007199254740991`, regardless of host integer capacity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ResourceByteLength(u64);

impl ResourceByteLength {
    /// The maximum byte count for one Resource in OMVCS 0.1.
    pub const MAX: Self = Self(MAX_RESOURCE_BYTE_LENGTH);

    /// Constructs a Resource byte count if it is within the OMVCS 0.1 range.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceByteLengthError::OutOfRange`] above [`Self::MAX`].
    pub const fn new(value: u64) -> Result<Self, ResourceByteLengthError> {
        if value <= MAX_RESOURCE_BYTE_LENGTH {
            Ok(Self(value))
        } else {
            Err(ResourceByteLengthError::OutOfRange)
        }
    }

    /// Returns the exact, already-bounded byte count.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl Serialize for ResourceByteLength {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(self.0)
    }
}

impl<'de> Deserialize<'de> for ResourceByteLength {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = Box::<RawValue>::deserialize(deserializer)?;
        parse_json_byte_length(raw.get()).map_err(D::Error::custom)
    }
}

/// An immutable content-addressed Resource Object.
///
/// The owned byte buffer is private and exposed only through a shared slice.
/// Its identifier is calculated from those complete bytes and cannot be
/// influenced by names, descriptive fields, or physical storage.
#[derive(Clone, PartialEq, Eq)]
pub struct ResourceObject {
    bytes: Box<[u8]>,
    resource_id: ResourceId,
}

impl ResourceObject {
    /// Creates an immutable Resource Object from its complete raw bytes.
    #[must_use]
    pub fn new(bytes: impl Into<Box<[u8]>>) -> Self {
        let bytes = bytes.into();
        let resource_id = hash_resource_bytes(&bytes);
        Self { bytes, resource_id }
    }

    /// Returns the complete raw Resource bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the identifier derived only from the complete raw bytes.
    #[must_use]
    pub const fn resource_id(&self) -> ResourceId {
        self.resource_id
    }

    /// Returns the exact byte count if it is representable by OMVCS 0.1.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceByteLengthError::OutOfRange`] when the complete
    /// Resource exceeds the OMVCS 0.1 per-Resource limit.
    pub fn byte_length(&self) -> Result<ResourceByteLength, ResourceByteLengthError> {
        let length =
            u64::try_from(self.bytes.len()).map_err(|_| ResourceByteLengthError::OutOfRange)?;
        ResourceByteLength::new(length)
    }

    /// Creates a required Resource Reference whose length matches these bytes.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceByteLengthError::OutOfRange`] if the complete
    /// Resource exceeds the OMVCS 0.1 per-Resource limit.
    pub fn reference(&self) -> Result<ResourceReference, ResourceByteLengthError> {
        Ok(ResourceReference::new(
            self.resource_id,
            self.byte_length()?,
        ))
    }
}

/// An explicitly unchecked Resource Reference candidate.
///
/// Only the generic fields approved for OMVCS 0.1 are represented. Unknown
/// fields (including names and physical storage/reconstruction fields) are
/// rejected during deserialization. Serialization supports preservation only;
/// it does not establish historical validity. Use [`Self::admit`] before
/// historical use, even when the property map is empty.
///
/// ```compile_fail
/// use omvcs_model::resource::ResourceReferenceCandidate;
/// fn historical_bytes(candidate: &ResourceReferenceCandidate) {
///     candidate.canonical_bytes(None);
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResourceReferenceCandidate {
    #[serde(with = "resource_id_serde")]
    resource_id: ResourceId,
    byte_length: ResourceByteLength,
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    media_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    properties: Option<BTreeMap<String, Value>>,
}

impl ResourceReferenceCandidate {
    /// Creates an unchecked candidate with its required typed identity and length.
    #[must_use]
    pub const fn new(resource_id: ResourceId, byte_length: ResourceByteLength) -> Self {
        Self {
            resource_id,
            byte_length,
            role: None,
            media_type: None,
            properties: None,
        }
    }

    /// Returns the typed Resource Identifier.
    #[must_use]
    pub const fn resource_id(&self) -> ResourceId {
        self.resource_id
    }

    /// Returns the required, bounded byte count for the complete Resource.
    #[must_use]
    pub const fn byte_length(&self) -> ResourceByteLength {
        self.byte_length
    }

    /// Checks that a retrieved immutable Resource Object matches this reference.
    ///
    /// This verifies both the raw-byte-derived identifier and the exact
    /// complete-resource byte count. It does not require the Resource to be
    /// available when a historical reference is decoded.
    #[must_use]
    pub fn matches_resource(&self, resource: &ResourceObject) -> bool {
        self.resource_id == resource.resource_id && resource.byte_length() == Ok(self.byte_length)
    }

    /// Returns the optional semantic role.
    #[must_use]
    pub fn role(&self) -> Option<&str> {
        self.role.as_deref()
    }

    /// Returns the optional intended media/content type.
    #[must_use]
    pub fn media_type(&self) -> Option<&str> {
        self.media_type.as_deref()
    }

    /// Returns the optional immutable interpretation properties map.
    #[must_use]
    pub const fn properties(&self) -> Option<&BTreeMap<String, Value>> {
        self.properties.as_ref()
    }

    /// Returns a new reference carrying the supplied semantic role.
    #[must_use]
    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.role = Some(role.into());
        self
    }

    /// Returns a new reference carrying the supplied intended media type.
    #[must_use]
    pub fn with_media_type(mut self, media_type: impl Into<String>) -> Self {
        self.media_type = Some(media_type.into());
        self
    }

    /// Returns an unchecked candidate carrying properties, without granting admission.
    #[must_use]
    pub fn with_properties(mut self, properties: BTreeMap<String, Value>) -> Self {
        self.properties = Some(properties);
        self
    }
}

impl<'de> Deserialize<'de> for ResourceReferenceCandidate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ReferenceVisitor;

        impl<'de> Visitor<'de> for ReferenceVisitor {
            type Value = ResourceReferenceWire;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a Resource Reference object with named fields")
            }

            fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                ResourceReferenceWire::deserialize(MapAccessDeserializer::new(map))
            }
        }

        let raw = Box::<RawValue>::deserialize(deserializer)?;
        crate::canonical::validate_unique_json_member_names(raw.get().as_bytes())
            .map_err(D::Error::custom)?;
        // Core §7 and ADR-0007 require an object. Only map access reaches
        // the derived fields; positional structs and enum forms cannot enter.
        let wire = serde_json::Deserializer::from_str(raw.get())
            .deserialize_map(ReferenceVisitor)
            .map_err(D::Error::custom)?;
        Ok(Self {
            resource_id: wire.resource_id,
            byte_length: wire.byte_length,
            role: wire.role,
            media_type: wire.media_type,
            properties: wire.properties,
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ResourceReferenceWire {
    #[serde(with = "resource_id_serde")]
    resource_id: ResourceId,
    byte_length: ResourceByteLength,
    #[serde(default, deserialize_with = "present_string")]
    role: Option<String>,
    #[serde(default, deserialize_with = "present_string")]
    media_type: Option<String>,
    #[serde(default, deserialize_with = "present_properties")]
    properties: Option<BTreeMap<String, Value>>,
}

fn present_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    // A present field must not silently disappear from the historical body.
    String::deserialize(deserializer).map(Some)
}

fn present_properties<'de, D>(deserializer: D) -> Result<Option<BTreeMap<String, Value>>, D::Error>
where
    D: Deserializer<'de>,
{
    // A present null is not an absent property map.
    BTreeMap::deserialize(deserializer).map(Some)
}

/// Operational context determined by the containing versioned schema contract.
///
/// This is not a Resource Reference field or an independent property-schema ID.
/// When an Adapter supplies the authority, its exact identity and state schema
/// version are included. Callers must derive this binding from the containing
/// schema/Adapter contract, never guess it or substitute a latest version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceValidationContext {
    /// Exact versioned schema of the containing historical use.
    pub containing_schema: String,
    /// Exact Adapter identity and versioned state schema, when applicable.
    pub adapter: Option<(String, String)>,
}

/// The applicable schema/Adapter's property authority, not a Core vocabulary.
///
/// The containing contract must bind to one exact authority/version. Core
/// selects only an exact context match and rejects multiple matches. Implementors
/// own meanings and must reject all ADR-0007-excluded semantics, including
/// alternate keys and nested values. The same authority supplies the complete
/// value-shape and nested collection schema and validates normalized properties.
pub trait ResourcePropertiesValidator {
    /// Returns the exact containing schema/Adapter binding served by this authority.
    fn context(&self) -> &ResourceValidationContext;
    /// Describes the property object and all recursive value/collection shapes.
    fn properties_schema(&self) -> MetadataSchema;
    /// Validates semantic admissibility under this exact context.
    ///
    /// # Errors
    ///
    /// Returns the authority's rejection reason; Core propagates it unchanged.
    fn validate_properties(&self, properties: &BTreeMap<String, Value>) -> Result<(), String>;
}

/// A failure to admit a candidate; the candidate remains unchecked and preservable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceAdmissionError {
    /// The containing schema/Adapter contract has no known exact binding.
    UnknownContext,
    /// No available validator matches the exact required context.
    UnavailableContext,
    /// More than one authority matches the required context.
    NonUniqueAuthority,
    /// An admitted reference is being used under a different containing context.
    ContextMismatch,
    /// Generic shape or canonical validation failed.
    Canonical(CanonicalMetadataError),
    /// The exact semantic authority rejected the properties.
    SemanticRejection(String),
}

impl fmt::Display for ResourceAdmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownContext => formatter.write_str("Resource properties context is unknown"),
            Self::UnavailableContext => {
                formatter.write_str("exact Resource properties authority is unavailable")
            }
            Self::NonUniqueAuthority => {
                formatter.write_str("Resource properties authority is not unique")
            }
            Self::ContextMismatch => formatter.write_str("Resource historical use context differs"),
            Self::Canonical(error) => error.fmt(formatter),
            Self::SemanticRejection(reason) => {
                write!(formatter, "Resource properties rejected: {reason}")
            }
        }
    }
}

impl std::error::Error for ResourceAdmissionError {}

impl From<CanonicalMetadataError> for ResourceAdmissionError {
    fn from(error: CanonicalMetadataError) -> Self {
        Self::Canonical(error)
    }
}

impl ResourceReferenceCandidate {
    /// Validates for historical admission, retaining this unchecked candidate.
    ///
    /// `context` is the exact binding from the containing schema/Adapter contract;
    /// `None` represents unknown or ambiguous binding. Only one matching authority
    /// is accepted. Without properties, generic Core validation alone suffices.
    ///
    /// # Errors
    ///
    /// Unknown, unavailable, or non-unique authority, shape/canonical failure,
    /// or semantic rejection cannot produce a validated reference.
    pub fn admit(
        &self,
        context: Option<&ResourceValidationContext>,
        validators: &[&dyn ResourcePropertiesValidator],
    ) -> Result<ResourceReference, ResourceAdmissionError> {
        let Some(_) = self.properties else {
            return Ok(ResourceReference {
                data: self.clone(),
                context: None,
            });
        };
        let context = context.ok_or(ResourceAdmissionError::UnknownContext)?;
        let mut applicable = validators
            .iter()
            .filter(|validator| validator.context() == context);
        let validator = applicable
            .next()
            .ok_or(ResourceAdmissionError::UnavailableContext)?;
        if applicable.next().is_some() {
            return Err(ResourceAdmissionError::NonUniqueAuthority);
        }

        let body =
            serde_json::to_vec(self).map_err(|_| CanonicalMetadataError::Canonicalization)?;
        let canonical =
            canonicalize_metadata_body(&body, &reference_schema(validator.properties_schema()))?;
        let normalized: Self =
            serde_json::from_slice(&canonical).map_err(|_| CanonicalMetadataError::InvalidJson)?;
        let properties = normalized
            .properties
            .as_ref()
            .ok_or(CanonicalMetadataError::InvalidJson)?;
        validator
            .validate_properties(properties)
            .map_err(ResourceAdmissionError::SemanticRejection)?;
        Ok(ResourceReference {
            data: normalized,
            context: Some(context.clone()),
        })
    }
}

/// An immutable Resource Reference validated for historical admission.
///
/// It cannot be decoded or constructed directly from unrestricted properties.
/// Historical bytes/values are available only through explicitly fallible,
/// context-checked methods, not unconditional `Serialize`. Validation context
/// is operational evidence and is never added to historical fields.
///
/// ```compile_fail
/// use omvcs_model::resource::ResourceReferenceCandidate;
/// fn commit_reference(_: &omvcs_model::resource::ResourceReference) {}
/// let candidate: ResourceReferenceCandidate = todo!();
/// commit_reference(&candidate);
/// ```
///
/// Unconditional historical serialization and direct validated decoding are
/// intentionally unavailable; both would bypass the containing-use binding.
///
/// ```compile_fail
/// use omvcs_model::resource::ResourceReference;
/// fn unchecked_serialization(reference: &ResourceReference) {
///     serde_json::to_vec(reference);
/// }
/// ```
///
/// ```compile_fail
/// use omvcs_model::resource::ResourceReference;
/// let reference = serde_json::from_str::<ResourceReference>("{}");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceReference {
    data: ResourceReferenceCandidate,
    context: Option<ResourceValidationContext>,
}

impl ResourceReference {
    /// Creates a generically valid reference with no properties.
    #[must_use]
    pub const fn new(resource_id: ResourceId, byte_length: ResourceByteLength) -> Self {
        Self {
            data: ResourceReferenceCandidate::new(resource_id, byte_length),
            context: None,
        }
    }

    /// Returns the immutable Resource identity.
    #[must_use]
    pub const fn resource_id(&self) -> ResourceId {
        self.data.resource_id()
    }

    /// Returns the bounded complete Resource byte count.
    #[must_use]
    pub const fn byte_length(&self) -> ResourceByteLength {
        self.data.byte_length()
    }

    /// Checks the identity and complete byte count of retrieved content.
    #[must_use]
    pub fn matches_resource(&self, resource: &ResourceObject) -> bool {
        self.data.matches_resource(resource)
    }

    /// Returns the semantic role, if present.
    #[must_use]
    pub fn role(&self) -> Option<&str> {
        self.data.role()
    }

    /// Returns the media type, if present.
    #[must_use]
    pub fn media_type(&self) -> Option<&str> {
        self.data.media_type()
    }

    /// Returns shared, immutable access to the validated property map.
    #[must_use]
    pub const fn properties(&self) -> Option<&BTreeMap<String, Value>> {
        self.data.properties()
    }

    /// Returns a new reference with the supplied semantic role.
    #[must_use]
    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.data = self.data.with_role(role);
        self
    }

    /// Returns a new reference with the supplied media type.
    #[must_use]
    pub fn with_media_type(mut self, media_type: impl Into<String>) -> Self {
        self.data = self.data.with_media_type(media_type);
        self
    }

    /// Adding/replacing properties always returns an unchecked candidate.
    #[must_use]
    pub fn with_properties(
        self,
        properties: BTreeMap<String, Value>,
    ) -> ResourceReferenceCandidate {
        self.data.with_properties(properties)
    }

    /// Returns the historical fields for embedding under the exact containing use.
    ///
    /// # Errors
    ///
    /// Property-bearing references reject missing or different context. Operational
    /// context/evidence is never serialized into this historical value.
    pub fn historical_value(
        &self,
        context: Option<&ResourceValidationContext>,
    ) -> Result<Value, ResourceAdmissionError> {
        if self.context.is_some() && self.context.as_ref() != context {
            return Err(ResourceAdmissionError::ContextMismatch);
        }
        serde_json::to_value(&self.data)
            .map_err(|_| CanonicalMetadataError::Canonicalization.into())
    }

    /// Returns canonical historical reference bytes for the exact containing use.
    ///
    /// Nested collections were normalized during admission using the authority's
    /// schema; immutable access prevents subsequent changes to those properties.
    ///
    /// # Errors
    ///
    /// Returns the same context errors as [`Self::historical_value`], or a
    /// canonical serialization failure. This embedded reference has no separate ID.
    pub fn canonical_bytes(
        &self,
        context: Option<&ResourceValidationContext>,
    ) -> Result<Vec<u8>, ResourceAdmissionError> {
        serde_jcs::to_vec(&self.historical_value(context)?)
            .map_err(|_| CanonicalMetadataError::Canonicalization.into())
    }
}

fn reference_schema(properties: MetadataSchema) -> MetadataSchema {
    MetadataSchema::structure([
        ("resource_id", MetadataSchema::Scalar),
        ("byte_length", MetadataSchema::Scalar),
        ("role", MetadataSchema::Scalar),
        ("media_type", MetadataSchema::Scalar),
        ("properties", properties),
    ])
}

mod resource_id_serde {
    use serde::de::Error as _;
    use serde::{Deserialize, Deserializer, Serializer};

    use crate::ResourceId;

    pub(super) fn serialize<S>(resource_id: &ResourceId, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&resource_id.to_string())
    }

    pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<ResourceId, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(D::Error::custom)
    }
}

fn parse_json_byte_length(raw: &str) -> Result<ResourceByteLength, ResourceByteLengthError> {
    let exponent_index = raw.find('e').or_else(|| raw.find('E'));
    let (mantissa, exponent) =
        exponent_index.map_or((raw, "0"), |index| (&raw[..index], &raw[index + 1..]));
    let (negative, unsigned_mantissa) = mantissa
        .strip_prefix('-')
        .map_or((false, mantissa), |unsigned| (true, unsigned));
    let (integer, fraction) = match unsigned_mantissa.split_once('.') {
        Some((integer, fraction)) => (integer, fraction),
        None => (unsigned_mantissa, ""),
    };
    if !integer.bytes().all(|byte| byte.is_ascii_digit())
        || !fraction.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(ResourceByteLengthError::NotAnInteger);
    }

    let mut digits = String::with_capacity(integer.len() + fraction.len());
    digits.push_str(integer);
    digits.push_str(fraction);
    if digits.bytes().all(|byte| byte == b'0') {
        return ResourceByteLength::new(0);
    }
    if negative {
        return Err(ResourceByteLengthError::Negative);
    }

    let exponent = exponent.parse::<i64>().map_err(|_| {
        if exponent.starts_with('-') {
            ResourceByteLengthError::NotAnInteger
        } else {
            ResourceByteLengthError::OutOfRange
        }
    })?;
    let fraction_length =
        i64::try_from(fraction.len()).map_err(|_| ResourceByteLengthError::NotAnInteger)?;
    let scale = exponent
        .checked_sub(fraction_length)
        .ok_or(ResourceByteLengthError::NotAnInteger)?;

    let significant_digits = if scale < 0 {
        let trailing_zero_count = usize::try_from(scale.unsigned_abs())
            .map_err(|_| ResourceByteLengthError::NotAnInteger)?;
        let available_trailing_zeros = digits
            .bytes()
            .rev()
            .take_while(|byte| *byte == b'0')
            .count();
        if trailing_zero_count > available_trailing_zeros {
            return Err(ResourceByteLengthError::NotAnInteger);
        }
        digits[..digits.len() - trailing_zero_count].trim_start_matches('0')
    } else {
        digits.trim_start_matches('0')
    };

    if significant_digits.is_empty() {
        return ResourceByteLength::new(0);
    }
    let appended_zeros =
        usize::try_from(scale.max(0)).map_err(|_| ResourceByteLengthError::OutOfRange)?;
    let result_length = significant_digits
        .len()
        .checked_add(appended_zeros)
        .ok_or(ResourceByteLengthError::OutOfRange)?;
    if result_length > MAX_BYTE_LENGTH_DECIMAL.len() {
        return Err(ResourceByteLengthError::OutOfRange);
    }

    let mut integer_value = String::with_capacity(result_length);
    integer_value.push_str(significant_digits);
    integer_value.extend(std::iter::repeat_n('0', appended_zeros));
    if integer_value.len() == MAX_BYTE_LENGTH_DECIMAL.len()
        && integer_value.as_str() > MAX_BYTE_LENGTH_DECIMAL
    {
        return Err(ResourceByteLengthError::OutOfRange);
    }

    let value = integer_value
        .parse::<u64>()
        .map_err(|_| ResourceByteLengthError::OutOfRange)?;
    ResourceByteLength::new(value)
}
