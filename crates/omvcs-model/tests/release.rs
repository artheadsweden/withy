#![allow(clippy::expect_used)]

use std::collections::BTreeMap;

use omvcs_model::canonical::{MetadataSchema, canonicalize_metadata_body};
use omvcs_model::hashing::hash_release_metadata;
use omvcs_model::project_state::{
    AdmittedAdapterStateResolver, ProjectStateCandidate, ProjectStateSchemaValidator,
};
use omvcs_model::release::{
    RELEASE_SCHEMA, ReleaseAdmissionError, ReleaseCandidate, ReleaseSchemaValidator,
};
use omvcs_model::revision::{
    AdmittedProjectStateResolver, AdmittedRevisionResolver, RevisionCandidate,
    RevisionSchemaValidator,
};
use omvcs_model::{
    ActorId, AdapterStateId, ProjectId, ProjectState, ProjectStateId, ReleaseId, Revision,
    RevisionId,
};
use serde_json::{Value, json};

const PROJECT_SCHEMA: &str = "release-test.project-state/1";
const REVISION_SCHEMA: &str = "release-test.revision/1";
const PROJECT_TEXT: &str = "019cc17d-1b22-7a41-9fe9-c345c468f82e";
const OTHER_PROJECT_TEXT: &str = "019cc17d-1b22-7a41-9fe9-c345c468f82f";
const ACTOR_TEXT: &str = "019cc17d-1b22-7a41-9fe9-c345c468f82c";
const OTHER_ACTOR_TEXT: &str = "019cc17d-1b22-7a41-9fe9-c345c468f82d";
const CREATED_AT: &str = "2026-10-08T11:02:17.000000000Z";

struct TestReleaseSchema {
    schema: &'static str,
    rejected: bool,
}

impl ReleaseSchemaValidator for TestReleaseSchema {
    fn schema(&self) -> &str {
        self.schema
    }

    fn validate_release(&self, _: &ReleaseCandidate) -> Result<(), String> {
        if self.rejected {
            Err("test schema rejection".to_owned())
        } else {
            Ok(())
        }
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

struct MismatchingRevisionResolver<'a>(&'a Revision);

impl AdmittedRevisionResolver for MismatchingRevisionResolver<'_> {
    fn resolve_admitted(&self, _: RevisionId) -> Option<&Revision> {
        Some(self.0)
    }
}

struct MismatchingProjectStateResolver<'a>(&'a ProjectState);

impl AdmittedProjectStateResolver for MismatchingProjectStateResolver<'_> {
    fn resolve_admitted(&self, _: ProjectStateId) -> Option<&ProjectState> {
        Some(self.0)
    }
}

fn actor_id(text: &str) -> ActorId {
    text.parse().expect("valid ActorId")
}

fn project_id(text: &str) -> ProjectId {
    text.parse().expect("valid ProjectId")
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
    .expect("valid Project State candidate");
    candidate
        .admit(
            &[&TestProjectSchema],
            &BTreeMap::new(),
            &AdapterResolver(adapter_id),
        )
        .expect("admitted Project State")
}

fn project_states() -> BTreeMap<ProjectStateId, ProjectState> {
    [
        project_state(PROJECT_TEXT),
        project_state(OTHER_PROJECT_TEXT),
    ]
    .into_iter()
    .map(|state| (state.project_state_id(), state))
    .collect()
}

fn revisions(states: &BTreeMap<ProjectStateId, ProjectState>) -> BTreeMap<RevisionId, Revision> {
    states
        .values()
        .map(|state| {
            let revision = RevisionCandidate::new(
                REVISION_SCHEMA,
                state.project_state_id(),
                vec![],
                actor_id(ACTOR_TEXT),
                CREATED_AT,
                "release target",
                vec![],
            )
            .admit(&[&TestRevisionSchema], states, &BTreeMap::new())
            .expect("admitted target Revision");
            (revision.revision_id(), revision)
        })
        .collect()
}

