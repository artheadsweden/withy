#![allow(clippy::expect_used)]

use std::collections::BTreeMap;

use omvcs_core::line::{
    InMemoryLineRepository, Line, LineGeneration, LineOperationBoundary, LineOperationError,
    MAX_LINE_GENERATION,
};
use omvcs_model::canonical::{MetadataSchema, canonicalize_metadata_body};
use omvcs_model::project_state::{
    AdmittedAdapterStateResolver, ProjectStateCandidate, ProjectStateSchemaValidator,
};
use omvcs_model::revision::{RevisionCandidate, RevisionSchemaValidator};
use omvcs_model::{
    ActorId, AdapterStateId, LineId, ProjectId, ProjectState, ProjectStateId, Revision, RevisionId,
};
use serde_json::{Value, json};

const REVISION_SCHEMA: &str = "line-test.revision/1";
const PROJECT_SCHEMA: &str = "line-test.project-state/1";
const PROJECT_A: &str = "019cc17d-1b22-7a41-9fe9-c345c468f82e";
const PROJECT_B: &str = "019cc17d-1b22-7a41-9fe9-c345c468f82f";
const ACTOR_ID: &str = "019cc17d-1b22-7a41-9fe9-c345c468f82c";

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

struct TestAdapterResolver(AdapterStateId);

impl AdmittedAdapterStateResolver for TestAdapterResolver {
    fn resolve_admitted(&self, id: AdapterStateId) -> Option<AdapterStateId> {
        (id == self.0).then_some(id)
    }
}

struct Fixture {
    project_ids: [ProjectId; 2],
    states: BTreeMap<ProjectStateId, ProjectState>,
    revisions: BTreeMap<RevisionId, Revision>,
    targets: [[RevisionId; 2]; 2],
    repository: InMemoryLineRepository,
}

impl Fixture {
    fn new() -> Self {
        let project_ids = [
            PROJECT_A.parse().expect("Project A ID"),
            PROJECT_B.parse().expect("Project B ID"),
        ];
        let first_state = admitted_project_state(PROJECT_A);
        let second_state = admitted_project_state(PROJECT_B);
        let states = BTreeMap::from([
            (first_state.project_state_id(), first_state),
            (second_state.project_state_id(), second_state),
        ]);
        let state_ids = states
            .values()
            .map(|state| (state.project_id(), state.project_state_id()))
            .collect::<BTreeMap<_, _>>();
        let mut revisions = BTreeMap::new();
        let mut targets = [[RevisionId::from_digest([0; 32]); 2]; 2];
        for (project_index, project_targets) in targets.iter_mut().enumerate() {
            let project_id = project_ids[project_index];
            let state_id = state_ids[&project_id];
            for (revision_index, target) in project_targets.iter_mut().enumerate() {
                let id = add_revision(
                    state_id,
                    &states,
                    &mut revisions,
                    &format!("project {project_index} revision {revision_index}"),
                );
                *target = id;
            }
        }
        Self {
            project_ids,
            states,
            revisions,
            targets,
            repository: InMemoryLineRepository::new(project_ids),
        }
    }

    fn create(&self, project: usize, target: usize, name: &str) -> Line {
        self.repository
            .create_line(
                self.project_ids[project],
                name.to_owned(),
                self.targets[project][target],
                &self.revisions,
                &self.states,
            )
            .expect("valid Line fixture")
    }
}

fn admitted_project_state(project_id: &str) -> ProjectState {
    let adapter_id = AdapterStateId::from_digest([0x31; 32]);
    let candidate: ProjectStateCandidate = serde_json::from_value(json!({
        "schema": PROJECT_SCHEMA,
        "project_id": project_id,
        "components": {},
        "adapter_state_id": adapter_id.to_string(),
        "project_metadata": {}
    }))
    .expect("valid Project State candidate");
    candidate
        .admit(
            &[&TestProjectSchema],
            &BTreeMap::new(),
            &TestAdapterResolver(adapter_id),
        )
        .expect("admitted Project State")
}

