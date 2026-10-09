#![allow(clippy::expect_used)]

use std::collections::{BTreeMap, HashMap};

use omvcs_core::revision_graph::{RevisionGraphDefect, RevisionGraphError, revision_ancestors};
use omvcs_model::canonical::MetadataSchema;
use omvcs_model::project_state::{
    AdmittedAdapterStateResolver, ProjectStateCandidate, ProjectStateSchemaValidator,
};
use omvcs_model::revision::{
    AdmittedRevisionResolver, RevisionAdmissionError, RevisionCandidate, RevisionSchemaValidator,
};
use omvcs_model::{ActorId, AdapterStateId, ProjectState, ProjectStateId, Revision, RevisionId};
use serde_json::{Value, json};

const REVISION_SCHEMA: &str = "test-only.revision/1";
const PROJECT_SCHEMA: &str = "test-only.project-state/1";
const PROJECT_ID: &str = "019cc17d-1b22-7a41-9fe9-c345c468f82e";
const OTHER_PROJECT_ID: &str = "019cc17d-1b22-7a41-9fe9-c345c468f82f";
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

struct HashMapResolver(HashMap<RevisionId, Revision>);

impl AdmittedRevisionResolver for HashMapResolver {
    fn resolve_admitted(&self, id: RevisionId) -> Option<&Revision> {
        self.0.get(&id)
    }
}

struct WrongRevisionResolver<'a>(&'a Revision);

impl AdmittedRevisionResolver for WrongRevisionResolver<'_> {
    fn resolve_admitted(&self, _: RevisionId) -> Option<&Revision> {
        Some(self.0)
    }
}

