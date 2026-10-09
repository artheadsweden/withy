//! Atomic Release creation and lookup at the Core operation boundary.
//!
//! This mutex-backed reference implementation demonstrates atomic admission,
//! Project-scoped name uniqueness, and exact idempotent retries. It is not a
//! durable Repository Home implementation.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::Mutex;

use omvcs_model::release::{ReleaseAdmissionError, ReleaseCandidate, ReleaseSchemaValidator};
use omvcs_model::revision::{AdmittedProjectStateResolver, AdmittedRevisionResolver};
use omvcs_model::{ProjectId, Release, ReleaseId};

#[derive(Default)]
struct RepositoryState {
    projects: BTreeSet<ProjectId>,
    releases: BTreeMap<ReleaseId, Release>,
}

/// Atomic operation contract for immutable Release metadata.
pub trait ReleaseOperationBoundary {
    /// Creates and admits a Release atomically.
    ///
    /// An exact duplicate succeeds idempotently only when the typed identifier,
    /// complete canonical bytes, and Project/name binding all match. A name
    /// already bound to another Release conflicts. A stored Release that
    /// disagrees with the requested identifier or body is an integrity failure.
    /// If both identifier/body integrity failure and name conflict apply,
    /// integrity failure is returned first. Any error leaves the repository's
    /// Release set unchanged.
    ///
    /// Resource bytes are not required or accessed.
    ///
    /// # Errors
    ///
    /// Returns an error when the Project does not exist, the candidate fails
    /// Release admission, the Project-scoped name is occupied, stored metadata
    /// violates its identifier/body pairing, or the repository is unavailable.
    fn create_release(
        &self,
        candidate: ReleaseCandidate,
        schemas: &[&dyn ReleaseSchemaValidator],
        revisions: &dyn AdmittedRevisionResolver,
        project_states: &dyn AdmittedProjectStateResolver,
    ) -> Result<Release, ReleaseOperationError>;

    /// Returns an admitted Release by its typed content-derived identifier.
    ///
    /// # Errors
    ///
    /// Returns an error if the repository cannot be read.
    fn get_release(&self, release_id: ReleaseId) -> Result<Option<Release>, ReleaseOperationError>;
}

/// Mutex-backed reference implementation of the Release operation boundary.
pub struct InMemoryReleaseRepository {
    state: Mutex<RepositoryState>,
}

/// Read-only enumeration of all admitted Releases across every repository Project.
///
/// Success MUST return the complete admitted set for this read, not a selected
/// Project or page. Failure MUST NOT masquerade as an empty or truncated set.
/// This does not promise a snapshot shared with other repositories or reads.
pub trait ReleaseEnumerationBoundary {
    /// Returns all admitted immutable Releases without modifying them.
    ///
    /// # Errors
    ///
    /// Returns an error when complete enumeration cannot be performed.
    fn admitted_releases(&self) -> Result<Vec<Release>, ReleaseOperationError>;
}

impl ReleaseEnumerationBoundary for InMemoryReleaseRepository {
    fn admitted_releases(&self) -> Result<Vec<Release>, ReleaseOperationError> {
        Ok(self.lock()?.releases.values().cloned().collect())
    }
}

