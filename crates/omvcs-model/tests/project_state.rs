#![allow(clippy::expect_used)]

use std::collections::{BTreeMap, HashSet};

use omvcs_model::canonical::{ArrayOrdering, MetadataSchema};
use omvcs_model::component_state::{ComponentStateCandidate, ComponentStateSchemaValidator};
use omvcs_model::hashing::hash_project_state_metadata;
use omvcs_model::project_state::{
    AdmittedAdapterStateResolver, ProjectStateAdmissionError, ProjectStateCandidate,
    ProjectStateSchemaValidator,
};
use omvcs_model::{
    AdapterStateId, ComponentState, ComponentStateId, CreativeComponentId, ProjectId,
};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const PROJECT_SCHEMA: &str = "test.project-state/1";
const COMPONENT_SCHEMA: &str = "test.component-state/1";
const PROJECT_ID: &str = "019cc17d-1b22-7a41-9fe9-c345c468f82c";
const COMPONENT_ID: &str = "019cc17d-1b22-7a41-9fe9-c345c468f82d";

struct ProjectSchema {
    schema: &'static str,
    metadata_schema: MetadataSchema,
    require_title: bool,
}

impl ProjectSchema {
    fn new() -> Self {
        Self {
            schema: PROJECT_SCHEMA,
            metadata_schema: MetadataSchema::structure([
                ("title", MetadataSchema::Scalar),
                (
                    "tags",
                    MetadataSchema::array(ArrayOrdering::SetLike, MetadataSchema::Scalar),
                ),
                (
                    "sequence",
                    MetadataSchema::array(ArrayOrdering::Ordered, MetadataSchema::Scalar),
                ),
                (
                    "nested",
                    MetadataSchema::structure([(
                        "labels",
                        MetadataSchema::array(ArrayOrdering::SetLike, MetadataSchema::Scalar),
                    )]),
                ),
            ]),
            require_title: false,
        }
    }

    const fn with_schema(mut self, schema: &'static str) -> Self {
        self.schema = schema;
        self
    }

    const fn requiring_title(mut self) -> Self {
        self.require_title = true;
        self
    }
}

impl ProjectStateSchemaValidator for ProjectSchema {
    fn schema(&self) -> &str {
        self.schema
    }

    fn project_metadata_schema(&self) -> MetadataSchema {
        self.metadata_schema.clone()
    }

