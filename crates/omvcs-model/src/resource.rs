//! Logical Resource references and immutable Resource Object bytes.
//!
//! Resource identity is always derived from complete raw bytes. Descriptive
//! reference fields belong to a containing historical object's canonical
//! body, not to the Resource Identifier. Physical storage and reconstruction
//! information is intentionally outside these types.

use std::collections::BTreeMap;
use std::fmt;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use serde_json::value::RawValue;

use crate::ResourceId;
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

/// A historical reference to an immutable Resource Object.
///
/// Only the generic fields approved for OMVCS 0.1 are represented. Unknown
/// fields (including names and physical storage/reconstruction fields) are
/// rejected during deserialization. Properties are the schema/Adapter-owned
/// immutable interpretation map; they are not a channel for excluded
/// historical or operational fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResourceReference {
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

impl ResourceReference {
    /// Creates a Resource Reference with its required typed identity and length.
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

    /// Returns a new reference carrying schema/Adapter-supplied properties.
    #[must_use]
    pub fn with_properties(mut self, properties: BTreeMap<String, Value>) -> Self {
        self.properties = Some(properties);
        self
    }
}

impl<'de> Deserialize<'de> for ResourceReference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = Box::<RawValue>::deserialize(deserializer)?;
        crate::canonical::validate_unique_json_member_names(raw.get().as_bytes())
            .map_err(D::Error::custom)?;
        let wire =
            serde_json::from_str::<ResourceReferenceWire>(raw.get()).map_err(D::Error::custom)?;
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
    role: Option<String>,
    media_type: Option<String>,
    properties: Option<BTreeMap<String, Value>>,
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
