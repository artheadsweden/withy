//! Core Working State operational metadata and lifecycle operations.
//!
//! This module keeps mutable local work outside immutable history. Its
//! in-memory repository is a reference implementation of the logical Core
//! commit boundary; durable storage and DAW/provider mechanics are adapters.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::{Arc, Mutex};

use omvcs_model::project_state::{ComponentStateResolver, ProjectState};
use omvcs_model::revision::{AdmittedProjectStateResolver, AdmittedRevisionResolver, Revision};
use omvcs_model::{ComponentStateId, CreativeComponentId, LineId, ProjectId, RevisionId};

use crate::line::{LineOperationBoundary, LineOperationError};

/// Derived Core comparison of the current Working State with its applicable
/// Base/source state. This is not DAW-native dirty/unsaved state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreChangeStatus {
    /// Core can establish semantic equality with the applicable source/base.
    Unchanged,
    /// Core can establish a relevant difference from the applicable source/base.
    Changed,
    /// Available evidence cannot establish equality or a relevant difference.
    Unknown,
}

impl CoreChangeStatus {
    /// Returns the normative machine-readable status token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unchanged => "unchanged",
            Self::Changed => "changed",
            Self::Unknown => "unknown",
        }
    }
}

/// Operational confidence in live Adapter state, separate from whether a
/// Working State or [`AdapterWorkingStateRef`] has been committed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkingStateRecoveryCondition {
    /// Live state is confirmed to correspond to the committed reference.
    Confirmed,
    /// A committed reference exists, but live correspondence is not confirmed.
    Unconfirmed,
    /// Adapter work may have left live state inconsistent with committed metadata.
    RecoveryRequired,
}

impl WorkingStateRecoveryCondition {
    /// Returns the normative machine-readable recovery token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Confirmed => "confirmed",
            Self::Unconfirmed => "unconfirmed",
            Self::RecoveryRequired => "recovery_required",
        }
    }
}

/// Authorization scoped to one Working State replacement invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReplacementAuthorization {
    /// Preserve current work; this is the default.
    #[default]
    Preserve,
    /// Explicitly authorize this invocation to replace current work.
    DiscardCurrentWorkingState,
}

/// Opaque Adapter-defined token for recoverable mutable Adapter Working State.
///
/// The bytes are only meaningful to the Adapter. They are not a `ResourceId`,
/// `AdapterStateId`, historical identity, provenance, or credential container.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct AdapterWorkingStateRef(Vec<u8>);

impl AdapterWorkingStateRef {
    /// Wraps the Adapter-defined opaque reference bytes.
    #[must_use]
    pub fn new(reference: impl Into<Vec<u8>>) -> Self {
        Self(reference.into())
    }

    /// Exposes the opaque token to the Adapter that owns its interpretation.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Debug for AdapterWorkingStateRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("AdapterWorkingStateRef([opaque])")
    }
}

/// Core-owned Working State metadata. This record has no assigned
/// `WorkingStateId` and no persisted `modified` Boolean.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkingState {
    project_id: ProjectId,
    base_revision_id: Option<RevisionId>,
    line_id: Option<LineId>,
    component_sources: BTreeMap<CreativeComponentId, Option<ComponentStateId>>,
    adapter_working_state_ref: Option<AdapterWorkingStateRef>,
}

/// Complete Core-owned operational record for persistence.
///
/// The store commits each value atomically. This type defines no wire
/// encoding or provider storage mechanics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistedWorkingState {
    project_id: ProjectId,
    working_state: Option<WorkingState>,
    recovery_condition: Option<WorkingStateRecoveryCondition>,
}

impl PersistedWorkingState {
    /// Returns the Project owning this operational record.
    #[must_use]
    pub const fn project_id(&self) -> ProjectId {
        self.project_id
    }

    /// Returns committed Working State metadata, if any. A recovery-only
    /// record does not fabricate a Working State or an Adapter reference.
    #[must_use]
    pub const fn working_state(&self) -> Option<&WorkingState> {
        self.working_state.as_ref()
    }

    /// Returns the separately persisted Adapter recovery condition.
    #[must_use]
    pub const fn recovery_condition(&self) -> Option<WorkingStateRecoveryCondition> {
        self.recovery_condition
    }
}

/// Errors from a Core Working State metadata persistence boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkingStatePersistenceError {
    /// The reference store could not load or atomically commit the record.
    Unavailable,
}

/// Reference-store contract for complete operational Working State records.
///
/// `commit` atomically replaces the persisted record for the Project. A
/// failed commit MUST leave its prior record authoritative. Implementations
/// own durable encoding and storage mechanics; Core owns the record contents
/// and logical commit order.
pub trait WorkingStateRecordStore: Send + Sync {
    /// Loads the latest committed record for a Project, if present.
    ///
    /// # Errors
    ///
    /// Returns `Unavailable` when the record cannot be loaded.
    fn load(
        &self,
        project_id: ProjectId,
    ) -> Result<Option<PersistedWorkingState>, WorkingStatePersistenceError>;

    /// Atomically commits the complete record for its Project.
    ///
    /// # Errors
    ///
    /// Returns `Unavailable` without replacing the prior committed record.
    fn commit(&self, record: &PersistedWorkingState) -> Result<(), WorkingStatePersistenceError>;
}

impl WorkingState {
    /// Returns the associated Project.
    #[must_use]
    pub const fn project_id(&self) -> ProjectId {
        self.project_id
    }

    /// Returns the optional Base Revision.
    #[must_use]
    pub const fn base_revision_id(&self) -> Option<RevisionId> {
        self.base_revision_id
    }

    /// Returns the optional associated Line.
    #[must_use]
    pub const fn line_id(&self) -> Option<LineId> {
        self.line_id
    }

