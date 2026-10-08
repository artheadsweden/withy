//! Canonical serialization for schema-described OMVCS metadata bodies.
//!
//! The caller supplies the historical object body and its schema. This module
//! does not extract bodies from storage or protocol wrappers and does not hash
//! the resulting bytes.

use std::collections::BTreeMap;
use std::fmt;
use std::str;

use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::{Map, Number, Value};

const DUPLICATE_MEMBER_MARKER: &str = "OMVCS_DUPLICATE_MEMBER_NAME";
const UNSUPPORTED_NUMBER_MARKER: &str = "OMVCS_UNSUPPORTED_NUMBER";

/// The declared ordering semantics for an array-valued collection field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrayOrdering {
    /// Preserve the array's specified semantic order.
    Ordered,
    /// Sort by each normalized element's RFC 8785 canonical bytes and reject duplicates.
    SetLike,
}

/// A recursive description of the JSON shape and collection semantics of a metadata body.
///
/// A struct lists its recognized fields; absent fields remain absent, and fields
/// not listed in the schema are rejected. A map's values all use the same
/// schema. Every array schema must declare its ordering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetadataSchema {
    /// A JSON null, boolean, number, or string.
    Scalar,
    /// A fixed-field JSON object.
    Struct(BTreeMap<String, Self>),
    /// A JSON object map whose values follow the contained schema.
    Map(Box<Self>),
    /// An array and the schema of each element.
    Array {
        /// `None` represents an incomplete schema and is invalid for serialization.
        ordering: Option<ArrayOrdering>,
        /// Schema applied recursively to each element.
        elements: Box<Self>,
    },
}

impl MetadataSchema {
    /// Creates a fixed-field object schema.
    pub fn structure<K>(fields: impl IntoIterator<Item = (K, Self)>) -> Self
    where
        K: Into<String>,
    {
        Self::Struct(
            fields
                .into_iter()
                .map(|(key, value)| (key.into(), value))
                .collect(),
        )
    }

    /// Creates a JSON object-map schema.
    #[must_use]
    pub fn map(values: Self) -> Self {
        Self::Map(Box::new(values))
    }

    /// Creates an explicitly classified array schema.
    #[must_use]
    pub fn array(ordering: ArrayOrdering, elements: Self) -> Self {
        Self::Array {
            ordering: Some(ordering),
            elements: Box::new(elements),
        }
    }

    /// Creates an incomplete array schema, which is rejected during serialization.
    #[must_use]
    pub fn unclassified_array(elements: Self) -> Self {
        Self::Array {
            ordering: None,
            elements: Box::new(elements),
        }
    }
}

/// Failures while validating or canonicalizing an OMVCS metadata body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonicalMetadataError {
    /// The supplied bytes are not valid UTF-8.
    InvalidUtf8,
    /// The JSON is invalid or contains a number that cannot be represented under JCS rules.
    InvalidJson,
    /// A raw JSON object contains a repeated member name.
    DuplicateMemberName,
    /// A value does not match its schema, or contains a field not described by the schema.
    SchemaMismatch {
        /// JSON Pointer-like path to the mismatching value.
        path: String,
    },
    /// An array has no ordered or set-like classification in its schema.
    UnclassifiedArray {
        /// JSON Pointer-like path to the unclassified array.
        path: String,
    },
    /// A set-like array contains repeated canonical element bytes.
    DuplicateSetLikeElement {
        /// JSON Pointer-like path to the set-like array.
        path: String,
    },
    /// RFC 8785 canonical serialization failed.
    Canonicalization,
}