fn add_revision(
    state_id: ProjectStateId,
    states: &BTreeMap<ProjectStateId, ProjectState>,
    revisions: &mut BTreeMap<RevisionId, Revision>,
    message: &str,
) -> RevisionId {
    let revision = RevisionCandidate::new(
        REVISION_SCHEMA,
        state_id,
        vec![],
        ACTOR_ID.parse::<ActorId>().expect("Actor ID"),
        "2026-10-09T07:00:00.000000000Z",
        message,
        vec![],
    )
    .admit(&[&TestRevisionSchema], states, revisions)
    .expect("admitted Revision");
    let id = revision.revision_id();
    revisions.insert(id, revision);
    id
}

fn generation(value: u64) -> LineGeneration {
    LineGeneration::try_from(value).expect("valid generation")
}

#[test]
fn line_serialization_has_exact_five_member_closed_record() {
    let fixture = Fixture::new();
    let line = fixture.create(0, 0, "Main");
    let value = serde_json::to_value(&line).expect("serialize Line");
    let object = value.as_object().expect("Line object");
    assert_eq!(
        object.keys().map(String::as_str).collect::<Vec<_>>(),
        [
            "generation",
            "line_id",
            "name",
            "project_id",
            "target_revision"
        ]
    );

    for member in [
        "generation",
        "line_id",
        "name",
        "project_id",
        "target_revision",
    ] {
        let mut missing = value.clone();
        missing.as_object_mut().expect("object").remove(member);
        assert!(
            serde_json::from_value::<Line>(missing).is_err(),
            "missing {member}"
        );
    }

    let mut extended = value;
    extended
        .as_object_mut()
        .expect("object")
        .insert("extension".to_owned(), Value::Null);
    assert!(serde_json::from_value::<Line>(extended).is_err());
    let duplicate_member = format!(
        r#"{{"line_id":"{}","project_id":"{}","name":"Main","target_revision":"{}","generation":0,"name":"Again"}}"#,
        line.line_id(),
        line.project_id(),
        line.target_revision()
    );
    assert!(serde_json::from_str::<Line>(&duplicate_member).is_err());
}

#[test]
fn line_identity_is_assigned_uuidv7_and_independent_of_name_and_target() {
    let fixture = Fixture::new();
    let line = fixture.create(0, 0, "Main");
    let line_id = line.line_id();
    let text = line_id.to_string();
    assert_eq!(text.parse::<LineId>(), Ok(line_id));
    assert_eq!(text, text.to_ascii_lowercase());
    assert_eq!(text.as_bytes()[14], b'7');

    let renamed = fixture
        .repository
        .rename_line(line_id, generation(0), "Arrangement".to_owned())
        .expect("rename succeeds");
    let moved = fixture
        .repository
        .move_line(
            line_id,
            fixture.targets[0][0],
            generation(1),
            fixture.targets[0][1],
            &fixture.revisions,
            &fixture.states,
        )
        .expect("move succeeds");
    assert_eq!(renamed.line_id(), line_id);
    assert_eq!(moved.line_id(), line_id);
    assert_eq!(moved.project_id(), line.project_id());
}

#[test]
fn line_names_use_exact_project_scoped_equality() {
    let fixture = Fixture::new();
    let first = fixture.create(0, 0, "Mix");
    let lower_case = fixture.create(0, 0, "mix");
    let other_project = fixture.create(1, 0, "Mix");
    assert_ne!(first.line_id(), lower_case.line_id());
    assert_ne!(first.line_id(), other_project.line_id());
    assert_eq!(
        fixture.repository.create_line(
            fixture.project_ids[0],
            "Mix".to_owned(),
            fixture.targets[0][1],
            &fixture.revisions,
            &fixture.states,
        ),
        Err(LineOperationError::NameConflict)
    );
    let composed = fixture.create(0, 0, "\u{00e9}");
    let decomposed = fixture.create(0, 0, "e\u{0301}");
    assert_ne!(composed.line_id(), decomposed.line_id());
}