fn actor_id() -> ActorId {
    ACTOR_ID.parse().expect("valid ActorId fixture")
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

fn add_revision(
    state_id: ProjectStateId,
    parents: Vec<RevisionId>,
    message: &str,
    created_at: &str,
    states: &BTreeMap<ProjectStateId, ProjectState>,
    revisions: &mut BTreeMap<RevisionId, Revision>,
) -> RevisionId {
    let revision = RevisionCandidate::new(
        REVISION_SCHEMA,
        state_id,
        parents,
        actor_id(),
        created_at,
        message,
        vec![],
    )
    .admit(&[&TestRevisionSchema], states, revisions)
    .expect("valid admitted Revision fixture");
    let id = revision.revision_id();
    revisions.insert(id, revision);
    id
}

#[test]
fn initial_revision_has_no_ancestors() {
    let states = project_states();
    let state_id = *states.keys().next().expect("one Project State");
    let mut revisions = BTreeMap::new();
    let root = add_revision(
        state_id,
        vec![],
        "initial",
        "2026-10-08T11:02:17.000000000Z",
        &states,
        &mut revisions,
    );

    assert_eq!(revision_ancestors(root, &revisions), Ok(vec![]));
}

#[test]
fn direct_and_transitive_parents_are_returned_even_when_timestamps_are_not_ordered() {
    let states = project_states();
    let state_id = *states.keys().next().expect("one Project State");
    let mut revisions = BTreeMap::new();
    let later_parent = add_revision(
        state_id,
        vec![],
        "parent has later timestamp",
        "2026-10-08T11:02:18.000000000Z",
        &states,
        &mut revisions,
    );
    let earlier_child = add_revision(
        state_id,
        vec![later_parent],
        "child has earlier timestamp",
        "2026-10-08T11:02:17.000000000Z",
        &states,
        &mut revisions,
    );

    assert_eq!(
        revision_ancestors(earlier_child, &revisions),
        Ok(vec![later_parent])
    );
}

#[test]
fn multi_parent_traversal_is_unique_and_stable_across_parent_and_map_order() {
    let states = project_states();
    let state_id = *states.keys().next().expect("one Project State");
    let mut revisions = BTreeMap::new();
    let root = add_revision(
        state_id,
        vec![],
        "shared root",
        "2026-10-08T11:02:17.000000000Z",
        &states,
        &mut revisions,
    );
    let left = add_revision(
        state_id,
        vec![root],
        "left",
        "2026-10-08T11:02:17.000000000Z",
        &states,
        &mut revisions,
    );
    let right = add_revision(
        state_id,
        vec![root],
        "right",
        "2026-10-08T11:02:17.000000000Z",
        &states,
        &mut revisions,
    );
    let merge = add_revision(
        state_id,
        vec![right, left],
        "two parents",
        "2026-10-08T11:02:17.000000000Z",
        &states,
        &mut revisions,
    );
    let permuted = RevisionCandidate::new(
        REVISION_SCHEMA,
        state_id,
        vec![left, right],
        actor_id(),
        "2026-10-08T11:02:17.000000000Z",
        "two parents",
        vec![],
    )
    .admit(&[&TestRevisionSchema], &states, &revisions)
    .expect("same multi-parent Revision with permuted parent array");
    assert_eq!(permuted.revision_id(), merge);

    let mut expected = vec![root, left, right];
    expected.sort();
    let sorted_map_result = revision_ancestors(merge, &revisions).expect("complete graph");
    assert_eq!(sorted_map_result, expected);

    let reverse_inserted = HashMapResolver(
        revisions
            .into_iter()
            .rev()
            .collect::<HashMap<RevisionId, Revision>>(),
    );
    assert_eq!(
        revision_ancestors(merge, &reverse_inserted).expect("same complete graph"),
        sorted_map_result
    );
}

#[test]
fn unresolved_parent_metadata_is_reported_without_skipping_it() {
    let states = project_states();
    let state_id = *states.keys().next().expect("one Project State");
    let mut complete = BTreeMap::new();
    let parent = add_revision(
        state_id,
        vec![],
        "parent",
        "2026-10-08T11:02:17.000000000Z",
        &states,
        &mut complete,
    );
    let child = add_revision(
        state_id,
        vec![parent],
        "child",
        "2026-10-08T11:02:17.000000000Z",
        &states,
        &mut complete,
    );
    let partial = BTreeMap::from([(child, complete.remove(&child).expect("child exists"))]);

    assert_eq!(
        revision_ancestors(child, &partial),
        Err(RevisionGraphError::UnresolvedRevision(parent))
    );
}

#[test]
fn resolver_identifier_mismatch_is_not_reported_as_unresolved() {
    let states = project_states();
    let state_id = *states.keys().next().expect("one Project State");
    let mut revisions = BTreeMap::new();
    let resolved = add_revision(
        state_id,
        vec![],
        "resolved object",
        "2026-10-08T11:02:17.000000000Z",
        &states,
        &mut revisions,
    );
    let requested = RevisionId::from_digest([0x77; 32]);
    let actual = revisions.get(&resolved).expect("Revision exists");

    assert_eq!(
        revision_ancestors(requested, &WrongRevisionResolver(actual)),
        Err(RevisionGraphError::InvalidStructure(
            RevisionGraphDefect::IdentifierMismatch {
                requested,
                resolved
            }
        ))
    );
}

#[test]
fn cross_project_parent_is_rejected_before_it_can_enter_admitted_graph_metadata() {
    let current_state = project_state(PROJECT_ID);
    let foreign_state = project_state(OTHER_PROJECT_ID);
    let current_id = current_state.project_state_id();
    let foreign_id = foreign_state.project_state_id();
    let states = BTreeMap::from([(current_id, current_state), (foreign_id, foreign_state)]);
    let foreign_parent = RevisionCandidate::new(
        REVISION_SCHEMA,
        foreign_id,
        vec![],
        actor_id(),
        "2026-10-08T11:02:17.000000000Z",
        "foreign project",
        vec![],
    )
    .admit(&[&TestRevisionSchema], &states, &BTreeMap::new())
    .expect("foreign Project root is independently admitted");
    let foreign_id = foreign_parent.revision_id();
    let revisions = BTreeMap::from([(foreign_id, foreign_parent)]);
    let child = RevisionCandidate::new(
        REVISION_SCHEMA,
        current_id,
        vec![foreign_id],
        actor_id(),
        "2026-10-08T11:02:17.000000000Z",
        "cross-project child",
        vec![],
    );

    assert_eq!(
        child
            .admit(&[&TestRevisionSchema], &states, &revisions)
            .expect_err("cross-Project parent cannot be admitted"),
        RevisionAdmissionError::ParentProjectMismatch
    );
}

#[test]
fn metadata_only_traversal_requires_no_resource_byte_resolver() {
    let states = project_states();
    let state_id = *states.keys().next().expect("one Project State");
    let mut revisions = BTreeMap::new();
    let parent = add_revision(
        state_id,
        vec![],
        "Resource bytes are not part of graph traversal",
        "2026-10-08T11:02:18.000000000Z",
        &states,
        &mut revisions,
    );
    let child = add_revision(
        state_id,
        vec![parent],
        "timestamp does not choose ancestry",
        "2026-10-08T11:02:17.000000000Z",
        &states,
        &mut revisions,
    );

    assert_eq!(revision_ancestors(child, &revisions), Ok(vec![parent]));
}