impl fmt::Display for CanonicalMetadataError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUtf8 => formatter.write_str("metadata body is not valid UTF-8"),
            Self::InvalidJson => {
                formatter.write_str("metadata body is not valid JCS-compatible JSON")
            }
            Self::DuplicateMemberName => {
                formatter.write_str("metadata body contains a duplicate object member name")
            }
            Self::SchemaMismatch { path } => {
                write!(
                    formatter,
                    "metadata value does not match its schema at {path}"
                )
            }
            Self::UnclassifiedArray { path } => {
                write!(
                    formatter,
                    "array collection has no ordering classification at {path}"
                )
            }
            Self::DuplicateSetLikeElement { path } => {
                write!(
                    formatter,
                    "set-like array has duplicate canonical elements at {path}"
                )
            }
            Self::Canonicalization => formatter.write_str("RFC 8785 canonicalization failed"),
        }
    }
}

impl std::error::Error for CanonicalMetadataError {}

/// Parses, schema-normalizes, and RFC 8785-canonicalizes one metadata object body.
///
/// `body` must contain only the historical object body. Storage wrappers,
/// transport headers, database keys, signatures, and timestamps outside that
/// object are not accepted as separate inputs and are not included implicitly.
/// Duplicate member names are rejected during parsing, before canonicalization.
///
/// # Errors
///
/// Returns an error for non-UTF-8 or invalid JSON, duplicate object member
/// names, schema mismatches, unclassified arrays, duplicate set-like elements,
/// or a failed RFC 8785 serialization.
pub fn canonicalize_metadata_body(
    body: &[u8],
    schema: &MetadataSchema,
) -> Result<Vec<u8>, CanonicalMetadataError> {
    let text = str::from_utf8(body).map_err(|_| CanonicalMetadataError::InvalidUtf8)?;
    let mut deserializer = serde_json::Deserializer::from_str(text);
    let parsed =
        UniqueJsonValue::deserialize(&mut deserializer).map_err(|error| parse_error(&error))?;
    deserializer.end().map_err(|error| parse_error(&error))?;
    if !parsed.0.is_object() {
        return Err(schema_mismatch(""));
    }
    validate_schema(schema, "")?;
    let normalized = normalize(parsed.0, schema, "")?;
    serde_jcs::to_vec(&normalized).map_err(|_| CanonicalMetadataError::Canonicalization)
}

fn parse_error(error: &serde_json::Error) -> CanonicalMetadataError {
    let message = error.to_string();
    if message.contains(DUPLICATE_MEMBER_MARKER) {
        CanonicalMetadataError::DuplicateMemberName
    } else {
        CanonicalMetadataError::InvalidJson
    }
}

