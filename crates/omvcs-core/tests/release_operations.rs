#![allow(clippy::expect_used)]

use std::collections::BTreeMap;
use std::sync::Arc;

use omvcs_core::release::{
    InMemoryReleaseRepository, ReleaseOperationBoundary, ReleaseOperationError,
};
use omvcs_model::canonical::MetadataSchema;
use omvcs_model::project_state::{
    AdmittedAdapterStateResolver, ProjectStateCandidate, ProjectStateSchemaValidator,
};
use omvcs_model::release::{RELEASE_SCHEMA, ReleaseCandidate, ReleaseSchemaValidator};
use omvcs_model::revision::{RevisionCandidate, RevisionSchemaValidator};
use omvcs_model::{
    ActorId, AdapterStateId, ProjectId, ProjectState, ProjectStateId, Release, ReleaseId, Revision,
    RevisionId,
};
use serde_json::{Value, json};

const PROJECT_SCHEMA: &str = "release-core-test.project-state/1";
const REVISION_SCHEMA: &str = "release-core-test.revision/1";
const PROJECT_TEXT: &str = "019cc17d-1b22-7a41-9fe9-c345c468f82e";
const OTHER_PROJECT_TEXT: &str = "019cc17d-1b22-7a41-9fe9-c345c468f82f";
const ACTOR_TEXT: &str = "019cc17d-1b22-7a41-9fe9-c345c468f82c";
const CREATED_AT: &str = "2026-10-08T11:02:17.000000000Z";

struct TestReleaseSchema;

