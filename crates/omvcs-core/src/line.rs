//! Line history references and their atomic Core operation boundary.
//!
//! This module models only Line operational metadata. It does not create or
//! mutate Revisions, implement durable storage, or define retention policy.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::Mutex;

use omvcs_model::revision::{AdmittedProjectStateResolver, AdmittedRevisionResolver};
use omvcs_model::{LineId, ProjectId, RevisionId};
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer, ser::SerializeStruct};

/// Largest exact non-negative JSON integer admitted for a Line generation.
pub const MAX_LINE_GENERATION: u64 = 9_007_199_254_740_991;

/// The exact interoperable generation value for a mutable Line record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LineGeneration(u64);

impl LineGeneration {
    /// The generation assigned to a newly created Line.
    pub const ZERO: Self = Self(0);

    /// Returns the numeric value of this generation.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    fn next(self) -> Option<Self> {
        (self.0 < MAX_LINE_GENERATION).then(|| Self(self.0 + 1))
    }
}

impl TryFrom<u64> for LineGeneration {
    type Error = LineOperationError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value <= MAX_LINE_GENERATION {
            Ok(Self(value))
        } else {
            Err(LineOperationError::InvalidGeneration)
        }
    }
}

impl Serialize for LineGeneration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(self.0)
    }
}

impl<'de> Deserialize<'de> for LineGeneration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u64::deserialize(deserializer)?;
        Self::try_from(value).map_err(D::Error::custom)
    }
}

/// The closed OMVCS 0.1 mutable Line record.
///
/// Its exact serialized member set is `line_id`, `project_id`, `name`,
/// `target_revision`, and `generation`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    id: LineId,
    project_id: ProjectId,
    name: String,
    target_revision: RevisionId,
    generation: LineGeneration,
}

impl Line {
    /// Returns the stable assigned identifier of this Line.
    #[must_use]
    pub const fn line_id(&self) -> LineId {
        self.id
    }

    /// Returns the immutable Project association of this Line.
    #[must_use]
    pub const fn project_id(&self) -> ProjectId {
        self.project_id
    }

    /// Returns the current exact Line name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the current admitted Revision target.
    #[must_use]
    pub const fn target_revision(&self) -> RevisionId {
        self.target_revision
    }

    /// Returns the current mutation generation.
    #[must_use]
    pub const fn generation(&self) -> LineGeneration {
        self.generation
    }
}

impl Serialize for Line {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut record = serializer.serialize_struct("Line", 5)?;
        record.serialize_field("line_id", &self.id.to_string())?;
        record.serialize_field("project_id", &self.project_id.to_string())?;
        record.serialize_field("name", &self.name)?;
        record.serialize_field("target_revision", &self.target_revision.to_string())?;
        record.serialize_field("generation", &self.generation)?;
        record.end()
    }
}

impl<'de> Deserialize<'de> for Line {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct LineWire {
            line_id: String,
            project_id: String,
            name: String,
            target_revision: String,
            generation: LineGeneration,
        }

        let wire = LineWire::deserialize(deserializer)?;
        Ok(Self {
            id: wire.line_id.parse().map_err(D::Error::custom)?,
            project_id: wire.project_id.parse().map_err(D::Error::custom)?,
            name: wire.name,
            target_revision: wire.target_revision.parse().map_err(D::Error::custom)?,
            generation: wire.generation,
        })
    }
}

