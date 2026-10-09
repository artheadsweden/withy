#![allow(clippy::expect_used)]

use std::collections::BTreeMap;

use omvcs_core::line::{
    InMemoryLineRepository, Line, LineEnumerationBoundary, LineOperationBoundary,
    LineOperationError,
};
use omvcs_core::reachability::{
    AdmittedAdapterStateResourceResolver, HistoricalId, PartialReachability, ReachabilityDefect,
    ReachabilityError, ReachabilityRoot, UnresolvedRootReference,
    partial_line_release_reachability,
};
use omvcs_core::release::{
    InMemoryReleaseRepository, ReleaseEnumerationBoundary, ReleaseOperationBoundary,
    ReleaseOperationError,
};
use omvcs_model::canonical::MetadataSchema;
use omvcs_model::component_state::{ComponentStateCandidate, ComponentStateSchemaValidator};
use omvcs_model::project_state::{
    AdmittedAdapterStateResolver, ComponentStateResolver, ProjectStateCandidate,
    ProjectStateSchemaValidator,
};
use omvcs_model::release::{RELEASE_SCHEMA, ReleaseCandidate, ReleaseSchemaValidator};
use omvcs_model::resource::{
    ResourceByteLength, ResourceReferenceCandidate, ResourceValidationContext,
};
use omvcs_model::revision::{
    AdmittedProjectStateResolver, AdmittedRevisionResolver, RevisionCandidate,
    RevisionSchemaValidator,
};
use omvcs_model::{
    ActorId, AdapterStateId, ComponentState, ComponentStateId, CreativeComponentId, ProjectId,
    ProjectState, ProjectStateId, Release, ResourceId, Revision, RevisionId,
};
use serde_json::{Value, json};

const SCHEMA: &str = "reachability-test/1";
const TIME: &str = "2026-10-08T11:02:17.000000000Z";

struct Schema;

fn empty_schema() -> MetadataSchema {
    MetadataSchema::structure(std::iter::empty::<(String, MetadataSchema)>())
}

impl ComponentStateSchemaValidator for Schema {
    fn schema(&self) -> &str {
        SCHEMA
    }
    fn metadata_schema(&self) -> MetadataSchema {
        empty_schema()
    }
    fn validate_metadata(&self, _: &BTreeMap<String, Value>) -> Result<(), String> {
        Ok(())
    }
    fn resource_validation_context(
        &self,
        _: &ResourceReferenceCandidate,
    ) -> Option<ResourceValidationContext> {
        None
    }
}

impl ProjectStateSchemaValidator for Schema {
    fn schema(&self) -> &str {
        SCHEMA
    }
    fn project_metadata_schema(&self) -> MetadataSchema {
        empty_schema()
    }
    fn validate_project_metadata(&self, _: &BTreeMap<String, Value>) -> Result<(), String> {
        Ok(())
    }
}

impl RevisionSchemaValidator for Schema {
    fn schema(&self) -> &str {
        SCHEMA
    }
    fn provenance_entry_schema(&self) -> MetadataSchema {
        empty_schema()
    }
    fn validate_provenance(&self, _: &[Value]) -> Result<(), String> {
        Ok(())
    }
}

impl ReleaseSchemaValidator for Schema {
    fn schema(&self) -> &str {
        RELEASE_SCHEMA
    }
    fn validate_release(&self, _: &ReleaseCandidate) -> Result<(), String> {
        Ok(())
    }
}

// A provider-neutral trusted test double, not a new Adapter body/admission
// implementation. Exact schema admission is a precondition of this boundary.
#[derive(Default)]
struct Adapters(BTreeMap<AdapterStateId, Vec<ResourceId>>);

impl AdmittedAdapterStateResolver for Adapters {
    fn resolve_admitted(&self, id: AdapterStateId) -> Option<AdapterStateId> {
        self.0.contains_key(&id).then_some(id)
    }
}

impl AdmittedAdapterStateResourceResolver for Adapters {
    fn resolve_resource_ids(
        &self,
        id: AdapterStateId,
    ) -> Option<(AdapterStateId, Vec<ResourceId>)> {
        self.0.get(&id).map(|ids| (id, ids.clone()))
    }
}

