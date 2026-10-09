//! Immutable, content-addressed Release metadata.
//!
//! A Release becomes historical only after its exact schema authority and
//! its admitted same-Project Revision target have been verified.

use std::fmt;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use serde_json::value::RawValue;

use crate::canonical::{
    CanonicalMetadataError, MetadataSchema, canonicalize_metadata_body,
    validate_unique_json_member_names,
};
use crate::hashing::hash_release_metadata;
use crate::revision::{AdmittedProjectStateResolver, AdmittedRevisionResolver};
use crate::timestamp::is_valid_utc_nanosecond_timestamp;
use crate::{ActorId, ProjectId, ReleaseId, RevisionId};

/// The only Release schema defined by OMVCS 0.1.
pub const RELEASE_SCHEMA: &str = "omvcs.release/0.1";

/// The exact versioned schema authority used to admit an OMVCS 0.1 Release.
pub trait ReleaseSchemaValidator {
    /// Returns the exact schema identifier handled by this authority.
    fn schema(&self) -> &str;

    /// Applies the schema authority's validation to the complete typed body.
    ///
    /// # Errors
    ///
    /// Returns the exact schema authority's rejection reason.
    fn validate_release(&self, candidate: &ReleaseCandidate) -> Result<(), String>;
}

/// A decoded but not yet admitted Release body.
///
/// The schema is retained from decoded input so unknown schema versions cannot
/// be silently treated as OMVCS 0.1. The constructor assigns the exact
/// OMVCS 0.1 schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseCandidate {
    schema: String,
    project_id: ProjectId,
    name: String,
    revision_id: RevisionId,
    created_at: String,
    creator_id: ActorId,
    description: String,
}

impl ReleaseCandidate {
    /// Creates an OMVCS 0.1 Release candidate.
    #[must_use]
    pub fn new(
        project_id: ProjectId,
        name: impl Into<String>,
        revision_id: RevisionId,
        created_at: impl Into<String>,
        creator_id: ActorId,
        description: impl Into<String>,
    ) -> Self {
        Self {
            schema: RELEASE_SCHEMA.to_owned(),
            project_id,
            name: name.into(),
            revision_id,
            created_at: created_at.into(),
            creator_id,
            description: description.into(),
        }
    }

    /// Returns the candidate's declared schema without asserting availability.
    #[must_use]
    pub fn schema(&self) -> &str {
        &self.schema
    }

    /// Returns the owning Project identifier.
    #[must_use]
    pub const fn project_id(&self) -> ProjectId {
        self.project_id
    }

    /// Returns the exact Release name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the direct target Revision identifier.
    #[must_use]
    pub const fn revision_id(&self) -> RevisionId {
        self.revision_id
    }

    /// Returns the candidate creation timestamp.
    #[must_use]
    pub fn created_at(&self) -> &str {
        &self.created_at
    }

    /// Returns the direct creator Actor identifier.
    #[must_use]
    pub const fn creator_id(&self) -> ActorId {
        self.creator_id
    }

    /// Returns the verbatim description.
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Verifies the canonical body-derived Identifier without resolving the
    /// target Revision or its Project State.
    ///
    /// The exact Release schema, closed body, name, timestamp, and
    /// schema-owned checks still apply. This does not establish historical
    /// admission or validate the target.
    ///
    /// # Errors
    ///
    /// Returns an error for an unavailable/ambiguous schema, invalid body,
    /// timestamp, name, or canonical representation.
    pub fn verify_body_identifier(
        &self,
        schemas: &[&dyn ReleaseSchemaValidator],
    ) -> Result<ReleaseId, ReleaseAdmissionError> {
        if self.schema != RELEASE_SCHEMA {
            return Err(ReleaseAdmissionError::UnavailableSchema);
        }
        let mut matching_schemas = schemas
            .iter()
            .copied()
            .filter(|schema| schema.schema() == self.schema);
        let schema = matching_schemas
            .next()
            .ok_or(ReleaseAdmissionError::UnavailableSchema)?;
        if matching_schemas.next().is_some() {
            return Err(ReleaseAdmissionError::NonUniqueSchemaAuthority);
        }
        if self.name.is_empty() {
            return Err(ReleaseAdmissionError::InvalidName);
        }
        if !is_valid_utc_nanosecond_timestamp(&self.created_at) {
            return Err(ReleaseAdmissionError::InvalidTimestamp);
        }
        schema
            .validate_release(self)
            .map_err(ReleaseAdmissionError::SchemaRejected)?;
        let body = self.body_value();
        let body_json =
            serde_json::to_vec(&body).map_err(|_| CanonicalMetadataError::Canonicalization)?;
        let canonical_body = canonicalize_metadata_body(&body_json, &release_body_schema())?;
        Ok(hash_release_metadata(&canonical_body))
    }