impl InMemoryReleaseRepository {
    /// Creates a reference repository with the supplied existing Projects.
    #[must_use]
    pub fn new(projects: impl IntoIterator<Item = ProjectId>) -> Self {
        Self {
            state: Mutex::new(RepositoryState {
                projects: projects.into_iter().collect(),
                ..RepositoryState::default()
            }),
        }
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, RepositoryState>, ReleaseOperationError> {
        self.state
            .lock()
            .map_err(|_| ReleaseOperationError::RepositoryUnavailable)
    }
}

impl ReleaseOperationBoundary for InMemoryReleaseRepository {
    fn create_release(
        &self,
        candidate: ReleaseCandidate,
        schemas: &[&dyn ReleaseSchemaValidator],
        revisions: &dyn AdmittedRevisionResolver,
        project_states: &dyn AdmittedProjectStateResolver,
    ) -> Result<Release, ReleaseOperationError> {
        let mut state = self.lock()?;
        if !state.projects.contains(&candidate.project_id()) {
            return Err(ReleaseOperationError::ProjectNotFound);
        }

        let release = candidate.admit(schemas, revisions, project_states)?;
        let release_id = release.release_id();

        if let Some(existing) = state.releases.get(&release_id) {
            if existing.release_id() != release_id
                || existing.canonical_body() != release.canonical_body()
                || existing.project_id() != release.project_id()
                || existing.name() != release.name()
            {
                return Err(ReleaseOperationError::IdentifierIntegrityViolation);
            }
        }

        if state.releases.values().any(|existing| {
            existing.project_id() == release.project_id()
                && existing.name() == release.name()
                && existing.release_id() != release_id
        }) {
            return Err(ReleaseOperationError::NameConflict);
        }

        if let Some(existing) = state.releases.get(&release_id) {
            return Ok(existing.clone());
        }

        state.releases.insert(release_id, release.clone());
        drop(state);
        Ok(release)
    }

    fn get_release(&self, release_id: ReleaseId) -> Result<Option<Release>, ReleaseOperationError> {
        Ok(self.lock()?.releases.get(&release_id).cloned())
    }
}

/// A failure returned by a Release operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReleaseOperationError {
    /// The Project does not exist in this repository.
    ProjectNotFound,
    /// Candidate body failed its exact Release admission checks.
    Admission(ReleaseAdmissionError),
    /// Another Release in this Project already has the exact requested name.
    NameConflict,
    /// A stored Release does not match its typed key or candidate body.
    IdentifierIntegrityViolation,
    /// The repository lock was poisoned after a thread panic.
    RepositoryUnavailable,
}

impl fmt::Display for ReleaseOperationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProjectNotFound => formatter.write_str("Project does not exist"),
            Self::Admission(error) => write!(formatter, "{error}"),
            Self::NameConflict => formatter.write_str("Release name is already used in Project"),
            Self::IdentifierIntegrityViolation => {
                formatter.write_str("stored Release identifier/body integrity violation")
            }
            Self::RepositoryUnavailable => formatter.write_str("Release repository is unavailable"),
        }
    }
}

impl std::error::Error for ReleaseOperationError {}