fn candidate(
    project: &str,
    target: RevisionId,
    name: &str,
    created_at: &str,
    creator: &str,
    description: &str,
) -> ReleaseCandidate {
    ReleaseCandidate::new(
        project_id(project),
        name,
        target,
        created_at,
        actor_id(creator),
        description,
    )
}

fn target_for(project: &str) -> RevisionId {
    let states = project_states();
    let revisions = revisions(&states);
    let project_id = project_id(project);
    revisions
        .values()
        .find(|revision| states[&revision.project_state_id()].project_id() == project_id)
        .expect("target Revision for Project")
        .revision_id()
}

fn raw_body() -> Value {
    json!({
        "schema": RELEASE_SCHEMA,
        "project_id": PROJECT_TEXT,
        "name": "v1",
        "revision_id": "omvcs:revision:sha256:1111111111111111111111111111111111111111111111111111111111111111",
        "created_at": CREATED_AT,
        "creator_id": ACTOR_TEXT,
        "description": "launch"
    })
}

#[test]
fn release_candidate_has_exact_closed_seven_member_body_and_rejects_unknown_or_null_fields() {
    let value = raw_body();
    let candidate: ReleaseCandidate =
        serde_json::from_value(value.clone()).expect("exact body is accepted");
    assert_eq!(candidate.schema(), RELEASE_SCHEMA);

    let mut unknown = value.clone();
    unknown["extra"] = json!("not allowed");
    assert!(serde_json::from_value::<ReleaseCandidate>(unknown).is_err());

    for member in [
        "schema",
        "project_id",
        "name",
        "revision_id",
        "created_at",
        "creator_id",
        "description",
    ] {
        let mut null_member = value.clone();
        null_member[member] = Value::Null;
        assert!(
            serde_json::from_value::<ReleaseCandidate>(null_member).is_err(),
            "null substitution for {member} must be rejected"
        );
    }

    let duplicate_member = format!(
        "{{\"schema\":\"{RELEASE_SCHEMA}\",\"schema\":\"{RELEASE_SCHEMA}\",\"project_id\":\"{PROJECT_TEXT}\",\"name\":\"v1\",\"revision_id\":\"omvcs:revision:sha256:{}\",\"created_at\":\"{CREATED_AT}\",\"creator_id\":\"{ACTOR_TEXT}\",\"description\":\"launch\"}}",
        "1".repeat(64)
    );
    assert!(serde_json::from_str::<ReleaseCandidate>(&duplicate_member).is_err());
}

#[test]
fn release_candidate_rejects_each_missing_required_top_level_member() {
    let valid = raw_body();
    for member in [
        "schema",
        "project_id",
        "name",
        "revision_id",
        "created_at",
        "creator_id",
        "description",
    ] {
        let mut missing_member = valid.clone();
        missing_member
            .as_object_mut()
            .expect("valid body is an object")
            .remove(member);
        assert!(
            serde_json::from_value::<ReleaseCandidate>(missing_member).is_err(),
            "omitted required member {member} must be rejected"
        );
    }
}

#[test]
fn release_wire_identifiers_are_decoded_as_their_typed_identifier_classes() {
    let value = raw_body();
    assert!(serde_json::from_value::<ReleaseCandidate>(value.clone()).is_ok());

    let mut wrong_project = value.clone();
    wrong_project["project_id"] = json!("not-a-ProjectId");
    assert!(serde_json::from_value::<ReleaseCandidate>(wrong_project).is_err());

    let mut wrong_revision_namespace = value.clone();
    wrong_revision_namespace["revision_id"] = json!(
        "omvcs:project-state:sha256:1111111111111111111111111111111111111111111111111111111111111111"
    );
    assert!(serde_json::from_value::<ReleaseCandidate>(wrong_revision_namespace).is_err());

    let mut invalid_actor = value;
    invalid_actor["creator_id"] = json!("actor:not-an-assigned-identifier");
    assert!(serde_json::from_value::<ReleaseCandidate>(invalid_actor).is_err());
}

