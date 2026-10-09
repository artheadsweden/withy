#![allow(clippy::expect_used)]

use std::collections::BTreeMap;

use omvcs_model::canonical::MetadataSchema;
use omvcs_model::component_state::{ComponentStateCandidate, ComponentStateSchemaValidator};
use omvcs_model::project_state::{
    AdmittedAdapterStateResolver, ComponentStateResolver, ProjectStateAdmissionError,
    ProjectStateCandidate, ProjectStateSchemaValidator,
};
use omvcs_model::release::{
    RELEASE_SCHEMA, ReleaseAdmissionError, ReleaseCandidate, ReleaseSchemaValidator,
};
use omvcs_model::revision::{
    AdmittedProjectStateResolver, AdmittedRevisionResolver, RevisionAdmissionError,
    RevisionCandidate, RevisionSchemaValidator,
};
use omvcs_model::{
    ActorId, AdapterStateId, ComponentStateId, ProjectId, ProjectState, ProjectStateId, ReleaseId,
    Revision, RevisionId,
};
use serde_json::{Value, json};

const PROJECT_SCHEMA: &str = "validation-test.project-state/1";
const REVISION_SCHEMA: &str = "validation-test.revision/1";
const COMPONENT_SCHEMA: &str = "validation-test.component-state/1";
const PROJECT_TEXT: &str = "019cc17d-1b22-7a41-9fe9-c345c468f82c";
const ACTOR_TEXT: &str = "019cc17d-1b22-7a41-9fe9-c345c468f82d";
const CREATED_AT: &str = "2026-10-08T11:02:17.000000000Z";

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

struct ComponentSchema;

impl ComponentStateSchemaValidator for ComponentSchema {
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

struct ReleaseSchema;

impl ReleaseSchemaValidator for ReleaseSchema {
    fn schema(&self) -> &str {
        RELEASE_SCHEMA
    }

    fn validate_release(&self, _: &ReleaseCandidate) -> Result<(), String> {
        Ok(())
    }
}

struct NoAdapter;

impl AdmittedAdapterStateResolver for NoAdapter {
    fn resolve_admitted(&self, _: AdapterStateId) -> Option<AdapterStateId> {
        None
    }
}

struct OneAdapter(AdapterStateId);

impl AdmittedAdapterStateResolver for OneAdapter {
    fn resolve_admitted(&self, id: AdapterStateId) -> Option<AdapterStateId> {
        (id == self.0).then_some(id)
    }
}

struct NoComponents;

impl ComponentStateResolver for NoComponents {
    fn resolve_admitted(&self, _: ComponentStateId) -> Option<&omvcs_model::ComponentState> {
        None
    }
}

struct NoProjectStates;

impl AdmittedProjectStateResolver for NoProjectStates {
    fn resolve_admitted(&self, _: ProjectStateId) -> Option<&ProjectState> {
        None
    }
}

struct OneProjectState<'a>(&'a ProjectState);

impl AdmittedProjectStateResolver for OneProjectState<'_> {
    fn resolve_admitted(&self, id: ProjectStateId) -> Option<&ProjectState> {
        (self.0.project_state_id() == id).then_some(self.0)
    }
}

struct NoRevisions;

impl AdmittedRevisionResolver for NoRevisions {
    fn resolve_admitted(&self, _: RevisionId) -> Option<&Revision> {
        None
    }
}

struct OneRevision<'a>(&'a Revision);

impl AdmittedRevisionResolver for OneRevision<'_> {
    fn resolve_admitted(&self, id: RevisionId) -> Option<&Revision> {
        (self.0.revision_id() == id).then_some(self.0)
    }
}

fn project_id() -> ProjectId {
    PROJECT_TEXT.parse().expect("ProjectId fixture")
}

fn actor_id() -> ActorId {
    ACTOR_TEXT.parse().expect("ActorId fixture")
}

fn admitted_project_state(adapter: AdapterStateId) -> ProjectState {
    let candidate: ProjectStateCandidate = serde_json::from_value(json!({
        "schema": PROJECT_SCHEMA,
        "project_id": project_id().to_string(),
        "components": {},
        "adapter_state_id": adapter.to_string(),
        "project_metadata": {}
    }))
    .expect("closed Project State candidate");
    candidate
        .admit(&[&ProjectSchema], &NoComponents, &OneAdapter(adapter))
        .expect("admitted Project State")
}

#[test]
fn project_state_body_identifier_is_independent_of_reference_admission() {
    let adapter = AdapterStateId::from_digest([31; 32]);
    let candidate: ProjectStateCandidate = serde_json::from_value(json!({
        "schema": PROJECT_SCHEMA,
        "project_id": project_id().to_string(),
        "components": {},
        "adapter_state_id": adapter.to_string(),
        "project_metadata": {}
    }))
    .expect("closed Project State candidate");

    let identifier = candidate
        .verify_body_identifier(&[&ProjectSchema])
        .expect("schema-valid body Identifier despite missing Adapter State");
    assert_eq!(
        candidate.admit(&[&ProjectSchema], &NoComponents, &NoAdapter),
        Err(ProjectStateAdmissionError::UnavailableAdapterState)
    );
    let admitted = serde_json::from_value::<ProjectStateCandidate>(json!({
        "schema": PROJECT_SCHEMA,
        "project_id": project_id().to_string(),
        "components": {},
        "adapter_state_id": adapter.to_string(),
        "project_metadata": {}
    }))
    .expect("identical Project State body")
    .admit(&[&ProjectSchema], &NoComponents, &OneAdapter(adapter))
    .expect("same body becomes admitted when target is available");
    assert_eq!(identifier, admitted.project_state_id());

    let unknown_schema: ProjectStateCandidate = serde_json::from_value(json!({
        "schema": "unknown.project-state/9",
        "project_id": project_id().to_string(),
        "components": {},
        "adapter_state_id": adapter.to_string(),
        "project_metadata": {}
    }))
    .expect("unknown schema remains a candidate");
    assert_eq!(
        unknown_schema.verify_body_identifier(&[&ProjectSchema]),
        Err(ProjectStateAdmissionError::UnavailableSchema)
    );
    assert_ne!(
        identifier,
        ProjectStateId::from_digest([0; 32]),
        "the body hash is a real typed identity, not an absent-target sentinel"
    );
}