    /// Validates and admits this candidate as immutable historical metadata.
    ///
    /// The exact Release schema authority must be uniquely available. The
    /// target Revision and its Project State must both resolve as admitted
    /// objects belonging to `project_id`; Resource bytes are not consulted.
    ///
    /// # Errors
    ///
    /// Returns an error for an unavailable or non-unique schema authority,
    /// invalid name or timestamp, schema rejection, unresolved or mismatched
    /// target metadata, a cross-Project target, or invalid canonical data.
    pub fn admit(
        self,
        schemas: &[&dyn ReleaseSchemaValidator],
        revisions: &dyn AdmittedRevisionResolver,
        project_states: &dyn AdmittedProjectStateResolver,
    ) -> Result<Release, ReleaseAdmissionError> {
        if self.schema != RELEASE_SCHEMA {
            return Err(ReleaseAdmissionError::UnavailableSchema);
        }

        let mut matching_schemas = schemas
            .iter()
            .copied()
            .filter(|schema| schema.schema() == self.schema);
        let schema = matching_schemas
            .next()
            .ok_or(ReleaseAdmissionError::UnavailableSchema)?;
        if matching_schemas.next().is_some() {
            return Err(ReleaseAdmissionError::NonUniqueSchemaAuthority);
        }

        if self.name.is_empty() {
            return Err(ReleaseAdmissionError::InvalidName);
        }
        if !is_valid_utc_nanosecond_timestamp(&self.created_at) {
            return Err(ReleaseAdmissionError::InvalidTimestamp);
        }

        let revision = revisions
            .resolve_admitted(self.revision_id)
            .ok_or(ReleaseAdmissionError::UnavailableRevision)?;
        if revision.revision_id() != self.revision_id {
            return Err(ReleaseAdmissionError::RevisionIdentifierMismatch);
        }
        let project_state_id = revision.project_state_id();
        let project_state = project_states
            .resolve_admitted(project_state_id)
            .ok_or(ReleaseAdmissionError::UnavailableProjectState)?;
        if project_state.project_state_id() != project_state_id {
            return Err(ReleaseAdmissionError::ProjectStateIdentifierMismatch);
        }
        if project_state.project_id() != self.project_id {
            return Err(ReleaseAdmissionError::TargetProjectMismatch);
        }

        schema
            .validate_release(&self)
            .map_err(ReleaseAdmissionError::SchemaRejected)?;

        let body = self.body_value();
        let body_json =
            serde_json::to_vec(&body).map_err(|_| CanonicalMetadataError::Canonicalization)?;
        let canonical_body = canonicalize_metadata_body(&body_json, &release_body_schema())?;
        let id = hash_release_metadata(&canonical_body);

        Ok(Release {
            schema: self.schema,
            project_id: self.project_id,
            name: self.name,
            revision_id: self.revision_id,
            created_at: self.created_at,
            creator_id: self.creator_id,
            description: self.description,
            canonical_body: canonical_body.into_boxed_slice(),
            id,
        })
    }

    fn body_value(&self) -> Value {
        let mut body = serde_json::Map::new();
        body.insert("schema".to_owned(), Value::String(self.schema.clone()));
        body.insert(
            "project_id".to_owned(),
            Value::String(self.project_id.to_string()),
        );
        body.insert("name".to_owned(), Value::String(self.name.clone()));
        body.insert(
            "revision_id".to_owned(),
            Value::String(self.revision_id.to_string()),
        );
        body.insert(
            "created_at".to_owned(),
            Value::String(self.created_at.clone()),
        );
        body.insert(
            "creator_id".to_owned(),
            Value::String(self.creator_id.to_string()),
        );
        body.insert(
            "description".to_owned(),
            Value::String(self.description.clone()),
        );
        Value::Object(body)
    }
}

