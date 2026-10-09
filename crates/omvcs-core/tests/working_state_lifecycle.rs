#![allow(clippy::expect_used)]

use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex};

use omvcs_core::line::{InMemoryLineRepository, LineGeneration, LineOperationBoundary};
use omvcs_core::working_state::{
    AdapterPrepareError, AdapterReferenceError, AdapterRestoreError, AdapterWorkingStateRef,
    ComponentSourceSelection, CoreChangeStatus, HistoricalComponentSource,
    InMemoryWorkingStateRepository, PersistedWorkingState, ReplacementAuthorization,
    WorkingStateAdapter, WorkingStateError, WorkingStatePersistenceError, WorkingStatePreparation,
    WorkingStateRecordStore, WorkingStateRecoveryCondition,
};
use omvcs_model::canonical::MetadataSchema;
use omvcs_model::component_state::{ComponentStateCandidate, ComponentStateSchemaValidator};
use omvcs_model::project_state::{
    AdmittedAdapterStateResolver, ComponentStateResolver, ProjectState, ProjectStateCandidate,
    ProjectStateSchemaValidator,
};
use omvcs_model::revision::{Revision, RevisionCandidate, RevisionSchemaValidator};
use omvcs_model::{
    ActorId, AdapterStateId, ComponentState, ComponentStateId, CreativeComponentId, ProjectId,
    ProjectStateId, RevisionId,
};
use serde_json::{Value, json};

const PROJECT_SCHEMA: &str = "working-state-test.project/1";
const COMPONENT_SCHEMA: &str = "working-state-test.component/1";
const REVISION_SCHEMA: &str = "working-state-test.revision/1";
const PROJECT_ID: &str = "019cc17d-1b22-7a41-9fe9-c345c468f82e";
const OTHER_PROJECT_ID: &str = "019cc17d-1b22-7a41-9fe9-c345c468f82f";
const ACTOR_ID: &str = "019cc17d-1b22-7a41-9fe9-c345c468f82c";

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

struct TestComponentSchema;

impl ComponentStateSchemaValidator for TestComponentSchema {
    fn schema(&self) -> &str {
        COMPONENT_SCHEMA
    }

    fn metadata_schema(&self) -> MetadataSchema {
        MetadataSchema::map(MetadataSchema::Scalar)
    }

    fn validate_metadata(&self, _: &BTreeMap<String, Value>) -> Result<(), String> {
        Ok(())
    }

    fn resource_validation_context(
        &self,
        _: &omvcs_model::resource::ResourceReferenceCandidate,
    ) -> Option<omvcs_model::resource::ResourceValidationContext> {
        None
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

struct TestAdapterResolver(AdapterStateId);

impl AdmittedAdapterStateResolver for TestAdapterResolver {
    fn resolve_admitted(&self, id: AdapterStateId) -> Option<AdapterStateId> {
        (id == self.0).then_some(id)
    }
}

struct WrongComponentStateResolver<'a> {
    states: &'a BTreeMap<ComponentStateId, ComponentState>,
    requested_id: ComponentStateId,
    returned_id: ComponentStateId,
}

impl ComponentStateResolver for WrongComponentStateResolver<'_> {
    fn resolve_admitted(&self, id: ComponentStateId) -> Option<&ComponentState> {
        let resolved_id = if id == self.requested_id {
            self.returned_id
        } else {
            id
        };
        self.states.get(&resolved_id)
    }
}

struct Fixture {
    project_id: ProjectId,
    other_project_id: ProjectId,
    component_a: CreativeComponentId,
    component_b: CreativeComponentId,
    local_component: CreativeComponentId,
    component_state_a1: ComponentStateId,
    component_state_a2: ComponentStateId,
    component_state_b1: ComponentStateId,
    revision_v1: RevisionId,
    revision_v2: RevisionId,
    other_revision: RevisionId,
    project_states: BTreeMap<ProjectStateId, ProjectState>,
    revisions: BTreeMap<RevisionId, Revision>,
    component_states: BTreeMap<ComponentStateId, ComponentState>,
}

impl Fixture {
    fn new() -> Self {
        let project_id = PROJECT_ID.parse().expect("valid project ID");
        let other_project_id = OTHER_PROJECT_ID.parse().expect("valid other project ID");
        let component_a = CreativeComponentId::new();
        let component_b = CreativeComponentId::new();
        let local_component = CreativeComponentId::new();
        let source_alpha_original = admit_component_state(component_a, "a1");
        let source_alpha_original_id = source_alpha_original.component_state_id();
        let source_alpha_alternate = admit_component_state(component_a, "a2");
        let source_alpha_alternate_id = source_alpha_alternate.component_state_id();
        let source_beta = admit_component_state(component_b, "b1");
        let source_beta_id = source_beta.component_state_id();
        let component_states = BTreeMap::from([
            (source_alpha_original_id, source_alpha_original),
            (source_alpha_alternate_id, source_alpha_alternate),
            (source_beta_id, source_beta),
        ]);

        let v1_components = BTreeMap::from([
            (component_a, source_alpha_original_id),
            (component_b, source_beta_id),
        ]);
        let v2_components = BTreeMap::from([
            (component_a, source_alpha_alternate_id),
            (component_b, source_beta_id),
        ]);
        let state_v1 = admit_project_state(project_id, &v1_components, &component_states);
        let state_id_v1 = state_v1.project_state_id();
        let state_v2 = admit_project_state(project_id, &v2_components, &component_states);
        let state_id_v2 = state_v2.project_state_id();
        let other_state =
            admit_project_state(other_project_id, &BTreeMap::new(), &component_states);
        let other_state_id = other_state.project_state_id();
        let project_states = BTreeMap::from([
            (state_id_v1, state_v1),
            (state_id_v2, state_v2),
            (other_state_id, other_state),
        ]);
        let revision_v1 = admit_revision(state_id_v1, &project_states, &BTreeMap::new(), "one");
        let revision_v2 = admit_revision(state_id_v2, &project_states, &BTreeMap::new(), "two");
        let other_revision =
            admit_revision(other_state_id, &project_states, &BTreeMap::new(), "other");
        let revision_v1_id = revision_v1.revision_id();
        let revision_v2_id = revision_v2.revision_id();
        let other_revision_id = other_revision.revision_id();
        let revisions = BTreeMap::from([
            (revision_v1_id, revision_v1),
            (revision_v2_id, revision_v2),
            (other_revision_id, other_revision),
        ]);

        Self {
            project_id,
            other_project_id,
            component_a,
            component_b,
            local_component,
            component_state_a1: source_alpha_original_id,
            component_state_a2: source_alpha_alternate_id,
            component_state_b1: source_beta_id,
            revision_v1: revision_v1_id,
            revision_v2: revision_v2_id,
            other_revision: other_revision_id,
            project_states,
            revisions,
            component_states,
        }
    }
}

fn admit_component_state(component_id: CreativeComponentId, label: &str) -> ComponentState {
    let mut metadata = BTreeMap::new();
    metadata.insert("label".to_owned(), json!(label));
    ComponentStateCandidate::new(COMPONENT_SCHEMA, component_id, vec![], metadata)
        .admit(&[&TestComponentSchema], &[])
        .expect("valid admitted Component State")
}

fn admit_project_state(
    project_id: ProjectId,
    components: &BTreeMap<CreativeComponentId, ComponentStateId>,
    component_states: &BTreeMap<ComponentStateId, ComponentState>,
) -> ProjectState {
    let adapter_state_id = AdapterStateId::from_digest([0x74; 32]);
    let candidate: ProjectStateCandidate = serde_json::from_value(json!({
        "schema": PROJECT_SCHEMA,
        "project_id": project_id.to_string(),
        "components": components.iter().map(|(component_id, state_id)| {
            (component_id.to_string(), Value::String(state_id.to_string()))
        }).collect::<serde_json::Map<_, _>>(),
        "adapter_state_id": adapter_state_id.to_string(),
        "project_metadata": {}
    }))
    .expect("valid Project State candidate");
    candidate
        .admit(
            &[&TestProjectSchema],
            component_states,
            &TestAdapterResolver(adapter_state_id),
        )
        .expect("valid admitted Project State")
}