#[test]
fn exact_schema_must_be_available_and_unique_and_must_accept_candidate() {
    let revision_id = target_for(PROJECT_TEXT);
    let candidate = candidate(PROJECT_TEXT, revision_id, "v1", CREATED_AT, ACTOR_TEXT, "");
    let states = project_states();
    let revisions = revisions(&states);

    assert_eq!(
        candidate.clone().admit(&[], &revisions, &states),
        Err(ReleaseAdmissionError::UnavailableSchema)
    );
    let authority = TestReleaseSchema {
        schema: RELEASE_SCHEMA,
        rejected: false,
    };
    let duplicate_authority = TestReleaseSchema {
        schema: RELEASE_SCHEMA,
        rejected: false,
    };
    assert_eq!(
        candidate
            .clone()
            .admit(&[&authority, &duplicate_authority], &revisions, &states),
        Err(ReleaseAdmissionError::NonUniqueSchemaAuthority)
    );
    let rejecting_authority = TestReleaseSchema {
        schema: RELEASE_SCHEMA,
        rejected: true,
    };
    assert!(matches!(
        candidate.admit(&[&rejecting_authority], &revisions, &states),
        Err(ReleaseAdmissionError::SchemaRejected(_))
    ));

    let wrong_schema: ReleaseCandidate = serde_json::from_value({
        let mut value = raw_body();
        value["schema"] = json!("omvcs.release/0.2");
        value
    })
    .expect("well-typed unsupported schema candidate");
    assert_eq!(
        wrong_schema.admit(&[&authority], &revisions, &states),
        Err(ReleaseAdmissionError::UnavailableSchema)
    );
}

#[test]
fn release_name_timestamp_actor_and_description_follow_the_approved_contract() {
    let states = project_states();
    let revisions = revisions(&states);
    let target = target_for(PROJECT_TEXT);
    let authority = TestReleaseSchema {
        schema: RELEASE_SCHEMA,
        rejected: false,
    };

    assert_eq!(
        candidate(PROJECT_TEXT, target, "", CREATED_AT, ACTOR_TEXT, "").admit(
            &[&authority],
            &revisions,
            &states
        ),
        Err(ReleaseAdmissionError::InvalidName)
    );
    for invalid in [
        "2026-10-08T11:02:17Z",
        "2026-10-08T11:02:17.00000000Z",
        "2026-10-08T11:02:17.0000000000Z",
        "2026-10-08T11:02:17.000000000+00:00",
        "2026-10-08t11:02:17.000000000z",
        "2026-02-29T11:02:17.000000000Z",
        "2026-10-08T24:02:17.000000000Z",
        "2026-10-08T11:60:17.000000000Z",
        "2026-10-08T11:02:61.000000000Z",
        "2017-12-31T23:59:60.000000000Z",
    ] {
        assert_eq!(
            candidate(PROJECT_TEXT, target, "v1", invalid, ACTOR_TEXT, "").admit(
                &[&authority],
                &revisions,
                &states
            ),
            Err(ReleaseAdmissionError::InvalidTimestamp),
            "invalid timestamp {invalid} must fail"
        );
    }
    assert!(
        candidate(
            PROJECT_TEXT,
            target,
            "v1",
            "2016-12-31T23:59:60.000000000Z",
            ACTOR_TEXT,
            ""
        )
        .admit(&[&authority], &revisions, &states)
        .is_ok()
    );

    let valid = candidate(PROJECT_TEXT, target, "v1", CREATED_AT, ACTOR_TEXT, "")
        .admit(&[&authority], &revisions, &states)
        .expect("empty description is allowed");
    assert_eq!(valid.creator_id(), actor_id(ACTOR_TEXT));
    assert_eq!(valid.description(), "");
    assert_eq!(valid.created_at(), CREATED_AT);
}