    fn validate_project_metadata(&self, metadata: &BTreeMap<String, Value>) -> Result<(), String> {
        if self.require_title && !metadata.contains_key("title") {
            return Err("title is required by this test schema".to_owned());
        }
        if metadata
            .get("title")
            .is_some_and(|value| !value.is_string())
        {
            return Err("title must be a string".to_owned());
        }
        if metadata
            .get("tags")
            .and_then(Value::as_array)
            .is_some_and(|values| values.iter().any(|value| !value.is_string()))
        {
            return Err("tags must contain strings".to_owned());
        }
        if metadata
            .get("nested")
            .and_then(Value::as_object)
            .and_then(|nested| nested.get("labels"))
            .and_then(Value::as_array)
            .is_some_and(|values| values.iter().any(|value| !value.is_string()))
        {
            return Err("nested labels must contain strings".to_owned());
        }
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

#[derive(Default)]
struct AdapterResolver(HashSet<AdapterStateId>);

impl AdmittedAdapterStateResolver for AdapterResolver {
    fn resolve_admitted(&self, id: AdapterStateId) -> Option<AdapterStateId> {
        self.0.contains(&id).then_some(id)
    }
}

const fn adapter_id(byte: u8) -> AdapterStateId {
    AdapterStateId::from_digest([byte; 32])
}

fn component_id() -> CreativeComponentId {
    COMPONENT_ID.parse().expect("valid UUIDv7 fixture")
}

fn project_id() -> ProjectId {
    PROJECT_ID.parse().expect("valid UUIDv7 fixture")
}

fn admitted_component(component_id: CreativeComponentId, revision: &str) -> ComponentState {
    let candidate: ComponentStateCandidate = serde_json::from_value(json!({
        "schema": COMPONENT_SCHEMA,
        "component_id": component_id.to_string(),
        "resources": [],
        "metadata": {"revision": revision}
    }))
    .expect("valid Component State candidate");
    candidate
        .admit(&[&ComponentSchema], &[])
        .expect("valid Component State")
}

fn project_body(
    project: ProjectId,
    components: Value,
    adapter: AdapterStateId,
    metadata: Value,
) -> Value {
    let mut body = serde_json::Map::new();
    body.insert("schema".to_owned(), json!(PROJECT_SCHEMA));
    body.insert("project_id".to_owned(), json!(project.to_string()));
    body.insert("components".to_owned(), components);
    body.insert("adapter_state_id".to_owned(), json!(adapter.to_string()));
    body.insert("project_metadata".to_owned(), metadata);
    Value::Object(body)
}

fn admit(
    body: Value,
    schema: &ProjectSchema,
    states: &BTreeMap<ComponentStateId, ComponentState>,
    adapters: &AdapterResolver,
) -> Result<omvcs_model::ProjectState, ProjectStateAdmissionError> {
    let candidate: ProjectStateCandidate =
        serde_json::from_value(body).expect("well-formed test candidate");
    candidate.admit(&[schema], states, adapters)
}

fn resolver_with(adapter: AdapterStateId) -> AdapterResolver {
    AdapterResolver(HashSet::from([adapter]))
}

#[test]
fn project_state_requires_exact_five_member_object_and_accepts_empty_maps() {
    let schema = ProjectSchema::new();
    let adapter = adapter_id(1);
    let adapters = resolver_with(adapter);
    let states = BTreeMap::new();
    let valid = project_body(project_id(), json!({}), adapter, json!({}));
    assert!(admit(valid.clone(), &schema, &states, &adapters).is_ok());

    for missing in [
        "schema",
        "project_id",
        "components",
        "adapter_state_id",
        "project_metadata",
    ] {
        let mut invalid = valid.clone();
        invalid.as_object_mut().expect("object").remove(missing);
        assert!(
            ProjectStateCandidate::deserialize(invalid).is_err(),
            "omitted required member {missing} was accepted"
        );
    }
    let mut extended = valid;
    extended["unexpected"] = json!(true);
    assert!(ProjectStateCandidate::deserialize(extended).is_err());
    assert!(ProjectStateCandidate::deserialize(json!([])).is_err());
}

#[test]
fn raw_duplicate_names_are_rejected_at_top_level_and_nested_metadata() {
    let adapter = adapter_id(2);
    let top_duplicate = format!(
        r#"{{"schema":"{PROJECT_SCHEMA}","schema":"{PROJECT_SCHEMA}","project_id":"{PROJECT_ID}","components":{{}},"adapter_state_id":"{adapter}","project_metadata":{{}}}}"#
    );
    let nested_duplicate = format!(
        r#"{{"schema":"{PROJECT_SCHEMA}","project_id":"{PROJECT_ID}","components":{{}},"adapter_state_id":"{adapter}","project_metadata":{{"title":"a","\u0074itle":"b"}}}}"#
    );
    let component_key_duplicate = format!(
        r#"{{"schema":"{PROJECT_SCHEMA}","project_id":"{PROJECT_ID}","components":{{"{COMPONENT_ID}":"{}","\u0030{}":"{}"}},"adapter_state_id":"{adapter}","project_metadata":{{}}}}"#,
        "omvcs:component-state:sha256:".to_owned() + &"11".repeat(32),
        &COMPONENT_ID[1..],
        "omvcs:component-state:sha256:".to_owned() + &"22".repeat(32)
    );
    for raw in [top_duplicate, nested_duplicate, component_key_duplicate] {
        assert!(
            serde_json::from_str::<ProjectStateCandidate>(&raw).is_err(),
            "duplicate raw JSON member accepted: {raw}"
        );
    }
}

#[test]
fn schema_must_be_exactly_available_and_unique() {
    let adapter = adapter_id(3);
    let adapters = resolver_with(adapter);
    let states = BTreeMap::new();
    let candidate = project_body(project_id(), json!({}), adapter, json!({}));
    let decoded =
        || serde_json::from_value::<ProjectStateCandidate>(candidate.clone()).expect("candidate");

    assert_eq!(
        decoded().admit(&[], &states, &adapters),
        Err(ProjectStateAdmissionError::UnavailableSchema)
    );
    let wrong_schema = ProjectSchema::new().with_schema("test.other/1");
    assert_eq!(
        decoded().admit(&[&wrong_schema], &states, &adapters),
        Err(ProjectStateAdmissionError::UnavailableSchema)
    );
    let first = ProjectSchema::new();
    let second = ProjectSchema::new();
    assert_eq!(
        decoded().admit(&[&first, &second], &states, &adapters),
        Err(ProjectStateAdmissionError::NonUniqueSchemaAuthority)
    );
    let invalid_schema = ProjectSchema {
        metadata_schema: MetadataSchema::Scalar,
        ..ProjectSchema::new()
    };
    assert_eq!(
        decoded().admit(&[&invalid_schema], &states, &adapters),
        Err(ProjectStateAdmissionError::InvalidSchemaDefinition)
    );
}

#[test]
fn component_keys_and_values_must_be_canonical_typed_identifiers() {
    let adapter = adapter_id(4);
    for (key, value) in [
        (
            "not-a-component",
            "omvcs:component-state:sha256:".to_owned() + &"11".repeat(32),
        ),
        (
            COMPONENT_ID,
            "omvcs:adapter-state:sha256:".to_owned() + &"11".repeat(32),
        ),
        (
            "019CC17D-1B22-7A41-9FE9-C345C468F82D",
            "omvcs:component-state:sha256:".to_owned() + &"11".repeat(32),
        ),
    ] {
        let body = project_body(project_id(), json!({key: value}), adapter, json!({}));
        assert!(
            serde_json::from_value::<ProjectStateCandidate>(body).is_err(),
            "noncanonical typed component reference was accepted: {key}"
        );
    }
    let body = project_body(
        project_id(),
        json!({COMPONENT_ID: "omvcs:component-state:sha256:".to_owned() + &"11".repeat(32)}),
        adapter,
        json!({}),
    );
    assert!(serde_json::from_value::<ProjectStateCandidate>(body).is_ok());

    let invalid_project = project_body(project_id(), json!({}), adapter, json!({}))
        .to_string()
        .replace(PROJECT_ID, "019CC17D-1B22-7A41-9FE9-C345C468F82C");
    assert!(serde_json::from_str::<ProjectStateCandidate>(&invalid_project).is_err());
    let wrong_adapter_type = project_body(project_id(), json!({}), adapter, json!({}))
        .to_string()
        .replace(
            &adapter.to_string(),
            &("omvcs:resource:sha256:".to_owned() + &"33".repeat(32)),
        );
    assert!(serde_json::from_str::<ProjectStateCandidate>(&wrong_adapter_type).is_err());
}

#[test]
fn every_component_reference_must_resolve_and_match_its_embedded_identity() {
    let schema = ProjectSchema::new();
    let adapter = adapter_id(5);
    let adapters = resolver_with(adapter);
    let matching = admitted_component(component_id(), "one");
    let matching_id = matching.component_state_id();
    let states = BTreeMap::from([(matching_id, matching)]);
    let body = project_body(
        project_id(),
        json!({COMPONENT_ID: matching_id.to_string()}),
        adapter,
        json!({}),
    );
    assert!(admit(body, &schema, &states, &adapters).is_ok());

    let absent_id = ComponentStateId::from_digest([99; 32]);
    let absent = project_body(
        project_id(),
        json!({COMPONENT_ID: absent_id.to_string()}),
        adapter,
        json!({}),
    );
    assert_eq!(
        admit(absent, &schema, &states, &adapters),
        Err(ProjectStateAdmissionError::UnavailableComponentState)
    );

    let other_component = CreativeComponentId::new();
    let wrong_state = admitted_component(other_component, "other");
    let wrong_state_id = wrong_state.component_state_id();
    let wrong_states = BTreeMap::from([(wrong_state_id, wrong_state)]);
    let mismatch = project_body(
        project_id(),
        json!({COMPONENT_ID: wrong_state_id.to_string()}),
        adapter,
        json!({}),
    );
    assert_eq!(
        admit(mismatch, &schema, &wrong_states, &adapters),
        Err(ProjectStateAdmissionError::ComponentIdentityMismatch)
    );
}

#[test]
fn referenced_metadata_can_be_admitted_without_materializing_resource_bytes() {
    let schema = ProjectSchema::new();
    let adapter = adapter_id(12);
    let adapters = resolver_with(adapter);
    let resource_bytes = b"not materialized here";
    let resource_id = omvcs_model::hashing::hash_resource_bytes(resource_bytes);
    let candidate: ComponentStateCandidate = serde_json::from_value(json!({
        "schema": COMPONENT_SCHEMA,
        "component_id": COMPONENT_ID,
        "resources": [{
            "resource_id": resource_id.to_string(),
            "byte_length": resource_bytes.len()
        }],
        "metadata": {}
    }))
    .expect("Component State candidate");
    // The model admits the immutable Resource Reference without needing a
    // materialized Resource Object byte buffer.
    let component_state = candidate.admit(&[&ComponentSchema], &[]).expect("state");
    let component_state_id = component_state.component_state_id();
    let states = BTreeMap::from([(component_state_id, component_state)]);
    let body = project_body(
        project_id(),
        json!({COMPONENT_ID: component_state_id.to_string()}),
        adapter,
        json!({}),
    );
    assert!(admit(body, &schema, &states, &adapters).is_ok());
}

#[test]
fn adapter_state_requires_trusted_resolvable_admitted_object() {
    let schema = ProjectSchema::new();
    let adapter = adapter_id(6);
    let body = project_body(project_id(), json!({}), adapter, json!({}));
    let states = BTreeMap::new();
    assert_eq!(
        admit(body.clone(), &schema, &states, &AdapterResolver::default()),
        Err(ProjectStateAdmissionError::UnavailableAdapterState)
    );
    assert!(admit(body, &schema, &states, &resolver_with(adapter)).is_ok());
}

#[test]
fn project_metadata_is_schema_checked_recursively_and_semantically() {
    let schema = ProjectSchema::new().requiring_title();
    let adapter = adapter_id(7);
    let adapters = resolver_with(adapter);
    let states = BTreeMap::new();
    let body = |metadata| project_body(project_id(), json!({}), adapter, metadata);

    assert!(matches!(
        admit(body(json!({})), &schema, &states, &adapters),
        Err(ProjectStateAdmissionError::MetadataRejected(_))
    ));
    for invalid in [
        json!({"unknown": "x"}),
        json!({"title": []}),
        json!({"title": "x", "tags": [1, "two"]}),
        json!({"title": "x", "nested": {"labels": [true]}}),
    ] {
        assert!(admit(body(invalid), &schema, &states, &adapters).is_err());
    }
    let valid = body(json!({
        "title": "Project",
        "tags": ["b", "a"],
        "sequence": [1, 2],
        "nested": {"labels": ["y", "x"]}
    }));
    assert!(admit(valid, &schema, &states, &adapters).is_ok());

    let unclassified = ProjectSchema {
        metadata_schema: MetadataSchema::structure([(
            "items",
            MetadataSchema::unclassified_array(MetadataSchema::Scalar),
        )]),
        ..ProjectSchema::new()
    };
    let invalid = project_body(project_id(), json!({}), adapter, json!({"items": []}));
    assert!(admit(invalid, &unclassified, &states, &adapters).is_err());
}

#[test]
fn canonical_body_is_exactly_five_members_and_hashes_without_type_prefix() {
    let schema = ProjectSchema::new();
    let adapter = adapter_id(8);
    let adapters = resolver_with(adapter);
    let state = admitted_component(component_id(), "one");
    let state_id = state.component_state_id();
    let states = BTreeMap::from([(state_id, state)]);
    let admitted = admit(
        project_body(
            project_id(),
            json!({COMPONENT_ID: state_id.to_string()}),
            adapter,
            json!({"title": "A", "tags": ["z", "a"]}),
        ),
        &schema,
        &states,
        &adapters,
    )
    .expect("valid Project State");
    let canonical: Value =
        serde_json::from_slice(admitted.canonical_body()).expect("canonical JSON");
    let object = canonical.as_object().expect("object body");
    assert_eq!(
        object.keys().map(String::as_str).collect::<Vec<_>>(),
        [
            "adapter_state_id",
            "components",
            "project_id",
            "project_metadata",
            "schema"
        ]
    );
    let direct = Sha256::digest(admitted.canonical_body());
    assert_eq!(
        admitted.project_state_id(),
        hash_project_state_metadata(admitted.canonical_body())
    );
    assert_eq!(
        admitted.project_state_id().digest().as_slice(),
        direct.as_slice()
    );
    assert!(
        admitted
            .canonical_body()
            .starts_with(b"{\"adapter_state_id\":")
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn every_canonical_member_changes_identity_but_map_order_does_not() {
    let schema = ProjectSchema::new();
    let first_adapter = adapter_id(9);
    let second_adapter = adapter_id(10);
    let adapters = AdapterResolver(HashSet::from([first_adapter, second_adapter]));
    let first_component = component_id();
    let second_component = CreativeComponentId::new();
    let component_a = admitted_component(first_component, "state-a");
    let component_b = admitted_component(first_component, "state-b");
    let component_c = admitted_component(second_component, "state-c");
    let id_a = component_a.component_state_id();
    let id_b = component_b.component_state_id();
    let id_c = component_c.component_state_id();
    let states = BTreeMap::from([
        (id_a, component_a),
        (id_b, component_b),
        (id_c, component_c),
    ]);
    let make = |schema_value: &ProjectSchema,
                project: ProjectId,
                components: Value,
                adapter: AdapterStateId,
                metadata: Value| {
        let mut body = project_body(project, components, adapter, metadata);
        body["schema"] = json!(schema_value.schema);
        let candidate: ProjectStateCandidate = serde_json::from_value(body).expect("candidate");
        candidate
            .admit(&[schema_value], &states, &adapters)
            .expect("valid Project State")
    };
    let base = make(
        &schema,
        project_id(),
        json!({COMPONENT_ID: id_a.to_string()}),
        first_adapter,
        json!({"title": "A", "tags": ["b", "a"]}),
    );
    let variants = [
        make(
            &ProjectSchema::new().with_schema("test.project-state/2"),
            project_id(),
            json!({COMPONENT_ID: id_a.to_string()}),
            first_adapter,
            json!({"title": "A", "tags": ["a", "b"]}),
        ),
        make(
            &schema,
            ProjectId::new(),
            json!({COMPONENT_ID: id_a.to_string()}),
            first_adapter,
            json!({"title": "A", "tags": ["a", "b"]}),
        ),
        make(
            &schema,
            project_id(),
            json!({COMPONENT_ID: id_b.to_string()}),
            first_adapter,
            json!({"title": "A", "tags": ["a", "b"]}),
        ),
        make(
            &schema,
            project_id(),
            json!({
                COMPONENT_ID: id_a.to_string(),
                second_component.to_string(): id_c.to_string()
            }),
            first_adapter,
            json!({"title": "A", "tags": ["a", "b"]}),
        ),
        make(
            &schema,
            project_id(),
            json!({COMPONENT_ID: id_a.to_string()}),
            second_adapter,
            json!({"title": "A", "tags": ["a", "b"]}),
        ),
        make(
            &schema,
            project_id(),
            json!({COMPONENT_ID: id_a.to_string()}),
            first_adapter,
            json!({"title": "B", "tags": ["a", "b"]}),
        ),
    ];
    for (index, changed) in variants.into_iter().enumerate() {
        assert_ne!(
            base.project_state_id(),
            changed.project_state_id(),
            "canonical member variant {index} did not change identity: base={}, changed={}",
            String::from_utf8_lossy(base.canonical_body()),
            String::from_utf8_lossy(changed.canonical_body())
        );
    }

    let reordered: ProjectStateCandidate = serde_json::from_str(&format!(
        r#"{{"project_metadata":{{"tags":["a","b"],"title":"A"}},"adapter_state_id":"{first_adapter}","components":{{"{second_component}":"{id_c}","{COMPONENT_ID}":"{id_a}"}},"project_id":"{PROJECT_ID}","schema":"{PROJECT_SCHEMA}"}}"#
    ))
    .expect("reordered candidate");
    let reordered = reordered
        .admit(&[&schema], &states, &adapters)
        .expect("valid reordered state");
    let map_base = make(
        &schema,
        project_id(),
        json!({
            COMPONENT_ID: id_a.to_string(),
            second_component.to_string(): id_c.to_string()
        }),
        first_adapter,
        json!({"title": "A", "tags": ["a", "b"]}),
    );
    assert_eq!(map_base.canonical_body(), reordered.canonical_body());
    assert_eq!(map_base.project_state_id(), reordered.project_state_id());
}

#[test]
fn noncanonical_map_keys_and_unknown_operational_fields_never_enter_history() {
    let adapter = adapter_id(11);
    let invalid_key = format!(
        r#"{{"schema":"{PROJECT_SCHEMA}","project_id":"{PROJECT_ID}","components":{{"{COMPONENT_ID}":"omvcs:component-state:sha256:{}"}},"adapter_state_id":"{adapter}","project_metadata":{{}},"storage_location":"local"}}"#,
        "12".repeat(32)
    );
    assert!(serde_json::from_str::<ProjectStateCandidate>(&invalid_key).is_err());

    let noncanonical_state_id = format!(
        r#"{{"schema":"{PROJECT_SCHEMA}","project_id":"{PROJECT_ID}","components":{{"{COMPONENT_ID}":"OMVCS:component-state:sha256:{}"}},"adapter_state_id":"{adapter}","project_metadata":{{}}}}"#,
        "12".repeat(32)
    );
    assert!(serde_json::from_str::<ProjectStateCandidate>(&noncanonical_state_id).is_err());
}