fn admit_revision(
    state_id: ProjectStateId,
    project_states: &BTreeMap<ProjectStateId, ProjectState>,
    parents: &BTreeMap<RevisionId, Revision>,
    message: &str,
) -> Revision {
    RevisionCandidate::new(
        REVISION_SCHEMA,
        state_id,
        parents.keys().copied().collect(),
        ACTOR_ID.parse::<ActorId>().expect("valid ActorId"),
        "2026-10-09T12:00:00.000000000Z",
        message,
        vec![],
    )
    .admit(&[&TestRevisionSchema], project_states, parents)
    .expect("valid admitted Revision")
}

struct AdapterControl {
    status: CoreChangeStatus,
    status_on_prepare: Option<CoreChangeStatus>,
    prepare_error: Option<AdapterPrepareError>,
    reference_error: Option<AdapterReferenceError>,
    restore_results: VecDeque<Result<(), AdapterRestoreError>>,
    prepare_calls: usize,
    restore_calls: usize,
    native_dirty: bool,
    preparations: Vec<WorkingStatePreparation>,
    restored_references: Vec<Vec<u8>>,
}

impl Default for AdapterControl {
    fn default() -> Self {
        Self {
            status: CoreChangeStatus::Unknown,
            status_on_prepare: None,
            prepare_error: None,
            reference_error: None,
            restore_results: VecDeque::new(),
            prepare_calls: 0,
            restore_calls: 0,
            native_dirty: false,
            preparations: Vec::new(),
            restored_references: Vec::new(),
        }
    }
}

struct TestAdapter(Mutex<AdapterControl>);

impl TestAdapter {
    fn new(status: CoreChangeStatus) -> Self {
        Self(Mutex::new(AdapterControl {
            status,
            ..AdapterControl::default()
        }))
    }

    fn set_status(&self, status: CoreChangeStatus) {
        self.0.lock().expect("adapter lock").status = status;
    }

    fn set_status_on_prepare(&self, status: Option<CoreChangeStatus>) {
        self.0.lock().expect("adapter lock").status_on_prepare = status;
    }

    fn set_prepare_error(&self, error: Option<AdapterPrepareError>) {
        self.0.lock().expect("adapter lock").prepare_error = error;
    }

    fn set_reference_error(&self, error: Option<AdapterReferenceError>) {
        self.0.lock().expect("adapter lock").reference_error = error;
    }

    fn push_restore_result(&self, result: Result<(), AdapterRestoreError>) {
        self.0
            .lock()
            .expect("adapter lock")
            .restore_results
            .push_back(result);
    }

    fn counts(&self) -> (usize, usize) {
        let control = self.0.lock().expect("adapter lock");
        (control.prepare_calls, control.restore_calls)
    }

    fn set_native_dirty(&self, dirty: bool) {
        self.0.lock().expect("adapter lock").native_dirty = dirty;
    }

    fn native_dirty(&self) -> bool {
        self.0.lock().expect("adapter lock").native_dirty
    }

    fn restored_references(&self) -> Vec<Vec<u8>> {
        self.0
            .lock()
            .expect("adapter lock")
            .restored_references
            .clone()
    }
}

impl WorkingStateAdapter for TestAdapter {
    fn detect_core_changes(&self, _: &omvcs_core::working_state::WorkingState) -> CoreChangeStatus {
        self.0.lock().expect("adapter lock").status
    }

    fn prepare_working_state(
        &self,
        preparation: &WorkingStatePreparation,
    ) -> Result<AdapterWorkingStateRef, AdapterPrepareError> {
        let mut control = self.0.lock().expect("adapter lock");
        control.prepare_calls += 1;
        control.preparations.push(preparation.clone());
        if let Some(error) = control.prepare_error {
            return Err(error);
        }
        if let Some(status) = control.status_on_prepare.take() {
            control.status = status;
        }
        let reference = AdapterWorkingStateRef::new(
            format!("opaque-ref-{}", control.prepare_calls).into_bytes(),
        );
        drop(control);
        Ok(reference)
    }

    fn validate_working_state_ref(
        &self,
        _: ProjectId,
        _: &AdapterWorkingStateRef,
    ) -> Result<(), AdapterReferenceError> {
        self.0
            .lock()
            .expect("adapter lock")
            .reference_error
            .map_or(Ok(()), Err)
    }

    fn restore_working_state(
        &self,
        _: ProjectId,
        reference: &AdapterWorkingStateRef,
    ) -> Result<(), AdapterRestoreError> {
        let mut control = self.0.lock().expect("adapter lock");
        control.restore_calls += 1;
        control
            .restored_references
            .push(reference.as_bytes().to_vec());
        control.restore_results.pop_front().unwrap_or(Ok(()))
    }
}

#[derive(Default)]
struct RecordStoreControl {
    records: BTreeMap<ProjectId, PersistedWorkingState>,
    commit_results: VecDeque<Result<(), WorkingStatePersistenceError>>,
    unavailable: bool,
}

#[derive(Default)]
struct TestWorkingStateRecordStore(Mutex<RecordStoreControl>);

impl TestWorkingStateRecordStore {
    fn fail_next_commit(&self) {
        self.0
            .lock()
            .expect("record-store lock")
            .commit_results
            .push_back(Err(WorkingStatePersistenceError::Unavailable));
    }

    fn set_unavailable(&self, unavailable: bool) {
        self.0.lock().expect("record-store lock").unavailable = unavailable;
    }
}

impl WorkingStateRecordStore for TestWorkingStateRecordStore {
    fn load(
        &self,
        project_id: ProjectId,
    ) -> Result<Option<PersistedWorkingState>, WorkingStatePersistenceError> {
        let control = self.0.lock().expect("record-store lock");
        if control.unavailable {
            return Err(WorkingStatePersistenceError::Unavailable);
        }
        Ok(control.records.get(&project_id).cloned())
    }

    fn commit(&self, record: &PersistedWorkingState) -> Result<(), WorkingStatePersistenceError> {
        let mut control = self.0.lock().expect("record-store lock");
        if control.unavailable {
            return Err(WorkingStatePersistenceError::Unavailable);
        }
        control.commit_results.pop_front().unwrap_or(Ok(()))?;
        control.records.insert(record.project_id(), record.clone());
        drop(control);
        Ok(())
    }
}

fn repository(fixture: &Fixture) -> InMemoryWorkingStateRepository {
    InMemoryWorkingStateRepository::new([fixture.project_id])
}

fn initial(repository: &InMemoryWorkingStateRepository, fixture: &Fixture) {
    repository
        .create_initial_working_state(fixture.project_id)
        .expect("initial Working State");
}

fn full_materialise(
    repository: &InMemoryWorkingStateRepository,
    fixture: &Fixture,
    adapter: &TestAdapter,
    line_id: Option<omvcs_model::LineId>,
    authorization: ReplacementAuthorization,
) -> Result<omvcs_core::working_state::WorkingState, WorkingStateError> {
    let lines = InMemoryLineRepository::new([fixture.project_id]);
    repository.materialise_working_state(
        fixture.project_id,
        fixture.revision_v1,
        line_id,
        authorization,
        adapter,
        &fixture.revisions,
        &fixture.project_states,
        &fixture.component_states,
        &lines,
    )
}

fn repository_with_committed_reference(
    fixture: &Fixture,
) -> (InMemoryWorkingStateRepository, TestAdapter) {
    let repository = repository(fixture);
    initial(&repository, fixture);
    let adapter = TestAdapter::new(CoreChangeStatus::Unchanged);
    full_materialise(
        &repository,
        fixture,
        &adapter,
        None,
        ReplacementAuthorization::DiscardCurrentWorkingState,
    )
    .expect("first materialisation");
    (repository, adapter)
}

