//! Immutable, content-addressed Revision objects.
//!
//! A Revision candidate becomes historical only after its exact versioned
//! schema, Project State, and parent Revisions have been resolved as admitted.
//! Provenance structure and semantics remain owned by that exact schema.

use std::collections::BTreeMap;
use std::fmt;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use serde_json::value::RawValue;

use crate::canonical::{
    ArrayOrdering, CanonicalMetadataError, MetadataSchema, canonicalize_metadata_body,
    validate_unique_json_member_names,
};
use crate::hashing::hash_revision_metadata;
use crate::timestamp::is_valid_utc_nanosecond_timestamp;
use crate::{ActorId, ProjectState, ProjectStateId, RevisionId};

/// The exact, versioned Revision schema authority used for admission.
///
/// The authority owns provenance entry fields, requiredness, value shapes,
/// meanings, nested schemas, and collection classifications. Core applies
/// those generic structural and canonical rules without defining entry
/// vocabulary.
pub trait RevisionSchemaValidator {
    /// Returns the exact versioned schema identifier handled by this authority.
    fn schema(&self) -> &str;

    /// Describes the schema-owned structure for each provenance object.
    ///
    /// The root must be a fixed-field object schema. Nested object and array
    /// rules are recursively applied by Core.
    fn provenance_entry_schema(&self) -> MetadataSchema;

    /// Applies schema-owned requiredness and semantic rules after
    /// canonicalization.
    ///
    /// # Errors
    ///
    /// Returns the exact schema's rejection reason.
    fn validate_provenance(&self, entries: &[Value]) -> Result<(), String>;
}

/// Trusted boundary for resolving typed identifiers to admitted Project States.
pub trait AdmittedProjectStateResolver {
    /// Resolves `id` only when it names a valid, admitted Project State.
    fn resolve_admitted(&self, id: ProjectStateId) -> Option<&ProjectState>;
}

impl AdmittedProjectStateResolver for BTreeMap<ProjectStateId, ProjectState> {
    fn resolve_admitted(&self, id: ProjectStateId) -> Option<&ProjectState> {
        self.get(&id)
    }
}

/// Trusted boundary for resolving typed identifiers to admitted Revisions.
pub trait AdmittedRevisionResolver {
    /// Resolves `id` only when it names a valid, admitted Revision.
    fn resolve_admitted(&self, id: RevisionId) -> Option<&Revision>;
}

impl AdmittedRevisionResolver for BTreeMap<RevisionId, Revision> {
    fn resolve_admitted(&self, id: RevisionId) -> Option<&Revision> {
        self.get(&id)
    }
}

/// A decoded but not yet historically admitted Revision.
///
/// Candidates intentionally expose neither canonical historical bytes nor a
/// Revision Identifier before exact-schema and reference admission.
///
/// ```compile_fail
/// use omvcs_model::revision::RevisionCandidate;
/// fn hash_unchecked(candidate: &RevisionCandidate) {
///     let _ = candidate.revision_id();
/// }
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct RevisionCandidate {
    schema: String,
    project_state_id: ProjectStateId,
    parents: Vec<RevisionId>,
    author_id: ActorId,
    created_at: String,
    message: String,
    provenance: Vec<Value>,
}

impl RevisionCandidate {
    /// Creates a candidate with the exact required OMVCS 0.1 Revision members.
    #[must_use]
    pub fn new(
        schema: impl Into<String>,
        project_state_id: ProjectStateId,
        parents: Vec<RevisionId>,
        author_id: ActorId,
        created_at: impl Into<String>,
        message: impl Into<String>,
        provenance: Vec<Value>,
    ) -> Self {
        Self {
            schema: schema.into(),
            project_state_id,
            parents,
            author_id,
            created_at: created_at.into(),
            message: message.into(),
            provenance,
        }
    }

    /// Returns the versioned schema identifier without asserting its availability.
    #[must_use]
    pub fn schema(&self) -> &str {
        &self.schema
    }

    /// Validates and admits this candidate as immutable historical state.
    ///
    /// The exact schema authority must be uniquely available. The referenced
    /// Project State and every parent Revision must resolve to admitted
    /// objects, and parent Project identity must match the Project State.
    ///
    /// # Errors
    ///
    /// Returns an error for unavailable or ambiguous schema authority,
    /// invalid provenance schema or values, invalid timestamp, unresolved or
    /// mismatched references, cross-Project parents, or invalid canonical data.
    pub fn admit(
        self,
        schemas: &[&dyn RevisionSchemaValidator],
        project_states: &dyn AdmittedProjectStateResolver,
        revisions: &dyn AdmittedRevisionResolver,
    ) -> Result<Revision, RevisionAdmissionError> {
        let mut matching_schemas = schemas
            .iter()
            .copied()
            .filter(|schema| schema.schema() == self.schema);
        let schema = matching_schemas
            .next()
            .ok_or(RevisionAdmissionError::UnavailableSchema)?;
        if matching_schemas.next().is_some() {
            return Err(RevisionAdmissionError::NonUniqueSchemaAuthority);
        }

        let provenance_entry_schema = schema.provenance_entry_schema();
        if !matches!(provenance_entry_schema, MetadataSchema::Struct(_)) {
            return Err(RevisionAdmissionError::InvalidProvenanceSchema);
        }
        validate_created_at(&self.created_at)?;

        let project_state = project_states
            .resolve_admitted(self.project_state_id)
            .ok_or(RevisionAdmissionError::UnavailableProjectState)?;
        if project_state.project_state_id() != self.project_state_id {
            return Err(RevisionAdmissionError::ProjectStateIdentifierMismatch);
        }
        let project_id = project_state.project_id();

        let mut parents = self.parents.clone();
        parents.sort_by_key(ToString::to_string);
        if parents.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(RevisionAdmissionError::DuplicateParent);
        }
        for parent_id in &parents {
            let parent = revisions
                .resolve_admitted(*parent_id)
                .ok_or(RevisionAdmissionError::UnavailableParent)?;
            if parent.revision_id() != *parent_id {
                return Err(RevisionAdmissionError::ParentIdentifierMismatch);
            }
            if parent.project_id != project_id {
                return Err(RevisionAdmissionError::ParentProjectMismatch);
            }
        }