/// A failure returned by a Line operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineOperationError {
    /// The Project does not exist in this repository.
    ProjectNotFound,
    /// The requested Line does not exist.
    LineNotFound,
    /// A target identifier does not resolve to admitted Revision metadata.
    RevisionNotAdmitted,
    /// A resolver returned a Revision different from the requested identifier.
    RevisionIdentifierMismatch,
    /// The target Revision's Project State is not admitted or resolvable.
    ProjectStateNotAdmitted,
    /// A Project State resolver returned a different identifier.
    ProjectStateIdentifierMismatch,
    /// The requested target belongs to a different Project.
    TargetProjectMismatch,
    /// Another Line in this Project already has the exact requested name.
    NameConflict,
    /// An expected Line generation is not the current generation.
    StaleGeneration,
    /// An expected target Revision is not the current target.
    StaleTarget,
    /// An expected Default Line does not equal the current preference.
    StaleDefaultLine,
    /// A Line being selected as Default belongs to a different Project.
    DefaultLineProjectMismatch,
    /// The current Default Line may not be deleted.
    DefaultLineProtected,
    /// A Line mutation cannot increment the maximum generation.
    GenerationExhausted,
    /// A raw generation value lies outside the interoperable domain.
    InvalidGeneration,
    /// The repository lock was poisoned after a thread panic.
    RepositoryUnavailable,
}

impl fmt::Display for LineOperationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::ProjectNotFound => "Project does not exist",
            Self::LineNotFound => "Line does not exist",
            Self::RevisionNotAdmitted => "target Revision is not admitted",
            Self::RevisionIdentifierMismatch => "Revision resolver returned a different identifier",
            Self::ProjectStateNotAdmitted => "Revision Project State is not admitted",
            Self::ProjectStateIdentifierMismatch => {
                "Project State resolver returned a different identifier"
            }
            Self::TargetProjectMismatch => "target Revision belongs to a different Project",
            Self::NameConflict => "Line name is already used in this Project",
            Self::StaleGeneration => "Line generation does not match the expected value",
            Self::StaleTarget => "Line target does not match the expected value",
            Self::StaleDefaultLine => "Default Line does not match the expected value",
            Self::DefaultLineProjectMismatch => "Default Line must belong to the specified Project",
            Self::DefaultLineProtected => "the current Default Line cannot be deleted",
            Self::GenerationExhausted => "Line generation cannot be incremented",
            Self::InvalidGeneration => "Line generation is outside the exact JSON integer domain",
            Self::RepositoryUnavailable => "Line repository is unavailable",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for LineOperationError {}

/// Atomic repository operation contract for mutable Line metadata.
///
/// Each mutation is one indivisible decision: all validation, compare-and-swap
/// checks, uniqueness/default-reference checks, and the resulting update or
/// removal either succeed together or leave repository state unchanged.
///
/// Authorization: this boundary is not an authorization mechanism. A Core
/// layer with a configured authorization mechanism should invoke it before
/// dispatching an operation.
///
/// Historical and Working State effects: none for any operation.
///
/// Operational effects are specified on each operation below. Read operations
/// have no effects. Mutations are not automatically retried, and this
/// boundary provides no general idempotency key or retry-on-behalf behavior;
/// compare-and-swap retries must use the caller's supplied expectations.
///
/// Implementations backed by Repository Home must persist the Default Line
/// there; the in-memory implementation below is only a contract reference
/// and test double.
pub trait LineOperationBoundary: Send + Sync {
    /// Atomically creates a generation-zero Line after validating its admitted
    /// target and Project-scoped exact name uniqueness.
    ///
    /// Historical and Working State effects: none.
    ///
    /// Operational effect: creates exactly one complete Line.
    ///
    /// Retry and idempotency: this operation does not retry automatically and
    /// does not deduplicate requests by an idempotency key. After an uncertain
    /// result, the caller must inspect current repository state before
    /// deciding whether to try again; a successful first creation ordinarily
    /// makes a same-name retry fail uniqueness.
    ///
    /// # Errors
    ///
    /// Returns an error if the Project or admitted target is unavailable,
    /// the target belongs to another Project, the name is already used, or
    /// the repository cannot complete the atomic operation.
    fn create_line(
        &self,
        project_id: ProjectId,
        name: String,
        target_revision: RevisionId,
        revisions: &dyn AdmittedRevisionResolver,
        project_states: &dyn AdmittedProjectStateResolver,
    ) -> Result<Line, LineOperationError>;

