//! Immutable, content-addressed Component State objects.
//!
//! Candidates are not historical objects until an exact versioned schema and
//! every applicable Resource Reference property authority have admitted them.

use std::collections::BTreeMap;
use std::fmt;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use serde_json::value::RawValue;

use crate::canonical::{
    CanonicalMetadataError, MetadataSchema, canonicalize_metadata_body,
    canonicalize_prevalidated_body, validate_unique_json_member_names,
};
use crate::hashing::hash_component_state_metadata;
use crate::resource::{
    ResourceAdmissionError, ResourcePropertiesValidator, ResourceReference,
    ResourceReferenceCandidate, ResourceValidationContext,
};
use crate::{ComponentStateId, CreativeComponentId};

/// The exact, versioned Component State schema authority used for admission.
///
/// Implementations describe the schema-owned metadata structure and meaning,
/// and bind each permitted property-bearing Resource Reference context to its
/// exact authority. Core does not select schemas by preference or infer the
/// binding from property names.
pub trait ComponentStateSchemaValidator {
    /// Returns the exact schema version identifier handled by this authority.
    fn schema(&self) -> &str;

    /// Describes the permitted metadata keys, value shapes, and array rules.
    ///
    /// The root schema must describe a JSON object (`Struct` or `Map`).
    fn metadata_schema(&self) -> MetadataSchema;

    /// Applies schema-owned requirements and semantic rules to normalized metadata.
    ///
    /// # Errors
    ///
    /// Returns the exact schema's rejection reason.
    fn validate_metadata(&self, metadata: &BTreeMap<String, Value>) -> Result<(), String>;

    /// Returns the exact Resource validation binding for this property-bearing use.
    ///
    /// Returning `None` leaves the Resource Reference unchecked and prevents
    /// Component State admission. Implementations must not use latest-version,
    /// installed-preference, or key-name heuristics.
    fn resource_validation_context(
        &self,
        reference: &ResourceReferenceCandidate,
    ) -> Option<ResourceValidationContext>;
}

/// A decoded but not yet historically admitted Component State.
///
/// Candidate values can be retained while their exact schema is unavailable,
/// but this type intentionally has no identity or canonical historical bytes.
///
/// ```compile_fail
/// use omvcs_model::component_state::ComponentStateCandidate;
/// fn hash_unchecked(candidate: &ComponentStateCandidate) {
///     let _ = candidate.component_state_id();
/// }
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct ComponentStateCandidate {
    schema: String,
    component_id: CreativeComponentId,
    parents: Option<Vec<ComponentStateId>>,
    resources: Vec<ResourceReferenceCandidate>,
    metadata: BTreeMap<String, Value>,
}

impl ComponentStateCandidate {
    /// Creates an unchecked candidate with the required Component State fields.
    #[must_use]
    pub fn new(
        schema: impl Into<String>,
        component_id: CreativeComponentId,
        resources: Vec<ResourceReferenceCandidate>,
        metadata: BTreeMap<String, Value>,
    ) -> Self {
        Self {
            schema: schema.into(),
            component_id,
            parents: None,
            resources,
            metadata,
        }
    }

    /// Returns a candidate with the specified parent set.
    #[must_use]
    pub fn with_parents(mut self, parents: Vec<ComponentStateId>) -> Self {
        self.parents = Some(parents);
        self
    }

    /// Returns the schema version string without asserting its availability.
    #[must_use]
    pub fn schema(&self) -> &str {
        &self.schema
    }

    /// Returns the typed Creative Component identity.
    #[must_use]
    pub const fn component_id(&self) -> CreativeComponentId {
        self.component_id
    }

    /// Returns optional lineage as asserted by the candidate.
    #[must_use]
    pub fn parents(&self) -> Option<&[ComponentStateId]> {
        self.parents.as_deref()
    }

    /// Returns the unchecked Resource Reference candidates.
    #[must_use]
    pub fn resources(&self) -> &[ResourceReferenceCandidate] {
        &self.resources
    }

    /// Returns the candidate metadata map.
    #[must_use]
    pub const fn metadata(&self) -> &BTreeMap<String, Value> {
        &self.metadata
    }