#[test]
fn concurrent_creation_preserves_project_name_uniqueness_atomically() {
    use std::sync::{Arc, Barrier};
    use std::thread;

    let fixture = Fixture::new();
    let repository = Arc::new(InMemoryLineRepository::new(fixture.project_ids));
    let barrier = Arc::new(Barrier::new(3));
    let project_id = fixture.project_ids[0];
    let target_id = fixture.targets[0][0];
    let revisions = &fixture.revisions;
    let states = &fixture.states;
    let create = |repository: Arc<InMemoryLineRepository>,
                  barrier: Arc<Barrier>,
                  revisions: &BTreeMap<RevisionId, Revision>,
                  states: &BTreeMap<ProjectStateId, ProjectState>| {
        barrier.wait();
        repository.create_line(
            project_id,
            "one exact name".to_owned(),
            target_id,
            revisions,
            states,
        )
    };
    thread::scope(|scope| {
        let first = scope.spawn({
            let repository = Arc::clone(&repository);
            let barrier = Arc::clone(&barrier);
            move || create(repository, barrier, revisions, states)
        });
        let second = scope.spawn({
            let repository = Arc::clone(&repository);
            let barrier = Arc::clone(&barrier);
            move || create(repository, barrier, revisions, states)
        });
        barrier.wait();
        let first = first.join().expect("first creation thread");
        let second = second.join().expect("second creation thread");
        assert_eq!(usize::from(first.is_ok()) + usize::from(second.is_ok()), 1);
        assert!(
            matches!(
                (&first, &second),
                (Ok(_), Err(LineOperationError::NameConflict))
                    | (Err(LineOperationError::NameConflict), Ok(_))
            ),
            "expected one creation and one exact-name conflict: {first:?}; {second:?}"
        );
    });
}

#[test]
fn create_requires_an_admitted_same_project_revision_and_allows_shared_targets() {
    let fixture = Fixture::new();
    let first = fixture.create(0, 0, "A");
    let second = fixture.create(0, 0, "B");
    assert_eq!(first.target_revision(), second.target_revision());
    assert_eq!(first.generation(), LineGeneration::ZERO);
    assert_eq!(
        fixture.repository.create_line(
            fixture.project_ids[0],
            "Foreign".to_owned(),
            fixture.targets[1][0],
            &fixture.revisions,
            &fixture.states,
        ),
        Err(LineOperationError::TargetProjectMismatch)
    );
    assert_eq!(
        fixture.repository.create_line(
            fixture.project_ids[0],
            "Unadmitted".to_owned(),
            RevisionId::from_digest([0xee; 32]),
            &fixture.revisions,
            &fixture.states,
        ),
        Err(LineOperationError::RevisionNotAdmitted)
    );
    assert!(
        fixture
            .repository
            .create_line(
                fixture.project_ids[0],
                "Unadmitted".to_owned(),
                fixture.targets[0][0],
                &fixture.revisions,
                &fixture.states,
            )
            .is_ok(),
        "failed creation must not leave a partial Line or reserve its name"
    );
}

#[test]
fn create_line_does_not_require_resource_bytes() {
    let fixture = Fixture::new();
    // The admitted Revision/Project State resolvers are the complete target
    // validation boundary; no Resource-byte resolver is supplied to Core.
    let line = fixture.create(0, 0, "Metadata-only target");
    assert_eq!(line.target_revision(), fixture.targets[0][0]);
}

#[test]
fn create_rejects_nonexistent_project_without_partial_line() {
    let fixture = Fixture::new();
    let repository = InMemoryLineRepository::new([fixture.project_ids[0]]);
    assert_eq!(
        repository.create_line(
            fixture.project_ids[1],
            "No Project".to_owned(),
            fixture.targets[1][0],
            &fixture.revisions,
            &fixture.states,
        ),
        Err(LineOperationError::ProjectNotFound)
    );
}

