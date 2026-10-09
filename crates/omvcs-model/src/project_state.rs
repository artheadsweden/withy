//! Immutable, content-addressed complete Project State objects.
//!
//! A candidate becomes historical only after validation by its exact schema,
//! resolution of every referenced admitted Component State, and resolution of
//! its Adapter State through the trusted Core/Adapter admission boundary.

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
use crate::hashing::hash_project_state_metadata;
use crate::{
    AdapterStateId, ComponentState, ComponentStateId, CreativeComponentId, ProjectId,
    ProjectStateId,
};

/// The exact, versioned Project State schema authority used for admission.
///
/// The authority declares the complete schema for `project_metadata`.
/// Core applies its recursive shape and collection rules; schema-owned
/// requiredness and semantic constraints are applied by
/// [`Self::validate_project_metadata`].
pub trait ProjectStateSchemaValidator {
    /// Returns the exact schema version identifier handled by this authority.
    fn schema(&self) -> &str;

    /// Describes permitted metadata keys, value shapes, and array rules.
    ///
    /// The root schema must describe a JSON object (`Struct` or `Map`).
    fn project_metadata_schema(&self) -> MetadataSchema;

    /// Applies schema-owned requirements and semantic rules to normalized metadata.
    ///
    /// # Errors
    ///
    /// Returns the exact schema's rejection reason.
    fn validate_project_metadata(&self, metadata: &BTreeMap<String, Value>) -> Result<(), String>;
}

/// Resolves Component State identifiers to their admitted immutable objects.
///
/// A returned [`ComponentState`] is structurally admitted: its fields and
/// identity can only be produced by the Component State admission API.
pub trait ComponentStateResolver {
    /// Resolves an identifier to its corresponding admitted Component State.
    fn resolve_admitted(&self, id: ComponentStateId) -> Option<&ComponentState>;
}

impl ComponentStateResolver for BTreeMap<ComponentStateId, ComponentState> {
    fn resolve_admitted(&self, id: ComponentStateId) -> Option<&ComponentState> {
        self.get(&id)
    }
}

/// Trusted boundary for resolving an already admitted Adapter State object.
///
/// Implementations MUST return `true` only when this exact typed identifier
/// resolves to one canonical Adapter State metadata object admitted after
/// generic Core structural/canonical checks and semantic validation by the
/// unique exact applicable Adapter schema/authority. This boundary consumes
/// that guarantee; it does not define or inspect an Adapter State body and
/// does not treat underlying Resource-byte availability as admission.
pub trait AdmittedAdapterStateResolver {
    /// Resolves `id` to the typed identity of its valid, historically admitted object.
    ///
    /// Returning `None` means the object is missing, unavailable, unchecked,
    /// or not admitted under its exact applicable schema/Adapter authority.
    fn resolve_admitted(&self, id: AdapterStateId) -> Option<AdapterStateId>;
}

/// A decoded but not yet historically admitted Project State.
///
/// Candidates intentionally expose neither canonical historical bytes nor a
/// Project State Identifier until exact-schema and reference admission passes.
#[derive(Debug, PartialEq, Eq)]
pub struct ProjectStateCandidate {
    schema: String,
    project_id: ProjectId,
    components: BTreeMap<CreativeComponentId, ComponentStateId>,
    adapter_state_id: AdapterStateId,
    project_metadata: BTreeMap<String, Value>,
}