    /// Validates and admits this candidate as immutable historical state.
    ///
    /// The exact schema version must match one available schema authority.
    /// Property-bearing Resource References additionally require one matching
    /// exact authority under the schema-provided context. No identity or
    /// canonical body is produced before all validation succeeds.
    ///
    /// # Errors
    ///
    /// Returns an error for an unavailable or ambiguous schema, invalid schema
    /// definition, invalid metadata, invalid parent/resource sets, or failed
    /// Resource Reference admission.
    pub fn admit(
        self,
        schemas: &[&dyn ComponentStateSchemaValidator],
        resource_validators: &[&dyn ResourcePropertiesValidator],
    ) -> Result<ComponentState, ComponentStateAdmissionError> {
        let mut matching_schemas = schemas
            .iter()
            .copied()
            .filter(|schema| schema.schema() == self.schema);
        let schema = matching_schemas
            .next()
            .ok_or(ComponentStateAdmissionError::UnavailableSchema)?;
        if matching_schemas.next().is_some() {
            return Err(ComponentStateAdmissionError::NonUniqueSchemaAuthority);
        }

        let metadata_schema = schema.metadata_schema();
        if !matches!(
            metadata_schema,
            MetadataSchema::Struct(_) | MetadataSchema::Map(_)
        ) {
            return Err(ComponentStateAdmissionError::InvalidSchemaDefinition);
        }
        let metadata_json = serde_json::to_vec(&self.metadata)
            .map_err(|_| CanonicalMetadataError::Canonicalization)?;
        let normalized_metadata_bytes =
            canonicalize_metadata_body(&metadata_json, &metadata_schema)?;
        let normalized_metadata: BTreeMap<String, Value> =
            serde_json::from_slice(&normalized_metadata_bytes)
                .map_err(|_| CanonicalMetadataError::InvalidJson)?;
        schema
            .validate_metadata(&normalized_metadata)
            .map_err(ComponentStateAdmissionError::MetadataRejected)?;

        let mut parents = self.parents;
        if let Some(parent_ids) = parents.as_mut() {
            parent_ids.sort_by_key(std::string::ToString::to_string);
            if parent_ids.windows(2).any(|pair| pair[0] == pair[1]) {
                return Err(ComponentStateAdmissionError::DuplicateParent);
            }
        }

        let mut resources = Vec::with_capacity(self.resources.len());
        for candidate in self.resources {
            let context = if candidate.properties().is_some() {
                schema.resource_validation_context(&candidate)
            } else {
                None
            };
            if context
                .as_ref()
                .is_some_and(|binding| binding.containing_schema != self.schema)
            {
                return Err(ComponentStateAdmissionError::InvalidResourceBinding);
            }
            let admitted = candidate.admit(context.as_ref(), resource_validators)?;
            let canonical = admitted.canonical_bytes(context.as_ref())?;
            resources.push((canonical, admitted));
        }
        resources.sort_by(|left, right| left.0.cmp(&right.0));
        if resources.windows(2).any(|pair| pair[0].0 == pair[1].0) {
            return Err(ComponentStateAdmissionError::DuplicateResource);
        }

        let resource_values = resources
            .iter()
            .map(|(bytes, _)| {
                serde_json::from_slice(bytes).map_err(|_| CanonicalMetadataError::InvalidJson)
            })
            .collect::<Result<Vec<Value>, _>>()?;
        let parents_value = parents.as_ref().map(|ids| {
            Value::Array(
                ids.iter()
                    .map(|parent| Value::String(parent.to_string()))
                    .collect(),
            )
        });
        let metadata_value = Value::Object(
            normalized_metadata
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
        );
        let mut body = serde_json::Map::new();
        body.insert("schema".to_owned(), Value::String(self.schema.clone()));
        body.insert(
            "component_id".to_owned(),
            Value::String(self.component_id.to_string()),
        );
        body.insert("resources".to_owned(), Value::Array(resource_values));
        body.insert("metadata".to_owned(), metadata_value);
        if let Some(parents) = parents_value {
            body.insert("parents".to_owned(), parents);
        }
        let canonical_body = canonicalize_prevalidated_body(&Value::Object(body))?;
        let component_state_id = hash_component_state_metadata(&canonical_body);

        Ok(ComponentState {
            schema: self.schema,
            component_id: self.component_id,
            parents: parents.map(Vec::into_boxed_slice),
            resources: resources
                .into_iter()
                .map(|(_, reference)| reference)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            metadata: normalized_metadata,
            canonical_body: canonical_body.into_boxed_slice(),
            id: component_state_id,
        })
    }
}