#[test]
fn line_generation_accepts_only_exact_nonnegative_safe_json_integers() {
    let fixture = Fixture::new();
    let line = fixture.create(0, 0, "Generation");
    let mut record = serde_json::to_value(&line).expect("Line record");
    for value in [json!(0), json!(7), json!(MAX_LINE_GENERATION)] {
        record["generation"] = value.clone();
        let parsed: Line = serde_json::from_value(record.clone()).expect("valid generation");
        assert_eq!(parsed.generation().get(), value.as_u64().expect("integer"));
    }
    for value in [
        json!(MAX_LINE_GENERATION + 1),
        json!(-1),
        json!(1.5),
        json!("1"),
        Value::Null,
    ] {
        record["generation"] = value;
        assert!(serde_json::from_value::<Line>(record.clone()).is_err());
    }
    assert_eq!(
        LineGeneration::try_from(MAX_LINE_GENERATION + 1),
        Err(LineOperationError::InvalidGeneration)
    );

    for number in ["-1", "1.0", "1e0", "9007199254740992", "\"1\""] {
        let source = format!(
            r#"{{"line_id":"{}","project_id":"{}","name":"Generation","target_revision":"{}","generation":{number}}}"#,
            line.line_id(),
            line.project_id(),
            line.target_revision()
        );
        assert!(
            serde_json::from_str::<Line>(&source).is_err(),
            "invalid generation representation {number}"
        );
    }
}