fn normalize(
    value: Value,
    schema: &MetadataSchema,
    path: &str,
) -> Result<Value, CanonicalMetadataError> {
    match schema {
        MetadataSchema::Scalar if !value.is_array() && !value.is_object() => Ok(value),
        MetadataSchema::Struct(fields) => {
            let Value::Object(input) = value else {
                return Err(schema_mismatch(path));
            };
            let mut output = Map::new();
            for (key, value) in input {
                let Some(field_schema) = fields.get(&key) else {
                    return Err(schema_mismatch(&member_path(path, &key)));
                };
                output.insert(
                    key.clone(),
                    normalize(value, field_schema, &member_path(path, &key))?,
                );
            }
            Ok(Value::Object(output))
        }
        MetadataSchema::Map(value_schema) => {
            let Value::Object(input) = value else {
                return Err(schema_mismatch(path));
            };
            let mut output = Map::new();
            for (key, value) in input {
                output.insert(
                    key.clone(),
                    normalize(value, value_schema, &member_path(path, &key))?,
                );
            }
            Ok(Value::Object(output))
        }
        MetadataSchema::Array { ordering, elements } => {
            let Value::Array(input) = value else {
                return Err(schema_mismatch(path));
            };
            let Some(ordering) = ordering else {
                return Err(CanonicalMetadataError::UnclassifiedArray {
                    path: display_path(path),
                });
            };

            let mut output = input
                .into_iter()
                .enumerate()
                .map(|(index, value)| normalize(value, elements, &index_path(path, index)))
                .collect::<Result<Vec<_>, _>>()?;
            if *ordering == ArrayOrdering::SetLike {
                let mut canonical_elements = output
                    .into_iter()
                    .map(|element| {
                        serde_jcs::to_vec(&element)
                            .map(|bytes| (bytes, element))
                            .map_err(|_| CanonicalMetadataError::Canonicalization)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                canonical_elements.sort_by(|left, right| left.0.cmp(&right.0));
                if canonical_elements
                    .windows(2)
                    .any(|pair| pair[0].0 == pair[1].0)
                {
                    return Err(CanonicalMetadataError::DuplicateSetLikeElement {
                        path: display_path(path),
                    });
                }
                output = canonical_elements
                    .into_iter()
                    .map(|(_, element)| element)
                    .collect();
            }
            Ok(Value::Array(output))
        }
        MetadataSchema::Scalar => Err(schema_mismatch(path)),
    }
}

fn validate_schema(schema: &MetadataSchema, path: &str) -> Result<(), CanonicalMetadataError> {
    match schema {
        MetadataSchema::Scalar => Ok(()),
        MetadataSchema::Struct(fields) => {
            for (field, field_schema) in fields {
                validate_schema(field_schema, &member_path(path, field))?;
            }
            Ok(())
        }
        MetadataSchema::Map(value_schema) => validate_schema(value_schema, &member_path(path, "*")),
        MetadataSchema::Array { ordering, elements } => {
            if ordering.is_none() {
                return Err(CanonicalMetadataError::UnclassifiedArray {
                    path: display_path(path),
                });
            }
            validate_schema(elements, &member_path(path, "*"))
        }
    }
}

fn schema_mismatch(path: &str) -> CanonicalMetadataError {
    CanonicalMetadataError::SchemaMismatch {
        path: display_path(path),
    }
}

fn display_path(path: &str) -> String {
    if path.is_empty() {
        "/".to_owned()
    } else {
        path.to_owned()
    }
}

fn member_path(path: &str, member: &str) -> String {
    let escaped = member.replace('~', "~0").replace('/', "~1");
    format!("{path}/{escaped}")
}

fn index_path(path: &str, index: usize) -> String {
    format!("{path}/{index}")
}

struct UniqueJsonValue(Value);

impl<'de> Deserialize<'de> for UniqueJsonValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(UniqueValueVisitor)
    }
}

struct UniqueValueVisitor;

impl<'de> Visitor<'de> for UniqueValueVisitor {
    type Value = UniqueJsonValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON value with unique object member names")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(UniqueJsonValue(Value::Null))
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(UniqueJsonValue(Value::Bool(value)))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if !integer_is_exactly_representable(value.unsigned_abs()) {
            return Err(E::custom(UNSUPPORTED_NUMBER_MARKER));
        }
        Ok(UniqueJsonValue(Value::Number(Number::from(value))))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if !integer_is_exactly_representable(value) {
            return Err(E::custom(UNSUPPORTED_NUMBER_MARKER));
        }
        Ok(UniqueJsonValue(Value::Number(Number::from(value))))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Number::from_f64(value)
            .map(|number| UniqueJsonValue(Value::Number(number)))
            .ok_or_else(|| E::custom(UNSUPPORTED_NUMBER_MARKER))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(UniqueJsonValue(Value::String(value.to_owned())))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(UniqueJsonValue(Value::String(value)))
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element::<UniqueJsonValue>()? {
            values.push(value.0);
        }
        Ok(UniqueJsonValue(Value::Array(values)))
    }

    fn visit_map<A>(self, mut object: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = Map::new();
        while let Some((key, value)) = object.next_entry::<String, UniqueJsonValue>()? {
            if values.contains_key(&key) {
                return Err(de::Error::custom(DUPLICATE_MEMBER_MARKER));
            }
            values.insert(key, value.0);
        }
        Ok(UniqueJsonValue(Value::Object(values)))
    }
}

const fn integer_is_exactly_representable(magnitude: u64) -> bool {
    if magnitude == 0 {
        return true;
    }
    let significant_bits = u64::BITS - magnitude.leading_zeros() - magnitude.trailing_zeros();
    significant_bits <= 53
}