impl ProjectStateCandidate {
    /// Verifies the canonical body-derived Identifier without resolving
    /// Component State or Adapter State references.
    ///
    /// The exact schema, complete closed body, nested metadata shape,
    /// collection classifications, and schema-owned metadata rules are still
    /// required. This method does not establish historical admission.
    ///
    /// # Errors
    ///
    /// Returns an error for an unavailable/ambiguous exact schema, invalid
    /// schema/body, rejected metadata, or invalid canonical metadata.
    pub fn verify_body_identifier(
        &self,
        schemas: &[&dyn ProjectStateSchemaValidator],
    ) -> Result<ProjectStateId, ProjectStateAdmissionError> {
        let mut matching_schemas = schemas
            .iter()
            .copied()
            .filter(|schema| schema.schema() == self.schema);
        let schema = matching_schemas
            .next()
            .ok_or(ProjectStateAdmissionError::UnavailableSchema)?;
        if matching_schemas.next().is_some() {
            return Err(ProjectStateAdmissionError::NonUniqueSchemaAuthority);
        }

        let metadata_schema = schema.project_metadata_schema();
        if !matches!(
            metadata_schema,
            MetadataSchema::Struct(_) | MetadataSchema::Map(_)
        ) {
            return Err(ProjectStateAdmissionError::InvalidSchemaDefinition);
        }
        let metadata_json = serde_json::to_vec(&self.project_metadata)
            .map_err(|_| CanonicalMetadataError::Canonicalization)?;
        let normalized_metadata_bytes =
            canonicalize_metadata_body(&metadata_json, &metadata_schema)?;
        let normalized_metadata: BTreeMap<String, Value> =
            serde_json::from_slice(&normalized_metadata_bytes)
                .map_err(|_| CanonicalMetadataError::InvalidJson)?;
        schema
            .validate_project_metadata(&normalized_metadata)
            .map_err(ProjectStateAdmissionError::MetadataRejected)?;

        let components = Value::Object(
            self.components
                .iter()
                .map(|(component_id, state_id)| {
                    (
                        component_id.to_string(),
                        Value::String(state_id.to_string()),
                    )
                })
                .collect(),
        );
        let project_metadata = Value::Object(
            normalized_metadata
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
        );
        let mut body = serde_json::Map::new();
        body.insert("schema".to_owned(), Value::String(self.schema.clone()));
        body.insert(
            "project_id".to_owned(),
            Value::String(self.project_id.to_string()),
        );
        body.insert("components".to_owned(), components);
        body.insert(
            "adapter_state_id".to_owned(),
            Value::String(self.adapter_state_id.to_string()),
        );
        body.insert("project_metadata".to_owned(), project_metadata);
        let canonical_body = canonicalize_prevalidated_body(&Value::Object(body))?;
        Ok(hash_project_state_metadata(&canonical_body))
    }

    /// Validates and admits this candidate as immutable historical Project State.
    ///
    /// The exact schema must be uniquely available. Every component reference
    /// must resolve to the corresponding admitted Component State, and the
    /// Adapter State identifier must pass the trusted admitted-object boundary.
    ///
    /// # Errors
    ///
    /// Returns an error for unavailable/ambiguous schema authority, invalid
    /// metadata, unresolved or mismatched Component State, an unadmitted
    /// Adapter State, or invalid canonical metadata.
    pub fn admit(
        self,
        schemas: &[&dyn ProjectStateSchemaValidator],
        component_states: &dyn ComponentStateResolver,
        adapter_states: &dyn AdmittedAdapterStateResolver,
    ) -> Result<ProjectState, ProjectStateAdmissionError> {
        let mut matching_schemas = schemas
            .iter()
            .copied()
            .filter(|schema| schema.schema() == self.schema);
        let schema = matching_schemas
            .next()
            .ok_or(ProjectStateAdmissionError::UnavailableSchema)?;
        if matching_schemas.next().is_some() {
            return Err(ProjectStateAdmissionError::NonUniqueSchemaAuthority);
        }

        let metadata_schema = schema.project_metadata_schema();
        if !matches!(
            metadata_schema,
            MetadataSchema::Struct(_) | MetadataSchema::Map(_)
        ) {
            return Err(ProjectStateAdmissionError::InvalidSchemaDefinition);
        }
        let metadata_json = serde_json::to_vec(&self.project_metadata)
            .map_err(|_| CanonicalMetadataError::Canonicalization)?;
        let normalized_metadata_bytes =
            canonicalize_metadata_body(&metadata_json, &metadata_schema)?;
        let normalized_metadata: BTreeMap<String, Value> =
            serde_json::from_slice(&normalized_metadata_bytes)
                .map_err(|_| CanonicalMetadataError::InvalidJson)?;
        schema
            .validate_project_metadata(&normalized_metadata)
            .map_err(ProjectStateAdmissionError::MetadataRejected)?;

        for (component_id, component_state_id) in &self.components {
            let component_state = component_states
                .resolve_admitted(*component_state_id)
                .ok_or(ProjectStateAdmissionError::UnavailableComponentState)?;
            if component_state.component_state_id() != *component_state_id {
                return Err(ProjectStateAdmissionError::ComponentStateIdentifierMismatch);
            }
            if component_state.component_id() != *component_id {
                return Err(ProjectStateAdmissionError::ComponentIdentityMismatch);
            }
        }

        if adapter_states.resolve_admitted(self.adapter_state_id) != Some(self.adapter_state_id) {
            return Err(ProjectStateAdmissionError::UnavailableAdapterState);
        }

        let components = Value::Object(
            self.components
                .iter()
                .map(|(component_id, state_id)| {
                    (
                        component_id.to_string(),
                        Value::String(state_id.to_string()),
                    )
                })
                .collect(),
        );
        let project_metadata = Value::Object(
            normalized_metadata
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
        );
        let mut body = serde_json::Map::new();
        body.insert("schema".to_owned(), Value::String(self.schema.clone()));
        body.insert(
            "project_id".to_owned(),
            Value::String(self.project_id.to_string()),
        );
        body.insert("components".to_owned(), components);
        body.insert(
            "adapter_state_id".to_owned(),
            Value::String(self.adapter_state_id.to_string()),
        );
        body.insert("project_metadata".to_owned(), project_metadata);
        let canonical_body = canonicalize_prevalidated_body(&Value::Object(body))?;
        let project_state_id = hash_project_state_metadata(&canonical_body);

        Ok(ProjectState {
            schema: self.schema,
            project_id: self.project_id,
            components: self.components,
            adapter_state_id: self.adapter_state_id,
            project_metadata: normalized_metadata,
            canonical_body: canonical_body.into_boxed_slice(),
            id: project_state_id,
        })
    }
}