impl ReleaseSchemaValidator for TestReleaseSchema {
    fn schema(&self) -> &str {
        RELEASE_SCHEMA
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

struct Fixture {
    project_ids: Vec<ProjectId>,
    states: BTreeMap<ProjectStateId, ProjectState>,
    revisions: BTreeMap<RevisionId, Revision>,
}

impl Fixture {
    fn new(project_texts: &[&str]) -> Self {
        let actor_id = ACTOR_TEXT.parse::<ActorId>().expect("valid ActorId");
        let states = project_texts
            .iter()
            .map(|project_text| {
                let adapter_id = AdapterStateId::from_digest([0x31; 32]);
                let candidate: ProjectStateCandidate = serde_json::from_value(json!({
                    "schema": PROJECT_SCHEMA,
                    "project_id": project_text,
                    "components": {},
                    "adapter_state_id": adapter_id.to_string(),
                    "project_metadata": {}
                }))
                .expect("valid Project State candidate");
                let state = candidate
                    .admit(
                        &[&TestProjectSchema],
                        &BTreeMap::new(),
                        &AdapterResolver(adapter_id),
                    )
                    .expect("admitted Project State");
                (state.project_state_id(), state)
            })
            .collect::<BTreeMap<_, _>>();
        let revisions = states
            .values()
            .map(|state| {
                let revision = RevisionCandidate::new(
                    REVISION_SCHEMA,
                    state.project_state_id(),
                    vec![],
                    actor_id,
                    CREATED_AT,
                    "Release target",
                    vec![],
                )
                .admit(&[&TestRevisionSchema], &states, &BTreeMap::new())
                .expect("admitted Revision");
                (revision.revision_id(), revision)
            })
            .collect();
        Self {
            project_ids: project_texts
                .iter()
                .map(|text| text.parse().expect("valid ProjectId"))
                .collect(),
            states,
            revisions,
        }
    }

    fn project(project_text: &str) -> ProjectId {
        project_text.parse().expect("valid ProjectId")
    }

    fn revision(&self, project_text: &str) -> RevisionId {
        let project_id = Self::project(project_text);
        self.revisions
            .values()
            .find(|revision| self.states[&revision.project_state_id()].project_id() == project_id)
            .expect("Revision for Project")
            .revision_id()
    }

    fn candidate(&self, project_text: &str, name: &str, description: &str) -> ReleaseCandidate {
        ReleaseCandidate::new(
            Self::project(project_text),
            name,
            self.revision(project_text),
            CREATED_AT,
            ACTOR_TEXT.parse().expect("valid ActorId"),
            description,
        )
    }

    fn repository(&self) -> InMemoryReleaseRepository {
        InMemoryReleaseRepository::new(self.project_ids.iter().copied())
    }
}

fn create(
    repository: &dyn ReleaseOperationBoundary,
    fixture: &Fixture,
    candidate: ReleaseCandidate,
) -> Result<Release, ReleaseOperationError> {
    repository.create_release(
        candidate,
        &[&TestReleaseSchema],
        &fixture.revisions,
        &fixture.states,
    )
}

#[test]
fn exact_duplicate_body_creation_is_idempotent_and_preserves_the_stored_object() {
    let fixture = Fixture::new(&[PROJECT_TEXT]);
    let repository = fixture.repository();
    let candidate = fixture.candidate(PROJECT_TEXT, "v1", "first public release");
    let first = create(&repository, &fixture, candidate.clone()).expect("create Release");
    let retry = create(&repository, &fixture, candidate).expect("exact duplicate succeeds");

    assert_eq!(retry, first);
    assert_eq!(retry.release_id(), first.release_id());
    assert_eq!(retry.canonical_body(), first.canonical_body());
    assert_eq!(repository.get_release(first.release_id()), Ok(Some(first)));
}

#[test]
fn another_body_cannot_claim_an_existing_project_scoped_name_and_failure_is_atomic() {
    let fixture = Fixture::new(&[PROJECT_TEXT]);
    let repository = fixture.repository();
    let original_candidate = fixture.candidate(PROJECT_TEXT, "v1", "original");
    let original =
        create(&repository, &fixture, original_candidate).expect("create original Release");
    let conflicting_candidate = fixture.candidate(PROJECT_TEXT, "v1", "different body");
    let conflicting = conflicting_candidate
        .clone()
        .admit(&[&TestReleaseSchema], &fixture.revisions, &fixture.states)
        .expect("candidate is valid independently");

    assert_eq!(
        create(&repository, &fixture, conflicting_candidate),
        Err(ReleaseOperationError::NameConflict)
    );
    assert_eq!(
        repository.get_release(conflicting.release_id()),
        Ok(None),
        "failed creation must not insert the candidate body"
    );
    assert_eq!(
        repository.get_release(original.release_id()),
        Ok(Some(original))
    );

    let retry_with_other_name = fixture.candidate(PROJECT_TEXT, "v2", "different body");
    let accepted = create(&repository, &fixture, retry_with_other_name)
        .expect("failed name conflict did not leave a partial name claim");
    assert_eq!(accepted.name(), "v2");
}

#[test]
fn release_names_are_exact_project_scoped_strings_without_normalization() {
    let fixture = Fixture::new(&[PROJECT_TEXT, OTHER_PROJECT_TEXT]);
    let repository = fixture.repository();
    let first = create(
        &repository,
        &fixture,
        fixture.candidate(PROJECT_TEXT, "v1", ""),
    )
    .expect("create first name");
    assert_eq!(first.description(), "");

    let other_case = create(
        &repository,
        &fixture,
        fixture.candidate(PROJECT_TEXT, "V1", ""),
    )
    .expect("case-distinct exact name");
    let composed = create(
        &repository,
        &fixture,
        fixture.candidate(PROJECT_TEXT, "\u{00e9}", ""),
    )
    .expect("composed Unicode name");
    let decomposed = create(
        &repository,
        &fixture,
        fixture.candidate(PROJECT_TEXT, "e\u{0301}", ""),
    )
    .expect("decomposed Unicode name");
    let other_project_same_name = create(
        &repository,
        &fixture,
        fixture.candidate(OTHER_PROJECT_TEXT, "v1", ""),
    )
    .expect("other Project has independent namespace");

    assert_ne!(first.release_id(), other_case.release_id());
    assert_ne!(composed.release_id(), decomposed.release_id());
    assert_eq!(other_project_same_name.name(), first.name());
    assert_eq!(
        other_project_same_name.project_id(),
        Fixture::project(OTHER_PROJECT_TEXT)
    );
}

#[test]
fn concurrent_creates_for_one_exact_name_commit_at_most_one_release() {
    let fixture = Arc::new(Fixture::new(&[PROJECT_TEXT]));
    let repository = Arc::new(fixture.repository());
    let left = fixture.candidate(PROJECT_TEXT, "same-name", "left description");
    let right = fixture.candidate(PROJECT_TEXT, "same-name", "right description");

    let (left_result, right_result) = std::thread::scope(|scope| {
        let left_thread = scope.spawn(|| create(&*repository, &fixture, left));
        let right_thread = scope.spawn(|| create(&*repository, &fixture, right));
        (
            left_thread.join().expect("left creation"),
            right_thread.join().expect("right creation"),
        )
    });
    let success_count = usize::from(left_result.is_ok()) + usize::from(right_result.is_ok());
    let conflict_count = usize::from(left_result == Err(ReleaseOperationError::NameConflict))
        + usize::from(right_result == Err(ReleaseOperationError::NameConflict));
    assert_eq!(success_count, 1);
    assert_eq!(conflict_count, 1);
}

#[test]
fn missing_project_and_cross_project_targets_have_distinct_failures() {
    let fixture = Fixture::new(&[PROJECT_TEXT, OTHER_PROJECT_TEXT]);
    let only_first_project_repository =
        InMemoryReleaseRepository::new([Fixture::project(PROJECT_TEXT)]);
    assert_eq!(
        only_first_project_repository.create_release(
            fixture.candidate(OTHER_PROJECT_TEXT, "foreign-owner", ""),
            &[&TestReleaseSchema],
            &fixture.revisions,
            &fixture.states,
        ),
        Err(ReleaseOperationError::ProjectNotFound)
    );

    let cross_target = ReleaseCandidate::new(
        Fixture::project(PROJECT_TEXT),
        "cross-target",
        fixture.revision(OTHER_PROJECT_TEXT),
        CREATED_AT,
        ACTOR_TEXT.parse().expect("valid ActorId"),
        "",
    );
    assert_eq!(
        create(&only_first_project_repository, &fixture, cross_target),
        Err(ReleaseOperationError::Admission(
            omvcs_model::release::ReleaseAdmissionError::TargetProjectMismatch
        ))
    );
}

#[test]
fn operation_lookup_exposes_release_root_target_without_mutation_operations() {
    let fixture = Fixture::new(&[PROJECT_TEXT]);
    let repository = fixture.repository();
    let target = fixture.revision(PROJECT_TEXT);
    let release = create(
        &repository,
        &fixture,
        fixture.candidate(PROJECT_TEXT, "root-edge", ""),
    )
    .expect("created Release");

    let resolved = repository
        .get_release(release.release_id())
        .expect("lookup")
        .expect("Release exists");
    assert_eq!(resolved.revision_id(), target);
    assert_eq!(resolved.canonical_body(), release.canonical_body());
    let _: ReleaseId = resolved.release_id();
}