    /// Returns the operational `CreativeComponentId -> Option<ComponentStateId>` map.
    #[must_use]
    pub const fn component_sources(
        &self,
    ) -> &BTreeMap<CreativeComponentId, Option<ComponentStateId>> {
        &self.component_sources
    }

    /// Returns the optional opaque Adapter reference currently committed by Core.
    #[must_use]
    pub const fn adapter_working_state_ref(&self) -> Option<&AdapterWorkingStateRef> {
        self.adapter_working_state_ref.as_ref()
    }
}

/// Read-only Working State view including derived status and recovery state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkingStateInspection {
    /// Core-owned committed metadata.
    pub working_state: WorkingState,
    /// Derived status; never inferred from native DAW dirty state.
    pub change_status: CoreChangeStatus,
    /// Separate operational recovery condition. Recovery may be required
    /// even when no Adapter reference has been committed.
    pub recovery_condition: Option<WorkingStateRecoveryCondition>,
}

/// A historical Component State selected as a source for one Working State
/// component. `revision_id` provides the Project context for validating it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HistoricalComponentSource {
    /// Same-Project admitted Revision in which this source occurs.
    pub revision_id: RevisionId,
    /// Admitted source Component State.
    pub component_state_id: ComponentStateId,
}

/// One component-source update in a custom Working State. `None` represents a
/// local component with no historical source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComponentSourceSelection {
    /// Component whose operational source mapping is updated.
    pub component_id: CreativeComponentId,
    /// Historical source, or `None` for a locally created component.
    pub source: Option<HistoricalComponentSource>,
}

/// Adapter-facing description of the target prepared mutable state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkingStatePreparation {
    /// Project whose mutable state is being prepared.
    pub project_id: ProjectId,
    /// Base Revision to commit, if this is full materialisation.
    pub base_revision_id: Option<RevisionId>,
    /// Optional Line association in the resulting record.
    pub line_id: Option<LineId>,
    /// Complete resulting operational source mapping.
    pub component_sources: BTreeMap<CreativeComponentId, Option<ComponentStateId>>,
}

/// Adapter preparation failure before a new reference is returned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterPrepareError {
    /// Adapter could not capture/prepare complete recoverable mutable state.
    CaptureFailed,
    /// Adapter could not stage the requested restoration.
    PrepareFailed,
}

/// Adapter validation result for a newly prepared opaque reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterReferenceError {
    /// The reference is malformed or invalid for this Project.
    Invalid,
    /// The reference points to state that is currently unavailable.
    Unavailable,
    /// The referenced state cannot be restored.
    Unrestorable,
}

/// Adapter restoration failure category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterRestoreError {
    /// Restoration failed without partially changing live state.
    Failed,
    /// The supplied reference is invalid.
    InvalidReference,
    /// The supplied reference is unavailable.
    ReferenceUnavailable,
    /// The supplied reference cannot be restored.
    ReferenceUnrestorable,
    /// Destructive Adapter work partially changed live state.
    PartialFailure,
}

/// Narrow Adapter boundary used by Core Working State operations.
///
/// Implementations own mutable state representation and all Adapter-specific
/// behavior. In particular, Core comparison evidence must not be implemented
/// by substituting native dirty/unsaved state.
pub trait WorkingStateAdapter: Send + Sync {
    /// Returns Adapter evidence for Core semantic comparison.
    fn detect_core_changes(&self, working_state: &WorkingState) -> CoreChangeStatus;

    /// Prepares a complete recoverable mutable state without replacing the
    /// currently committed Core reference. The Adapter must keep the old
    /// committed mutable state recoverable until Core commits the new
    /// reference.
    ///
    /// # Errors
    ///
    /// Returns an explicit capture or preparation failure.
    fn prepare_working_state(
        &self,
        preparation: &WorkingStatePreparation,
    ) -> Result<AdapterWorkingStateRef, AdapterPrepareError>;

    /// Validates a prepared reference before any destructive restoration.
    ///
    /// # Errors
    ///
    /// Returns the explicit Adapter validation failure for the supplied token.
    fn validate_working_state_ref(
        &self,
        project_id: ProjectId,
        reference: &AdapterWorkingStateRef,
    ) -> Result<(), AdapterReferenceError>;

    /// Restores the supplied reference into live mutable Adapter state.
    ///
    /// An implementation must report `PartialFailure` if it has destructively
    /// changed live state and cannot complete or roll back the operation.
    ///
    /// # Errors
    ///
    /// Returns a clean failure, reference failure, or partial destructive
    /// failure requiring recovery.
    fn restore_working_state(
        &self,
        project_id: ProjectId,
        reference: &AdapterWorkingStateRef,
    ) -> Result<(), AdapterRestoreError>;
}

#[derive(Default)]
struct RepositoryState {
    projects: BTreeSet<ProjectId>,
    working_states: BTreeMap<ProjectId, WorkingState>,
    recovery_conditions: BTreeMap<ProjectId, WorkingStateRecoveryCondition>,
}

/// Mutex-backed reference implementation of the Core Working State boundary.
///
/// Its lock covers validation-to-commit coordination and prevents concurrent
/// Core mutations from interleaving around Adapter work.
pub struct InMemoryWorkingStateRepository {
    state: Mutex<RepositoryState>,
    record_store: Option<Arc<dyn WorkingStateRecordStore>>,
}

impl InMemoryWorkingStateRepository {
    /// Creates a reference repository for the supplied existing Projects.
    #[must_use]
    pub fn new(projects: impl IntoIterator<Item = ProjectId>) -> Self {
        Self {
            state: Mutex::new(RepositoryState {
                projects: projects.into_iter().collect(),
                ..RepositoryState::default()
            }),
            record_store: None,
        }
    }