impl<'de> Deserialize<'de> for ProjectStateCandidate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = Box::<RawValue>::deserialize(deserializer)?;
        let raw_text = raw.get();
        if !raw_text.trim_start().starts_with('{') {
            return Err(D::Error::custom(
                "Project State candidate must be a JSON object",
            ));
        }
        validate_unique_json_member_names(raw_text.as_bytes()).map_err(D::Error::custom)?;
        let wire: ProjectStateCandidateWire =
            serde_json::from_str(raw_text).map_err(D::Error::custom)?;
        Ok(Self {
            schema: wire.schema,
            project_id: wire.project_id,
            components: wire.components,
            adapter_state_id: wire.adapter_state_id,
            project_metadata: wire.project_metadata,
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectStateCandidateWire {
    schema: String,
    #[serde(with = "project_id_serde")]
    project_id: ProjectId,
    #[serde(deserialize_with = "components_serde::deserialize")]
    components: BTreeMap<CreativeComponentId, ComponentStateId>,
    #[serde(with = "adapter_state_id_serde")]
    adapter_state_id: AdapterStateId,
    project_metadata: BTreeMap<String, Value>,
}

mod project_id_serde {
    use serde::de::Error as _;
    use serde::{Deserialize, Deserializer};

    use crate::ProjectId;

    pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<ProjectId, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(D::Error::custom)
    }
}

mod components_serde {
    use std::collections::BTreeMap;

    use serde::de::Error as _;
    use serde::{Deserialize, Deserializer};

    use crate::{ComponentStateId, CreativeComponentId};

    pub(super) fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<BTreeMap<CreativeComponentId, ComponentStateId>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw: BTreeMap<String, String> = BTreeMap::deserialize(deserializer)?;
        raw.into_iter()
            .map(|(component_id, state_id)| {
                let component_id = component_id.parse().map_err(D::Error::custom)?;
                let state_id = state_id.parse().map_err(D::Error::custom)?;
                Ok((component_id, state_id))
            })
            .collect()
    }
}

mod adapter_state_id_serde {
    use serde::de::Error as _;
    use serde::{Deserialize, Deserializer};

    use crate::AdapterStateId;

    pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<AdapterStateId, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(D::Error::custom)
    }
}

/// An admitted, immutable complete Project State with a content-derived identity.
///
/// Its private fields and lack of an unchecked constructor preserve the
/// historical admission boundary.
#[derive(Debug)]
pub struct ProjectState {
    schema: String,
    project_id: ProjectId,
    components: BTreeMap<CreativeComponentId, ComponentStateId>,
    adapter_state_id: AdapterStateId,
    project_metadata: BTreeMap<String, Value>,
    canonical_body: Box<[u8]>,
    id: ProjectStateId,
}

impl PartialEq for ProjectState {
    fn eq(&self, other: &Self) -> bool {
        self.canonical_body == other.canonical_body
    }
}

impl Eq for ProjectState {}

impl ProjectState {
    /// Returns the exact versioned Project State schema identifier.
    #[must_use]
    pub fn schema(&self) -> &str {
        &self.schema
    }

    /// Returns the assigned Project Identifier represented by this state.
    #[must_use]
    pub const fn project_id(&self) -> ProjectId {
        self.project_id
    }

    /// Returns the complete Creative Component to Component State membership map.
    #[must_use]
    pub const fn components(&self) -> &BTreeMap<CreativeComponentId, ComponentStateId> {
        &self.components
    }

    /// Returns the identifier of the admitted Adapter State object.
    #[must_use]
    pub const fn adapter_state_id(&self) -> AdapterStateId {
        self.adapter_state_id
    }

    /// Returns schema-validated Project metadata.
    #[must_use]
    pub const fn project_metadata(&self) -> &BTreeMap<String, Value> {
        &self.project_metadata
    }

    /// Returns canonical bytes of exactly the five-member historical body.
    #[must_use]
    pub fn canonical_body(&self) -> &[u8] {
        &self.canonical_body
    }

    /// Returns the typed content-derived Project State Identifier.
    #[must_use]
    pub const fn project_state_id(&self) -> ProjectStateId {
        self.id
    }
}

/// A failure to admit a Project State candidate as historical state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectStateAdmissionError {
    /// No available validator handles the candidate's exact `schema` value.
    UnavailableSchema,
    /// Multiple validators claim the same exact versioned schema.
    NonUniqueSchemaAuthority,
    /// The metadata schema does not describe a JSON object map.
    InvalidSchemaDefinition,
    /// Schema-owned metadata semantics rejected the normalized metadata.
    MetadataRejected(String),
    /// A Component State reference is not resolvable to an admitted object.
    UnavailableComponentState,
    /// A resolver returned a Component State whose identity does not match its key.
    ComponentStateIdentifierMismatch,
    /// A Component State's embedded `component_id` differs from its map key.
    ComponentIdentityMismatch,
    /// The Adapter State reference is not resolvable as valid/admitted history.
    UnavailableAdapterState,
    /// Generic metadata or canonical validation failed.
    Canonical(CanonicalMetadataError),
}

impl fmt::Display for ProjectStateAdmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnavailableSchema => formatter.write_str("Project State schema is unavailable"),
            Self::NonUniqueSchemaAuthority => {
                formatter.write_str("Project State schema authority is non-unique")
            }
            Self::InvalidSchemaDefinition => {
                formatter.write_str("Project State metadata schema is not an object schema")
            }
            Self::MetadataRejected(reason) => {
                write!(formatter, "Project State metadata rejected: {reason}")
            }
            Self::UnavailableComponentState => {
                formatter.write_str("Component State reference is unavailable")
            }
            Self::ComponentStateIdentifierMismatch => {
                formatter.write_str("resolved Component State identifier does not match reference")
            }
            Self::ComponentIdentityMismatch => {
                formatter.write_str("Component State component_id does not match map key")
            }
            Self::UnavailableAdapterState => {
                formatter.write_str("Adapter State reference is unavailable or not admitted")
            }
            Self::Canonical(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for ProjectStateAdmissionError {}

impl From<CanonicalMetadataError> for ProjectStateAdmissionError {
    fn from(error: CanonicalMetadataError) -> Self {
        Self::Canonical(error)
    }
}