#[test]
fn component_state_body_identifier_uses_its_schema_without_resource_bytes() {
    let component: omvcs_model::CreativeComponentId = "019cc17d-1b22-7a41-9fe9-c345c468f82e"
        .parse()
        .expect("Creative Component fixture");
    let missing_parent = ComponentStateId::from_digest([37; 32]);
    let candidate =
        ComponentStateCandidate::new(COMPONENT_SCHEMA, component, Vec::new(), BTreeMap::new())
            .with_parents(vec![missing_parent]);
    let identifier = candidate
        .verify_body_identifier(&[&ComponentSchema], &[])
        .expect("empty Resource set needs no local Resource bytes");
    let admitted = candidate
        .admit(&[&ComponentSchema], &[])
        .expect("schema-valid Component State");
    assert_eq!(identifier, admitted.component_state_id());
}

#[test]
fn revision_body_identifier_is_independent_of_project_state_and_parent_resolution() {
    let project_state = admitted_project_state(AdapterStateId::from_digest([38; 32]));
    let missing_state = project_state.project_state_id();
    let parent = RevisionCandidate::new(
        REVISION_SCHEMA,
        missing_state,
        Vec::new(),
        actor_id(),
        CREATED_AT,
        "existing parent",
        vec![],
    )
    .admit(
        &[&RevisionSchema],
        &OneProjectState(&project_state),
        &BTreeMap::new(),
    )
    .expect("admitted parent Revision");
    let candidate = RevisionCandidate::new(
        REVISION_SCHEMA,
        missing_state,
        vec![parent.revision_id()],
        actor_id(),
        CREATED_AT,
        "immutable body",
        vec![],
    );
    let identifier = candidate
        .verify_body_identifier(&[&RevisionSchema])
        .expect("exact-schema body Identifier is independently verifiable");
    assert_eq!(
        candidate.admit(&[&RevisionSchema], &NoProjectStates, &NoRevisions),
        Err(RevisionAdmissionError::UnavailableProjectState)
    );
    let admitted = RevisionCandidate::new(
        REVISION_SCHEMA,
        missing_state,
        vec![parent.revision_id()],
        actor_id(),
        CREATED_AT,
        "immutable body",
        vec![],
    )
    .admit(
        &[&RevisionSchema],
        &OneProjectState(&project_state),
        &OneRevision(&parent),
    )
    .expect("same body becomes admitted when Project State is available");
    assert_eq!(identifier, admitted.revision_id());
}

#[test]
fn release_body_identifier_is_independent_of_target_admission() {
    let project_state = admitted_project_state(AdapterStateId::from_digest([39; 32]));
    let target_revision = RevisionCandidate::new(
        REVISION_SCHEMA,
        project_state.project_state_id(),
        Vec::new(),
        actor_id(),
        CREATED_AT,
        "Release target",
        Vec::new(),
    )
    .admit(
        &[&RevisionSchema],
        &OneProjectState(&project_state),
        &BTreeMap::new(),
    )
    .expect("admitted target Revision");
    let missing_revision = target_revision.revision_id();
    let candidate = ReleaseCandidate::new(
        project_id(),
        "release",
        missing_revision,
        CREATED_AT,
        actor_id(),
        "body hash does not admit target",
    );
    let identifier = candidate
        .verify_body_identifier(&[&ReleaseSchema])
        .expect("exact Release schema body can be hashed");
    assert_eq!(
        candidate.admit(&[&ReleaseSchema], &NoRevisions, &NoProjectStates),
        Err(ReleaseAdmissionError::UnavailableRevision)
    );
    let admitted = ReleaseCandidate::new(
        project_id(),
        "release",
        missing_revision,
        CREATED_AT,
        actor_id(),
        "body hash does not admit target",
    )
    .admit(
        &[&ReleaseSchema],
        &OneRevision(&target_revision),
        &OneProjectState(&project_state),
    )
    .expect("same body becomes admitted when target is available");
    assert_eq!(identifier, admitted.release_id());
    assert_ne!(identifier, ReleaseId::from_digest([0; 32]));
}

#[test]
fn body_identity_methods_do_not_turn_reference_absence_into_schema_success() {
    let candidate = RevisionCandidate::new(
        "unknown.revision/99",
        ProjectStateId::from_digest([35; 32]),
        Vec::new(),
        actor_id(),
        CREATED_AT,
        "",
        Vec::new(),
    );
    assert_eq!(
        candidate.verify_body_identifier(&[&RevisionSchema]),
        Err(RevisionAdmissionError::UnavailableSchema)
    );

    let malformed_timestamp = RevisionCandidate::new(
        REVISION_SCHEMA,
        ProjectStateId::from_digest([36; 32]),
        Vec::new(),
        actor_id(),
        "not-a-canonical-time",
        "",
        Vec::new(),
    );
    assert_eq!(
        malformed_timestamp.verify_body_identifier(&[&RevisionSchema]),
        Err(RevisionAdmissionError::InvalidTimestamp)
    );
}