    /// Reconstructs repository state from a Core/reference-store boundary.
    ///
    /// A committed `recovery_required` condition is preserved. Other records
    /// with Adapter references become `unconfirmed` after reload; successful
    /// restoration is required before Core reports them confirmed. No
    /// serialization format or provider behavior is prescribed here.
    ///
    /// # Errors
    ///
    /// Returns `RepositoryUnavailable` if any record cannot be loaded.
    pub fn load_from_record_store(
        projects: impl IntoIterator<Item = ProjectId>,
        record_store: Arc<dyn WorkingStateRecordStore>,
    ) -> Result<Self, WorkingStateError> {
        let projects: BTreeSet<_> = projects.into_iter().collect();
        let mut state = RepositoryState {
            projects: projects.clone(),
            ..RepositoryState::default()
        };
        for project_id in projects {
            let Some(persisted) = record_store
                .load(project_id)
                .map_err(|_| WorkingStateError::RepositoryUnavailable)?
            else {
                continue;
            };
            if persisted.project_id != project_id
                || persisted
                    .working_state
                    .as_ref()
                    .is_some_and(|working_state| working_state.project_id != project_id)
            {
                return Err(WorkingStateError::PersistedRecordProjectMismatch);
            }
            let recovery_condition = if persisted.recovery_condition
                == Some(WorkingStateRecoveryCondition::RecoveryRequired)
            {
                Some(WorkingStateRecoveryCondition::RecoveryRequired)
            } else if persisted
                .working_state
                .as_ref()
                .is_some_and(|working_state| working_state.adapter_working_state_ref.is_some())
            {
                Some(WorkingStateRecoveryCondition::Unconfirmed)
            } else {
                None
            };
            if let Some(working_state) = persisted.working_state {
                state.working_states.insert(project_id, working_state);
            }
            if let Some(condition) = recovery_condition {
                state.recovery_conditions.insert(project_id, condition);
            }
        }
        Ok(Self {
            state: Mutex::new(state),
            record_store: Some(record_store),
        })
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, RepositoryState>, WorkingStateError> {
        self.state
            .lock()
            .map_err(|_| WorkingStateError::RepositoryUnavailable)
    }

    /// Creates the valid pre-first-Revision record. No synthetic Revision,
    /// operation identifier, or Adapter reference is created.
    ///
    /// # Errors
    ///
    /// Returns `ProjectNotFound`, `WorkingStateAlreadyExists`, or
    /// `RepositoryUnavailable`.
    pub fn create_initial_working_state(
        &self,
        project_id: ProjectId,
    ) -> Result<WorkingState, WorkingStateError> {
        let mut state = self.lock()?;
        require_project(&state, project_id)?;
        if state.working_states.contains_key(&project_id) {
            return Err(WorkingStateError::WorkingStateAlreadyExists);
        }
        let working_state = WorkingState {
            project_id,
            base_revision_id: None,
            line_id: None,
            component_sources: BTreeMap::new(),
            adapter_working_state_ref: None,
        };
        self.commit_persisted_record(
            &working_state,
            state.recovery_conditions.get(&project_id).copied(),
        )?;
        state
            .working_states
            .insert(project_id, working_state.clone());
        drop(state);
        Ok(working_state)
    }

    /// Read-only inspection. A status other than confirmed recovery is
    /// conservatively reported as `Unknown`.
    ///
    /// # Errors
    ///
    /// Returns `ProjectNotFound`, `WorkingStateNotFound`, or
    /// `RepositoryUnavailable`.
    pub fn inspect_working_state(
        &self,
        project_id: ProjectId,
        adapter: &dyn WorkingStateAdapter,
    ) -> Result<WorkingStateInspection, WorkingStateError> {
        let state = self.lock()?;
        require_project(&state, project_id)?;
        let working_state = state
            .working_states
            .get(&project_id)
            .ok_or(WorkingStateError::WorkingStateNotFound)?;
        let recovery_condition = state.recovery_conditions.get(&project_id).copied();
        let change_status = if working_state.adapter_working_state_ref.is_some()
            && recovery_condition == Some(WorkingStateRecoveryCondition::Confirmed)
        {
            adapter.detect_core_changes(working_state)
        } else {
            CoreChangeStatus::Unknown
        };
        let inspection = WorkingStateInspection {
            working_state: working_state.clone(),
            change_status,
            recovery_condition,
        };
        drop(state);
        Ok(inspection)
    }

    /// Reads operational recovery status even if no Working State exists.
    /// This is idempotent, performs no Adapter work, and creates no history.
    ///
    /// # Errors
    ///
    /// Returns `ProjectNotFound` or `RepositoryUnavailable`.
    pub fn recovery_condition(
        &self,
        project_id: ProjectId,
    ) -> Result<Option<WorkingStateRecoveryCondition>, WorkingStateError> {
        let state = self.lock()?;
        require_project(&state, project_id)?;
        Ok(state.recovery_conditions.get(&project_id).copied())
    }

    /// Explicitly retries persistence of current operational status after a
    /// store outage. This does not restore live state, clear recovery status,
    /// create Working State metadata, or retry destructive Adapter work.
    /// Repeating it commits the same current record.
    ///
    /// # Errors
    ///
    /// Returns `ProjectNotFound` or `RepositoryUnavailable`. A failed commit
    /// leaves the prior stored record authoritative and runtime status intact.
    pub fn persist_recovery_condition(
        &self,
        project_id: ProjectId,
    ) -> Result<(), WorkingStateError> {
        let state = self.lock()?;
        require_project(&state, project_id)?;
        self.commit_current_record(project_id, &state)
    }

    /// Performs full materialisation. The supplied optional Line association
    /// is the association committed by this invocation; pass `None` for no
    /// association.
    /// The Base Revision, source map, Line, reference, and recovery condition
    /// become authoritative together only after Adapter prepare/validation/
    /// restore succeeds.
    ///
    /// # Errors
    ///
    /// Returns typed Project/Revision/Line, authorization, Adapter, or
    /// repository errors. No Revision or other historical object is created.
    #[allow(clippy::too_many_arguments)]
    pub fn materialise_working_state(
        &self,
        project_id: ProjectId,
        revision_id: RevisionId,
        line_id: Option<LineId>,
        authorization: ReplacementAuthorization,
        adapter: &dyn WorkingStateAdapter,
        revisions: &dyn AdmittedRevisionResolver,
        project_states: &dyn AdmittedProjectStateResolver,
        component_states: &dyn ComponentStateResolver,
        lines: &dyn LineOperationBoundary,
    ) -> Result<WorkingState, WorkingStateError> {
        let mut state = self.lock()?;
        require_project(&state, project_id)?;
        let target = resolve_revision(project_id, revision_id, revisions, project_states)?;
        let sources = resolve_full_component_sources(target, component_states)?;
        if let Some(line_id) = line_id {
            validate_line(project_id, line_id, lines)?;
        }
        let resulting_line = line_id;
        let current = state.working_states.get(&project_id).cloned();
        if let Some(current) = &current {
            enforce_replacement_authorization(
                current,
                state.recovery_conditions.get(&project_id).copied(),
                authorization,
                adapter,
            )?;
        }

        let preparation = WorkingStatePreparation {
            project_id,
            base_revision_id: Some(revision_id),
            line_id: resulting_line,
            component_sources: sources,
        };
        let reference = adapter
            .prepare_working_state(&preparation)
            .map_err(WorkingStateError::AdapterPrepareFailed)?;
        adapter
            .validate_working_state_ref(project_id, &reference)
            .map_err(map_reference_error)?;
        if let Some(current) = &current {
            enforce_replacement_authorization(
                current,
                state.recovery_conditions.get(&project_id).copied(),
                authorization,
                adapter,
            )?;
        }
        if let Err(error) = restore_prepared(project_id, &reference, adapter, &mut state) {
            if state.recovery_conditions.get(&project_id)
                == Some(&WorkingStateRecoveryCondition::RecoveryRequired)
            {
                self.commit_current_record(project_id, &state)?;
            }
            return Err(error);
        }

        let committed = WorkingState {
            project_id,
            base_revision_id: Some(revision_id),
            line_id: resulting_line,
            component_sources: preparation.component_sources,
            adapter_working_state_ref: Some(reference),
        };
        self.commit_after_adapter_restore(project_id, &committed, &mut state)?;
        state.working_states.insert(project_id, committed.clone());
        state
            .recovery_conditions
            .insert(project_id, WorkingStateRecoveryCondition::Confirmed);
        drop(state);
        Ok(committed)
    }

    /// Materialises only the selected components from one admitted
    /// same-Project Revision. It leaves the existing Base Revision unchanged.
    ///
    /// # Errors
    ///
    /// Returns typed Project/Revision/component/authorization/Adapter or
    /// repository errors; failure before commit preserves the prior record.
    #[allow(clippy::too_many_arguments)]
    pub fn materialise_selective_working_state(
        &self,
        project_id: ProjectId,
        source_revision_id: RevisionId,
        component_ids: &[CreativeComponentId],
        authorization: ReplacementAuthorization,
        adapter: &dyn WorkingStateAdapter,
        revisions: &dyn AdmittedRevisionResolver,
        project_states: &dyn AdmittedProjectStateResolver,
        component_states: &dyn ComponentStateResolver,
    ) -> Result<WorkingState, WorkingStateError> {
        let mut state = self.lock()?;
        require_project(&state, project_id)?;
        let current = state
            .working_states
            .get(&project_id)
            .cloned()
            .ok_or(WorkingStateError::WorkingStateNotFound)?;
        let source_revision =
            resolve_revision(project_id, source_revision_id, revisions, project_states)?;
        let source_map = resolve_full_component_sources(source_revision, component_states)?;
        let mut selected = BTreeSet::new();
        for component_id in component_ids {
            if !selected.insert(*component_id) {
                return Err(WorkingStateError::DuplicateComponentSelection);
            }
            if !source_map.contains_key(component_id) {
                return Err(WorkingStateError::ComponentNotInRevision);
            }
        }

        enforce_replacement_authorization(
            &current,
            state.recovery_conditions.get(&project_id).copied(),
            authorization,
            adapter,
        )?;
        let mut resulting_sources = current.component_sources.clone();
        for component_id in selected {
            if let Some(source) = source_map.get(&component_id) {
                resulting_sources.insert(component_id, *source);
            }
        }
        let result = self.prepare_restore_commit(
            project_id,
            current.base_revision_id,
            current.line_id,
            resulting_sources,
            &current,
            state.recovery_conditions.get(&project_id).copied(),
            authorization,
            adapter,
            &mut state,
        );
        drop(state);
        result
    }

    /// Applies explicit custom component sources from admitted Revisions in
    /// this Project, or marks a component as locally created with no source.
    /// The Base Revision and Line association are retained.
    ///
    /// # Errors
    ///
    /// Returns typed Project/source/authorization/Adapter or repository
    /// errors. All selected sources are validated before Adapter preparation.
    #[allow(clippy::too_many_arguments)]
    pub fn materialise_custom_working_state(
        &self,
        project_id: ProjectId,
        selections: &[ComponentSourceSelection],
        authorization: ReplacementAuthorization,
        adapter: &dyn WorkingStateAdapter,
        revisions: &dyn AdmittedRevisionResolver,
        project_states: &dyn AdmittedProjectStateResolver,
        component_states: &dyn ComponentStateResolver,
    ) -> Result<WorkingState, WorkingStateError> {
        let mut state = self.lock()?;
        require_project(&state, project_id)?;
        let current = state
            .working_states
            .get(&project_id)
            .cloned()
            .ok_or(WorkingStateError::WorkingStateNotFound)?;
        let mut validated = BTreeMap::new();
        for selection in selections {
            if validated.contains_key(&selection.component_id) {
                return Err(WorkingStateError::DuplicateComponentSelection);
            }
            let source = match selection.source {
                Some(source) => Some(validate_component_source(
                    project_id,
                    selection.component_id,
                    source,
                    revisions,
                    project_states,
                    component_states,
                )?),
                None => None,
            };
            validated.insert(selection.component_id, source);
        }

        enforce_replacement_authorization(
            &current,
            state.recovery_conditions.get(&project_id).copied(),
            authorization,
            adapter,
        )?;
        let mut resulting_sources = current.component_sources.clone();
        resulting_sources.extend(validated);
        let result = self.prepare_restore_commit(
            project_id,
            current.base_revision_id,
            current.line_id,
            resulting_sources,
            &current,
            state.recovery_conditions.get(&project_id).copied(),
            authorization,
            adapter,
            &mut state,
        );
        drop(state);
        result
    }

    #[allow(clippy::too_many_arguments)]
    fn prepare_restore_commit(
        &self,
        project_id: ProjectId,
        base_revision_id: Option<RevisionId>,
        line_id: Option<LineId>,
        component_sources: BTreeMap<CreativeComponentId, Option<ComponentStateId>>,
        current: &WorkingState,
        recovery_condition: Option<WorkingStateRecoveryCondition>,
        authorization: ReplacementAuthorization,
        adapter: &dyn WorkingStateAdapter,
        state: &mut RepositoryState,
    ) -> Result<WorkingState, WorkingStateError> {
        let preparation = WorkingStatePreparation {
            project_id,
            base_revision_id,
            line_id,
            component_sources,
        };
        let reference = adapter
            .prepare_working_state(&preparation)
            .map_err(WorkingStateError::AdapterPrepareFailed)?;
        adapter
            .validate_working_state_ref(project_id, &reference)
            .map_err(map_reference_error)?;
        enforce_replacement_authorization(current, recovery_condition, authorization, adapter)?;
        if let Err(error) = restore_prepared(project_id, &reference, adapter, state) {
            if state.recovery_conditions.get(&project_id)
                == Some(&WorkingStateRecoveryCondition::RecoveryRequired)
            {
                self.commit_persisted_record(
                    current,
                    Some(WorkingStateRecoveryCondition::RecoveryRequired),
                )?;
            }
            return Err(error);
        }

        let committed = WorkingState {
            project_id,
            base_revision_id,
            line_id,
            component_sources: preparation.component_sources,
            adapter_working_state_ref: Some(reference),
        };
        self.commit_after_adapter_restore(project_id, &committed, state)?;
        state.working_states.insert(project_id, committed.clone());
        state
            .recovery_conditions
            .insert(project_id, WorkingStateRecoveryCondition::Confirmed);
        // The caller owns the repository lock and releases it after this
        // method returns so that the operation result remains atomic.
        Ok(committed)
    }

    /// Associates or clears a Line without touching Adapter content, Base
    /// Revision, component sources, or history. Repeating the same value is
    /// an explicit successful no-op.
    ///
    /// # Errors
    ///
    /// Returns typed Project/Line or repository errors.
    pub fn associate_working_state_line(
        &self,
        project_id: ProjectId,
        line_id: Option<LineId>,
        lines: &dyn LineOperationBoundary,
    ) -> Result<LineAssociationResult, WorkingStateError> {
        let mut state = self.lock()?;
        require_project(&state, project_id)?;
        let current = state
            .working_states
            .get(&project_id)
            .ok_or(WorkingStateError::WorkingStateNotFound)?;
        if current.line_id == line_id {
            drop(state);
            return Ok(LineAssociationResult::NoOp);
        }
        if let Some(line_id) = line_id {
            validate_line(project_id, line_id, lines)?;
        }
        let mut updated = current.clone();
        updated.line_id = line_id;
        self.commit_persisted_record(
            &updated,
            state.recovery_conditions.get(&project_id).copied(),
        )?;
        state.working_states.insert(project_id, updated);
        drop(state);
        Ok(LineAssociationResult::Associated)
    }

    /// Restores the committed Adapter reference, for example after process or
    /// DAW restart. A clean failure leaves Core metadata intact and recovery
    /// unconfirmed; invalid/unavailable/unrestorable references and partial
    /// destructive failure are explicitly reported.
    ///
    /// # Errors
    ///
    /// Returns typed reference/Adapter/recovery or repository errors.
    pub fn restore_committed_working_state(
        &self,
        project_id: ProjectId,
        adapter: &dyn WorkingStateAdapter,
    ) -> Result<(), WorkingStateError> {
        let mut state = self.lock()?;
        require_project(&state, project_id)?;
        let current = state
            .working_states
            .get(&project_id)
            .ok_or(WorkingStateError::WorkingStateNotFound)?;
        let reference = current
            .adapter_working_state_ref
            .clone()
            .ok_or(WorkingStateError::AdapterWorkingStateReferenceMissing)?;
        state
            .recovery_conditions
            .insert(project_id, WorkingStateRecoveryCondition::Unconfirmed);
        self.commit_current_record(project_id, &state)?;
        if let Err(error) = adapter.validate_working_state_ref(project_id, &reference) {
            set_recovery_required(project_id, &mut state);
            self.commit_current_record(project_id, &state)?;
            return Err(map_reference_error(error));
        }
        match adapter.restore_working_state(project_id, &reference) {
            Ok(()) => {
                state
                    .recovery_conditions
                    .insert(project_id, WorkingStateRecoveryCondition::Confirmed);
                self.commit_current_record(project_id, &state)?;
                drop(state);
                Ok(())
            }
            Err(AdapterRestoreError::Failed) => {
                self.commit_current_record(project_id, &state)?;
                drop(state);
                Err(WorkingStateError::AdapterRestoreFailed)
            }
            Err(AdapterRestoreError::InvalidReference) => {
                set_recovery_required(project_id, &mut state);
                self.commit_current_record(project_id, &state)?;
                drop(state);
                Err(WorkingStateError::AdapterWorkingStateReferenceInvalid)
            }
            Err(AdapterRestoreError::ReferenceUnavailable) => {
                set_recovery_required(project_id, &mut state);
                self.commit_current_record(project_id, &state)?;
                drop(state);
                Err(WorkingStateError::AdapterWorkingStateReferenceUnavailable)
            }
            Err(AdapterRestoreError::ReferenceUnrestorable) => {
                set_recovery_required(project_id, &mut state);
                self.commit_current_record(project_id, &state)?;
                drop(state);
                Err(WorkingStateError::AdapterWorkingStateReferenceUnrestorable)
            }
            Err(AdapterRestoreError::PartialFailure) => {
                set_recovery_required(project_id, &mut state);
                self.commit_current_record(project_id, &state)?;
                drop(state);
                Err(WorkingStateError::AdapterPartialFailureRecoveryRequired)
            }
        }
    }

    /// Sets the restart boundary explicitly: any committed reference is no
    /// longer assumed to match live Adapter state until restored/validated.
    ///
    /// # Errors
    ///
    /// Returns `ProjectNotFound`, `WorkingStateNotFound`, or
    /// `RepositoryUnavailable`.
    pub fn mark_unconfirmed_after_restart(
        &self,
        project_id: ProjectId,
    ) -> Result<(), WorkingStateError> {
        let mut state = self.lock()?;
        require_project(&state, project_id)?;
        let working_state = state
            .working_states
            .get(&project_id)
            .ok_or(WorkingStateError::WorkingStateNotFound)?;
        if working_state.adapter_working_state_ref.is_some()
            && state.recovery_conditions.get(&project_id)
                != Some(&WorkingStateRecoveryCondition::RecoveryRequired)
        {
            state
                .recovery_conditions
                .insert(project_id, WorkingStateRecoveryCondition::Unconfirmed);
        }
        self.commit_current_record(project_id, &state)?;
        drop(state);
        Ok(())
    }

    /// Exports current authoritative Working State metadata (if present) and
    /// operational recovery condition for reference-store integration.
    ///
    /// # Errors
    ///
    /// Returns `ProjectNotFound`, `WorkingStateNotFound`, or
    /// `RepositoryUnavailable`.
    pub fn export_persisted_working_state(
        &self,
        project_id: ProjectId,
    ) -> Result<PersistedWorkingState, WorkingStateError> {
        let state = self.lock()?;
        require_project(&state, project_id)?;
        let working_state = state.working_states.get(&project_id).cloned();
        let recovery_condition = state.recovery_conditions.get(&project_id).copied();
        drop(state);
        if working_state.is_none() && recovery_condition.is_none() {
            return Err(WorkingStateError::WorkingStateNotFound);
        }
        Ok(PersistedWorkingState {
            project_id,
            working_state,
            recovery_condition,
        })
    }

    fn commit_persisted_record(
        &self,
        working_state: &WorkingState,
        recovery_condition: Option<WorkingStateRecoveryCondition>,
    ) -> Result<(), WorkingStateError> {
        if let Some(record_store) = &self.record_store {
            let record = PersistedWorkingState {
                project_id: working_state.project_id,
                working_state: Some(working_state.clone()),
                recovery_condition,
            };
            record_store
                .commit(&record)
                .map_err(|_| WorkingStateError::RepositoryUnavailable)?;
        }
        Ok(())
    }

    fn commit_current_record(
        &self,
        project_id: ProjectId,
        state: &RepositoryState,
    ) -> Result<(), WorkingStateError> {
        let working_state = state.working_states.get(&project_id).cloned();
        let recovery_condition = state.recovery_conditions.get(&project_id).copied();
        if let Some(record_store) = &self.record_store {
            if working_state.is_some() || recovery_condition.is_some() {
                record_store
                    .commit(&PersistedWorkingState {
                        project_id,
                        working_state,
                        recovery_condition,
                    })
                    .map_err(|_| WorkingStateError::RepositoryUnavailable)?;
            }
        }
        Ok(())
    }

    fn commit_after_adapter_restore(
        &self,
        project_id: ProjectId,
        new_record: &WorkingState,
        state: &mut RepositoryState,
    ) -> Result<(), WorkingStateError> {
        match self
            .commit_persisted_record(new_record, Some(WorkingStateRecoveryCondition::Confirmed))
        {
            Ok(()) => Ok(()),
            Err(error) => {
                set_recovery_required(project_id, state);
                // The failed new-state commit leaves prior metadata (including
                // absence) authoritative. Persist only that metadata plus the
                // recovery marker. If the store is still unavailable, propagate
                // it; callers can explicitly retry status persistence later.
                self.commit_current_record(project_id, state)?;
                Err(error)
            }
        }
    }

    /// Returns a cloned committed Core record without deriving comparison
    /// status. Intended for tests and repository integrations.
    ///
    /// # Errors
    ///
    /// Returns `ProjectNotFound`, `WorkingStateNotFound`, or
    /// `RepositoryUnavailable`.
    pub fn get_working_state(
        &self,
        project_id: ProjectId,
    ) -> Result<WorkingState, WorkingStateError> {
        let state = self.lock()?;
        require_project(&state, project_id)?;
        let working_state = state
            .working_states
            .get(&project_id)
            .cloned()
            .ok_or(WorkingStateError::WorkingStateNotFound)?;
        drop(state);
        Ok(working_state)
    }
}

/// Explicit success result for Line association, including its established
/// same-value no-op.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineAssociationResult {
    /// The association changed.
    Associated,
    /// The requested association already matched the committed value.
    NoOp,
}