impl<'de> Deserialize<'de> for ComponentStateCandidate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = Box::<RawValue>::deserialize(deserializer)?;
        let raw_text = raw.get();
        if !raw_text.trim_start().starts_with('{') {
            return Err(D::Error::custom(
                "Component State candidate must be a JSON object",
            ));
        }
        validate_unique_json_member_names(raw_text.as_bytes()).map_err(D::Error::custom)?;
        let wire: ComponentStateCandidateWire =
            serde_json::from_str(raw_text).map_err(D::Error::custom)?;
        Ok(Self {
            schema: wire.schema,
            component_id: wire.component_id,
            parents: wire.parents,
            resources: wire.resources,
            metadata: wire.metadata,
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ComponentStateCandidateWire {
    schema: String,
    #[serde(with = "creative_component_id_serde")]
    component_id: CreativeComponentId,
    #[serde(default, deserialize_with = "present_parents")]
    parents: Option<Vec<ComponentStateId>>,
    resources: Vec<ResourceReferenceCandidate>,
    metadata: BTreeMap<String, Value>,
}

fn present_parents<'de, D>(deserializer: D) -> Result<Option<Vec<ComponentStateId>>, D::Error>
where
    D: Deserializer<'de>,
{
    let strings = Vec::<String>::deserialize(deserializer)?;
    strings
        .into_iter()
        .map(|value| value.parse().map_err(D::Error::custom))
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

mod creative_component_id_serde {
    use serde::de::Error as _;
    use serde::{Deserialize, Deserializer};

    use crate::CreativeComponentId;

    pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<CreativeComponentId, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(D::Error::custom)
    }
}

/// An admitted, immutable Component State with a content-derived identity.
///
/// Fields are private and the type has no mutating or unchecked constructor.
///
/// ```compile_fail
/// use omvcs_model::ComponentState;
/// fn mutate(state: &mut ComponentState) {
///     state.schema = "other/1".to_owned();
/// }
/// ```
#[derive(Debug)]
pub struct ComponentState {
    schema: String,
    component_id: CreativeComponentId,
    parents: Option<Box<[ComponentStateId]>>,
    resources: Box<[ResourceReference]>,
    metadata: BTreeMap<String, Value>,
    canonical_body: Box<[u8]>,
    id: ComponentStateId,
}

impl PartialEq for ComponentState {
    fn eq(&self, other: &Self) -> bool {
        self.canonical_body == other.canonical_body
    }
}

impl Eq for ComponentState {}

impl ComponentState {
    /// Returns the exact versioned Component State schema identifier.
    #[must_use]
    pub fn schema(&self) -> &str {
        &self.schema
    }

    /// Returns the typed Creative Component identity represented by this state.
    #[must_use]
    pub const fn component_id(&self) -> CreativeComponentId {
        self.component_id
    }

    /// Returns `None` for unknown/unasserted lineage and `Some([])` for a known
    /// initial state.
    #[must_use]
    pub fn parents(&self) -> Option<&[ComponentStateId]> {
        self.parents.as_deref()
    }

    /// Returns the historically admitted Resource References.
    #[must_use]
    pub fn resources(&self) -> &[ResourceReference] {
        &self.resources
    }

    /// Returns the schema-validated metadata map.
    #[must_use]
    pub const fn metadata(&self) -> &BTreeMap<String, Value> {
        &self.metadata
    }

    /// Returns the canonical bytes of exactly the historical Component State body.
    #[must_use]
    pub fn canonical_body(&self) -> &[u8] {
        &self.canonical_body
    }

    /// Returns the typed content-derived Component State Identifier.
    #[must_use]
    pub const fn component_state_id(&self) -> ComponentStateId {
        self.id
    }
}

/// A failure to admit a Component State candidate as historical state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentStateAdmissionError {
    /// No available validator handles the candidate's exact `schema` value.
    UnavailableSchema,
    /// Multiple validators claim the same exact versioned schema.
    NonUniqueSchemaAuthority,
    /// The metadata schema does not describe a JSON object map.
    InvalidSchemaDefinition,
    /// Schema-owned metadata semantics rejected the normalized metadata.
    MetadataRejected(String),
    /// The schema supplied a Resource binding for a different containing schema.
    InvalidResourceBinding,
    /// Two parents have identical typed Component State identity.
    DuplicateParent,
    /// Two admitted Resource References have identical canonical bytes.
    DuplicateResource,
    /// Generic metadata or canonical validation failed.
    Canonical(CanonicalMetadataError),
    /// Resource Reference admission failed.
    Resource(ResourceAdmissionError),
}

impl fmt::Display for ComponentStateAdmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnavailableSchema => {
                formatter.write_str("exact Component State schema is unavailable")
            }
            Self::NonUniqueSchemaAuthority => {
                formatter.write_str("Component State schema authority is not unique")
            }
            Self::InvalidSchemaDefinition => {
                formatter.write_str("Component State metadata schema must describe an object map")
            }
            Self::MetadataRejected(reason) => {
                write!(formatter, "Component State metadata rejected: {reason}")
            }
            Self::InvalidResourceBinding => formatter
                .write_str("Resource validation binding names a different Component State schema"),
            Self::DuplicateParent => {
                formatter.write_str("Component State parent set contains a duplicate")
            }
            Self::DuplicateResource => {
                formatter.write_str("Component State resource set contains a duplicate")
            }
            Self::Canonical(error) => error.fmt(formatter),
            Self::Resource(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for ComponentStateAdmissionError {}

impl From<CanonicalMetadataError> for ComponentStateAdmissionError {
    fn from(error: CanonicalMetadataError) -> Self {
        Self::Canonical(error)
    }
}

impl From<ResourceAdmissionError> for ComponentStateAdmissionError {
    fn from(error: ResourceAdmissionError) -> Self {
        Self::Resource(error)
    }
}