impl<'de> Deserialize<'de> for ReleaseCandidate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = Box::<RawValue>::deserialize(deserializer)?;
        let raw_text = raw.get();
        if !raw_text.trim_start().starts_with('{') {
            return Err(D::Error::custom("Release candidate must be a JSON object"));
        }
        validate_unique_json_member_names(raw_text.as_bytes()).map_err(D::Error::custom)?;
        let wire: ReleaseCandidateWire =
            serde_json::from_str(raw_text).map_err(D::Error::custom)?;
        Ok(Self {
            schema: wire.schema,
            project_id: wire.project_id.parse().map_err(D::Error::custom)?,
            name: wire.name,
            revision_id: wire.revision_id.parse().map_err(D::Error::custom)?,
            created_at: wire.created_at,
            creator_id: wire.creator_id.parse().map_err(D::Error::custom)?,
            description: wire.description,
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReleaseCandidateWire {
    schema: String,
    project_id: String,
    name: String,
    revision_id: String,
    created_at: String,
    creator_id: String,
    description: String,
}

fn release_body_schema() -> MetadataSchema {
    MetadataSchema::structure([
        ("schema", MetadataSchema::Scalar),
        ("project_id", MetadataSchema::Scalar),
        ("name", MetadataSchema::Scalar),
        ("revision_id", MetadataSchema::Scalar),
        ("created_at", MetadataSchema::Scalar),
        ("creator_id", MetadataSchema::Scalar),
        ("description", MetadataSchema::Scalar),
    ])
}

/// An admitted, immutable Release whose exact seven-member body defines its
/// typed content-derived identity.
#[derive(Debug, Clone)]
pub struct Release {
    schema: String,
    project_id: ProjectId,
    name: String,
    revision_id: RevisionId,
    created_at: String,
    creator_id: ActorId,
    description: String,
    canonical_body: Box<[u8]>,
    id: ReleaseId,
}

impl PartialEq for Release {
    fn eq(&self, other: &Self) -> bool {
        self.canonical_body == other.canonical_body
    }
}

impl Eq for Release {}

impl Release {
    /// Returns the exact versioned Release schema identifier.
    #[must_use]
    pub fn schema(&self) -> &str {
        &self.schema
    }

    /// Returns the owning Project identifier.
    #[must_use]
    pub const fn project_id(&self) -> ProjectId {
        self.project_id
    }

    /// Returns the exact immutable Release name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the immutable direct target Revision identifier.
    ///
    /// This is the root edge consumed by future reachability traversal; this
    /// model does not traverse it.
    #[must_use]
    pub const fn revision_id(&self) -> RevisionId {
        self.revision_id
    }

    /// Returns the exact UTC nanosecond creation timestamp.
    #[must_use]
    pub fn created_at(&self) -> &str {
        &self.created_at
    }

    /// Returns the direct creator Actor identifier.
    #[must_use]
    pub const fn creator_id(&self) -> ActorId {
        self.creator_id
    }

    /// Returns the exact immutable description.
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns canonical RFC 8785 bytes of the complete seven-member body.
    #[must_use]
    pub fn canonical_body(&self) -> &[u8] {
        &self.canonical_body
    }

    /// Returns the typed content-derived Release Identifier.
    #[must_use]
    pub const fn release_id(&self) -> ReleaseId {
        self.id
    }
}

/// A failure to admit a Release candidate as historical metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReleaseAdmissionError {
    /// No available authority handles the exact OMVCS 0.1 Release schema.
    UnavailableSchema,
    /// Multiple authorities claim the exact Release schema.
    NonUniqueSchemaAuthority,
    /// The exact schema authority rejected the candidate.
    SchemaRejected(String),
    /// A Release name is empty.
    InvalidName,
    /// `created_at` is not a valid Core §15 UTC nanosecond timestamp.
    InvalidTimestamp,
    /// The target Revision does not resolve as admitted.
    UnavailableRevision,
    /// The resolver returned a Revision with a different identifier.
    RevisionIdentifierMismatch,
    /// The target Revision's Project State does not resolve as admitted.
    UnavailableProjectState,
    /// The resolver returned a Project State with a different identifier.
    ProjectStateIdentifierMismatch,
    /// The target Revision belongs to another Project.
    TargetProjectMismatch,
    /// Generic metadata or canonical validation failed.
    Canonical(CanonicalMetadataError),
}

impl fmt::Display for ReleaseAdmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnavailableSchema => formatter.write_str("Release schema is unavailable"),
            Self::NonUniqueSchemaAuthority => {
                formatter.write_str("Release schema authority is non-unique")
            }
            Self::SchemaRejected(reason) => write!(formatter, "Release schema rejected: {reason}"),
            Self::InvalidName => formatter.write_str("Release name must not be empty"),
            Self::InvalidTimestamp => {
                formatter.write_str("created_at is not a valid UTC nanosecond timestamp")
            }
            Self::UnavailableRevision => formatter.write_str("target Revision is unavailable"),
            Self::RevisionIdentifierMismatch => {
                formatter.write_str("resolved Revision identifier does not match reference")
            }
            Self::UnavailableProjectState => {
                formatter.write_str("Revision Project State is unavailable")
            }
            Self::ProjectStateIdentifierMismatch => {
                formatter.write_str("resolved Project State identifier does not match reference")
            }
            Self::TargetProjectMismatch => {
                formatter.write_str("target Revision belongs to a different Project")
            }
            Self::Canonical(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for ReleaseAdmissionError {}

impl From<CanonicalMetadataError> for ReleaseAdmissionError {
    fn from(error: CanonicalMetadataError) -> Self {
        Self::Canonical(error)
    }
}