/// Machine-distinguishable Working State operation failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkingStateError {
    /// No Project with the supplied typed identifier exists in this repository.
    ProjectNotFound,
    /// No Working State currently exists for the Project.
    WorkingStateNotFound,
    /// Initial creation was requested but a Working State already exists.
    WorkingStateAlreadyExists,
    /// The target Revision is missing or not admitted.
    RevisionNotAdmitted,
    /// The resolver returned a Revision different from the requested ID.
    RevisionIdentifierMismatch,
    /// The Revision's Project State is missing or not admitted.
    ProjectStateNotAdmitted,
    /// The Project State resolver returned a different identifier.
    ProjectStateIdentifierMismatch,
    /// The admitted Revision belongs to another Project.
    RevisionProjectMismatch,
    /// A source Component State is missing or not admitted.
    ComponentStateNotAdmitted,
    /// A resolver returned a Component State different from the requested ID.
    ComponentStateIdentifierMismatch,
    /// The Component State resolver returned a different component.
    ComponentStateComponentMismatch,
    /// A source Revision does not contain the requested Component State.
    ComponentStateNotInRevision,
    /// A selected component is not represented in the selected Revision.
    ComponentNotInRevision,
    /// A component was selected more than once in one operation.
    DuplicateComponentSelection,
    /// The optional Line does not exist.
    LineNotFound,
    /// The Line belongs to another Project.
    LineProjectMismatch,
    /// `changed` or `unknown` current state requires explicit discard approval.
    ReplacementRequiresAuthorization,
    /// Adapter capture/preparation failed before Core commit.
    AdapterPrepareFailed(AdapterPrepareError),
    /// Adapter restore failed cleanly before changing live state.
    AdapterRestoreFailed,
    /// A prepared or committed Adapter reference is invalid.
    AdapterWorkingStateReferenceInvalid,
    /// A committed Adapter reference is missing or unavailable.
    AdapterWorkingStateReferenceMissing,
    /// A committed Adapter reference is unavailable.
    AdapterWorkingStateReferenceUnavailable,
    /// A committed Adapter reference is not restorable.
    AdapterWorkingStateReferenceUnrestorable,
    /// Partial destructive Adapter work requires explicit recovery.
    AdapterPartialFailureRecoveryRequired,
    /// The reference repository is unavailable.
    RepositoryUnavailable,
    /// A loaded persisted record identifies a different Project.
    PersistedRecordProjectMismatch,
    /// The Line repository failed while validating an association.
    LineRepositoryUnavailable,
}