fn project(n: u8) -> ProjectId {
    format!("019cc17d-1b22-7a41-9fe9-c345c468f8{n:02x}")
        .parse()
        .expect("Project UUIDv7")
}

fn actor() -> ActorId {
    "019cc17d-1b22-7a41-9fe9-c345c468f82c"
        .parse()
        .expect("Actor UUIDv7")
}

fn component() -> CreativeComponentId {
    "019cc17d-1b22-7a41-9fe9-c345c468f82d"
        .parse()
        .expect("Component UUIDv7")
}

struct Fixture {
    lines: InMemoryLineRepository,
    releases: InMemoryReleaseRepository,
    revisions: BTreeMap<RevisionId, Revision>,
    projects: BTreeMap<ProjectStateId, ProjectState>,
    components: BTreeMap<ComponentStateId, ComponentState>,
    adapters: Adapters,
    parent_component: ComponentStateId,
    child_component: ComponentStateId,
    state: ProjectStateId,
    adapter: AdapterStateId,
    initial: RevisionId,
    child: RevisionId,
    resource: ResourceId,
    parent_resource: ResourceId,
    adapter_resource: ResourceId,
}

impl Fixture {
    fn new() -> Self {
        let resource = ResourceId::from_digest([1; 32]);
        let parent_resource = ResourceId::from_digest([2; 32]);
        let adapter_resource = ResourceId::from_digest([3; 32]);
        let reference = |id| {
            ResourceReferenceCandidate::new(id, ResourceByteLength::new(4096).expect("length"))
        };
        let parent = ComponentStateCandidate::new(
            SCHEMA,
            component(),
            vec![reference(parent_resource)],
            BTreeMap::new(),
        )
        .with_parents(vec![])
        .admit(&[&Schema], &[])
        .expect("initial Component State");
        let parent_component = parent.component_state_id();
        let child = ComponentStateCandidate::new(
            SCHEMA,
            component(),
            vec![reference(resource)],
            BTreeMap::new(),
        )
        .with_parents(vec![parent_component])
        .admit(&[&Schema], &[])
        .expect("derived Component State");
        let child_component = child.component_state_id();
        let components = BTreeMap::from([(parent_component, parent), (child_component, child)]);
        let adapter = AdapterStateId::from_digest([4; 32]);
        let adapters = Adapters(BTreeMap::from([(
            adapter,
            vec![adapter_resource, resource],
        )]));
        let candidate: ProjectStateCandidate = serde_json::from_value(json!({
            "schema": SCHEMA,
            "project_id": project(1).to_string(),
            "components": {component().to_string(): child_component.to_string()},
            "adapter_state_id": adapter.to_string(),
            "project_metadata": {}
        }))
        .expect("Project State candidate");
        let state = candidate
            .admit(&[&Schema], &components, &adapters)
            .expect("Project State");
        let state_id = state.project_state_id();
        let mut fixture = Self {
            lines: InMemoryLineRepository::new([project(1), project(2)]),
            releases: InMemoryReleaseRepository::new([project(1), project(2)]),
            revisions: BTreeMap::new(),
            projects: BTreeMap::from([(state_id, state)]),
            components,
            adapters,
            parent_component,
            child_component,
            state: state_id,
            adapter,
            initial: RevisionId::from_digest([0; 32]),
            child: RevisionId::from_digest([0; 32]),
            resource,
            parent_resource,
            adapter_resource,
        };
        fixture.initial = fixture.revision(state_id, vec![], "initial");
        fixture.child = fixture.revision(state_id, vec![fixture.initial], "child");
        fixture
    }

    fn revision(
        &mut self,
        state: ProjectStateId,
        parents: Vec<RevisionId>,
        message: &str,
    ) -> RevisionId {
        let revision =
            RevisionCandidate::new(SCHEMA, state, parents, actor(), TIME, message, vec![])
                .admit(&[&Schema], &self.projects, &self.revisions)
                .expect("admitted Revision");
        let id = revision.revision_id();
        self.revisions.insert(id, revision);
        id
    }