#[test]
fn initial_state_is_pre_revision_and_existing_creation_is_distinct_failure() {
    let fixture = Fixture::new();
    let repository = repository(&fixture);
    let initial = repository
        .create_initial_working_state(fixture.project_id)
        .expect("create valid pre-first-Revision state");

    assert_eq!(initial.base_revision_id(), None);
    assert_eq!(initial.line_id(), None);
    assert!(initial.component_sources().is_empty());
    assert!(initial.adapter_working_state_ref().is_none());
    assert_eq!(
        repository.restore_committed_working_state(
            fixture.project_id,
            &TestAdapter::new(CoreChangeStatus::Unknown)
        ),
        Err(WorkingStateError::AdapterWorkingStateReferenceMissing)
    );
    assert_eq!(
        repository.create_initial_working_state(fixture.project_id),
        Err(WorkingStateError::WorkingStateAlreadyExists)
    );
    assert_eq!(
        repository.create_initial_working_state(fixture.other_project_id),
        Err(WorkingStateError::ProjectNotFound)
    );
    assert_eq!(fixture.revisions.len(), 3);
    assert_eq!(fixture.project_states.len(), 3);
}

#[test]
fn full_materialisation_creates_missing_working_state_only_after_success() {
    let fixture = Fixture::new();
    let record_store = Arc::new(TestWorkingStateRecordStore::default());
    let store_boundary: Arc<dyn WorkingStateRecordStore> = record_store.clone();
    let repository = InMemoryWorkingStateRepository::load_from_record_store(
        [fixture.project_id],
        store_boundary,
    )
    .expect("load empty record store");
    let adapter = TestAdapter::new(CoreChangeStatus::Unknown);
    adapter.set_prepare_error(Some(AdapterPrepareError::PrepareFailed));

    assert_eq!(
        full_materialise(
            &repository,
            &fixture,
            &adapter,
            None,
            ReplacementAuthorization::Preserve,
        ),
        Err(WorkingStateError::AdapterPrepareFailed(
            AdapterPrepareError::PrepareFailed
        ))
    );
    assert_eq!(
        repository.get_working_state(fixture.project_id),
        Err(WorkingStateError::WorkingStateNotFound)
    );
    assert_eq!(
        record_store
            .load(fixture.project_id)
            .expect("load failed attempt"),
        None,
        "failed initial materialisation must not persist a partial record"
    );

    adapter.set_prepare_error(None);
    let materialised = full_materialise(
        &repository,
        &fixture,
        &adapter,
        None,
        ReplacementAuthorization::Preserve,
    )
    .expect("first full materialisation without an existing record");
    assert_eq!(materialised.base_revision_id(), Some(fixture.revision_v1));
    assert_eq!(
        materialised.component_sources(),
        &BTreeMap::from([
            (fixture.component_a, Some(fixture.component_state_a1)),
            (fixture.component_b, Some(fixture.component_state_b1)),
        ])
    );
    assert!(materialised.adapter_working_state_ref().is_some());
    assert_eq!(
        repository
            .inspect_working_state(fixture.project_id, &adapter)
            .expect("inspect newly materialised Working State")
            .recovery_condition,
        Some(WorkingStateRecoveryCondition::Confirmed)
    );
    assert!(
        record_store
            .load(fixture.project_id)
            .expect("load successful initial materialisation")
            .is_some()
    );
}

#[test]
fn full_materialisation_commits_base_sources_reference_and_line_together() {
    let fixture = Fixture::new();
    let repository = repository(&fixture);
    initial(&repository, &fixture);
    let adapter = TestAdapter::new(CoreChangeStatus::Unchanged);
    let line_repo = InMemoryLineRepository::new([fixture.project_id]);
    let line = line_repo
        .create_line(
            fixture.project_id,
            "main".to_owned(),
            fixture.revision_v1,
            &fixture.revisions,
            &fixture.project_states,
        )
        .expect("create Line");

    let working_state = repository
        .materialise_working_state(
            fixture.project_id,
            fixture.revision_v1,
            Some(line.line_id()),
            ReplacementAuthorization::DiscardCurrentWorkingState,
            &adapter,
            &fixture.revisions,
            &fixture.project_states,
            &fixture.component_states,
            &line_repo,
        )
        .expect("materialise full Revision");
    assert_eq!(working_state.base_revision_id(), Some(fixture.revision_v1));
    assert_eq!(working_state.line_id(), Some(line.line_id()));
    assert_eq!(
        working_state.component_sources(),
        &BTreeMap::from([
            (fixture.component_a, Some(fixture.component_state_a1)),
            (fixture.component_b, Some(fixture.component_state_b1)),
        ])
    );
    assert_eq!(
        working_state
            .adapter_working_state_ref()
            .expect("committed opaque reference")
            .as_bytes(),
        b"opaque-ref-1"
    );
    assert_eq!(
        format!(
            "{:?}",
            working_state
                .adapter_working_state_ref()
                .expect("opaque reference")
        ),
        "AdapterWorkingStateRef([opaque])"
    );
    assert_eq!(
        fixture.revisions.len(),
        3,
        "materialisation creates no Revision"
    );
    assert_eq!(
        repository
            .inspect_working_state(fixture.project_id, &adapter)
            .expect("inspect committed state"),
        omvcs_core::working_state::WorkingStateInspection {
            working_state,
            change_status: CoreChangeStatus::Unchanged,
            recovery_condition: Some(WorkingStateRecoveryCondition::Confirmed),
        }
    );
}

#[test]
fn changed_and_unknown_work_refuse_replacement_before_adapter_calls() {
    for status in [CoreChangeStatus::Changed, CoreChangeStatus::Unknown] {
        let fixture = Fixture::new();
        let repository = repository(&fixture);
        initial(&repository, &fixture);
        let adapter = TestAdapter::new(CoreChangeStatus::Unchanged);
        full_materialise(
            &repository,
            &fixture,
            &adapter,
            None,
            ReplacementAuthorization::DiscardCurrentWorkingState,
        )
        .expect("first replacement of unknown initial state is explicitly authorized below");

        adapter.set_status(status);
        let before = repository
            .get_working_state(fixture.project_id)
            .expect("current record");
        let calls_before = adapter.counts();
        assert_eq!(
            full_materialise(
                &repository,
                &fixture,
                &adapter,
                None,
                ReplacementAuthorization::Preserve,
            ),
            Err(WorkingStateError::ReplacementRequiresAuthorization)
        );
        assert_eq!(
            repository
                .get_working_state(fixture.project_id)
                .expect("unchanged current record"),
            before
        );
        assert_eq!(adapter.counts(), calls_before);
    }
}

#[test]
fn unchanged_work_can_be_replaced_without_discard_authorization() {
    let fixture = Fixture::new();
    let repository = repository(&fixture);
    initial(&repository, &fixture);
    let adapter = TestAdapter::new(CoreChangeStatus::Unchanged);
    full_materialise(
        &repository,
        &fixture,
        &adapter,
        None,
        ReplacementAuthorization::DiscardCurrentWorkingState,
    )
    .expect("authorize first materialisation while status is unknown");
    adapter.set_status(CoreChangeStatus::Unchanged);

    let result = repository
        .materialise_working_state(
            fixture.project_id,
            fixture.revision_v2,
            None,
            ReplacementAuthorization::Preserve,
            &adapter,
            &fixture.revisions,
            &fixture.project_states,
            &fixture.component_states,
            &InMemoryLineRepository::new([fixture.project_id]),
        )
        .expect("unchanged replacement needs no authorization");
    assert_eq!(result.base_revision_id(), Some(fixture.revision_v2));
}