#[test]
fn release_target_requires_an_admitted_revision_and_same_project_state_without_resource_bytes() {
    let states = project_states();
    let revisions = revisions(&states);
    let authority = TestReleaseSchema {
        schema: RELEASE_SCHEMA,
        rejected: false,
    };

    assert_eq!(
        candidate(
            PROJECT_TEXT,
            RevisionId::from_digest([0x99; 32]),
            "v1",
            CREATED_AT,
            ACTOR_TEXT,
            ""
        )
        .admit(&[&authority], &revisions, &states),
        Err(ReleaseAdmissionError::UnavailableRevision)
    );

    let other_project_target = target_for(OTHER_PROJECT_TEXT);
    assert_eq!(
        candidate(
            PROJECT_TEXT,
            other_project_target,
            "v1",
            CREATED_AT,
            ACTOR_TEXT,
            ""
        )
        .admit(&[&authority], &revisions, &states),
        Err(ReleaseAdmissionError::TargetProjectMismatch)
    );

    let target = target_for(PROJECT_TEXT);
    let release = candidate(
        PROJECT_TEXT,
        target,
        "metadata only",
        CREATED_AT,
        ACTOR_TEXT,
        "",
    )
    .admit(&[&authority], &revisions, &states)
    .expect("no Resource-byte resolver is required for metadata admission");
    assert_eq!(release.revision_id(), target);
}

#[test]
fn release_target_resolvers_must_return_objects_matching_the_requested_identifiers() {
    let states = project_states();
    let revisions = revisions(&states);
    let target_id = target_for(PROJECT_TEXT);
    let foreign_target = target_for(OTHER_PROJECT_TEXT);
    let authority = TestReleaseSchema {
        schema: RELEASE_SCHEMA,
        rejected: false,
    };
    let target_candidate = candidate(
        PROJECT_TEXT,
        target_id,
        "resolver-mismatch",
        CREATED_AT,
        ACTOR_TEXT,
        "",
    );

    assert_eq!(
        target_candidate.clone().admit(
            &[&authority],
            &MismatchingRevisionResolver(&revisions[&foreign_target]),
            &states,
        ),
        Err(ReleaseAdmissionError::RevisionIdentifierMismatch)
    );

    let revision = &revisions[&target_id];
    let expected_state_id = revision.project_state_id();
    let mismatching_state = states
        .values()
        .find(|state| state.project_state_id() != expected_state_id)
        .expect("a distinct admitted Project State");
    assert_eq!(
        target_candidate.admit(
            &[&authority],
            &revisions,
            &MismatchingProjectStateResolver(mismatching_state),
        ),
        Err(ReleaseAdmissionError::ProjectStateIdentifierMismatch)
    );
}

#[test]
fn canonical_body_and_release_identifier_match_an_independent_golden_vector() {
    const EXPECTED_CANONICAL: &str = "{\"created_at\":\"2026-10-08T11:02:17.000000000Z\",\"creator_id\":\"019cc17d-1b22-7a41-9fe9-c345c468f82c\",\"description\":\"launch\",\"name\":\"v1\",\"project_id\":\"019cc17d-1b22-7a41-9fe9-c345c468f82e\",\"revision_id\":\"omvcs:revision:sha256:1084fda5317292d132b34316cd00fb969c25d789727dfddb6c440caf3f60371a\",\"schema\":\"omvcs.release/0.1\"}";
    const EXPECTED_ID: &str =
        "omvcs:release:sha256:1100b5f5f4b51f56b60e55a729b485320712923654b5b208efe55421845fcbcc";

    let states = project_states();
    let revisions = revisions(&states);
    let target = target_for(PROJECT_TEXT);
    let authority = TestReleaseSchema {
        schema: RELEASE_SCHEMA,
        rejected: false,
    };
    let release = candidate(PROJECT_TEXT, target, "v1", CREATED_AT, ACTOR_TEXT, "launch")
        .admit(&[&authority], &revisions, &states)
        .expect("admitted golden Release");

    assert_eq!(release.canonical_body(), EXPECTED_CANONICAL.as_bytes());
    assert_eq!(release.release_id().to_string(), EXPECTED_ID);
    assert_eq!(
        EXPECTED_ID.parse::<ReleaseId>().expect("typed ReleaseId"),
        release.release_id()
    );
    assert_eq!(
        hash_release_metadata(EXPECTED_CANONICAL.as_bytes()),
        release.release_id()
    );
}

