#![allow(clippy::expect_used)]

use std::collections::BTreeMap;

use omvcs_model::canonical::{ArrayOrdering, CanonicalMetadataError, MetadataSchema};
use omvcs_model::component_state::{ComponentStateCandidate, ComponentStateSchemaValidator};
use omvcs_model::hashing::hash_revision_metadata;
use omvcs_model::project_state::{
    AdmittedAdapterStateResolver, ProjectStateCandidate, ProjectStateSchemaValidator,
};
use omvcs_model::revision::{RevisionAdmissionError, RevisionCandidate, RevisionSchemaValidator};
use omvcs_model::{
    ActorId, AdapterStateId, ComponentState, CreativeComponentId, ProjectId, ProjectState,
    ProjectStateId, Revision, RevisionId,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const REVISION_SCHEMA: &str = "test-only.revision/1";
const SECOND_REVISION_SCHEMA: &str = "test-only.revision/2";
const PROJECT_SCHEMA: &str = "test-only.project-state/1";
const PROJECT_ID: &str = "019cc17d-1b22-7a41-9fe9-c345c468f82e";
const OTHER_PROJECT_ID: &str = "019cc17d-1b22-7a41-9fe9-c345c468f82f";
const ACTOR_ID: &str = "019cc17d-1b22-7a41-9fe9-c345c468f82c";
const OTHER_ACTOR_ID: &str = "019cc17d-1b22-7a41-9fe9-c345c468f82d";
const COMPONENT_SCHEMA: &str = "test-only.component-state/1";
const COMPONENT_ID: &str = "019cc17d-1b22-7a41-9fe9-c345c468f830";
const CREATED_AT: &str = "2026-10-08T11:02:17.000000000Z";

/// This deliberately test-only schema exercises validator plumbing. Its keys
/// and values have no production or OMVCS provenance meaning.
struct TestRevisionSchema {
    schema: &'static str,
    entry_schema: MetadataSchema,
}

#[test]
fn revision_identifier_members_are_decoded_as_their_typed_identifier_classes() {
    let base = json!({
        "schema": REVISION_SCHEMA,
        "project_state_id": "omvcs:project-state:sha256:0000000000000000000000000000000000000000000000000000000000000000",
        "parents": [],
        "author_id": ACTOR_ID,
        "created_at": CREATED_AT,
        "message": "",
        "provenance": []
    });
    assert!(serde_json::from_value::<RevisionCandidate>(base.clone()).is_ok());

    let mut wrong_project_namespace = base.clone();
    wrong_project_namespace["project_state_id"] = json!(
        "omvcs:revision:sha256:0000000000000000000000000000000000000000000000000000000000000000"
    );
    assert!(serde_json::from_value::<RevisionCandidate>(wrong_project_namespace).is_err());

    let mut wrong_parent_namespace = base.clone();
    wrong_parent_namespace["parents"] = json!([
        "omvcs:project-state:sha256:0000000000000000000000000000000000000000000000000000000000000000"
    ]);
    assert!(serde_json::from_value::<RevisionCandidate>(wrong_parent_namespace).is_err());

    let mut invalid_actor = base;
    invalid_actor["author_id"] = json!("actor:not-an-assigned-identifier");
    assert!(serde_json::from_value::<RevisionCandidate>(invalid_actor).is_err());
}

impl TestRevisionSchema {
    fn new(schema: &'static str) -> Self {
        Self {
            schema,
            entry_schema: MetadataSchema::structure([
                ("fixture_label", MetadataSchema::Scalar),
                (
                    "fixture_members",
                    MetadataSchema::array(ArrayOrdering::SetLike, MetadataSchema::Scalar),
                ),
                (
                    "fixture_sequence",
                    MetadataSchema::array(ArrayOrdering::Ordered, MetadataSchema::Scalar),
                ),
            ]),
        }
    }

    fn unclassified_nested_array() -> Self {
        Self {
            schema: REVISION_SCHEMA,
            entry_schema: MetadataSchema::structure([(
                "fixture_unclassified",
                MetadataSchema::unclassified_array(MetadataSchema::Scalar),
            )]),
        }
    }

    fn entry(label: &str) -> Value {
        json!({"fixture_label": label})
    }

    fn rich_entry(label: &str, members: &[&str], sequence: &[&str]) -> Value {
        json!({
            "fixture_label": label,
            "fixture_members": members,
            "fixture_sequence": sequence
        })
    }
}

impl RevisionSchemaValidator for TestRevisionSchema {
    fn schema(&self) -> &str {
        self.schema
    }

    fn provenance_entry_schema(&self) -> MetadataSchema {
        self.entry_schema.clone()
    }

    fn validate_provenance(&self, entries: &[Value]) -> Result<(), String> {
        if entries.iter().any(|entry| {
            entry
                .get("fixture_label")
                .is_none_or(|label| !label.is_string())
        }) {
            return Err("test-only fixture requires fixture_label".to_owned());
        }
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

struct TestAdapterResolver(AdapterStateId);

impl AdmittedAdapterStateResolver for TestAdapterResolver {
    fn resolve_admitted(&self, id: AdapterStateId) -> Option<AdapterStateId> {
        (id == self.0).then_some(id)
    }
}

fn actor_id(text: &str) -> ActorId {
    text.parse().expect("valid UUIDv7 ActorId fixture")
}

fn project_id(text: &str) -> ProjectId {
    text.parse().expect("valid UUIDv7 ProjectId fixture")
}

fn project_state(project: &str) -> ProjectState {
    let adapter_id = AdapterStateId::from_digest([0x31; 32]);
    let candidate: ProjectStateCandidate = serde_json::from_value(json!({
        "schema": PROJECT_SCHEMA,
        "project_id": project,
        "components": {},
        "adapter_state_id": adapter_id.to_string(),
        "project_metadata": {}
    }))
    .expect("valid test Project State candidate");
    candidate
        .admit(
            &[&TestProjectSchema],
            &BTreeMap::new(),
            &TestAdapterResolver(adapter_id),
        )
        .expect("valid test Project State")
}

fn project_states() -> BTreeMap<ProjectStateId, ProjectState> {
    let state = project_state(PROJECT_ID);
    BTreeMap::from([(state.project_state_id(), state)])
}

fn candidate(
    project_state_id: ProjectStateId,
    parents: Vec<RevisionId>,
    message: &str,
    created_at: &str,
    provenance: Vec<Value>,
) -> RevisionCandidate {
    RevisionCandidate::new(
        REVISION_SCHEMA,
        project_state_id,
        parents,
        actor_id(ACTOR_ID),
        created_at,
        message,
        provenance,
    )
}

fn admit(
    candidate: RevisionCandidate,
    schemas: &[&dyn RevisionSchemaValidator],
    project_states: &BTreeMap<ProjectStateId, ProjectState>,
    revisions: &BTreeMap<RevisionId, Revision>,
) -> Result<Revision, RevisionAdmissionError> {
    candidate.admit(schemas, project_states, revisions)
}

fn fixture_entry(label: &str) -> Vec<Value> {
    vec![TestRevisionSchema::entry(label)]
}

#[test]
fn revision_body_is_exactly_seven_members_and_hashes_only_canonical_body() {
    let schema = TestRevisionSchema::new(REVISION_SCHEMA);
    let states = project_states();
    let state_id = *states.keys().next().expect("one Project State");
    let revision = admit(
        candidate(state_id, vec![], "New take", CREATED_AT, vec![]),
        &[&schema],
        &states,
        &BTreeMap::new(),
    )
    .expect("valid initial Revision");

    let body: Value =
        serde_json::from_slice(revision.canonical_body()).expect("canonical JSON body");
    let object = body.as_object().expect("body is object");
    assert_eq!(
        object.keys().map(String::as_str).collect::<Vec<_>>(),
        [
            "author_id",
            "created_at",
            "message",
            "parents",
            "project_state_id",
            "provenance",
            "schema"
        ]
    );
    assert!(!object.contains_key("project_id"));
    let expected_body = format!(
        r#"{{"author_id":"{ACTOR_ID}","created_at":"{CREATED_AT}","message":"New take","parents":[],"project_state_id":"{state_id}","provenance":[],"schema":"{REVISION_SCHEMA}"}}"#
    );
    assert_eq!(revision.canonical_body(), expected_body.as_bytes());
    let digest = Sha256::digest(revision.canonical_body());
    assert_eq!(
        revision.revision_id().digest().as_slice(),
        digest.as_slice()
    );
    assert_eq!(
        revision.revision_id(),
        hash_revision_metadata(revision.canonical_body())
    );
}

#[test]
fn every_required_member_is_required_and_unknown_top_level_members_are_rejected() {
    let required = [
        "schema",
        "project_state_id",
        "parents",
        "author_id",
        "created_at",
        "message",
        "provenance",
    ];
    let valid = json!({
        "schema": REVISION_SCHEMA,
        "project_state_id": "omvcs:project-state:sha256:0000000000000000000000000000000000000000000000000000000000000000",
        "parents": [],
        "author_id": ACTOR_ID,
        "created_at": CREATED_AT,
        "message": "",
        "provenance": []
    });
    for member in required {
        let mut body = valid.clone();
        body.as_object_mut().expect("JSON object").remove(member);
        assert!(
            serde_json::from_value::<RevisionCandidate>(body).is_err(),
            "missing {member} must be rejected"
        );
    }
    for member in [
        "project_id",
        "author",
        "signature",
        "line_id",
        "release_id",
        "replica",
        "validation_evidence",
        "storage_location",
    ] {
        let mut body = valid.clone();
        body.as_object_mut()
            .expect("JSON object")
            .insert(member.to_owned(), json!("outside the closed body"));
        assert!(
            serde_json::from_value::<RevisionCandidate>(body).is_err(),
            "unknown member {member} must be rejected"
        );
    }
}

#[test]
fn duplicate_raw_json_members_are_rejected_before_candidate_decoding() {
    let duplicate_top_level = format!(
        r#"{{"schema":"{REVISION_SCHEMA}","project_state_id":"{}","parents":[],"author_id":"{ACTOR_ID}","created_at":"{CREATED_AT}","message":"","message":"duplicate","provenance":[]}}"#,
        "omvcs:project-state:sha256:".to_owned() + &"00".repeat(32)
    );
    assert!(serde_json::from_str::<RevisionCandidate>(&duplicate_top_level).is_err());

    let duplicate_nested = format!(
        r#"{{"schema":"{REVISION_SCHEMA}","project_state_id":"{}","parents":[],"author_id":"{ACTOR_ID}","created_at":"{CREATED_AT}","message":"","provenance":[{{"fixture_label":"first","fixture_label":"second"}}]}}"#,
        "omvcs:project-state:sha256:".to_owned() + &"00".repeat(32)
    );
    assert!(serde_json::from_str::<RevisionCandidate>(&duplicate_nested).is_err());
}

#[test]
fn exact_revision_schema_must_be_available_unique_and_object_directed() {
    let schema = TestRevisionSchema::new(REVISION_SCHEMA);
    let duplicate_schema = TestRevisionSchema::new(REVISION_SCHEMA);
    let invalid_schema = TestRevisionSchema {
        schema: REVISION_SCHEMA,
        entry_schema: MetadataSchema::map(MetadataSchema::Scalar),
    };
    let states = project_states();
    let state_id = *states.keys().next().expect("one Project State");

    assert_eq!(
        candidate(state_id, vec![], "", CREATED_AT, vec![])
            .admit(&[], &states, &BTreeMap::new())
            .expect_err("missing exact schema authority"),
        RevisionAdmissionError::UnavailableSchema
    );
    assert_eq!(
        RevisionCandidate::new(
            SECOND_REVISION_SCHEMA,
            state_id,
            vec![],
            actor_id(ACTOR_ID),
            CREATED_AT,
            "",
            vec![],
        )
        .admit(&[&schema], &states, &BTreeMap::new())
        .expect_err("schema version mismatch"),
        RevisionAdmissionError::UnavailableSchema
    );
    assert_eq!(
        candidate(state_id, vec![], "", CREATED_AT, vec![])
            .admit(&[&schema, &duplicate_schema], &states, &BTreeMap::new())
            .expect_err("duplicate schema authority"),
        RevisionAdmissionError::NonUniqueSchemaAuthority
    );
    assert_eq!(
        candidate(state_id, vec![], "", CREATED_AT, vec![])
            .admit(&[&invalid_schema], &states, &BTreeMap::new())
            .expect_err("invalid provenance schema"),
        RevisionAdmissionError::InvalidProvenanceSchema
    );
    assert!(matches!(
        candidate(
            state_id,
            vec![],
            "",
            CREATED_AT,
            vec![json!({"not_a_fixture_field": true})]
        )
        .admit(&[&schema], &states, &BTreeMap::new())
        .expect_err("schema must reject invalid provenance shape"),
        RevisionAdmissionError::Canonical(CanonicalMetadataError::SchemaMismatch { .. })
    ));
}

#[test]
fn revision_requires_an_admitted_resolvable_project_state() {
    let schema = TestRevisionSchema::new(REVISION_SCHEMA);
    let state = project_state(PROJECT_ID);
    let state_id = state.project_state_id();
    assert_eq!(
        admit(
            candidate(state_id, vec![], "", CREATED_AT, vec![]),
            &[&schema],
            &BTreeMap::new(),
            &BTreeMap::new()
        )
        .expect_err("Project State must resolve"),
        RevisionAdmissionError::UnavailableProjectState
    );
}

#[test]
fn parents_are_required_set_like_admitted_same_project_revision_references() {
    let schema = TestRevisionSchema::new(REVISION_SCHEMA);
    let states = project_states();
    let state_id = *states.keys().next().expect("one Project State");
    let root = admit(
        candidate(state_id, vec![], "root", CREATED_AT, vec![]),
        &[&schema],
        &states,
        &BTreeMap::new(),
    )
    .expect("valid root");
    let root_id = root.revision_id();
    let revisions = BTreeMap::from([(root_id, root)]);

    assert_eq!(
        admit(
            candidate(state_id, vec![root_id], "child", CREATED_AT, vec![]),
            &[&schema],
            &states,
            &BTreeMap::new()
        )
        .expect_err("parent Revision must resolve"),
        RevisionAdmissionError::UnavailableParent
    );
    assert_eq!(
        admit(
            candidate(
                state_id,
                vec![root_id, root_id],
                "duplicate",
                CREATED_AT,
                vec![]
            ),
            &[&schema],
            &states,
            &revisions
        )
        .expect_err("duplicate parents must be rejected"),
        RevisionAdmissionError::DuplicateParent
    );
    let child = admit(
        candidate(state_id, vec![root_id], "child", CREATED_AT, vec![]),
        &[&schema],
        &states,
        &revisions,
    )
    .expect("valid derived Revision");
    assert_eq!(child.parents(), &[root_id]);
}

#[test]
fn parent_order_is_nonsemantic_and_multiple_parents_are_supported() {
    let schema = TestRevisionSchema::new(REVISION_SCHEMA);
    let states = project_states();
    let state_id = *states.keys().next().expect("one Project State");
    let first = admit(
        candidate(state_id, vec![], "first parent", CREATED_AT, vec![]),
        &[&schema],
        &states,
        &BTreeMap::new(),
    )
    .expect("first parent");
    let second = admit(
        candidate(state_id, vec![], "second parent", CREATED_AT, vec![]),
        &[&schema],
        &states,
        &BTreeMap::new(),
    )
    .expect("second parent");
    let first_id = first.revision_id();
    let second_id = second.revision_id();
    let revisions = BTreeMap::from([(first_id, first), (second_id, second)]);

    let forward = admit(
        candidate(
            state_id,
            vec![first_id, second_id],
            "integration",
            CREATED_AT,
            vec![],
        ),
        &[&schema],
        &states,
        &revisions,
    )
    .expect("multiple parents");
    let reverse = admit(
        candidate(
            state_id,
            vec![second_id, first_id],
            "integration",
            CREATED_AT,
            vec![],
        ),
        &[&schema],
        &states,
        &revisions,
    )
    .expect("permuted multiple parents");
    assert_eq!(forward.revision_id(), reverse.revision_id());
    assert_eq!(forward.canonical_body(), reverse.canonical_body());
    assert_eq!(forward.parents().len(), 2);
}

#[test]
fn parent_project_must_match_project_state_and_timestamps_do_not_establish_ancestry() {
    let schema = TestRevisionSchema::new(REVISION_SCHEMA);
    let current_state = project_state(PROJECT_ID);
    let other_state = project_state(OTHER_PROJECT_ID);
    let current_id = current_state.project_state_id();
    let other_id = other_state.project_state_id();
    let states = BTreeMap::from([(current_id, current_state), (other_id, other_state)]);
    let parent = admit(
        candidate(
            other_id,
            vec![],
            "parent",
            "2026-10-08T11:02:18.000000000Z",
            vec![],
        ),
        &[&schema],
        &states,
        &BTreeMap::new(),
    )
    .expect("parent from other Project");
    let parent_id = parent.revision_id();
    let revisions = BTreeMap::from([(parent_id, parent)]);

    assert_eq!(
        admit(
            candidate(
                current_id,
                vec![parent_id],
                "earlier timestamp",
                CREATED_AT,
                vec![]
            ),
            &[&schema],
            &states,
            &revisions
        )
        .expect_err("cross-Project parent must be rejected"),
        RevisionAdmissionError::ParentProjectMismatch
    );

    let same_project_parent = admit(
        candidate(
            current_id,
            vec![],
            "later",
            "2026-10-08T11:02:18.000000000Z",
            vec![],
        ),
        &[&schema],
        &states,
        &BTreeMap::new(),
    )
    .expect("same Project parent");
    let parent_id = same_project_parent.revision_id();
    let same_project_revisions = BTreeMap::from([(parent_id, same_project_parent)]);
    let earlier_child = admit(
        candidate(
            current_id,
            vec![parent_id],
            "earlier timestamp",
            CREATED_AT,
            vec![],
        ),
        &[&schema],
        &states,
        &same_project_revisions,
    );
    assert!(earlier_child.is_ok());
}

#[test]
fn canonical_timestamp_requires_exact_utc_nanosecond_profile_and_valid_calendar_values() {
    let schema = TestRevisionSchema::new(REVISION_SCHEMA);
    let states = project_states();
    let state_id = *states.keys().next().expect("one Project State");
    for invalid in [
        "2026-10-08T11:02:17Z",
        "2026-10-08T11:02:17.00000000Z",
        "2026-10-08T11:02:17.0000000000Z",
        "2026-10-08T11:02:17.000000000+00:00",
        "2026-10-08t11:02:17.000000000z",
        "2026-02-29T11:02:17.000000000Z",
        "2026-13-08T11:02:17.000000000Z",
        "2026-10-32T11:02:17.000000000Z",
        "2026-10-08T24:02:17.000000000Z",
        "2026-10-08T11:60:17.000000000Z",
        "2026-10-08T11:02:61.000000000Z",
        "2026-10-31T23:59:60.000000000Z",
        "2016-12-30T23:59:60.000000000Z",
        "2016-12-31T22:59:60.000000000Z",
        "2017-12-31T23:59:60.000000000Z",
        "2026-06-29T23:59:60.000000000Z",
    ] {
        assert_eq!(
            admit(
                candidate(state_id, vec![], "", invalid, vec![]),
                &[&schema],
                &states,
                &BTreeMap::new()
            )
            .expect_err("invalid timestamp must be rejected"),
            RevisionAdmissionError::InvalidTimestamp,
            "rejected timestamp {invalid}"
        );
    }
    assert!(
        admit(
            candidate(
                state_id,
                vec![],
                "",
                "2024-02-29T23:59:59.999999999Z",
                vec![]
            ),
            &[&schema],
            &states,
            &BTreeMap::new()
        )
        .is_ok()
    );
}

#[test]
fn only_announced_utc_leap_second_instants_are_accepted() {
    let schema = TestRevisionSchema::new(REVISION_SCHEMA);
    let states = project_states();
    let state_id = *states.keys().next().expect("one Project State");
    let announced_dates = [
        (1972, 6, 30),
        (1972, 12, 31),
        (1973, 12, 31),
        (1974, 12, 31),
        (1975, 12, 31),
        (1976, 12, 31),
        (1977, 12, 31),
        (1978, 12, 31),
        (1979, 12, 31),
        (1981, 6, 30),
        (1982, 6, 30),
        (1983, 6, 30),
        (1985, 6, 30),
        (1987, 12, 31),
        (1989, 12, 31),
        (1990, 12, 31),
        (1992, 6, 30),
        (1993, 6, 30),
        (1994, 6, 30),
        (1995, 12, 31),
        (1997, 6, 30),
        (1998, 12, 31),
        (2005, 12, 31),
        (2008, 12, 31),
        (2012, 6, 30),
        (2015, 6, 30),
        (2016, 12, 31),
    ];
    for (year, month, day) in announced_dates {
        let timestamp = format!("{year:04}-{month:02}-{day:02}T23:59:60.000000000Z");
        assert!(
            admit(
                candidate(state_id, vec![], "", &timestamp, vec![]),
                &[&schema],
                &states,
                &BTreeMap::new()
            )
            .is_ok(),
            "announced UTC leap second {timestamp} must be accepted"
        );
    }
}

#[test]
fn message_is_verbatim_and_empty_message_is_valid() {
    let schema = TestRevisionSchema::new(REVISION_SCHEMA);
    let states = project_states();
    let state_id = *states.keys().next().expect("one Project State");
    let empty = admit(
        candidate(state_id, vec![], "", CREATED_AT, vec![]),
        &[&schema],
        &states,
        &BTreeMap::new(),
    )
    .expect("empty message is valid");
    let exact = "  Case-sensitive café\n\t";
    let with_text = admit(
        candidate(state_id, vec![], exact, CREATED_AT, vec![]),
        &[&schema],
        &states,
        &BTreeMap::new(),
    )
    .expect("string message is valid");
    assert_eq!(with_text.message(), exact);
    assert_ne!(empty.revision_id(), with_text.revision_id());
}

#[test]
#[allow(clippy::too_many_lines)]
fn each_canonical_revision_member_contributes_to_identity() {
    let schema_one = TestRevisionSchema::new(REVISION_SCHEMA);
    let schema_two = TestRevisionSchema::new(SECOND_REVISION_SCHEMA);
    let states = {
        let first = project_state(PROJECT_ID);
        let second = project_state(OTHER_PROJECT_ID);
        BTreeMap::from([
            (first.project_state_id(), first),
            (second.project_state_id(), second),
        ])
    };
    let project_ids = states.keys().copied().collect::<Vec<_>>();
    let first_state_id = project_ids[0];
    let second_state_id = project_ids[1];
    let baseline = admit(
        candidate(first_state_id, vec![], "same", CREATED_AT, vec![]),
        &[&schema_one, &schema_two],
        &states,
        &BTreeMap::new(),
    )
    .expect("baseline");
    let other_schema = RevisionCandidate::new(
        SECOND_REVISION_SCHEMA,
        first_state_id,
        vec![],
        actor_id(ACTOR_ID),
        CREATED_AT,
        "same",
        vec![],
    )
    .admit(&[&schema_one, &schema_two], &states, &BTreeMap::new())
    .expect("other schema");
    let other_project_state = admit(
        candidate(second_state_id, vec![], "same", CREATED_AT, vec![]),
        &[&schema_one, &schema_two],
        &states,
        &BTreeMap::new(),
    )
    .expect("other Project State");
    let other_author = RevisionCandidate::new(
        REVISION_SCHEMA,
        first_state_id,
        vec![],
        actor_id(OTHER_ACTOR_ID),
        CREATED_AT,
        "same",
        vec![],
    )
    .admit(&[&schema_one, &schema_two], &states, &BTreeMap::new())
    .expect("other ActorId");
    let other_time = admit(
        candidate(
            first_state_id,
            vec![],
            "same",
            "2026-10-08T11:02:18.000000000Z",
            vec![],
        ),
        &[&schema_one, &schema_two],
        &states,
        &BTreeMap::new(),
    )
    .expect("other timestamp");
    let other_message = admit(
        candidate(first_state_id, vec![], "different", CREATED_AT, vec![]),
        &[&schema_one, &schema_two],
        &states,
        &BTreeMap::new(),
    )
    .expect("other message");
    let other_provenance = admit(
        candidate(
            first_state_id,
            vec![],
            "same",
            CREATED_AT,
            fixture_entry("different"),
        ),
        &[&schema_one, &schema_two],
        &states,
        &BTreeMap::new(),
    )
    .expect("valid test-only provenance");

    let parent = admit(
        candidate(first_state_id, vec![], "parent", CREATED_AT, vec![]),
        &[&schema_one, &schema_two],
        &states,
        &BTreeMap::new(),
    )
    .expect("admitted parent");
    let parent_id = parent.revision_id();
    let parent_map = BTreeMap::from([(parent_id, parent)]);
    let other_parents = admit(
        candidate(first_state_id, vec![parent_id], "same", CREATED_AT, vec![]),
        &[&schema_one, &schema_two],
        &states,
        &parent_map,
    )
    .expect("different parent set");

    for changed in [
        other_schema.revision_id(),
        other_project_state.revision_id(),
        other_parents.revision_id(),
        other_author.revision_id(),
        other_time.revision_id(),
        other_message.revision_id(),
        other_provenance.revision_id(),
    ] {
        assert_ne!(baseline.revision_id(), changed);
    }
}

#[test]
fn provenance_uses_exact_schema_for_nested_validation_sorting_and_duplicates() {
    let schema = TestRevisionSchema::new(REVISION_SCHEMA);
    let states = project_states();
    let state_id = *states.keys().next().expect("one Project State");
    let first = TestRevisionSchema::rich_entry("z", &["second", "first"], &["one", "two"]);
    let second = TestRevisionSchema::rich_entry("a", &["member"], &["two", "one"]);
    let admitted = admit(
        candidate(state_id, vec![], "", CREATED_AT, vec![first, second]),
        &[&schema],
        &states,
        &BTreeMap::new(),
    )
    .expect("test-only schema validates entries");
    let entries = admitted.provenance();
    assert_eq!(entries[0]["fixture_label"], "a");
    assert_eq!(entries[1]["fixture_label"], "z");
    assert_eq!(entries[1]["fixture_members"], json!(["first", "second"]));
    assert_eq!(entries[1]["fixture_sequence"], json!(["one", "two"]));
    let permuted = admit(
        candidate(
            state_id,
            vec![],
            "",
            CREATED_AT,
            vec![
                TestRevisionSchema::rich_entry("a", &["member"], &["two", "one"]),
                TestRevisionSchema::rich_entry("z", &["first", "second"], &["one", "two"]),
            ],
        ),
        &[&schema],
        &states,
        &BTreeMap::new(),
    )
    .expect("permuted test-only provenance");
    assert_eq!(admitted.revision_id(), permuted.revision_id());

    let one = TestRevisionSchema::entry("duplicate");
    assert!(matches!(
        candidate(state_id, vec![], "", CREATED_AT, vec![one.clone(), one]).admit(
            &[&schema],
            &states,
            &BTreeMap::new()
        ),
        Err(RevisionAdmissionError::Canonical(
            CanonicalMetadataError::DuplicateSetLikeElement { .. }
        ))
    ));
    assert!(matches!(
        candidate(
            state_id,
            vec![],
            "",
            CREATED_AT,
            vec![TestRevisionSchema::rich_entry(
                "nested duplicates",
                &["same", "same"],
                &[]
            )]
        )
        .admit(&[&schema], &states, &BTreeMap::new()),
        Err(RevisionAdmissionError::Canonical(
            CanonicalMetadataError::DuplicateSetLikeElement { .. }
        ))
    ));
    let nested_schema = TestRevisionSchema::unclassified_nested_array();
    assert!(matches!(
        candidate(
            state_id,
            vec![],
            "",
            CREATED_AT,
            vec![json!({"fixture_unclassified": []})]
        )
        .admit(&[&nested_schema], &states, &BTreeMap::new()),
        Err(RevisionAdmissionError::Canonical(
            CanonicalMetadataError::UnclassifiedArray { .. }
        ))
    ));
}

#[test]
fn actor_and_project_identity_are_typed_and_platform_independent() {
    let schema = TestRevisionSchema::new(REVISION_SCHEMA);
    let states = project_states();
    let state_id = *states.keys().next().expect("one Project State");
    let revision = admit(
        candidate(state_id, vec![], "", CREATED_AT, vec![]),
        &[&schema],
        &states,
        &BTreeMap::new(),
    )
    .expect("valid Revision");
    assert_eq!(revision.author_id(), actor_id(ACTOR_ID));
    assert_eq!(project_id(PROJECT_ID).to_string(), PROJECT_ID);
    assert_eq!(revision.created_at(), CREATED_AT);
}

#[test]
fn project_state_without_resource_byte_materialization_remains_sufficient() {
    // This admitted Component State references a Resource by its typed ID and
    // complete byte length, but no Resource bytes or storage object exist.
    let schema = TestRevisionSchema::new(REVISION_SCHEMA);
    let adapter_id = AdapterStateId::from_digest([0x31; 32]);
    let component_id: CreativeComponentId = COMPONENT_ID.parse().expect("typed component ID");
    let component_candidate: ComponentStateCandidate = serde_json::from_value(json!({
        "schema": COMPONENT_SCHEMA,
        "component_id": COMPONENT_ID,
        "resources": [{
            "resource_id": "omvcs:resource:sha256:0000000000000000000000000000000000000000000000000000000000000000",
            "byte_length": 42
        }],
        "metadata": {}
    }))
    .expect("valid metadata-only Component State candidate");
    let component_state: ComponentState = component_candidate
        .admit(&[&TestComponentSchema], &[])
        .expect("valid Component State without Resource bytes");
    let component_state_id = component_state.component_state_id();
    let components = BTreeMap::from([(component_state_id, component_state)]);
    let project_candidate: ProjectStateCandidate = serde_json::from_value(json!({
        "schema": PROJECT_SCHEMA,
        "project_id": PROJECT_ID,
        "components": {
            (component_id.to_string()): component_state_id.to_string()
        },
        "adapter_state_id": adapter_id.to_string(),
        "project_metadata": {}
    }))
    .expect("valid Project State candidate");
    let project = project_candidate
        .admit(
            &[&TestProjectSchema],
            &components,
            &TestAdapterResolver(adapter_id),
        )
        .expect("valid Project State referencing metadata-only Component State");
    let state_id = project.project_state_id();
    let states = BTreeMap::from([(state_id, project)]);
    assert!(
        admit(
            candidate(state_id, vec![], "", CREATED_AT, vec![]),
            &[&schema],
            &states,
            &BTreeMap::new()
        )
        .is_ok()
    );
}

#[test]
fn schema_authority_rejection_prevents_history_identity() {
    let rejecting = RejectingRevisionSchema(TestRevisionSchema::new(REVISION_SCHEMA));
    let states = project_states();
    let state_id = *states.keys().next().expect("one Project State");
    assert_eq!(
        candidate(
            state_id,
            vec![],
            "",
            CREATED_AT,
            fixture_entry("allowed by shape but rejected semantically")
        )
        .admit(&[&rejecting], &states, &BTreeMap::new())
        .expect_err("exact schema rejection must prevent admission"),
        RevisionAdmissionError::ProvenanceRejected("test-only rejection".to_owned())
    );
}

struct RejectingRevisionSchema(TestRevisionSchema);

impl RevisionSchemaValidator for RejectingRevisionSchema {
    fn schema(&self) -> &str {
        self.0.schema()
    }

    fn provenance_entry_schema(&self) -> MetadataSchema {
        self.0.provenance_entry_schema()
    }

    fn validate_provenance(&self, _: &[Value]) -> Result<(), String> {
        Err("test-only rejection".to_owned())
    }
}

#[test]
fn timestamp_or_profile_changes_never_order_parentage_by_creation_time() {
    let schema = TestRevisionSchema::new(REVISION_SCHEMA);
    let states = project_states();
    let state_id = *states.keys().next().expect("one Project State");
    let root = admit(
        candidate(
            state_id,
            vec![],
            "later root",
            "2026-10-08T11:02:19.000000000Z",
            vec![],
        ),
        &[&schema],
        &states,
        &BTreeMap::new(),
    )
    .expect("valid root");
    let root_id = root.revision_id();
    let revisions = BTreeMap::from([(root_id, root)]);
    let child = admit(
        candidate(state_id, vec![root_id], "earlier child", CREATED_AT, vec![]),
        &[&schema],
        &states,
        &revisions,
    )
    .expect("parent relationship, not timestamp, defines ancestry");
    assert_eq!(child.parents(), &[root_id]);
}