#[test]
fn line_generation_integer_serialization_is_stable_and_canonicalizable() {
    let fixture = Fixture::new();
    let mut record = serde_json::to_value(fixture.create(0, 0, "Stable")).expect("Line record");
    record["generation"] = json!(MAX_LINE_GENERATION);
    let line: Line = serde_json::from_value(record).expect("maximum generation");
    let body = serde_json::to_vec(&line).expect("serialize Line");
    let schema = MetadataSchema::structure([
        ("line_id", MetadataSchema::Scalar),
        ("project_id", MetadataSchema::Scalar),
        ("name", MetadataSchema::Scalar),
        ("target_revision", MetadataSchema::Scalar),
        ("generation", MetadataSchema::Scalar),
    ]);
    let first =
        canonicalize_metadata_body(&body, &schema).expect("canonicalize Line operational data");
    let second = canonicalize_metadata_body(&body, &schema).expect("canonicalize same Line again");
    assert_eq!(first, second);
    assert!(
        String::from_utf8(first)
            .expect("canonical JSON UTF-8")
            .contains(r#""generation":9007199254740991"#)
    );
}

#[test]
fn move_line_increments_once_even_when_target_does_not_change() {
    let fixture = Fixture::new();
    let line = fixture.create(0, 0, "Move");
    let same_target = fixture
        .repository
        .move_line(
            line.line_id(),
            line.target_revision(),
            line.generation(),
            line.target_revision(),
            &fixture.revisions,
            &fixture.states,
        )
        .expect("same-target CAS succeeds");
    assert_eq!(same_target.generation().get(), 1);
    assert_eq!(same_target.target_revision(), line.target_revision());
    let moved = fixture
        .repository
        .move_line(
            line.line_id(),
            same_target.target_revision(),
            same_target.generation(),
            fixture.targets[0][1],
            &fixture.revisions,
            &fixture.states,
        )
        .expect("movement succeeds");
    assert_eq!(moved.generation().get(), 2);
    assert_eq!(moved.line_id(), line.line_id());
    assert_eq!(moved.project_id(), line.project_id());
    assert_eq!(moved.name(), line.name());
}

#[test]
fn move_line_distinguishes_stale_expected_target_and_generation() {
    let fixture = Fixture::new();
    let line = fixture.create(0, 0, "CAS");
    assert_eq!(
        fixture.repository.move_line(
            line.line_id(),
            fixture.targets[0][1],
            generation(0),
            fixture.targets[0][1],
            &fixture.revisions,
            &fixture.states,
        ),
        Err(LineOperationError::StaleTarget)
    );
    assert_eq!(
        fixture.repository.move_line(
            line.line_id(),
            line.target_revision(),
            generation(1),
            fixture.targets[0][1],
            &fixture.revisions,
            &fixture.states,
        ),
        Err(LineOperationError::StaleGeneration)
    );
    assert_eq!(fixture.repository.get_line(line.line_id()), Ok(Some(line)));
}

#[test]
fn move_line_rejects_missing_and_cross_project_targets_without_partial_updates() {
    let fixture = Fixture::new();
    let line = fixture.create(0, 0, "Validation");
    assert_eq!(
        fixture.repository.move_line(
            line.line_id(),
            line.target_revision(),
            line.generation(),
            fixture.targets[1][0],
            &fixture.revisions,
            &fixture.states,
        ),
        Err(LineOperationError::TargetProjectMismatch)
    );
    assert_eq!(
        fixture.repository.move_line(
            line.line_id(),
            line.target_revision(),
            line.generation(),
            RevisionId::from_digest([0xdd; 32]),
            &fixture.revisions,
            &fixture.states,
        ),
        Err(LineOperationError::RevisionNotAdmitted)
    );
    assert_eq!(fixture.repository.get_line(line.line_id()), Ok(Some(line)));
}

#[test]
fn rename_line_changes_only_name_and_increments_even_for_same_name() {
    let fixture = Fixture::new();
    let line = fixture.create(0, 0, "Original");
    let same = fixture
        .repository
        .rename_line(line.line_id(), line.generation(), "Original".to_owned())
        .expect("same-name rename succeeds");
    assert_eq!(same.generation().get(), 1);
    assert_eq!(same.name(), "Original");
    assert_eq!(same.line_id(), line.line_id());
    assert_eq!(same.project_id(), line.project_id());
    assert_eq!(same.target_revision(), line.target_revision());

    let renamed = fixture
        .repository
        .rename_line(same.line_id(), same.generation(), "New".to_owned())
        .expect("rename succeeds");
    assert_eq!(renamed.generation().get(), 2);
    assert_eq!(renamed.name(), "New");
    assert_eq!(renamed.target_revision(), line.target_revision());
}

#[test]
fn rename_line_rejects_stale_generation_and_exact_name_conflict_atomically() {
    let fixture = Fixture::new();
    let first = fixture.create(0, 0, "First");
    let second = fixture.create(0, 0, "Second");
    assert_eq!(
        fixture
            .repository
            .rename_line(first.line_id(), generation(1), "New".to_owned()),
        Err(LineOperationError::StaleGeneration)
    );
    assert_eq!(
        fixture.repository.rename_line(
            first.line_id(),
            first.generation(),
            second.name().to_owned()
        ),
        Err(LineOperationError::NameConflict)
    );
    assert_eq!(
        fixture.repository.get_line(first.line_id()),
        Ok(Some(first))
    );
    assert_eq!(
        fixture.repository.get_line(second.line_id()),
        Ok(Some(second))
    );
}

#[test]
fn deleting_a_missing_line_is_distinguishable_from_stale_generation() {
    let fixture = Fixture::new();
    let line = fixture.create(0, 0, "Delete");
    assert_eq!(
        fixture
            .repository
            .delete_line(line.line_id(), generation(1)),
        Err(LineOperationError::StaleGeneration)
    );
    assert_eq!(
        fixture.repository.delete_line(
            "019cc17d-1b22-7a41-9fe9-c345c468f831"
                .parse()
                .expect("Line ID"),
            generation(0)
        ),
        Err(LineOperationError::LineNotFound)
    );
    assert_eq!(fixture.repository.get_line(line.line_id()), Ok(Some(line)));
}

#[test]
fn deleting_current_generation_removes_only_line_record() {
    let fixture = Fixture::new();
    let line = fixture.create(0, 0, "Delete");
    assert_eq!(
        fixture
            .repository
            .delete_line(line.line_id(), line.generation()),
        Ok(())
    );
    assert_eq!(fixture.repository.get_line(line.line_id()), Ok(None));
    assert!(fixture.revisions.contains_key(&line.target_revision()));
}

#[test]
fn default_line_starts_absent_and_creation_does_not_assign_it() {
    let fixture = Fixture::new();
    assert_eq!(
        fixture.repository.default_line(fixture.project_ids[0]),
        Ok(None)
    );
    let _line = fixture.create(0, 0, "No implicit default");
    assert_eq!(
        fixture.repository.default_line(fixture.project_ids[0]),
        Ok(None)
    );
}

#[test]
fn default_line_can_be_set_changed_cleared_and_is_independent_of_local_selection() {
    let fixture = Fixture::new();
    let first = fixture.create(0, 0, "First");
    let second = fixture.create(0, 1, "Second");
    let first_before = fixture
        .repository
        .get_line(first.line_id())
        .expect("read first Line")
        .expect("first Line");
    let second_before = fixture
        .repository
        .get_line(second.line_id())
        .expect("read second Line")
        .expect("second Line");
    let revision_ids = fixture.revisions.keys().copied().collect::<Vec<_>>();
    fixture
        .repository
        .set_default_line(fixture.project_ids[0], None, Some(first.line_id()))
        .expect("set default");
    assert_eq!(
        fixture.repository.default_line(fixture.project_ids[0]),
        Ok(Some(first.line_id()))
    );
    assert_eq!(
        fixture.repository.get_line(first.line_id()),
        Ok(Some(first_before.clone()))
    );
    assert_eq!(
        fixture.repository.get_line(second.line_id()),
        Ok(Some(second_before.clone()))
    );
    fixture
        .repository
        .set_default_line(
            fixture.project_ids[0],
            Some(first.line_id()),
            Some(second.line_id()),
        )
        .expect("change default");
    assert_eq!(
        fixture.repository.default_line(fixture.project_ids[0]),
        Ok(Some(second.line_id()))
    );

    let mut local_selection = Some(first.line_id());
    assert_eq!(local_selection, Some(first.line_id()));
    local_selection = Some(second.line_id());
    assert_eq!(local_selection, Some(second.line_id()));
    assert_eq!(
        fixture.repository.default_line(fixture.project_ids[0]),
        Ok(Some(second.line_id()))
    );
    local_selection = None;
    assert_eq!(local_selection, None);
    assert_eq!(
        fixture.repository.default_line(fixture.project_ids[0]),
        Ok(Some(second.line_id()))
    );

    fixture
        .repository
        .set_default_line(fixture.project_ids[0], Some(second.line_id()), None)
        .expect("clear default");
    assert_eq!(
        fixture.repository.default_line(fixture.project_ids[0]),
        Ok(None)
    );
    assert_eq!(
        fixture.repository.get_line(first.line_id()),
        Ok(Some(first_before))
    );
    assert_eq!(
        fixture.repository.get_line(second.line_id()),
        Ok(Some(second_before))
    );
    assert_eq!(
        fixture.revisions.keys().copied().collect::<Vec<_>>(),
        revision_ids
    );
}

#[test]
fn default_line_cas_rejects_stale_missing_and_cross_project_values_atomically() {
    let fixture = Fixture::new();
    let first = fixture.create(0, 0, "First");
    let foreign = fixture.create(1, 0, "Foreign");
    assert_eq!(
        fixture
            .repository
            .set_default_line(fixture.project_ids[0], Some(first.line_id()), None,),
        Err(LineOperationError::StaleDefaultLine)
    );
    assert_eq!(
        fixture
            .repository
            .set_default_line(fixture.project_ids[0], None, Some(foreign.line_id()),),
        Err(LineOperationError::DefaultLineProjectMismatch)
    );
    assert_eq!(
        fixture.repository.set_default_line(
            fixture.project_ids[0],
            None,
            Some(
                "019cc17d-1b22-7a41-9fe9-c345c468f832"
                    .parse()
                    .expect("Line ID")
            ),
        ),
        Err(LineOperationError::LineNotFound)
    );
    assert_eq!(
        fixture.repository.default_line(fixture.project_ids[0]),
        Ok(None)
    );
}

#[test]
fn default_line_operations_do_not_mutate_lines_when_renamed_or_moved() {
    let fixture = Fixture::new();
    let line = fixture.create(0, 0, "Default");
    fixture
        .repository
        .set_default_line(fixture.project_ids[0], None, Some(line.line_id()))
        .expect("set default");
    let renamed = fixture
        .repository
        .rename_line(line.line_id(), line.generation(), "Renamed".to_owned())
        .expect("rename");
    let moved = fixture
        .repository
        .move_line(
            renamed.line_id(),
            renamed.target_revision(),
            renamed.generation(),
            fixture.targets[0][1],
            &fixture.revisions,
            &fixture.states,
        )
        .expect("move");
    assert_eq!(moved.generation().get(), 2);
    assert_eq!(
        fixture.repository.default_line(fixture.project_ids[0]),
        Ok(Some(line.line_id()))
    );
}

#[test]
fn default_line_requires_existing_project_and_delete_protects_current_default() {
    let fixture = Fixture::new();
    let line = fixture.create(0, 0, "Protected");
    let missing_project = "019cc17d-1b22-7a41-9fe9-c345c468f830"
        .parse::<ProjectId>()
        .expect("Project ID");
    assert_eq!(
        fixture
            .repository
            .set_default_line(missing_project, None, None),
        Err(LineOperationError::ProjectNotFound)
    );
    fixture
        .repository
        .set_default_line(fixture.project_ids[0], None, Some(line.line_id()))
        .expect("set default");
    assert_eq!(
        fixture
            .repository
            .delete_line(line.line_id(), line.generation()),
        Err(LineOperationError::DefaultLineProtected)
    );
    assert_eq!(
        fixture.repository.get_line(line.line_id()),
        Ok(Some(line.clone()))
    );
    assert_eq!(
        fixture.repository.default_line(fixture.project_ids[0]),
        Ok(Some(line.line_id()))
    );
}

#[test]
fn changing_or_clearing_default_permits_only_explicit_line_deletion() {
    let fixture = Fixture::new();
    let first = fixture.create(0, 0, "First");
    let second = fixture.create(0, 1, "Second");
    fixture
        .repository
        .set_default_line(fixture.project_ids[0], None, Some(first.line_id()))
        .expect("set first default");
    fixture
        .repository
        .set_default_line(
            fixture.project_ids[0],
            Some(first.line_id()),
            Some(second.line_id()),
        )
        .expect("explicitly change default");
    fixture
        .repository
        .delete_line(first.line_id(), first.generation())
        .expect("former default can be deleted");
    assert_eq!(
        fixture.repository.default_line(fixture.project_ids[0]),
        Ok(Some(second.line_id()))
    );

    fixture
        .repository
        .set_default_line(fixture.project_ids[0], Some(second.line_id()), None)
        .expect("explicitly clear default");
    fixture
        .repository
        .delete_line(second.line_id(), second.generation())
        .expect("cleared default can be deleted");
    assert_eq!(
        fixture.repository.default_line(fixture.project_ids[0]),
        Ok(None),
        "deleting a Line must not implicitly choose a fallback"
    );
    assert!(fixture.revisions.contains_key(&first.target_revision()));
    assert!(fixture.revisions.contains_key(&second.target_revision()));
}

#[test]
fn competing_default_assignment_and_deletion_cannot_leave_a_dangling_reference() {
    use std::sync::{Arc, Barrier};
    use std::thread;

    let fixture = Fixture::new();
    let line = fixture.create(0, 0, "Concurrent");
    let repo = Arc::new(fixture.repository);
    let barrier = Arc::new(Barrier::new(3));
    let set_repo = Arc::clone(&repo);
    let set_barrier = Arc::clone(&barrier);
    let project_id = fixture.project_ids[0];
    let line_id = line.line_id();
    let set = thread::spawn(move || {
        set_barrier.wait();
        set_repo.set_default_line(project_id, None, Some(line_id))
    });
    let delete_repo = Arc::clone(&repo);
    let delete_barrier = Arc::clone(&barrier);
    let generation = line.generation();
    let delete = thread::spawn(move || {
        delete_barrier.wait();
        delete_repo.delete_line(line_id, generation)
    });
    barrier.wait();

    let set_result = set.join().expect("set thread");
    let delete_result = delete.join().expect("delete thread");
    let default_won = matches!(
        (&set_result, &delete_result),
        (Ok(()), Err(LineOperationError::DefaultLineProtected))
    );
    let deletion_won = matches!(
        (&set_result, &delete_result),
        (Err(LineOperationError::LineNotFound), Ok(()))
    );
    assert!(
        default_won || deletion_won,
        "unexpected atomic outcomes: {set_result:?}, {delete_result:?}"
    );
    if default_won {
        assert_eq!(repo.default_line(project_id), Ok(Some(line_id)));
        assert!(repo.get_line(line_id).expect("read Line").is_some());
    } else {
        assert_eq!(repo.default_line(project_id), Ok(None));
        assert_eq!(repo.get_line(line_id), Ok(None));
    }
}