#[test]
fn replacement_status_is_rechecked_after_prepare_before_destructive_restore() {
    let fixture = Fixture::new();
    let repository = repository(&fixture);
    initial(&repository, &fixture);
    let adapter = TestAdapter::new(CoreChangeStatus::Unchanged);
    full_materialise(
        &repository,
        &fixture,
        &adapter,
        None,
        ReplacementAuthorization::DiscardCurrentWorkingState,
    )
    .expect("first materialisation");
    adapter.set_status(CoreChangeStatus::Unchanged);
    adapter.set_status_on_prepare(Some(CoreChangeStatus::Changed));
    let before = repository
        .get_working_state(fixture.project_id)
        .expect("committed record");
    let calls_before = adapter.counts();

    assert_eq!(
        full_materialise(
            &repository,
            &fixture,
            &adapter,
            None,
            ReplacementAuthorization::Preserve,
        ),
        Err(WorkingStateError::ReplacementRequiresAuthorization)
    );
    assert_eq!(
        repository
            .get_working_state(fixture.project_id)
            .expect("old committed record"),
        before
    );
    assert_eq!(
        adapter.counts(),
        (calls_before.0 + 1, calls_before.1),
        "Core must refuse before the destructive restore call"
    );
}

#[test]
fn replacement_authorization_is_fresh_after_prepare_failure() {
    let fixture = Fixture::new();
    let repository = repository(&fixture);
    initial(&repository, &fixture);
    let adapter = TestAdapter::new(CoreChangeStatus::Unchanged);
    full_materialise(
        &repository,
        &fixture,
        &adapter,
        None,
        ReplacementAuthorization::DiscardCurrentWorkingState,
    )
    .expect("authorize first materialisation");
    adapter.set_status(CoreChangeStatus::Changed);
    adapter.set_prepare_error(Some(AdapterPrepareError::CaptureFailed));
    assert_eq!(
        full_materialise(
            &repository,
            &fixture,
            &adapter,
            None,
            ReplacementAuthorization::DiscardCurrentWorkingState,
        ),
        Err(WorkingStateError::AdapterPrepareFailed(
            AdapterPrepareError::CaptureFailed
        ))
    );
    adapter.set_prepare_error(None);
    let before = adapter.counts();
    assert_eq!(
        full_materialise(
            &repository,
            &fixture,
            &adapter,
            None,
            ReplacementAuthorization::Preserve,
        ),
        Err(WorkingStateError::ReplacementRequiresAuthorization)
    );
    assert_eq!(adapter.counts(), before);
}

#[test]
fn prepare_failure_preserves_authoritative_reference_and_all_core_metadata() {
    let fixture = Fixture::new();
    let repository = repository(&fixture);
    initial(&repository, &fixture);
    let adapter = TestAdapter::new(CoreChangeStatus::Unchanged);
    full_materialise(
        &repository,
        &fixture,
        &adapter,
        None,
        ReplacementAuthorization::DiscardCurrentWorkingState,
    )
    .expect("first materialisation");
    let before = repository
        .get_working_state(fixture.project_id)
        .expect("committed record");
    adapter.set_prepare_error(Some(AdapterPrepareError::PrepareFailed));

    assert_eq!(
        full_materialise(
            &repository,
            &fixture,
            &adapter,
            None,
            ReplacementAuthorization::Preserve,
        ),
        Err(WorkingStateError::AdapterPrepareFailed(
            AdapterPrepareError::PrepareFailed
        ))
    );
    assert_eq!(
        repository
            .get_working_state(fixture.project_id)
            .expect("old record remains authoritative"),
        before
    );
}

#[test]
fn custom_sources_can_mix_revisions_and_local_components_without_advancing_base() {
    let fixture = Fixture::new();
    let repository = repository(&fixture);
    initial(&repository, &fixture);
    let adapter = TestAdapter::new(CoreChangeStatus::Unchanged);
    full_materialise(
        &repository,
        &fixture,
        &adapter,
        None,
        ReplacementAuthorization::DiscardCurrentWorkingState,
    )
    .expect("first materialisation");

    let custom = repository
        .materialise_custom_working_state(
            fixture.project_id,
            &[
                ComponentSourceSelection {
                    component_id: fixture.component_a,
                    source: Some(HistoricalComponentSource {
                        revision_id: fixture.revision_v2,
                        component_state_id: fixture.component_state_a2,
                    }),
                },
                ComponentSourceSelection {
                    component_id: fixture.local_component,
                    source: None,
                },
            ],
            ReplacementAuthorization::Preserve,
            &adapter,
            &fixture.revisions,
            &fixture.project_states,
            &fixture.component_states,
        )
        .expect("custom same-Project mixed source state");
    assert_eq!(custom.base_revision_id(), Some(fixture.revision_v1));
    assert_eq!(
        custom.component_sources().get(&fixture.component_a),
        Some(&Some(fixture.component_state_a2))
    );
    assert_eq!(
        custom.component_sources().get(&fixture.local_component),
        Some(&None)
    );
    assert_eq!(
        fixture.revisions.len(),
        3,
        "custom materialisation creates no history"
    );
}

#[test]
fn custom_source_must_be_admitted_same_project_and_present_in_its_revision() {
    let fixture = Fixture::new();
    let repository = repository(&fixture);
    initial(&repository, &fixture);
    let adapter = TestAdapter::new(CoreChangeStatus::Unchanged);
    let calls_before = adapter.counts();
    assert_eq!(
        repository.materialise_custom_working_state(
            fixture.project_id,
            &[ComponentSourceSelection {
                component_id: fixture.component_a,
                source: Some(HistoricalComponentSource {
                    revision_id: fixture.revision_v1,
                    component_state_id: fixture.component_state_a2,
                }),
            }],
            ReplacementAuthorization::DiscardCurrentWorkingState,
            &adapter,
            &fixture.revisions,
            &fixture.project_states,
            &fixture.component_states,
        ),
        Err(WorkingStateError::ComponentStateNotInRevision)
    );
    assert_eq!(adapter.counts(), calls_before);

    assert_eq!(
        repository.materialise_custom_working_state(
            fixture.project_id,
            &[ComponentSourceSelection {
                component_id: fixture.component_a,
                source: Some(HistoricalComponentSource {
                    revision_id: fixture.revision_v1,
                    component_state_id: ComponentStateId::from_digest([0xee; 32]),
                }),
            }],
            ReplacementAuthorization::DiscardCurrentWorkingState,
            &adapter,
            &fixture.revisions,
            &fixture.project_states,
            &fixture.component_states,
        ),
        Err(WorkingStateError::ComponentStateNotAdmitted)
    );
    assert_eq!(adapter.counts(), calls_before);

    assert_eq!(
        repository.materialise_custom_working_state(
            fixture.project_id,
            &[ComponentSourceSelection {
                component_id: fixture.component_a,
                source: Some(HistoricalComponentSource {
                    revision_id: fixture.other_revision,
                    component_state_id: fixture.component_state_a1,
                }),
            }],
            ReplacementAuthorization::DiscardCurrentWorkingState,
            &adapter,
            &fixture.revisions,
            &fixture.project_states,
            &fixture.component_states,
        ),
        Err(WorkingStateError::RevisionProjectMismatch)
    );
    assert_eq!(adapter.counts(), calls_before);
}

#[test]
fn selective_materialisation_keeps_base_and_changes_only_selected_source() {
    let fixture = Fixture::new();
    let repository = repository(&fixture);
    initial(&repository, &fixture);
    let adapter = TestAdapter::new(CoreChangeStatus::Unchanged);
    full_materialise(
        &repository,
        &fixture,
        &adapter,
        None,
        ReplacementAuthorization::DiscardCurrentWorkingState,
    )
    .expect("initial full materialisation");
    adapter.set_status(CoreChangeStatus::Changed);

    let result = repository
        .materialise_selective_working_state(
            fixture.project_id,
            fixture.revision_v2,
            &[fixture.component_a],
            ReplacementAuthorization::DiscardCurrentWorkingState,
            &adapter,
            &fixture.revisions,
            &fixture.project_states,
            &fixture.component_states,
        )
        .expect("selective source update");
    assert_eq!(result.base_revision_id(), Some(fixture.revision_v1));
    assert_eq!(
        result.component_sources().get(&fixture.component_a),
        Some(&Some(fixture.component_state_a2))
    );
    assert_eq!(
        result.component_sources().get(&fixture.component_b),
        Some(&Some(fixture.component_state_b1))
    );
}