    /// Returns the current Line record, if present. Inputs: `line_id`.
    /// Preconditions: none beyond repository availability. Authorization is
    /// handled by the calling Core layer as described on this trait.
    /// Historical, operational, and Working State effects: none. The result
    /// is the current Line or `None`; repeating the read is safe and has no
    /// mutation to retry.
    ///
    /// # Errors
    ///
    /// Returns an error if the repository cannot be read.
    fn get_line(&self, line_id: LineId) -> Result<Option<Line>, LineOperationError>;

    /// Atomically compares expected target and generation, then changes only
    /// the target and increments generation once.
    ///
    /// Historical and Working State effects: none.
    ///
    /// Operational effect: changes the target and generation as described
    /// above and returns the updated Line.
    ///
    /// Retry and idempotency: a retry with the old expected target or
    /// generation conflicts after a successful mutation, including a
    /// successful same-target move. The caller must obtain current state
    /// before deciding whether to issue another operation.
    ///
    /// # Errors
    ///
    /// Returns an error if the Line is missing, either expected value is
    /// stale, the new target is not admitted in the same Project, generation
    /// is exhausted, or the repository cannot complete the operation.
    fn move_line(
        &self,
        line_id: LineId,
        expected_target: RevisionId,
        expected_generation: LineGeneration,
        new_target: RevisionId,
        revisions: &dyn AdmittedRevisionResolver,
        project_states: &dyn AdmittedProjectStateResolver,
    ) -> Result<Line, LineOperationError>;

    /// Atomically compares generation and exact Project-scoped name
    /// uniqueness, then changes only name and increments generation once.
    ///
    /// Historical and Working State effects: none.
    ///
    /// Operational effect: changes the name and generation as described
    /// above and returns the updated Line.
    ///
    /// Retry and idempotency: a retry with the old expected generation
    /// conflicts after a successful mutation, including a successful
    /// same-name rename. The caller must obtain current state before deciding
    /// whether to issue another operation.
    ///
    /// # Errors
    ///
    /// Returns an error if the Line is missing, generation is stale, the
    /// requested name is already used, generation is exhausted, or the
    /// repository cannot complete the operation.
    fn rename_line(
        &self,
        line_id: LineId,
        expected_generation: LineGeneration,
        new_name: String,
    ) -> Result<Line, LineOperationError>;

    /// Atomically compares the expected shared Default Line, validates the
    /// requested Line, and sets or clears the repository-owned preference.
    ///
    /// Historical and Working State effects: none. No Line generation is
    /// changed.
    ///
    /// Operational effect: changes or clears only the Project's Default Line
    /// preference and returns `()`.
    ///
    /// Retry and idempotency: a stale expected value conflicts; the caller
    /// must read the current preference before deciding whether to issue
    /// another operation. Repeating a request does not bypass the supplied
    /// expected-current-value comparison.
    ///
    /// # Errors
    ///
    /// Returns an error if the Project or requested Line is missing, the
    /// expected value is stale, the Line belongs to another Project, or the
    /// repository cannot complete the operation.
    fn set_default_line(
        &self,
        project_id: ProjectId,
        expected_current: Option<LineId>,
        requested: Option<LineId>,
    ) -> Result<(), LineOperationError>;

    /// Returns the repository's current Default Line for an existing Project.
    /// Inputs: `project_id`. Preconditions: the Project exists. Authorization
    /// is handled by the calling Core layer as described on this trait.
    /// Historical, operational, and Working State effects: none. The result
    /// is the current optional Line ID; repeating the read is safe and has no
    /// mutation to retry.
    ///
    /// # Errors
    ///
    /// Returns an error if the Project does not exist or the repository
    /// cannot be read.
    fn default_line(&self, project_id: ProjectId) -> Result<Option<LineId>, LineOperationError>;

    /// Atomically compares existence and generation, protects the current
    /// Default Line, and removes only the specified Line record.
    ///
    /// Historical and Working State effects: none.
    ///
    /// Operational effect: removes exactly the Line record and returns `()`.
    ///
    /// Retry and idempotency: this operation does not retry automatically.
    /// After a stale-generation conflict, the caller must obtain and reason
    /// from current state. After success, a repeat is reported as missing;
    /// the boundary never retries using a newly observed generation.
    ///
    /// # Errors
    ///
    /// Returns an error if the Line is missing, generation is stale, the
    /// Line is the current Default Line, or the repository cannot complete
    /// the operation.
    fn delete_line(
        &self,
        line_id: LineId,
        expected_generation: LineGeneration,
    ) -> Result<(), LineOperationError>;
}