        let body = self.body_value(&parents);
        let body_json =
            serde_json::to_vec(&body).map_err(|_| CanonicalMetadataError::Canonicalization)?;
        let canonical_body =
            canonicalize_metadata_body(&body_json, &revision_body_schema(provenance_entry_schema))?;
        let canonical_value: Value = serde_json::from_slice(&canonical_body)
            .map_err(|_| CanonicalMetadataError::InvalidJson)?;
        let provenance = canonical_value
            .get("provenance")
            .and_then(Value::as_array)
            .ok_or(RevisionAdmissionError::InvalidProvenanceSchema)?
            .clone();

        schema
            .validate_provenance(&provenance)
            .map_err(RevisionAdmissionError::ProvenanceRejected)?;

        let id = hash_revision_metadata(&canonical_body);
        if parents.contains(&id) {
            return Err(RevisionAdmissionError::SelfParent);
        }

        Ok(Revision {
            schema: self.schema,
            project_state_id: self.project_state_id,
            parents,
            author_id: self.author_id,
            created_at: self.created_at,
            message: self.message,
            provenance,
            project_id,
            canonical_body: canonical_body.into_boxed_slice(),
            id,
        })
    }

    fn body_value(&self, parents: &[RevisionId]) -> Value {
        let mut body = serde_json::Map::new();
        body.insert("schema".to_owned(), Value::String(self.schema.clone()));
        body.insert(
            "project_state_id".to_owned(),
            Value::String(self.project_state_id.to_string()),
        );
        body.insert(
            "parents".to_owned(),
            Value::Array(
                parents
                    .iter()
                    .map(|id| Value::String(id.to_string()))
                    .collect(),
            ),
        );
        body.insert(
            "author_id".to_owned(),
            Value::String(self.author_id.to_string()),
        );
        body.insert(
            "created_at".to_owned(),
            Value::String(self.created_at.clone()),
        );
        body.insert("message".to_owned(), Value::String(self.message.clone()));
        body.insert(
            "provenance".to_owned(),
            Value::Array(self.provenance.clone()),
        );
        Value::Object(body)
    }
}

impl<'de> Deserialize<'de> for RevisionCandidate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = Box::<RawValue>::deserialize(deserializer)?;
        let raw_text = raw.get();
        if !raw_text.trim_start().starts_with('{') {
            return Err(D::Error::custom("Revision candidate must be a JSON object"));
        }
        validate_unique_json_member_names(raw_text.as_bytes()).map_err(D::Error::custom)?;
        let wire: RevisionCandidateWire =
            serde_json::from_str(raw_text).map_err(D::Error::custom)?;
        let project_state_id = wire.project_state_id.parse().map_err(D::Error::custom)?;
        let parents = wire
            .parents
            .into_iter()
            .map(|parent| parent.parse().map_err(D::Error::custom))
            .collect::<Result<Vec<_>, _>>()?;
        let author_id = wire.author_id.parse().map_err(D::Error::custom)?;
        Ok(Self::new(
            wire.schema,
            project_state_id,
            parents,
            author_id,
            wire.created_at,
            wire.message,
            wire.provenance,
        ))
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RevisionCandidateWire {
    schema: String,
    project_state_id: String,
    parents: Vec<String>,
    author_id: String,
    created_at: String,
    message: String,
    provenance: Vec<Value>,
}

fn revision_body_schema(provenance_entry_schema: MetadataSchema) -> MetadataSchema {
    MetadataSchema::structure([
        ("schema", MetadataSchema::Scalar),
        ("project_state_id", MetadataSchema::Scalar),
        (
            "parents",
            MetadataSchema::array(ArrayOrdering::SetLike, MetadataSchema::Scalar),
        ),
        ("author_id", MetadataSchema::Scalar),
        ("created_at", MetadataSchema::Scalar),
        ("message", MetadataSchema::Scalar),
        (
            "provenance",
            MetadataSchema::array(ArrayOrdering::SetLike, provenance_entry_schema),
        ),
    ])
}