    fn line(&self, project: ProjectId, name: &str, revision: RevisionId) -> Line {
        self.lines
            .create_line(
                project,
                name.to_owned(),
                revision,
                &self.revisions,
                &self.projects,
            )
            .expect("Line")
    }

    fn release(&self, project: ProjectId, name: &str, revision: RevisionId) -> Release {
        self.releases
            .create_release(
                ReleaseCandidate::new(project, name, revision, TIME, actor(), ""),
                &[&Schema],
                &self.revisions,
                &self.projects,
            )
            .expect("Release")
    }

    fn reach(&self) -> PartialReachability {
        self.reach_with(
            &self.revisions,
            &self.projects,
            &self.components,
            &self.adapters,
        )
    }

    fn reach_with(
        &self,
        revisions: &dyn AdmittedRevisionResolver,
        projects: &dyn AdmittedProjectStateResolver,
        components: &dyn ComponentStateResolver,
        adapters: &dyn AdmittedAdapterStateResourceResolver,
    ) -> PartialReachability {
        partial_line_release_reachability(
            &self.lines,
            &self.releases,
            revisions,
            projects,
            components,
            adapters,
        )
        .expect("enumerated roots")
    }

    fn other_project_revision(&mut self) -> RevisionId {
        let candidate: ProjectStateCandidate = serde_json::from_value(json!({
            "schema": SCHEMA,
            "project_id": project(2).to_string(),
            "components": {},
            "adapter_state_id": self.adapter.to_string(),
            "project_metadata": {}
        }))
        .expect("other Project State candidate");
        let state = candidate
            .admit(&[&Schema], &self.components, &self.adapters)
            .expect("other Project State");
        let id = state.project_state_id();
        self.projects.insert(id, state);
        self.revision(id, vec![], "other Project")
    }
}

// Core §62; ADR-0016/0017: enumerate all roots, not just a selected Project.
#[test]
fn every_retained_line_across_projects_is_a_root() {
    let mut f = Fixture::new();
    let other = f.other_project_revision();
    let first = f.line(project(1), "first", f.child);
    let second = f.line(project(2), "second", other);
    let result = f.reach();
    assert_eq!(result.lines.len(), 2);
    assert!(result.lines.contains(&first.line_id()));
    assert!(result.lines.contains(&second.line_id()));
    assert!(result.revisions.contains(&other));
    assert!(result.revisions.contains(&f.child));
    assert_eq!(result.releases, Vec::<omvcs_model::ReleaseId>::new());
}

#[test]
fn every_admitted_release_is_a_root_even_without_lines() {
    let mut f = Fixture::new();
    let other = f.other_project_revision();
    let first = f.release(project(1), "first", f.child);
    let second = f.release(project(2), "second", other);
    let result = f.reach();
    assert_eq!(result.lines, Vec::<omvcs_model::LineId>::new());
    assert_eq!(result.releases.len(), 2);
    assert!(result.releases.contains(&first.release_id()));
    assert!(result.releases.contains(&second.release_id()));
    assert!(result.revisions.contains(&f.initial));
    assert!(result.revisions.contains(&other));
}

#[test]
fn default_line_preference_is_not_a_second_root() {
    let f = Fixture::new();
    let line = f.line(project(1), "main", f.child);
    let before = f.reach();
    f.lines
        .set_default_line(project(1), None, Some(line.line_id()))
        .expect("set Default Line");
    assert_eq!(f.reach(), before);
    assert_eq!(before.lines, vec![line.line_id()]);
}

#[test]
fn deleted_lines_are_not_retained_roots_and_deletion_does_not_erase_history() {
    let f = Fixture::new();
    let line = f.line(project(1), "removed", f.child);
    f.lines
        .delete_line(line.line_id(), line.generation())
        .expect("remove Line");
    assert_eq!(f.reach(), PartialReachability::default());
    assert!(f.revisions.contains_key(&f.child));
    assert!(f.components.contains_key(&f.child_component));
}