impl fmt::Display for WorkingStateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::ProjectNotFound => "Project does not exist",
            Self::WorkingStateNotFound => "Working State does not exist",
            Self::WorkingStateAlreadyExists => "working_state_already_exists",
            Self::RevisionNotAdmitted => "Revision is missing or not admitted",
            Self::RevisionIdentifierMismatch => "Revision resolver returned a different identifier",
            Self::ProjectStateNotAdmitted => "Revision Project State is not admitted",
            Self::ProjectStateIdentifierMismatch => {
                "Project State resolver returned a different identifier"
            }
            Self::RevisionProjectMismatch => "Revision belongs to a different Project",
            Self::ComponentStateNotAdmitted => "Component State is missing or not admitted",
            Self::ComponentStateIdentifierMismatch => {
                "Component State resolver returned a different identifier"
            }
            Self::ComponentStateComponentMismatch => {
                "Component State identifies a different Creative Component"
            }
            Self::ComponentStateNotInRevision => {
                "Component State is not a source in the selected same-Project Revision"
            }
            Self::ComponentNotInRevision => "Creative Component is not represented in Revision",
            Self::DuplicateComponentSelection => "component selected more than once",
            Self::LineNotFound => "Line does not exist",
            Self::LineProjectMismatch => "Line belongs to a different Project",
            Self::ReplacementRequiresAuthorization => "replacement_requires_authorization",
            Self::AdapterPrepareFailed(AdapterPrepareError::CaptureFailed) => {
                "Adapter working-state capture failed"
            }
            Self::AdapterPrepareFailed(AdapterPrepareError::PrepareFailed) => {
                "Adapter working-state preparation failed"
            }
            Self::AdapterRestoreFailed => "Adapter working-state restore failed",
            Self::AdapterWorkingStateReferenceInvalid => "AdapterWorkingStateRef is invalid",
            Self::AdapterWorkingStateReferenceMissing => "AdapterWorkingStateRef is missing",
            Self::AdapterWorkingStateReferenceUnavailable => {
                "AdapterWorkingStateRef is unavailable"
            }
            Self::AdapterWorkingStateReferenceUnrestorable => {
                "AdapterWorkingStateRef is unrestorable"
            }
            Self::AdapterPartialFailureRecoveryRequired => {
                "adapter_partial_failure_recovery_required"
            }
            Self::RepositoryUnavailable => "Working State repository is unavailable",
            Self::PersistedRecordProjectMismatch => {
                "persisted Working State record identifies a different Project"
            }
            Self::LineRepositoryUnavailable => "Line repository is unavailable",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for WorkingStateError {}