#[derive(Default)]
struct RepositoryState {
    projects: BTreeSet<ProjectId>,
    lines: BTreeMap<LineId, Line>,
    default_lines: BTreeMap<ProjectId, LineId>,
}

/// Mutex-backed reference implementation of the Line operation boundary.
///
/// This demonstrates the required atomic operation shape for model/Core
/// conformance. It is not a durable Repository Home implementation.
pub struct InMemoryLineRepository {
    state: Mutex<RepositoryState>,
}

impl InMemoryLineRepository {
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

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, RepositoryState>, LineOperationError> {
        self.state
            .lock()
            .map_err(|_| LineOperationError::RepositoryUnavailable)
    }
}

impl LineOperationBoundary for InMemoryLineRepository {
    fn create_line(
        &self,
        project_id: ProjectId,
        name: String,
        target_revision: RevisionId,
        revisions: &dyn AdmittedRevisionResolver,
        project_states: &dyn AdmittedProjectStateResolver,
    ) -> Result<Line, LineOperationError> {
        let target_project = admitted_revision_project(target_revision, revisions, project_states)?;
        if target_project != project_id {
            return Err(LineOperationError::TargetProjectMismatch);
        }

        let mut state = self.lock()?;
        if !state.projects.contains(&project_id) {
            return Err(LineOperationError::ProjectNotFound);
        }
        if state
            .lines
            .values()
            .any(|line| line.project_id == project_id && line.name == name)
        {
            return Err(LineOperationError::NameConflict);
        }

        let line = Line {
            id: LineId::new(),
            project_id,
            name,
            target_revision,
            generation: LineGeneration::ZERO,
        };
        state.lines.insert(line.id, line.clone());
        drop(state);
        Ok(line)
    }

    fn get_line(&self, line_id: LineId) -> Result<Option<Line>, LineOperationError> {
        Ok(self.lock()?.lines.get(&line_id).cloned())
    }

    fn move_line(
        &self,
        line_id: LineId,
        expected_target: RevisionId,
        expected_generation: LineGeneration,
        new_target: RevisionId,
        revisions: &dyn AdmittedRevisionResolver,
        project_states: &dyn AdmittedProjectStateResolver,
    ) -> Result<Line, LineOperationError> {
        let mut state = self.lock()?;
        let current = state
            .lines
            .get(&line_id)
            .ok_or(LineOperationError::LineNotFound)?;
        if current.target_revision != expected_target {
            return Err(LineOperationError::StaleTarget);
        }
        if current.generation != expected_generation {
            return Err(LineOperationError::StaleGeneration);
        }
        let target_project = admitted_revision_project(new_target, revisions, project_states)?;
        if target_project != current.project_id {
            return Err(LineOperationError::TargetProjectMismatch);
        }
        let next_generation = current
            .generation
            .next()
            .ok_or(LineOperationError::GenerationExhausted)?;

        let updated = state
            .lines
            .get_mut(&line_id)
            .ok_or(LineOperationError::LineNotFound)?;
        updated.target_revision = new_target;
        updated.generation = next_generation;
        let updated = updated.clone();
        drop(state);
        Ok(updated)
    }

    fn rename_line(
        &self,
        line_id: LineId,
        expected_generation: LineGeneration,
        new_name: String,
    ) -> Result<Line, LineOperationError> {
        let mut state = self.lock()?;
        let current = state
            .lines
            .get(&line_id)
            .ok_or(LineOperationError::LineNotFound)?;
        if current.generation != expected_generation {
            return Err(LineOperationError::StaleGeneration);
        }
        if state.lines.values().any(|line| {
            line.id != line_id && line.project_id == current.project_id && line.name == new_name
        }) {
            return Err(LineOperationError::NameConflict);
        }
        let next_generation = current
            .generation
            .next()
            .ok_or(LineOperationError::GenerationExhausted)?;

        let updated = state
            .lines
            .get_mut(&line_id)
            .ok_or(LineOperationError::LineNotFound)?;
        updated.name = new_name;
        updated.generation = next_generation;
        let updated = updated.clone();
        drop(state);
        Ok(updated)
    }