#[test]
fn moving_a_line_changes_future_reachability_without_mutating_history() {
    let mut f = Fixture::new();
    let line = f.line(project(1), "main", f.child);
    let new_target = f.revision(f.state, vec![], "divergent target");
    let child_bytes = f.revisions[&f.child].canonical_body().to_vec();
    let initial_bytes = f.revisions[&f.initial].canonical_body().to_vec();
    let new_target_bytes = f.revisions[&new_target].canonical_body().to_vec();

    let before = f.reach();
    assert!(before.revisions.contains(&f.child));
    assert!(before.revisions.contains(&f.initial));
    assert!(!before.revisions.contains(&new_target));

    f.lines
        .move_line(
            line.line_id(),
            line.target_revision(),
            line.generation(),
            new_target,
            &f.revisions,
            &f.projects,
        )
        .expect("move Line");

    let after = f.reach();
    assert!(after.revisions.contains(&new_target));
    assert!(!after.revisions.contains(&f.child));
    assert!(!after.revisions.contains(&f.initial));
    assert_eq!(f.revisions[&f.child].canonical_body(), child_bytes);
    assert_eq!(f.revisions[&f.initial].canonical_body(), initial_bytes);
    assert_eq!(f.revisions[&new_target].canonical_body(), new_target_bytes);
}

#[test]
fn convergence_across_roots_and_multiple_parents_is_sorted_and_deduplicated() {
    let mut f = Fixture::new();
    let sibling = f.revision(f.state, vec![f.initial], "sibling");
    let merge = f.revision(f.state, vec![sibling, f.child], "merge");
    f.line(project(1), "merge", merge);
    f.line(project(1), "same target", merge);
    f.release(project(1), "merge", merge);
    f.release(project(1), "sibling", sibling);
    let mut expected = vec![f.initial, f.child, sibling, merge];
    expected.sort();
    let result = f.reach();
    assert_eq!(result.revisions, expected);
    assert_eq!(result.project_states, vec![f.state]);
    assert_eq!(result.adapter_states, vec![f.adapter]);
    let mut resources = vec![f.resource, f.parent_resource, f.adapter_resource];
    resources.sort();
    assert_eq!(result.resources, resources);
    assert_eq!(result.unresolved, Vec::<HistoricalId>::new());
    assert_eq!(result.defects, Vec::<ReachabilityDefect>::new());
}

#[test]
fn project_component_asserted_parent_and_adapter_resource_edges_are_followed() {
    let f = Fixture::new();
    f.line(project(1), "main", f.child);
    let result = f.reach();
    assert_eq!(result.revisions.len(), 2);
    assert_eq!(result.project_states, vec![f.state]);
    let mut components = vec![f.parent_component, f.child_component];
    components.sort();
    assert_eq!(result.component_states, components);
    assert_eq!(result.adapter_states, vec![f.adapter]);
    assert!(result.resources.contains(&f.parent_resource));
    assert!(result.resources.contains(&f.adapter_resource));
}

#[test]
fn omitted_component_lineage_does_not_infer_parents() {
    let mut f = Fixture::new();
    let state = ComponentStateCandidate::new(SCHEMA, component(), vec![], BTreeMap::new())
        .admit(&[&Schema], &[])
        .expect("unasserted lineage");
    let id = state.component_state_id();
    f.components.insert(id, state);
    let candidate: ProjectStateCandidate = serde_json::from_value(json!({
        "schema": SCHEMA, "project_id": project(1).to_string(),
        "components": {component().to_string(): id.to_string()},
        "adapter_state_id": f.adapter.to_string(), "project_metadata": {}
    }))
    .expect("candidate");
    let project_state = candidate
        .admit(&[&Schema], &f.components, &f.adapters)
        .expect("Project State");
    let state_id = project_state.project_state_id();
    f.projects.insert(state_id, project_state);
    let revision = f.revision(state_id, vec![], "unasserted lineage");
    f.line(project(1), "main", revision);
    assert_eq!(f.reach().component_states, vec![id]);
}

#[test]
fn resource_bytes_replicas_and_materialisation_are_not_lookup_requirements() {
    let f = Fixture::new();
    f.release(project(1), "sparse", f.child);
    // All references use typed IDs/lengths without creating any ResourceObject.
    // A local cache cannot be passed to this operation at all.
    let without_bytes = f.reach();
    let local_cache = BTreeMap::from([(f.resource, vec![0_u8; 4096])]);
    assert_eq!(f.reach(), without_bytes);
    drop(local_cache);
    assert_eq!(f.reach(), without_bytes);
    assert_eq!(without_bytes.resources.len(), 3);
    assert_eq!(without_bytes.unresolved, Vec::<HistoricalId>::new());
}