fn require_project(
    state: &RepositoryState,
    project_id: ProjectId,
) -> Result<(), WorkingStateError> {
    if state.projects.contains(&project_id) {
        Ok(())
    } else {
        Err(WorkingStateError::ProjectNotFound)
    }
}

fn resolve_revision<'a>(
    project_id: ProjectId,
    revision_id: RevisionId,
    revisions: &'a dyn AdmittedRevisionResolver,
    project_states: &'a dyn AdmittedProjectStateResolver,
) -> Result<(&'a Revision, &'a ProjectState), WorkingStateError> {
    let revision = revisions
        .resolve_admitted(revision_id)
        .ok_or(WorkingStateError::RevisionNotAdmitted)?;
    if revision.revision_id() != revision_id {
        return Err(WorkingStateError::RevisionIdentifierMismatch);
    }
    let project_state = project_states
        .resolve_admitted(revision.project_state_id())
        .ok_or(WorkingStateError::ProjectStateNotAdmitted)?;
    if project_state.project_state_id() != revision.project_state_id() {
        return Err(WorkingStateError::ProjectStateIdentifierMismatch);
    }
    if project_state.project_id() != project_id {
        return Err(WorkingStateError::RevisionProjectMismatch);
    }
    Ok((revision, project_state))
}

fn resolve_full_component_sources(
    resolved: (&Revision, &ProjectState),
    component_states: &dyn ComponentStateResolver,
) -> Result<BTreeMap<CreativeComponentId, Option<ComponentStateId>>, WorkingStateError> {
    let (_, project_state) = resolved;
    project_state
        .components()
        .iter()
        .map(|(component_id, component_state_id)| {
            let component_state = component_states
                .resolve_admitted(*component_state_id)
                .ok_or(WorkingStateError::ComponentStateNotAdmitted)?;
            if component_state.component_state_id() != *component_state_id {
                return Err(WorkingStateError::ComponentStateIdentifierMismatch);
            }
            if component_state.component_id() != *component_id {
                return Err(WorkingStateError::ComponentStateComponentMismatch);
            }
            Ok((*component_id, Some(*component_state_id)))
        })
        .collect()
}