impl From<ReleaseAdmissionError> for ReleaseOperationError {
    fn from(error: ReleaseAdmissionError) -> Self {
        Self::Admission(error)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::{InMemoryReleaseRepository, ReleaseOperationBoundary, ReleaseOperationError};
    use omvcs_model::canonical::MetadataSchema;
    use omvcs_model::project_state::{
        AdmittedAdapterStateResolver, ProjectStateCandidate, ProjectStateSchemaValidator,
    };
    use omvcs_model::release::{ReleaseCandidate, ReleaseSchemaValidator};
    use omvcs_model::revision::{
        AdmittedProjectStateResolver, AdmittedRevisionResolver, RevisionCandidate,
        RevisionSchemaValidator,
    };
    use omvcs_model::{
        ActorId, AdapterStateId, ProjectId, ProjectState, ProjectStateId, Revision, RevisionId,
    };
    use serde_json::{Value, json};
    use std::collections::BTreeMap;

    const PROJECT_SCHEMA: &str = "release-test.project-state/1";
    const REVISION_SCHEMA: &str = "release-test.revision/1";
    struct TestSchema;

    impl ReleaseSchemaValidator for TestSchema {
        fn schema(&self) -> &str {
            omvcs_model::release::RELEASE_SCHEMA
        }

        fn validate_release(&self, _: &ReleaseCandidate) -> Result<(), String> {
            Ok(())
        }
    }

    struct TestProjectSchema;

    impl ProjectStateSchemaValidator for TestProjectSchema {
        fn schema(&self) -> &str {
            PROJECT_SCHEMA
        }

        fn project_metadata_schema(&self) -> MetadataSchema {
            MetadataSchema::structure(std::iter::empty::<(String, MetadataSchema)>())
        }

        fn validate_project_metadata(&self, _: &BTreeMap<String, Value>) -> Result<(), String> {
            Ok(())
        }
    }

    struct TestRevisionSchema;

    impl RevisionSchemaValidator for TestRevisionSchema {
        fn schema(&self) -> &str {
            REVISION_SCHEMA
        }

        fn provenance_entry_schema(&self) -> MetadataSchema {
            MetadataSchema::structure(std::iter::empty::<(String, MetadataSchema)>())
        }

        fn validate_provenance(&self, _: &[Value]) -> Result<(), String> {
            Ok(())
        }
    }

    struct AdapterResolver(AdapterStateId);

    impl AdmittedAdapterStateResolver for AdapterResolver {
        fn resolve_admitted(&self, id: AdapterStateId) -> Option<AdapterStateId> {
            (id == self.0).then_some(id)
        }
    }

    struct RevisionResolver(BTreeMap<RevisionId, Revision>);

    impl AdmittedRevisionResolver for RevisionResolver {
        fn resolve_admitted(&self, id: RevisionId) -> Option<&Revision> {
            self.0.get(&id)
        }
    }

    struct ProjectStateResolver(BTreeMap<ProjectStateId, ProjectState>);

    impl AdmittedProjectStateResolver for ProjectStateResolver {
        fn resolve_admitted(&self, id: ProjectStateId) -> Option<&ProjectState> {
            self.0.get(&id)
        }
    }

    fn admitted_fixture() -> (
        ProjectId,
        ProjectStateResolver,
        RevisionResolver,
        RevisionId,
    ) {
        let project_id = "019cc17d-1b22-7a41-9fe9-c345c468f82e"
            .parse::<ProjectId>()
            .expect("valid ProjectId");
        let actor_id = "019cc17d-1b22-7a41-9fe9-c345c468f82c"
            .parse::<ActorId>()
            .expect("valid ActorId");
        let adapter_id = AdapterStateId::from_digest([0x31; 32]);
        let candidate: ProjectStateCandidate = serde_json::from_value(json!({
            "schema": PROJECT_SCHEMA,
            "project_id": project_id.to_string(),
            "components": {},
            "adapter_state_id": adapter_id.to_string(),
            "project_metadata": {}
        }))
        .expect("valid Project State candidate");
        let state: ProjectState = candidate
            .admit(
                &[&TestProjectSchema],
                &BTreeMap::new(),
                &AdapterResolver(adapter_id),
            )
            .expect("admitted Project State");
        let state_id = state.project_state_id();
        let states = ProjectStateResolver(BTreeMap::from([(state_id, state)]));
        let revision = RevisionCandidate::new(
            REVISION_SCHEMA,
            state_id,
            vec![],
            actor_id,
            "2026-10-08T11:02:17.000000000Z",
            "release target",
            vec![],
        )
        .admit(&[&TestRevisionSchema], &states.0, &BTreeMap::new())
        .expect("admitted target Revision");
        let revision_id = revision.revision_id();
        let revisions = RevisionResolver(BTreeMap::from([(revision_id, revision)]));
        (project_id, states, revisions, revision_id)
    }

    fn actor_id() -> ActorId {
        "019cc17d-1b22-7a41-9fe9-c345c468f82c"
            .parse()
            .expect("valid ActorId")
    }

    fn release_candidate(
        project_id: ProjectId,
        revision_id: RevisionId,
        name: &str,
        description: &str,
    ) -> ReleaseCandidate {
        ReleaseCandidate::new(
            project_id,
            name,
            revision_id,
            "2026-10-08T11:02:17.000000000Z",
            actor_id(),
            description,
        )
    }

    #[test]
    fn a_corrupt_stored_identifier_binding_is_an_integrity_failure() {
        let (project_id, states, revisions, revision_id) = admitted_fixture();
        let repository = InMemoryReleaseRepository::new([project_id]);
        let first = release_candidate(project_id, revision_id, "first", "body one")
            .admit(&[&TestSchema], &revisions, &states)
            .expect("valid first Release");
        let second = release_candidate(project_id, revision_id, "second", "body two")
            .admit(&[&TestSchema], &revisions, &states)
            .expect("valid second Release");
        let first_id = first.release_id();
        let second_id = second.release_id();

        // Corrupt the in-memory key/body pairing to exercise detection of
        // different canonical bytes stored under an existing requested ID.
        repository
            .state
            .lock()
            .expect("repository lock")
            .releases
            .insert(first_id, second);

        assert_eq!(
            repository.create_release(
                release_candidate(project_id, revision_id, "first", "body one"),
                &[&TestSchema],
                &revisions,
                &states,
            ),
            Err(ReleaseOperationError::IdentifierIntegrityViolation)
        );
        let stored = repository
            .get_release(first_id)
            .expect("read corrupted record")
            .expect("corrupted record remains unchanged");
        assert_eq!(stored.release_id(), second_id);
        assert_ne!(stored.canonical_body(), first.canonical_body());
    }

    #[test]
    fn overlapping_identifier_integrity_and_name_conflict_reports_integrity_without_mutation() {
        let (project_id, states, revisions, revision_id) = admitted_fixture();
        let repository = InMemoryReleaseRepository::new([project_id]);

        let request = release_candidate(project_id, revision_id, "occupied", "requested body");
        let request_release = request
            .clone()
            .admit(&[&TestSchema], &revisions, &states)
            .expect("valid request body");
        let corrupted_record = release_candidate(project_id, revision_id, "other", "corrupt body")
            .admit(&[&TestSchema], &revisions, &states)
            .expect("valid conflicting stored body");
        let name_occupant = release_candidate(project_id, revision_id, "occupied", "name owner")
            .admit(&[&TestSchema], &revisions, &states)
            .expect("valid name occupant");
        let request_id = request_release.release_id();
        let corrupted_id = corrupted_record.release_id();
        let occupant_id = name_occupant.release_id();

        {
            let mut state = repository.state.lock().expect("repository lock");
            // The first mapping represents corrupt stored bytes under the
            // requested content-derived key; the second independently
            // occupies the requested Project/name binding.
            state.releases.insert(request_id, corrupted_record);
            state.releases.insert(occupant_id, name_occupant);
        }

        assert_eq!(
            repository.create_release(request, &[&TestSchema], &revisions, &states),
            Err(ReleaseOperationError::IdentifierIntegrityViolation)
        );

        let state = repository.state.lock().expect("repository lock");
        assert_eq!(state.releases.len(), 2);
        assert_eq!(state.releases[&request_id].release_id(), corrupted_id);
        assert_eq!(state.releases[&request_id].name(), "other");
        assert_eq!(state.releases[&occupant_id].release_id(), occupant_id);
        assert_eq!(state.releases[&occupant_id].name(), "occupied");
        assert_ne!(
            state.releases[&request_id].canonical_body(),
            request_release.canonical_body()
        );
    }

    #[test]
    fn an_unresolved_target_failure_does_not_claim_a_release_name() {
        let (project_id, states, revisions, revision_id) = admitted_fixture();
        let repository = InMemoryReleaseRepository::new([project_id]);
        let candidate = release_candidate(
            project_id,
            RevisionId::from_digest([1; 32]),
            "retryable",
            "",
        );
        assert!(matches!(
            repository.create_release(candidate, &[&TestSchema], &revisions, &states),
            Err(ReleaseOperationError::Admission(
                omvcs_model::release::ReleaseAdmissionError::UnavailableRevision
            ))
        ));

        let created = repository
            .create_release(
                release_candidate(project_id, revision_id, "retryable", ""),
                &[&TestSchema],
                &revisions,
                &states,
            )
            .expect("failed attempt left the name unclaimed");
        assert_eq!(
            repository.get_release(created.release_id()),
            Ok(Some(created))
        );
    }
}