#[test]
fn unresolved_revision_is_still_reached_and_independent_branches_continue() {
    let mut f = Fixture::new();
    let sibling = f.revision(f.state, vec![], "independent");
    let merge = f.revision(f.state, vec![sibling, f.child], "merge");
    f.line(project(1), "merge", merge);
    f.revisions.remove(&f.child);
    let result = f.reach();
    assert_eq!(result.unresolved, vec![HistoricalId::Revision(f.child)]);
    assert!(result.revisions.contains(&f.child));
    assert!(result.revisions.contains(&sibling));
    assert!(!result.revisions.contains(&f.initial));
    assert_eq!(result.resources.len(), 3);
    assert_eq!(result.defects, Vec::<ReachabilityDefect>::new());
}

#[test]
fn unresolved_line_target_does_not_hide_an_independent_release_root() {
    let mut f = Fixture::new();
    let line = f.line(project(1), "main", f.child);
    f.release(project(1), "retained", f.initial);
    f.revisions.remove(&f.child);
    let result = f.reach();
    assert_eq!(result.unresolved, vec![HistoricalId::Revision(f.child)]);
    assert_eq!(
        result.unresolved_root_references,
        vec![UnresolvedRootReference {
            root: ReachabilityRoot::Line(line.line_id()),
            target: f.child,
        }]
    );
    assert!(result.revisions.contains(&f.child));
    assert!(result.revisions.contains(&f.initial));
    assert_eq!(result.resources.len(), 3);
    assert_eq!(result.defects, Vec::<ReachabilityDefect>::new());
}

#[test]
fn unresolved_component_mapping_is_reached_without_guessing_its_parent_edges() {
    let mut f = Fixture::new();
    f.line(project(1), "main", f.child);
    f.components.remove(&f.child_component);
    let result = f.reach();
    assert_eq!(result.component_states, vec![f.child_component]);
    assert_eq!(
        result.unresolved,
        vec![HistoricalId::ComponentState(f.child_component)]
    );
    assert!(!result.resources.contains(&f.parent_resource));
    assert_eq!(result.resources.len(), 2); // independently asserted Adapter edges
}

#[test]
fn unresolved_references_from_convergent_roots_are_sorted_by_type_and_deduplicated() {
    let mut f = Fixture::new();
    f.line(project(1), "main", f.child);
    f.line(project(1), "duplicate target", f.child);
    f.release(project(1), "release", f.child);
    f.revisions.remove(&f.initial);
    f.components.clear();
    f.adapters.0.clear();
    let result = f.reach();
    assert_eq!(
        result.unresolved,
        vec![
            HistoricalId::Revision(f.initial),
            HistoricalId::ComponentState(f.child_component),
            HistoricalId::AdapterState(f.adapter),
        ]
    );
    assert_eq!(result.defects, Vec::<ReachabilityDefect>::new());
}

#[test]
fn ancestry_keeps_resources_of_earlier_project_states_not_only_the_current_target() {
    let mut f = Fixture::new();
    let adapter = AdapterStateId::from_digest([55; 32]);
    f.adapters.0.insert(adapter, vec![]);
    let candidate: ProjectStateCandidate = serde_json::from_value(json!({
        "schema": SCHEMA, "project_id": project(1).to_string(),
        "components": {}, "adapter_state_id": adapter.to_string(), "project_metadata": {}
    }))
    .expect("later Project State candidate");
    let state = candidate
        .admit(&[&Schema], &f.components, &f.adapters)
        .expect("later Project State");
    let state_id = state.project_state_id();
    f.projects.insert(state_id, state);
    let later = f.revision(
        state_id,
        vec![f.child],
        "Components removed from latest state",
    );
    f.line(project(1), "main", later);
    let result = f.reach();
    assert_eq!(result.revisions.len(), 3);
    assert_eq!(result.project_states.len(), 2);
    assert_eq!(result.adapter_states.len(), 2);
    assert_eq!(result.resources.len(), 3);
    assert!(result.resources.contains(&f.parent_resource));
    assert_eq!(result.unresolved, Vec::<HistoricalId>::new());
}