fn validate_component_source(
    project_id: ProjectId,
    component_id: CreativeComponentId,
    source: HistoricalComponentSource,
    revisions: &dyn AdmittedRevisionResolver,
    project_states: &dyn AdmittedProjectStateResolver,
    component_states: &dyn ComponentStateResolver,
) -> Result<ComponentStateId, WorkingStateError> {
    let (_, project_state) =
        resolve_revision(project_id, source.revision_id, revisions, project_states)?;
    let component_state = component_states
        .resolve_admitted(source.component_state_id)
        .ok_or(WorkingStateError::ComponentStateNotAdmitted)?;
    if component_state.component_state_id() != source.component_state_id {
        return Err(WorkingStateError::ComponentStateIdentifierMismatch);
    }
    if component_state.component_id() != component_id {
        return Err(WorkingStateError::ComponentStateComponentMismatch);
    }
    if project_state.components().get(&component_id) != Some(&source.component_state_id) {
        return Err(WorkingStateError::ComponentStateNotInRevision);
    }
    Ok(source.component_state_id)
}

fn validate_line(
    project_id: ProjectId,
    line_id: LineId,
    lines: &dyn LineOperationBoundary,
) -> Result<(), WorkingStateError> {
    let line = lines
        .get_line(line_id)
        .map_err(map_line_error)?
        .ok_or(WorkingStateError::LineNotFound)?;
    if line.project_id() != project_id {
        return Err(WorkingStateError::LineProjectMismatch);
    }
    Ok(())
}