#[test]
fn line_association_is_operational_idempotent_and_does_not_follow_line_movement() {
    let fixture = Fixture::new();
    let repository = repository(&fixture);
    initial(&repository, &fixture);
    let line_repo = InMemoryLineRepository::new([fixture.project_id]);
    let line = line_repo
        .create_line(
            fixture.project_id,
            "main".to_owned(),
            fixture.revision_v1,
            &fixture.revisions,
            &fixture.project_states,
        )
        .expect("create Line");
    assert_eq!(
        repository
            .associate_working_state_line(fixture.project_id, Some(line.line_id()), &line_repo)
            .expect("associate Line"),
        omvcs_core::working_state::LineAssociationResult::Associated
    );
    assert_eq!(
        repository
            .associate_working_state_line(fixture.project_id, Some(line.line_id()), &line_repo)
            .expect("repeat same association"),
        omvcs_core::working_state::LineAssociationResult::NoOp
    );

    let adapter = TestAdapter::new(CoreChangeStatus::Unchanged);
    let based_working_state = repository
        .materialise_working_state(
            fixture.project_id,
            fixture.revision_v1,
            Some(line.line_id()),
            ReplacementAuthorization::DiscardCurrentWorkingState,
            &adapter,
            &fixture.revisions,
            &fixture.project_states,
            &fixture.component_states,
            &line_repo,
        )
        .expect("materialise on associated Line");
    let moved = line_repo
        .move_line(
            line.line_id(),
            fixture.revision_v1,
            LineGeneration::ZERO,
            fixture.revision_v2,
            &fixture.revisions,
            &fixture.project_states,
        )
        .expect("move Line independently");
    let working_state = repository
        .get_working_state(fixture.project_id)
        .expect("association remains operational");
    assert_eq!(moved.target_revision(), fixture.revision_v2);
    assert_eq!(working_state.line_id(), Some(line.line_id()));
    assert_eq!(
        based_working_state.base_revision_id(),
        Some(fixture.revision_v1)
    );
    assert_eq!(working_state.base_revision_id(), Some(fixture.revision_v1));
    assert_eq!(
        repository
            .associate_working_state_line(fixture.project_id, None, &line_repo)
            .expect("clear only Line association"),
        omvcs_core::working_state::LineAssociationResult::Associated
    );
    let association_only = repository
        .get_working_state(fixture.project_id)
        .expect("read association-only update");
    assert_eq!(association_only.line_id(), None);
    assert_eq!(
        association_only.base_revision_id(),
        based_working_state.base_revision_id()
    );
    assert_eq!(
        association_only.component_sources(),
        based_working_state.component_sources()
    );
    assert_eq!(
        association_only.adapter_working_state_ref(),
        based_working_state.adapter_working_state_ref()
    );

    let rematerialised = repository
        .materialise_working_state(
            fixture.project_id,
            fixture.revision_v2,
            None,
            ReplacementAuthorization::DiscardCurrentWorkingState,
            &TestAdapter::new(CoreChangeStatus::Unchanged),
            &fixture.revisions,
            &fixture.project_states,
            &fixture.component_states,
            &line_repo,
        )
        .expect("explicitly materialise without an associated Line");
    assert_eq!(rematerialised.line_id(), None);
    assert_eq!(rematerialised.base_revision_id(), Some(fixture.revision_v2));
}

#[test]
fn core_change_status_is_tri_state_and_not_native_dirty_or_recovery_status() {
    let fixture = Fixture::new();
    let repository = repository(&fixture);
    initial(&repository, &fixture);
    let adapter = TestAdapter::new(CoreChangeStatus::Unknown);
    full_materialise(
        &repository,
        &fixture,
        &adapter,
        None,
        ReplacementAuthorization::DiscardCurrentWorkingState,
    )
    .expect("initial materialisation");

    adapter.set_status(CoreChangeStatus::Unchanged);
    adapter.set_native_dirty(true);
    assert!(adapter.native_dirty());
    assert_eq!(
        repository
            .inspect_working_state(fixture.project_id, &adapter)
            .expect("inspect with native dirty")
            .change_status,
        CoreChangeStatus::Unchanged
    );
    adapter.set_status(CoreChangeStatus::Changed);
    assert_eq!(
        repository
            .inspect_working_state(fixture.project_id, &adapter)
            .expect("inspect changed state")
            .change_status,
        CoreChangeStatus::Changed
    );
    adapter.set_status(CoreChangeStatus::Unknown);
    assert_eq!(
        repository
            .inspect_working_state(fixture.project_id, &adapter)
            .expect("inspect uncertain state")
            .change_status,
        CoreChangeStatus::Unknown
    );
}

#[test]
fn partial_failure_keeps_old_commit_sets_recovery_required_and_does_not_retry() {
    let fixture = Fixture::new();
    let repository = repository(&fixture);
    initial(&repository, &fixture);
    let adapter = TestAdapter::new(CoreChangeStatus::Unchanged);
    full_materialise(
        &repository,
        &fixture,
        &adapter,
        None,
        ReplacementAuthorization::DiscardCurrentWorkingState,
    )
    .expect("first materialisation");
    adapter.set_status(CoreChangeStatus::Changed);
    adapter.push_restore_result(Err(AdapterRestoreError::PartialFailure));
    let before = repository
        .get_working_state(fixture.project_id)
        .expect("old committed metadata");
    let calls_before = adapter.counts();

    assert_eq!(
        full_materialise(
            &repository,
            &fixture,
            &adapter,
            None,
            ReplacementAuthorization::DiscardCurrentWorkingState,
        ),
        Err(WorkingStateError::AdapterPartialFailureRecoveryRequired)
    );
    assert_eq!(
        repository
            .get_working_state(fixture.project_id)
            .expect("old reference/base/source map remain authoritative"),
        before
    );
    assert_eq!(adapter.counts(), (calls_before.0 + 1, calls_before.1 + 1));
    let inspection = repository
        .inspect_working_state(fixture.project_id, &adapter)
        .expect("partial operation needs recovery");
    assert_eq!(
        inspection.recovery_condition,
        Some(WorkingStateRecoveryCondition::RecoveryRequired)
    );
    assert_eq!(inspection.change_status, CoreChangeStatus::Unknown);
    assert_eq!(
        full_materialise(
            &repository,
            &fixture,
            &adapter,
            None,
            ReplacementAuthorization::Preserve,
        ),
        Err(WorkingStateError::ReplacementRequiresAuthorization)
    );
    assert_eq!(adapter.counts(), (calls_before.0 + 1, calls_before.1 + 1));
}

#[test]
fn restart_uses_committed_reference_and_reports_unavailable_reference() {
    let fixture = Fixture::new();
    let (repository, adapter) = repository_with_committed_reference(&fixture);

    repository
        .mark_unconfirmed_after_restart(fixture.project_id)
        .expect("mark restart uncertainty");
    assert_eq!(
        repository
            .inspect_working_state(fixture.project_id, &adapter)
            .expect("inspect before restore")
            .change_status,
        CoreChangeStatus::Unknown
    );
    repository
        .restore_committed_working_state(fixture.project_id, &adapter)
        .expect("restore through committed reference");
    assert_eq!(
        repository
            .inspect_working_state(fixture.project_id, &adapter)
            .expect("inspect restored record")
            .recovery_condition,
        Some(WorkingStateRecoveryCondition::Confirmed)
    );

    adapter.set_reference_error(Some(AdapterReferenceError::Unavailable));
    repository
        .mark_unconfirmed_after_restart(fixture.project_id)
        .expect("mark restart uncertainty again");
    assert_eq!(
        repository.restore_committed_working_state(fixture.project_id, &adapter),
        Err(WorkingStateError::AdapterWorkingStateReferenceUnavailable)
    );
    assert_eq!(
        repository
            .inspect_working_state(fixture.project_id, &adapter)
            .expect("inspect unavailable reference")
            .recovery_condition,
        Some(WorkingStateRecoveryCondition::RecoveryRequired)
    );
}