#[test]
fn unresolved_project_metadata_is_distinct_from_missing_resource_bytes() {
    let mut f = Fixture::new();
    f.release(project(1), "release", f.child);
    f.projects.clear();
    let result = f.reach();
    assert_eq!(result.project_states, vec![f.state]);
    assert_eq!(result.unresolved, vec![HistoricalId::ProjectState(f.state)]);
    assert_eq!(result.resources, Vec::<ResourceId>::new());
    assert_eq!(result.defects, Vec::<ReachabilityDefect>::new());
}

#[test]
fn unresolved_asserted_component_parent_is_not_discarded_as_unreachable() {
    let mut f = Fixture::new();
    f.line(project(1), "main", f.child);
    f.components.remove(&f.parent_component);
    let result = f.reach();
    assert!(result.component_states.contains(&f.parent_component));
    assert_eq!(
        result.unresolved,
        vec![HistoricalId::ComponentState(f.parent_component)]
    );
    assert!(result.resources.contains(&f.resource));
    assert!(!result.resources.contains(&f.parent_resource));
    assert_eq!(result.defects, Vec::<ReachabilityDefect>::new());
}

#[test]
fn unresolved_adapter_metadata_does_not_hide_component_resources() {
    let mut f = Fixture::new();
    f.line(project(1), "main", f.child);
    f.adapters.0.clear();
    let result = f.reach();
    assert_eq!(result.adapter_states, vec![f.adapter]);
    assert_eq!(
        result.unresolved,
        vec![HistoricalId::AdapterState(f.adapter)]
    );
    assert!(result.resources.contains(&f.parent_resource));
    assert!(!result.resources.contains(&f.adapter_resource));
}

struct WrongRevision<'a>(&'a Revision);
impl AdmittedRevisionResolver for WrongRevision<'_> {
    fn resolve_admitted(&self, _: RevisionId) -> Option<&Revision> {
        Some(self.0)
    }
}

#[test]
fn revision_identifier_mismatch_is_a_defect_not_an_unresolved_reference() {
    let f = Fixture::new();
    f.line(project(1), "main", f.child);
    let result = f.reach_with(
        &WrongRevision(&f.revisions[&f.initial]),
        &f.projects,
        &f.components,
        &f.adapters,
    );
    assert_eq!(result.revisions, vec![f.child]);
    assert_eq!(result.project_states, Vec::<ProjectStateId>::new());
    assert_eq!(result.unresolved, Vec::<HistoricalId>::new());
    assert_eq!(
        result.defects,
        vec![ReachabilityDefect::IdentifierMismatch {
            requested: HistoricalId::Revision(f.child),
            resolved: HistoricalId::Revision(f.initial),
        }]
    );
}

struct WrongProject<'a>(&'a ProjectState);
impl AdmittedProjectStateResolver for WrongProject<'_> {
    fn resolve_admitted(&self, _: ProjectStateId) -> Option<&ProjectState> {
        Some(self.0)
    }
}

struct WrongComponent<'a>(&'a ComponentState);
impl ComponentStateResolver for WrongComponent<'_> {
    fn resolve_admitted(&self, _: ComponentStateId) -> Option<&ComponentState> {
        Some(self.0)
    }
}