fn validate_created_at(value: &str) -> Result<(), RevisionAdmissionError> {
    if is_valid_utc_nanosecond_timestamp(value) {
        Ok(())
    } else {
        Err(RevisionAdmissionError::InvalidTimestamp)
    }
}

/// An admitted, immutable Revision with canonical seven-member identity.
///
/// The cached Project identifier is derived from the admitted Project State
/// solely to validate the Project-consistency of later parent references. It
/// is not a historical body member and never participates in identity.
#[derive(Debug)]
pub struct Revision {
    schema: String,
    project_state_id: ProjectStateId,
    parents: Vec<RevisionId>,
    author_id: ActorId,
    created_at: String,
    message: String,
    provenance: Vec<Value>,
    project_id: crate::ProjectId,
    canonical_body: Box<[u8]>,
    id: RevisionId,
}

impl PartialEq for Revision {
    fn eq(&self, other: &Self) -> bool {
        self.canonical_body == other.canonical_body
    }
}

impl Eq for Revision {}

impl Revision {
    /// Returns the exact versioned Revision schema identifier.
    #[must_use]
    pub fn schema(&self) -> &str {
        &self.schema
    }

    /// Returns the identifier of the complete admitted Project State.
    #[must_use]
    pub const fn project_state_id(&self) -> ProjectStateId {
        self.project_state_id
    }

    /// Returns the canonically ordered direct parent Revision identifiers.
    #[must_use]
    pub fn parents(&self) -> &[RevisionId] {
        &self.parents
    }

    /// Returns the direct `ActorId` author.
    #[must_use]
    pub const fn author_id(&self) -> ActorId {
        self.author_id
    }

    /// Returns the exact canonical UTC nanosecond timestamp string.
    #[must_use]
    pub fn created_at(&self) -> &str {
        &self.created_at
    }

    /// Returns the verbatim Revision message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the schema-validated, canonically ordered provenance objects.
    #[must_use]
    pub fn provenance(&self) -> &[Value] {
        &self.provenance
    }

    /// Returns canonical bytes of exactly the seven-member historical body.
    #[must_use]
    pub fn canonical_body(&self) -> &[u8] {
        &self.canonical_body
    }

    /// Returns the typed content-derived Revision Identifier.
    #[must_use]
    pub const fn revision_id(&self) -> RevisionId {
        self.id
    }
}

/// A failure to admit a Revision candidate as historical state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RevisionAdmissionError {
    /// No available validator handles the candidate's exact `schema` value.
    UnavailableSchema,
    /// Multiple validators claim the same exact versioned schema.
    NonUniqueSchemaAuthority,
    /// The exact schema does not provide a fixed object schema for entries.
    InvalidProvenanceSchema,
    /// Schema-owned provenance semantics rejected the normalized entries.
    ProvenanceRejected(String),
    /// The Project State reference is not resolvable to an admitted object.
    UnavailableProjectState,
    /// A resolver returned a Project State whose identifier differs from its key.
    ProjectStateIdentifierMismatch,
    /// A parent Revision reference is not resolvable to an admitted object.
    UnavailableParent,
    /// A resolver returned a parent Revision whose identifier differs from its key.
    ParentIdentifierMismatch,
    /// A parent Revision belongs to another Project.
    ParentProjectMismatch,
    /// The set-like parent collection contains an identifier more than once.
    DuplicateParent,
    /// The candidate identifier is also named as one of its own parents.
    SelfParent,
    /// `created_at` is not a valid canonical UTC nanosecond timestamp.
    InvalidTimestamp,
    /// Generic metadata or canonical validation failed.
    Canonical(CanonicalMetadataError),
}

impl fmt::Display for RevisionAdmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnavailableSchema => formatter.write_str("Revision schema is unavailable"),
            Self::NonUniqueSchemaAuthority => {
                formatter.write_str("Revision schema authority is non-unique")
            }
            Self::InvalidProvenanceSchema => {
                formatter.write_str("Revision provenance schema is not an object schema")
            }
            Self::ProvenanceRejected(reason) => {
                write!(formatter, "Revision provenance rejected: {reason}")
            }
            Self::UnavailableProjectState => {
                formatter.write_str("Project State reference is unavailable")
            }
            Self::ProjectStateIdentifierMismatch => {
                formatter.write_str("resolved Project State identifier does not match reference")
            }
            Self::UnavailableParent => {
                formatter.write_str("parent Revision reference is unavailable")
            }
            Self::ParentIdentifierMismatch => {
                formatter.write_str("resolved parent Revision identifier does not match reference")
            }
            Self::ParentProjectMismatch => {
                formatter.write_str("parent Revision belongs to another Project")
            }
            Self::DuplicateParent => formatter.write_str("parent Revision is duplicated"),
            Self::SelfParent => formatter.write_str("Revision cannot be its own parent"),
            Self::InvalidTimestamp => {
                formatter.write_str("created_at is not a canonical UTC nanosecond timestamp")
            }
            Self::Canonical(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for RevisionAdmissionError {}

impl From<CanonicalMetadataError> for RevisionAdmissionError {
    fn from(error: CanonicalMetadataError) -> Self {
        Self::Canonical(error)
    }
}