#[test]
fn recovery_required_survives_restart_transition() {
    let fixture = Fixture::new();
    let (repository, adapter) = repository_with_committed_reference(&fixture);
    adapter.set_status(CoreChangeStatus::Changed);
    adapter.push_restore_result(Err(AdapterRestoreError::PartialFailure));
    assert_eq!(
        full_materialise(
            &repository,
            &fixture,
            &adapter,
            None,
            ReplacementAuthorization::DiscardCurrentWorkingState,
        ),
        Err(WorkingStateError::AdapterPartialFailureRecoveryRequired)
    );

    repository
        .mark_unconfirmed_after_restart(fixture.project_id)
        .expect("restart does not downgrade a required recovery condition");
    assert_eq!(
        repository
            .inspect_working_state(fixture.project_id, &adapter)
            .expect("inspect after restart transition")
            .recovery_condition,
        Some(WorkingStateRecoveryCondition::RecoveryRequired)
    );
}

#[test]
fn initial_partial_restore_persists_recovery_without_fabricating_working_state() {
    // Core §§21/83, ADR-0025/0026: live mismatch is operational status,
    // independent of the absent authoritative Working State.
    let fixture = Fixture::new();
    let store = Arc::new(TestWorkingStateRecordStore::default());
    let boundary: Arc<dyn WorkingStateRecordStore> = store.clone();
    let repository = InMemoryWorkingStateRepository::load_from_record_store(
        [fixture.project_id],
        Arc::clone(&boundary),
    )
    .expect("empty store");
    let adapter = TestAdapter::new(CoreChangeStatus::Unknown);
    adapter.push_restore_result(Err(AdapterRestoreError::PartialFailure));
    assert_eq!(
        full_materialise(
            &repository,
            &fixture,
            &adapter,
            None,
            ReplacementAuthorization::Preserve
        ),
        Err(WorkingStateError::AdapterPartialFailureRecoveryRequired)
    );
    assert_recovery_without_working_state(&repository, &fixture);
    let marker = store
        .load(fixture.project_id)
        .expect("load recovery marker")
        .expect("marker");
    assert_eq!(marker.project_id(), fixture.project_id);
    assert!(
        marker.working_state().is_none(),
        "no Base, source map, Line or ref fabricated"
    );
    assert_eq!(
        marker.recovery_condition(),
        Some(WorkingStateRecoveryCondition::RecoveryRequired)
    );
    let reloaded =
        InMemoryWorkingStateRepository::load_from_record_store([fixture.project_id], boundary)
            .expect("reload recovery-only record");
    assert_recovery_without_working_state(&reloaded, &fixture);
    assert_eq!(adapter.counts(), (1, 1), "no automatic destructive retry");
    // Explicit caller rematerialisation is possible without a committed ref.
    let recovered = full_materialise(
        &reloaded,
        &fixture,
        &adapter,
        None,
        ReplacementAuthorization::DiscardCurrentWorkingState,
    )
    .expect("explicit full recovery");
    assert_eq!(recovered.base_revision_id(), Some(fixture.revision_v1));
    assert_eq!(
        reloaded
            .recovery_condition(fixture.project_id)
            .expect("status"),
        Some(WorkingStateRecoveryCondition::Confirmed)
    );
}

fn assert_recovery_without_working_state(
    repository: &InMemoryWorkingStateRepository,
    fixture: &Fixture,
) {
    for _ in 0..2 {
        assert_eq!(
            repository
                .recovery_condition(fixture.project_id)
                .expect("query without Working State"),
            Some(WorkingStateRecoveryCondition::RecoveryRequired)
        );
    }
    assert_eq!(
        repository.get_working_state(fixture.project_id),
        Err(WorkingStateError::WorkingStateNotFound)
    );
    let exported = repository
        .export_persisted_working_state(fixture.project_id)
        .expect("export status-only record");
    assert!(exported.working_state().is_none());
    assert_eq!(
        exported.recovery_condition(),
        Some(WorkingStateRecoveryCondition::RecoveryRequired)
    );
    assert_eq!(
        repository.recovery_condition(fixture.other_project_id),
        Err(WorkingStateError::ProjectNotFound)
    );
    assert_eq!(fixture.revisions.len(), 3);
    assert_eq!(fixture.project_states.len(), 3);
}

#[test]
fn initial_post_restore_commit_failure_keeps_absent_metadata_and_reloads_marker() {
    let fixture = Fixture::new();
    let store = Arc::new(TestWorkingStateRecordStore::default());
    let boundary: Arc<dyn WorkingStateRecordStore> = store.clone();
    let repository = InMemoryWorkingStateRepository::load_from_record_store(
        [fixture.project_id],
        Arc::clone(&boundary),
    )
    .expect("empty store");
    let adapter = TestAdapter::new(CoreChangeStatus::Unchanged);
    store.fail_next_commit();
    assert_eq!(
        full_materialise(
            &repository,
            &fixture,
            &adapter,
            None,
            ReplacementAuthorization::Preserve
        ),
        Err(WorkingStateError::RepositoryUnavailable)
    );
    assert_recovery_without_working_state(&repository, &fixture);
    assert!(
        store
            .load(fixture.project_id)
            .expect("load")
            .expect("marker")
            .working_state()
            .is_none()
    );
    let reloaded =
        InMemoryWorkingStateRepository::load_from_record_store([fixture.project_id], boundary)
            .expect("reload marker after one-shot commit failure");
    assert_recovery_without_working_state(&reloaded, &fixture);
    assert_eq!(adapter.counts(), (1, 1));
}

#[test]
fn initial_commit_outage_reports_unavailability_and_allows_explicit_marker_persistence() {
    let fixture = Fixture::new();
    let store = Arc::new(TestWorkingStateRecordStore::default());
    let boundary: Arc<dyn WorkingStateRecordStore> = store.clone();
    let repository = InMemoryWorkingStateRepository::load_from_record_store(
        [fixture.project_id],
        Arc::clone(&boundary),
    )
    .expect("empty store");
    let adapter = TestAdapter::new(CoreChangeStatus::Unchanged);
    store.set_unavailable(true);
    assert_eq!(
        full_materialise(
            &repository,
            &fixture,
            &adapter,
            None,
            ReplacementAuthorization::Preserve
        ),
        Err(WorkingStateError::RepositoryUnavailable)
    );
    assert_recovery_without_working_state(&repository, &fixture);
    assert_eq!(
        repository.persist_recovery_condition(fixture.project_id),
        Err(WorkingStateError::RepositoryUnavailable)
    );
    assert!(matches!(
        InMemoryWorkingStateRepository::load_from_record_store(
            [fixture.project_id],
            Arc::clone(&boundary),
        ),
        Err(WorkingStateError::RepositoryUnavailable)
    ));
    store.set_unavailable(false);
    assert_eq!(
        store
            .load(fixture.project_id)
            .expect("prior authoritative record"),
        None
    );
    repository
        .persist_recovery_condition(fixture.project_id)
        .expect("explicit recovery persistence");
    repository
        .persist_recovery_condition(fixture.project_id)
        .expect("idempotent persistence retry");
    let reloaded =
        InMemoryWorkingStateRepository::load_from_record_store([fixture.project_id], boundary)
            .expect("reload marker after store recovers");
    assert_recovery_without_working_state(&reloaded, &fixture);
    assert_eq!(
        adapter.counts(),
        (1, 1),
        "status persistence invokes no Adapter"
    );
}