#[test]
fn project_and_component_identifier_mismatches_do_not_follow_wrong_object_edges() {
    let mut f = Fixture::new();
    f.line(project(1), "main", f.child);
    let other = f.other_project_revision();
    let other_state = f.revisions[&other].project_state_id();
    let result = f.reach_with(
        &f.revisions,
        &WrongProject(&f.projects[&other_state]),
        &f.components,
        &f.adapters,
    );
    assert_eq!(result.component_states, Vec::<ComponentStateId>::new());
    assert_eq!(result.adapter_states, Vec::<AdapterStateId>::new());
    assert_eq!(result.unresolved, Vec::<HistoricalId>::new());
    assert_eq!(
        result.defects,
        vec![ReachabilityDefect::IdentifierMismatch {
            requested: HistoricalId::ProjectState(f.state),
            resolved: HistoricalId::ProjectState(other_state),
        }]
    );
    let result = f.reach_with(
        &f.revisions,
        &f.projects,
        &WrongComponent(&f.components[&f.parent_component]),
        &f.adapters,
    );
    assert_eq!(result.component_states, vec![f.child_component]);
    assert!(!result.resources.contains(&f.parent_resource));
    assert_eq!(result.unresolved, Vec::<HistoricalId>::new());
    assert_eq!(
        result.defects,
        vec![ReachabilityDefect::IdentifierMismatch {
            requested: HistoricalId::ComponentState(f.child_component),
            resolved: HistoricalId::ComponentState(f.parent_component),
        }]
    );
}

struct AdapterProjection {
    admitted: AdapterStateId,
    projection: Option<(AdapterStateId, Vec<ResourceId>)>,
}
impl AdmittedAdapterStateResolver for AdapterProjection {
    fn resolve_admitted(&self, _: AdapterStateId) -> Option<AdapterStateId> {
        Some(self.admitted)
    }
}
impl AdmittedAdapterStateResourceResolver for AdapterProjection {
    fn resolve_resource_ids(&self, _: AdapterStateId) -> Option<(AdapterStateId, Vec<ResourceId>)> {
        self.projection.clone()
    }
}

#[test]
fn adapter_projection_requires_exact_admitted_identity_and_never_guesses_resources() {
    let f = Fixture::new();
    f.line(project(1), "main", f.child);
    let wrong = AdapterStateId::from_digest([99; 32]);
    for adapter in [
        AdapterProjection {
            admitted: wrong,
            projection: Some((f.adapter, vec![f.adapter_resource])),
        },
        AdapterProjection {
            admitted: f.adapter,
            projection: Some((wrong, vec![f.adapter_resource])),
        },
    ] {
        let result = f.reach_with(&f.revisions, &f.projects, &f.components, &adapter);
        assert!(!result.resources.contains(&f.adapter_resource));
        assert_eq!(result.unresolved, Vec::<HistoricalId>::new());
        assert_eq!(
            result.defects,
            vec![ReachabilityDefect::IdentifierMismatch {
                requested: HistoricalId::AdapterState(f.adapter),
                resolved: HistoricalId::AdapterState(wrong),
            }]
        );
    }
}

#[test]
fn missing_adapter_projection_is_unresolved_but_empty_projection_asserts_no_edges() {
    let f = Fixture::new();
    f.line(project(1), "main", f.child);
    let adapter = AdapterProjection {
        admitted: f.adapter,
        projection: None,
    };
    let missing = f.reach_with(&f.revisions, &f.projects, &f.components, &adapter);
    assert_eq!(
        missing.unresolved,
        vec![HistoricalId::AdapterState(f.adapter)]
    );
    let adapter = AdapterProjection {
        admitted: f.adapter,
        projection: Some((f.adapter, vec![])),
    };
    let empty = f.reach_with(&f.revisions, &f.projects, &f.components, &adapter);
    assert_eq!(empty.unresolved, Vec::<HistoricalId>::new());
    assert_eq!(empty.resources, missing.resources);
}

#[test]
fn reachability_does_not_mutate_roots_preferences_or_immutable_metadata() {
    let f = Fixture::new();
    let line = f.line(project(1), "main", f.child);
    f.lines
        .set_default_line(project(1), None, Some(line.line_id()))
        .expect("Default Line");
    let release = f.release(project(1), "release", f.child);
    let lines = f.lines.retained_lines().expect("Lines");
    let release_bytes = release.canonical_body().to_vec();
    let revision_bytes = f.revisions[&f.child].canonical_body().to_vec();
    let project_bytes = f.projects[&f.state].canonical_body().to_vec();
    let component_bytes = f.components[&f.child_component].canonical_body().to_vec();
    assert_eq!(f.reach(), f.reach());
    assert_eq!(f.lines.retained_lines().expect("Lines"), lines);
    assert_eq!(f.lines.default_line(project(1)), Ok(Some(line.line_id())));
    assert_eq!(
        f.releases.admitted_releases().expect("Releases")[0].canonical_body(),
        release_bytes
    );
    assert_eq!(f.revisions[&f.child].canonical_body(), revision_bytes);
    assert_eq!(f.projects[&f.state].canonical_body(), project_bytes);
    assert_eq!(
        f.components[&f.child_component].canonical_body(),
        component_bytes
    );
}