const fn map_line_error(error: LineOperationError) -> WorkingStateError {
    match error {
        LineOperationError::RepositoryUnavailable => WorkingStateError::LineRepositoryUnavailable,
        _ => WorkingStateError::LineNotFound,
    }
}

fn enforce_replacement_authorization(
    current: &WorkingState,
    recovery_condition: Option<WorkingStateRecoveryCondition>,
    authorization: ReplacementAuthorization,
    adapter: &dyn WorkingStateAdapter,
) -> Result<(), WorkingStateError> {
    let status = if current.adapter_working_state_ref.is_some()
        && recovery_condition == Some(WorkingStateRecoveryCondition::Confirmed)
    {
        adapter.detect_core_changes(current)
    } else {
        CoreChangeStatus::Unknown
    };
    if status != CoreChangeStatus::Unchanged
        && authorization != ReplacementAuthorization::DiscardCurrentWorkingState
    {
        return Err(WorkingStateError::ReplacementRequiresAuthorization);
    }
    Ok(())
}

fn restore_prepared(
    project_id: ProjectId,
    reference: &AdapterWorkingStateRef,
    adapter: &dyn WorkingStateAdapter,
    state: &mut RepositoryState,
) -> Result<(), WorkingStateError> {
    match adapter.restore_working_state(project_id, reference) {
        Ok(()) => Ok(()),
        Err(AdapterRestoreError::Failed) => Err(WorkingStateError::AdapterRestoreFailed),
        Err(AdapterRestoreError::InvalidReference) => {
            Err(WorkingStateError::AdapterWorkingStateReferenceInvalid)
        }
        Err(AdapterRestoreError::ReferenceUnavailable) => {
            Err(WorkingStateError::AdapterWorkingStateReferenceUnavailable)
        }
        Err(AdapterRestoreError::ReferenceUnrestorable) => {
            Err(WorkingStateError::AdapterWorkingStateReferenceUnrestorable)
        }
        Err(AdapterRestoreError::PartialFailure) => {
            set_recovery_required(project_id, state);
            Err(WorkingStateError::AdapterPartialFailureRecoveryRequired)
        }
    }
}

const fn map_reference_error(error: AdapterReferenceError) -> WorkingStateError {
    match error {
        AdapterReferenceError::Invalid => WorkingStateError::AdapterWorkingStateReferenceInvalid,
        AdapterReferenceError::Unavailable => {
            WorkingStateError::AdapterWorkingStateReferenceUnavailable
        }
        AdapterReferenceError::Unrestorable => {
            WorkingStateError::AdapterWorkingStateReferenceUnrestorable
        }
    }
}

fn set_recovery_required(project_id: ProjectId, state: &mut RepositoryState) {
    state
        .recovery_conditions
        .insert(project_id, WorkingStateRecoveryCondition::RecoveryRequired);
}