#[test]
fn initial_partial_restore_with_unavailable_store_reports_repository_unavailable() {
    let fixture = Fixture::new();
    let store = Arc::new(TestWorkingStateRecordStore::default());
    let boundary: Arc<dyn WorkingStateRecordStore> = store.clone();
    let repository = InMemoryWorkingStateRepository::load_from_record_store(
        [fixture.project_id],
        Arc::clone(&boundary),
    )
    .expect("empty store");
    let adapter = TestAdapter::new(CoreChangeStatus::Unknown);
    adapter.push_restore_result(Err(AdapterRestoreError::PartialFailure));
    store.set_unavailable(true);
    assert_eq!(
        full_materialise(
            &repository,
            &fixture,
            &adapter,
            None,
            ReplacementAuthorization::Preserve
        ),
        Err(WorkingStateError::RepositoryUnavailable)
    );
    assert_recovery_without_working_state(&repository, &fixture);
    store.set_unavailable(false);
    assert_eq!(
        store.load(fixture.project_id).expect("load prior record"),
        None
    );
    repository
        .persist_recovery_condition(fixture.project_id)
        .expect("persist pending marker");
    let reloaded =
        InMemoryWorkingStateRepository::load_from_record_store([fixture.project_id], boundary)
            .expect("reload partial-failure marker");
    assert_recovery_without_working_state(&reloaded, &fixture);
    assert_eq!(adapter.counts(), (1, 1));
}

#[test]
fn existing_post_restore_commit_failure_preserves_all_old_metadata_and_reloads_recovery() {
    let fixture = Fixture::new();
    let store = Arc::new(TestWorkingStateRecordStore::default());
    let boundary: Arc<dyn WorkingStateRecordStore> = store.clone();
    let repository = InMemoryWorkingStateRepository::load_from_record_store(
        [fixture.project_id],
        Arc::clone(&boundary),
    )
    .expect("empty store");
    let adapter = TestAdapter::new(CoreChangeStatus::Unchanged);
    let lines = InMemoryLineRepository::new([fixture.project_id]);
    let line = lines
        .create_line(
            fixture.project_id,
            "original".to_owned(),
            fixture.revision_v1,
            &fixture.revisions,
            &fixture.project_states,
        )
        .expect("create Line");
    repository
        .materialise_working_state(
            fixture.project_id,
            fixture.revision_v1,
            Some(line.line_id()),
            ReplacementAuthorization::Preserve,
            &adapter,
            &fixture.revisions,
            &fixture.project_states,
            &fixture.component_states,
            &lines,
        )
        .expect("old authoritative state");
    let old = repository
        .get_working_state(fixture.project_id)
        .expect("old state");
    store.fail_next_commit();
    assert_eq!(
        repository.materialise_working_state(
            fixture.project_id,
            fixture.revision_v2,
            None,
            ReplacementAuthorization::Preserve,
            &adapter,
            &fixture.revisions,
            &fixture.project_states,
            &fixture.component_states,
            &lines,
        ),
        Err(WorkingStateError::RepositoryUnavailable)
    );
    assert_eq!(
        repository
            .get_working_state(fixture.project_id)
            .expect("unchanged metadata"),
        old
    );
    let stored = store
        .load(fixture.project_id)
        .expect("load")
        .expect("old record plus marker");
    assert_eq!(stored.working_state(), Some(&old));
    assert_eq!(
        stored.recovery_condition(),
        Some(WorkingStateRecoveryCondition::RecoveryRequired)
    );
    let reloaded =
        InMemoryWorkingStateRepository::load_from_record_store([fixture.project_id], boundary)
            .expect("reload old record plus marker");
    let inspection = reloaded
        .inspect_working_state(fixture.project_id, &adapter)
        .expect("inspection");
    assert_eq!(inspection.working_state, old);
    assert_eq!(
        inspection.recovery_condition,
        Some(WorkingStateRecoveryCondition::RecoveryRequired)
    );
    assert_eq!(inspection.change_status, CoreChangeStatus::Unknown);
    assert_eq!(adapter.counts(), (2, 2));
}

#[test]
fn persisted_record_reload_reconstructs_repository_and_restores_committed_ref() {
    let fixture = Fixture::new();
    let record_store = Arc::new(TestWorkingStateRecordStore::default());
    let record_store_boundary: Arc<dyn WorkingStateRecordStore> = record_store;
    let repository = InMemoryWorkingStateRepository::load_from_record_store(
        [fixture.project_id],
        Arc::clone(&record_store_boundary),
    )
    .expect("load initially empty record store");
    let adapter = TestAdapter::new(CoreChangeStatus::Unchanged);
    full_materialise(
        &repository,
        &fixture,
        &adapter,
        None,
        ReplacementAuthorization::Preserve,
    )
    .expect("persist full materialisation");
    let exported = repository
        .export_persisted_working_state(fixture.project_id)
        .expect("export complete committed record");
    let committed_reference = exported
        .working_state()
        .expect("committed Working State")
        .adapter_working_state_ref()
        .expect("full materialisation committed Adapter ref")
        .as_bytes()
        .to_vec();
    assert_eq!(
        exported
            .working_state()
            .expect("committed Working State")
            .base_revision_id(),
        Some(fixture.revision_v1)
    );
    assert_eq!(
        exported.recovery_condition(),
        Some(WorkingStateRecoveryCondition::Confirmed)
    );

    // Construct a distinct Core repository from the record-store boundary,
    // not by calling an in-process restart marker on the original instance.
    let reloaded = InMemoryWorkingStateRepository::load_from_record_store(
        [fixture.project_id],
        record_store_boundary,
    )
    .expect("reconstruct repository from persisted Core record");
    let before_restore = reloaded
        .inspect_working_state(fixture.project_id, &adapter)
        .expect("inspect reloaded record");
    assert_eq!(
        before_restore.working_state,
        *exported.working_state().expect("committed Working State")
    );
    assert_eq!(
        before_restore.recovery_condition,
        Some(WorkingStateRecoveryCondition::Unconfirmed)
    );
    reloaded
        .restore_committed_working_state(fixture.project_id, &adapter)
        .expect("restore after loading the committed reference");
    assert_eq!(
        adapter.restored_references().last(),
        Some(&committed_reference)
    );
    assert_eq!(
        reloaded
            .inspect_working_state(fixture.project_id, &adapter)
            .expect("inspect restored persisted record")
            .recovery_condition,
        Some(WorkingStateRecoveryCondition::Confirmed)
    );
}

#[test]
fn persisted_recovery_required_survives_repository_reconstruction() {
    let fixture = Fixture::new();
    let record_store = Arc::new(TestWorkingStateRecordStore::default());
    let store_boundary: Arc<dyn WorkingStateRecordStore> = record_store;
    let repository = InMemoryWorkingStateRepository::load_from_record_store(
        [fixture.project_id],
        Arc::clone(&store_boundary),
    )
    .expect("load empty reference store");
    let adapter = TestAdapter::new(CoreChangeStatus::Unchanged);
    full_materialise(
        &repository,
        &fixture,
        &adapter,
        None,
        ReplacementAuthorization::Preserve,
    )
    .expect("commit reference before injected failure");
    adapter.set_status(CoreChangeStatus::Changed);
    adapter.push_restore_result(Err(AdapterRestoreError::PartialFailure));
    assert_eq!(
        full_materialise(
            &repository,
            &fixture,
            &adapter,
            None,
            ReplacementAuthorization::DiscardCurrentWorkingState,
        ),
        Err(WorkingStateError::AdapterPartialFailureRecoveryRequired)
    );
    assert_eq!(
        repository
            .export_persisted_working_state(fixture.project_id)
            .expect("export partial-failure record")
            .recovery_condition(),
        Some(WorkingStateRecoveryCondition::RecoveryRequired)
    );

    let reloaded = InMemoryWorkingStateRepository::load_from_record_store(
        [fixture.project_id],
        store_boundary,
    )
    .expect("reload durable record and recovery condition");
    assert_eq!(
        reloaded
            .inspect_working_state(fixture.project_id, &adapter)
            .expect("inspect reloaded recovery condition")
            .recovery_condition,
        Some(WorkingStateRecoveryCondition::RecoveryRequired)
    );
}