    fn set_default_line(
        &self,
        project_id: ProjectId,
        expected_current: Option<LineId>,
        requested: Option<LineId>,
    ) -> Result<(), LineOperationError> {
        let mut state = self.lock()?;
        if !state.projects.contains(&project_id) {
            return Err(LineOperationError::ProjectNotFound);
        }
        let current = state.default_lines.get(&project_id).copied();
        if current != expected_current {
            return Err(LineOperationError::StaleDefaultLine);
        }
        if let Some(line_id) = requested {
            let line = state
                .lines
                .get(&line_id)
                .ok_or(LineOperationError::LineNotFound)?;
            if line.project_id != project_id {
                return Err(LineOperationError::DefaultLineProjectMismatch);
            }
        }
        match requested {
            Some(line_id) => {
                state.default_lines.insert(project_id, line_id);
            }
            None => {
                state.default_lines.remove(&project_id);
            }
        }
        drop(state);
        Ok(())
    }

    fn default_line(&self, project_id: ProjectId) -> Result<Option<LineId>, LineOperationError> {
        let state = self.lock()?;
        if !state.projects.contains(&project_id) {
            return Err(LineOperationError::ProjectNotFound);
        }
        Ok(state.default_lines.get(&project_id).copied())
    }

    fn delete_line(
        &self,
        line_id: LineId,
        expected_generation: LineGeneration,
    ) -> Result<(), LineOperationError> {
        let mut state = self.lock()?;
        let current = state
            .lines
            .get(&line_id)
            .ok_or(LineOperationError::LineNotFound)?;
        if current.generation != expected_generation {
            return Err(LineOperationError::StaleGeneration);
        }
        if state.default_lines.get(&current.project_id) == Some(&line_id) {
            return Err(LineOperationError::DefaultLineProtected);
        }
        state.lines.remove(&line_id);
        drop(state);
        Ok(())
    }
}