#[test]
fn every_canonical_body_member_participates_in_the_release_digest() {
    let base = r#"{"created_at":"2026-10-08T11:02:17.000000000Z","creator_id":"019cc17d-1b22-7a41-9fe9-c345c468f82c","description":"launch","name":"v1","project_id":"019cc17d-1b22-7a41-9fe9-c345c468f82e","revision_id":"omvcs:revision:sha256:1111111111111111111111111111111111111111111111111111111111111111","schema":"omvcs.release/0.1"}"#;
    let schema = MetadataSchema::structure([
        ("schema", MetadataSchema::Scalar),
        ("project_id", MetadataSchema::Scalar),
        ("name", MetadataSchema::Scalar),
        ("revision_id", MetadataSchema::Scalar),
        ("created_at", MetadataSchema::Scalar),
        ("creator_id", MetadataSchema::Scalar),
        ("description", MetadataSchema::Scalar),
    ]);
    let baseline = hash_release_metadata(
        &canonicalize_metadata_body(base.as_bytes(), &schema).expect("canonical base"),
    );

    let field_variations = [
        ("schema", json!("omvcs.release/0.2")),
        ("project_id", json!(OTHER_PROJECT_TEXT)),
        ("name", json!("V1")),
        (
            "revision_id",
            json!(
                "omvcs:revision:sha256:2222222222222222222222222222222222222222222222222222222222222222"
            ),
        ),
        ("created_at", json!("2026-10-08T11:02:18.000000000Z")),
        ("creator_id", json!(OTHER_ACTOR_TEXT)),
        ("description", json!("launch!")),
    ];
    for (member, changed_value) in field_variations {
        let mut body: Value = serde_json::from_str(base).expect("valid base JSON");
        body[member] = changed_value;
        let serialized = serde_json::to_vec(&body).expect("body JSON");
        let varied_digest = hash_release_metadata(
            &canonicalize_metadata_body(&serialized, &schema).expect("canonical changed body"),
        );
        assert_ne!(varied_digest, baseline, "{member} must affect identity");
    }
}

#[test]
fn admitted_release_keeps_all_body_values_and_has_no_mutation_surface() {
    let states = project_states();
    let revisions = revisions(&states);
    let target = target_for(PROJECT_TEXT);
    let release = candidate(
        PROJECT_TEXT,
        target,
        "v1",
        CREATED_AT,
        ACTOR_TEXT,
        "immutable",
    )
    .admit(
        &[&TestReleaseSchema {
            schema: RELEASE_SCHEMA,
            rejected: false,
        }],
        &revisions,
        &states,
    )
    .expect("admitted Release");
    let body: Value =
        serde_json::from_slice(release.canonical_body()).expect("canonical object body");
    assert_eq!(body.as_object().expect("object").len(), 7);
    assert_eq!(body["schema"], RELEASE_SCHEMA);
    assert_eq!(body["project_id"], PROJECT_TEXT);
    assert_eq!(body["name"], "v1");
    assert_eq!(body["revision_id"], target.to_string());
    assert_eq!(body["created_at"], CREATED_AT);
    assert_eq!(body["creator_id"], ACTOR_TEXT);
    assert_eq!(body["description"], "immutable");
    assert_eq!(release.name(), "v1");
    assert_eq!(release.description(), "immutable");
    assert_eq!(release.revision_id(), target);
}

#[test]
fn release_identifier_text_requires_its_typed_lowercase_digest_format() {
    let expected =
        "omvcs:release:sha256:0000000000000000000000000000000000000000000000000000000000000000";
    let id = expected.parse::<ReleaseId>().expect("valid ReleaseId");
    assert_eq!(id.to_string(), expected);
    assert!(
        expected
            .replace("omvcs:release", "omvcs:revision")
            .parse::<ReleaseId>()
            .is_err()
    );
    assert!(
        "omvcs:release:sha256:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
            .parse::<ReleaseId>()
            .is_err()
    );
    assert!(
        "omvcs:release:sha256:000000000000000000000000000000000000000000000000000000000000000"
            .parse::<ReleaseId>()
            .is_err()
    );
}