#[test]
fn partial_result_does_not_invent_roots_or_label_unselected_history_globally_unreachable() {
    let mut f = Fixture::new();
    // Resolvers contain history, but existence or an operational use/name cannot
    // make it a root. No Working State, Contribution, pin or transaction input
    // exists in the operation's public boundary.
    let unselected = f.revision(f.state, vec![], "operational reference, not a root");
    assert_eq!(f.reach(), PartialReachability::default());
    f.line(project(1), "retained", f.initial);
    let PartialReachability {
        lines,
        releases,
        revisions,
        project_states,
        component_states,
        adapter_states,
        resources,
        unresolved,
        unresolved_root_references,
        defects,
    } = f.reach(); // exhaustive result shape: no global unreachable/completeness field
    assert_eq!(lines.len(), 1);
    assert_eq!(releases, Vec::<omvcs_model::ReleaseId>::new());
    assert_eq!(revisions, vec![f.initial]);
    assert!(!revisions.contains(&unselected));
    assert_eq!(project_states.len(), 1);
    assert_eq!(component_states.len(), 2);
    assert_eq!(adapter_states.len(), 1);
    assert_eq!(resources.len(), 3);
    assert_eq!(unresolved, Vec::<HistoricalId>::new());
    assert_eq!(
        unresolved_root_references,
        Vec::<UnresolvedRootReference>::new()
    );
    assert_eq!(defects, Vec::<ReachabilityDefect>::new());
    assert!(f.revisions.contains_key(&unselected));
}

struct Roots {
    lines: Vec<Line>,
    releases: Vec<Release>,
    fail_lines: bool,
    fail_releases: bool,
}
impl LineEnumerationBoundary for Roots {
    fn retained_lines(&self) -> Result<Vec<Line>, LineOperationError> {
        if self.fail_lines {
            Err(LineOperationError::RepositoryUnavailable)
        } else {
            Ok(self.lines.clone())
        }
    }
}
impl ReleaseEnumerationBoundary for Roots {
    fn admitted_releases(&self) -> Result<Vec<Release>, ReleaseOperationError> {
        if self.fail_releases {
            Err(ReleaseOperationError::RepositoryUnavailable)
        } else {
            Ok(self.releases.clone())
        }
    }
}

#[test]
fn enumeration_order_and_duplicate_root_records_do_not_change_the_result() {
    let f = Fixture::new();
    let first = f.line(project(1), "first", f.child);
    let second = f.line(project(1), "second", f.initial);
    let release = f.release(project(1), "release", f.child);
    let roots = Roots {
        lines: vec![second, first.clone(), first],
        releases: vec![release.clone(), release],
        fail_lines: false,
        fail_releases: false,
    };
    assert_eq!(
        partial_line_release_reachability(
            &roots,
            &roots,
            &f.revisions,
            &f.projects,
            &f.components,
            &f.adapters
        ),
        Ok(f.reach())
    );
}

#[test]
fn failed_root_enumeration_is_not_reported_as_an_empty_successful_result() {
    let f = Fixture::new();
    for (fail_lines, fail_releases, expected) in [
        (
            true,
            false,
            ReachabilityError::Lines(LineOperationError::RepositoryUnavailable),
        ),
        (
            false,
            true,
            ReachabilityError::Releases(ReleaseOperationError::RepositoryUnavailable),
        ),
    ] {
        let roots = Roots {
            lines: vec![],
            releases: vec![],
            fail_lines,
            fail_releases,
        };
        assert_eq!(
            partial_line_release_reachability(
                &roots,
                &roots,
                &f.revisions,
                &f.projects,
                &f.components,
                &f.adapters
            ),
            Err(expected)
        );
    }
}