fn admitted_revision_project(
    revision_id: RevisionId,
    revisions: &dyn AdmittedRevisionResolver,
    project_states: &dyn AdmittedProjectStateResolver,
) -> Result<ProjectId, LineOperationError> {
    let revision = revisions
        .resolve_admitted(revision_id)
        .ok_or(LineOperationError::RevisionNotAdmitted)?;
    if revision.revision_id() != revision_id {
        return Err(LineOperationError::RevisionIdentifierMismatch);
    }
    let project_state_id = revision.project_state_id();
    let project_state = project_states
        .resolve_admitted(project_state_id)
        .ok_or(LineOperationError::ProjectStateNotAdmitted)?;
    if project_state.project_state_id() != project_state_id {
        return Err(LineOperationError::ProjectStateIdentifierMismatch);
    }
    Ok(project_state.project_id())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use std::collections::BTreeMap;

    use super::{
        InMemoryLineRepository, LineGeneration, LineOperationBoundary, LineOperationError,
    };
    use omvcs_model::canonical::MetadataSchema;
    use omvcs_model::project_state::{
        AdmittedAdapterStateResolver, ProjectStateCandidate, ProjectStateSchemaValidator,
    };
    use omvcs_model::revision::{RevisionCandidate, RevisionSchemaValidator};
    use omvcs_model::{ActorId, AdapterStateId, ProjectId, ProjectState, ProjectStateId, Revision};
    use serde_json::{Value, json};

    const PROJECT_SCHEMA: &str = "line-unit.project-state/1";
    const REVISION_SCHEMA: &str = "line-unit.revision/1";
    const PROJECT_TEXT: &str = "019cc17d-1b22-7a41-9fe9-c345c468f82e";
    const ACTOR_TEXT: &str = "019cc17d-1b22-7a41-9fe9-c345c468f82c";

    struct ProjectSchema;

    impl ProjectStateSchemaValidator for ProjectSchema {
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

    struct RevisionSchema;

    impl RevisionSchemaValidator for RevisionSchema {
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

    fn admitted_data() -> (
        ProjectId,
        BTreeMap<ProjectStateId, ProjectState>,
        BTreeMap<omvcs_model::RevisionId, Revision>,
        omvcs_model::RevisionId,
    ) {
        let project_id = PROJECT_TEXT.parse().expect("Project ID");
        let adapter_id = AdapterStateId::from_digest([0x31; 32]);
        let candidate: ProjectStateCandidate = serde_json::from_value(json!({
            "schema": PROJECT_SCHEMA,
            "project_id": PROJECT_TEXT,
            "components": {},
            "adapter_state_id": adapter_id.to_string(),
            "project_metadata": {}
        }))
        .expect("Project State candidate");
        let project_state = candidate
            .admit(
                &[&ProjectSchema],
                &BTreeMap::new(),
                &AdapterResolver(adapter_id),
            )
            .expect("admitted Project State");
        let state_id = project_state.project_state_id();
        let states = BTreeMap::from([(state_id, project_state)]);
        let revision = RevisionCandidate::new(
            REVISION_SCHEMA,
            state_id,
            vec![],
            ACTOR_TEXT.parse::<ActorId>().expect("Actor ID"),
            "2026-10-09T07:00:00.000000000Z",
            "unit target",
            vec![],
        )
        .admit(&[&RevisionSchema], &states, &BTreeMap::new())
        .expect("admitted Revision");
        let revision_id = revision.revision_id();
        let revisions = BTreeMap::from([(revision_id, revision)]);
        (project_id, states, revisions, revision_id)
    }

    #[test]
    fn move_and_rename_at_maximum_generation_fail_without_mutation() {
        let (project_id, states, revisions, target_id) = admitted_data();
        let repository = InMemoryLineRepository::new([project_id]);
        let line = repository
            .create_line(
                project_id,
                "terminal".to_owned(),
                target_id,
                &revisions,
                &states,
            )
            .expect("create Line");
        let maximum =
            LineGeneration::try_from(super::MAX_LINE_GENERATION).expect("maximum safe generation");
        {
            let mut state = repository.lock().expect("repository lock");
            state
                .lines
                .get_mut(&line.line_id())
                .expect("stored Line")
                .generation = maximum;
        }
        let before = repository
            .get_line(line.line_id())
            .expect("read Line")
            .expect("Line exists");

        assert_eq!(
            repository.move_line(
                line.line_id(),
                target_id,
                maximum,
                target_id,
                &revisions,
                &states
            ),
            Err(LineOperationError::GenerationExhausted)
        );
        assert_eq!(
            repository.rename_line(line.line_id(), maximum, "changed".to_owned()),
            Err(LineOperationError::GenerationExhausted)
        );
        assert_eq!(repository.get_line(line.line_id()), Ok(Some(before)));
    }

    #[test]
    fn delete_at_maximum_generation_succeeds_without_incrementing_or_touching_history() {
        let (project_id, states, revisions, target_id) = admitted_data();
        let repository = InMemoryLineRepository::new([project_id]);
        let line = repository
            .create_line(
                project_id,
                "terminal delete".to_owned(),
                target_id,
                &revisions,
                &states,
            )
            .expect("create Line");
        let maximum =
            LineGeneration::try_from(super::MAX_LINE_GENERATION).expect("maximum safe generation");
        repository
            .lock()
            .expect("repository lock")
            .lines
            .get_mut(&line.line_id())
            .expect("stored Line")
            .generation = maximum;

        repository
            .delete_line(line.line_id(), maximum)
            .expect("delete at current generation");
        assert_eq!(repository.get_line(line.line_id()), Ok(None));
        assert!(revisions.contains_key(&target_id));
    }
}