#[test]
fn restart_restore_failures_report_distinct_recovery_conditions() {
    let fixture = Fixture::new();
    let (repository, adapter) = repository_with_committed_reference(&fixture);

    repository
        .mark_unconfirmed_after_restart(fixture.project_id)
        .expect("mark clean restore attempt unconfirmed");
    adapter.push_restore_result(Err(AdapterRestoreError::Failed));
    assert_eq!(
        repository.restore_committed_working_state(fixture.project_id, &adapter),
        Err(WorkingStateError::AdapterRestoreFailed)
    );
    assert_eq!(
        repository
            .inspect_working_state(fixture.project_id, &adapter)
            .expect("clean failure leaves recovery unconfirmed")
            .recovery_condition,
        Some(WorkingStateRecoveryCondition::Unconfirmed)
    );

    repository
        .mark_unconfirmed_after_restart(fixture.project_id)
        .expect("mark partial restore attempt unconfirmed");
    adapter.push_restore_result(Err(AdapterRestoreError::PartialFailure));
    assert_eq!(
        repository.restore_committed_working_state(fixture.project_id, &adapter),
        Err(WorkingStateError::AdapterPartialFailureRecoveryRequired)
    );
    assert_eq!(
        repository
            .inspect_working_state(fixture.project_id, &adapter)
            .expect("partial restore needs recovery")
            .recovery_condition,
        Some(WorkingStateRecoveryCondition::RecoveryRequired)
    );

    for (reference_error, expected) in [
        (
            AdapterReferenceError::Invalid,
            WorkingStateError::AdapterWorkingStateReferenceInvalid,
        ),
        (
            AdapterReferenceError::Unavailable,
            WorkingStateError::AdapterWorkingStateReferenceUnavailable,
        ),
        (
            AdapterReferenceError::Unrestorable,
            WorkingStateError::AdapterWorkingStateReferenceUnrestorable,
        ),
    ] {
        adapter.set_reference_error(Some(reference_error));
        repository
            .mark_unconfirmed_after_restart(fixture.project_id)
            .expect("mark reference unconfirmed");
        assert_eq!(
            repository.restore_committed_working_state(fixture.project_id, &adapter),
            Err(expected)
        );
        assert_eq!(
            repository
                .inspect_working_state(fixture.project_id, &adapter)
                .expect("inspect unrecoverable reference")
                .recovery_condition,
            Some(WorkingStateRecoveryCondition::RecoveryRequired)
        );
    }
}

#[test]
fn adapter_prepare_reference_and_clean_restore_failures_preserve_commit() {
    for (failure, expected) in [
        (
            AdapterRestoreError::Failed,
            WorkingStateError::AdapterRestoreFailed,
        ),
        (
            AdapterRestoreError::InvalidReference,
            WorkingStateError::AdapterWorkingStateReferenceInvalid,
        ),
        (
            AdapterRestoreError::ReferenceUnavailable,
            WorkingStateError::AdapterWorkingStateReferenceUnavailable,
        ),
        (
            AdapterRestoreError::ReferenceUnrestorable,
            WorkingStateError::AdapterWorkingStateReferenceUnrestorable,
        ),
    ] {
        let fixture = Fixture::new();
        let repository = repository(&fixture);
        initial(&repository, &fixture);
        let adapter = TestAdapter::new(CoreChangeStatus::Unchanged);
        full_materialise(
            &repository,
            &fixture,
            &adapter,
            None,
            ReplacementAuthorization::DiscardCurrentWorkingState,
        )
        .expect("initial materialisation");
        let before = repository
            .get_working_state(fixture.project_id)
            .expect("old record");
        adapter.set_status(CoreChangeStatus::Unchanged);
        adapter.push_restore_result(Err(failure));
        let result = full_materialise(
            &repository,
            &fixture,
            &adapter,
            None,
            ReplacementAuthorization::Preserve,
        );
        assert_eq!(result, Err(expected));
        assert_eq!(
            repository
                .get_working_state(fixture.project_id)
                .expect("previous commit still authoritative"),
            before
        );
    }
}

#[test]
fn invalid_prepared_reference_fails_before_destructive_restore() {
    let fixture = Fixture::new();
    let repository = repository(&fixture);
    initial(&repository, &fixture);
    let adapter = TestAdapter::new(CoreChangeStatus::Unchanged);
    adapter.set_reference_error(Some(AdapterReferenceError::Invalid));
    assert_eq!(
        full_materialise(
            &repository,
            &fixture,
            &adapter,
            None,
            ReplacementAuthorization::DiscardCurrentWorkingState,
        ),
        Err(WorkingStateError::AdapterWorkingStateReferenceInvalid)
    );
    assert_eq!(adapter.counts(), (1, 0));
    let record = repository
        .get_working_state(fixture.project_id)
        .expect("initial record unchanged");
    assert_eq!(record.base_revision_id(), None);
    assert!(record.adapter_working_state_ref().is_none());
}

#[test]
fn missing_revision_fails_before_adapter_preparation() {
    let fixture = Fixture::new();
    let repository = repository(&fixture);
    initial(&repository, &fixture);
    let adapter = TestAdapter::new(CoreChangeStatus::Unchanged);
    let missing_revision = RevisionId::from_digest([0xfa; 32]);
    let result = repository.materialise_working_state(
        fixture.project_id,
        missing_revision,
        None,
        ReplacementAuthorization::DiscardCurrentWorkingState,
        &adapter,
        &fixture.revisions,
        &fixture.project_states,
        &fixture.component_states,
        &InMemoryLineRepository::new([fixture.project_id]),
    );

    assert_eq!(result, Err(WorkingStateError::RevisionNotAdmitted));
    assert_eq!(adapter.counts(), (0, 0));
}

#[test]
fn component_state_resolver_identifier_mismatch_fails_before_adapter_work() {
    let fixture = Fixture::new();
    let repository = repository(&fixture);
    initial(&repository, &fixture);
    let adapter = TestAdapter::new(CoreChangeStatus::Unknown);
    let wrong_component_states = WrongComponentStateResolver {
        states: &fixture.component_states,
        requested_id: fixture.component_state_a1,
        returned_id: fixture.component_state_a2,
    };

    assert_eq!(
        repository.materialise_working_state(
            fixture.project_id,
            fixture.revision_v1,
            None,
            ReplacementAuthorization::DiscardCurrentWorkingState,
            &adapter,
            &fixture.revisions,
            &fixture.project_states,
            &wrong_component_states,
            &InMemoryLineRepository::new([fixture.project_id]),
        ),
        Err(WorkingStateError::ComponentStateIdentifierMismatch)
    );
    assert_eq!(adapter.counts(), (0, 0));
}

#[test]
fn derived_and_recovery_statuses_have_normative_tokens() {
    assert_eq!(CoreChangeStatus::Unchanged.as_str(), "unchanged");
    assert_eq!(CoreChangeStatus::Changed.as_str(), "changed");
    assert_eq!(CoreChangeStatus::Unknown.as_str(), "unknown");
    assert_eq!(
        WorkingStateRecoveryCondition::Confirmed.as_str(),
        "confirmed"
    );
    assert_eq!(
        WorkingStateRecoveryCondition::Unconfirmed.as_str(),
        "unconfirmed"
    );
    assert_eq!(
        WorkingStateRecoveryCondition::RecoveryRequired.as_str(),
        "recovery_required"
    );
}

#[test]
fn custom_operation_without_adapter_reference_is_conservatively_authorized() {
    let fixture = Fixture::new();
    let repository = repository(&fixture);
    initial(&repository, &fixture);
    let adapter = TestAdapter::new(CoreChangeStatus::Unchanged);
    assert_eq!(
        repository.materialise_custom_working_state(
            fixture.project_id,
            &[ComponentSourceSelection {
                component_id: fixture.local_component,
                source: None,
            }],
            ReplacementAuthorization::Preserve,
            &adapter,
            &fixture.revisions,
            &fixture.project_states,
            &fixture.component_states,
        ),
        Err(WorkingStateError::ReplacementRequiresAuthorization)
    );
    assert_eq!(adapter.counts(), (0, 0));
}
