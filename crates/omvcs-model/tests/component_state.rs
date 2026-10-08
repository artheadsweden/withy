#![allow(clippy::expect_used)]

use std::cell::Cell;
use std::collections::BTreeMap;

use omvcs_model::canonical::{ArrayOrdering, CanonicalMetadataError, MetadataSchema};
use omvcs_model::component_state::{
    ComponentStateAdmissionError, ComponentStateCandidate, ComponentStateSchemaValidator,
};
use omvcs_model::hashing::{hash_component_state_metadata, hash_resource_bytes};
use omvcs_model::resource::{
    ResourceAdmissionError, ResourceByteLength, ResourcePropertiesValidator, ResourceReference,
    ResourceReferenceCandidate, ResourceValidationContext,
};
use omvcs_model::{ComponentStateId, CreativeComponentId};
use serde_json::{Value, json};

const SCHEMA: &str = "test.component-state/1";

fn component_id() -> CreativeComponentId {
    CreativeComponentId::new()
}

fn state_schema() -> MetadataSchema {
    MetadataSchema::structure([
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
            MetadataSchema::structure([
                (
                    "set",
                    MetadataSchema::array(ArrayOrdering::SetLike, MetadataSchema::Scalar),
                ),
                (
                    "sequence",
                    MetadataSchema::array(ArrayOrdering::Ordered, MetadataSchema::Scalar),
                ),
            ]),
        ),
    ])
}

struct TestSchema {
    schema: String,
    metadata_schema: MetadataSchema,
    required_keys: Vec<String>,
    resource_context: Option<ResourceValidationContext>,
}

impl TestSchema {
    fn new(schema: impl Into<String>) -> Self {
        Self {
            schema: schema.into(),
            metadata_schema: state_schema(),
            required_keys: Vec::new(),
            resource_context: Some(ResourceValidationContext {
                containing_schema: SCHEMA.to_owned(),
                adapter: Some(("test.adapter".to_owned(), "test.adapter-state/4".to_owned())),
            }),
        }
    }

    fn requiring(mut self, fields: &[&str]) -> Self {
        self.required_keys = fields.iter().map(|field| (*field).to_owned()).collect();
        self
    }

    fn with_metadata_schema(mut self, metadata_schema: MetadataSchema) -> Self {
        self.metadata_schema = metadata_schema;
        self
    }

    fn without_resource_binding(mut self) -> Self {
        self.resource_context = None;
        self
    }
}

impl ComponentStateSchemaValidator for TestSchema {
    fn schema(&self) -> &str {
        &self.schema
    }

    fn metadata_schema(&self) -> MetadataSchema {
        self.metadata_schema.clone()
    }

    fn validate_metadata(&self, metadata: &BTreeMap<String, Value>) -> Result<(), String> {
        if let Some(missing) = self
            .required_keys
            .iter()
            .find(|field| !metadata.contains_key(*field))
        {
            return Err(format!("required test metadata key is missing: {missing}"));
        }
        Ok(())
    }

    fn resource_validation_context(
        &self,
        reference: &ResourceReferenceCandidate,
    ) -> Option<ResourceValidationContext> {
        reference
            .properties()
            .and(self.resource_context.as_ref())
            .cloned()
    }
}

struct TestResourceValidator {
    context: ResourceValidationContext,
    properties_schema: MetadataSchema,
    reject: bool,
    calls: Cell<usize>,
}

impl TestResourceValidator {
    fn new(context: ResourceValidationContext) -> Self {
        Self {
            context,
            properties_schema: MetadataSchema::structure([
                ("approved", MetadataSchema::Scalar),
                (
                    "tags",
                    MetadataSchema::array(ArrayOrdering::SetLike, MetadataSchema::Scalar),
                ),
                (
                    "sequence",
                    MetadataSchema::array(ArrayOrdering::Ordered, MetadataSchema::Scalar),
                ),
            ]),
            reject: false,
            calls: Cell::new(0),
        }
    }

    const fn rejecting(mut self) -> Self {
        self.reject = true;
        self
    }
}

impl ResourcePropertiesValidator for TestResourceValidator {
    fn context(&self) -> &ResourceValidationContext {
        &self.context
    }

    fn properties_schema(&self) -> MetadataSchema {
        self.properties_schema.clone()
    }

    fn validate_properties(&self, _properties: &BTreeMap<String, Value>) -> Result<(), String> {
        self.calls.set(self.calls.get() + 1);
        if self.reject {
            Err("synthetic authority rejection".to_owned())
        } else {
            Ok(())
        }
    }
}

const fn empty_metadata() -> BTreeMap<String, Value> {
    BTreeMap::new()
}

fn reference_candidate(label: &str, length: u64) -> ResourceReferenceCandidate {
    ResourceReferenceCandidate::new(
        hash_resource_bytes(label.as_bytes()),
        ResourceByteLength::new(length).expect("test byte length is in range"),
    )
}

fn candidate(
    schema: &str,
    component: CreativeComponentId,
    resources: Vec<ResourceReferenceCandidate>,
    metadata: BTreeMap<String, Value>,
) -> ComponentStateCandidate {
    ComponentStateCandidate::new(schema, component, resources, metadata)
}

fn admit(
    candidate: ComponentStateCandidate,
    schema: &TestSchema,
    validators: &[&dyn ResourcePropertiesValidator],
) -> Result<omvcs_model::ComponentState, ComponentStateAdmissionError> {
    candidate.admit(&[schema], validators)
}

fn context_for_test_schema() -> ResourceValidationContext {
    ResourceValidationContext {
        containing_schema: SCHEMA.to_owned(),
        adapter: Some(("test.adapter".to_owned(), "test.adapter-state/4".to_owned())),
    }
}

fn properties_candidate() -> ResourceReferenceCandidate {
    let mut properties = BTreeMap::new();
    properties.insert("approved".to_owned(), json!(true));
    properties.insert("tags".to_owned(), json!(["z", "a"]));
    properties.insert("sequence".to_owned(), json!(["last", "first"]));
    reference_candidate("property-bearing", 17).with_properties(properties)
}

#[allow(clippy::needless_pass_by_value)]
fn body_json(
    schema: &str,
    component_id: &str,
    resources: Value,
    metadata: Value,
    parents: Option<Value>,
) -> Value {
    let mut body = json!({
        "schema": schema,
        "component_id": component_id,
        "resources": resources,
        "metadata": metadata
    });
    if let Some(parents) = parents {
        body["parents"] = parents;
    }
    body
}

#[test]
fn component_state_requires_closed_body_and_typed_component_identity() {
    let id = component_id();
    let valid = body_json(SCHEMA, &id.to_string(), json!([]), json!({}), None);
    let parsed = serde_json::from_value::<ComponentStateCandidate>(valid.clone());
    assert!(parsed.is_ok());
    let parsed = parsed.expect("valid candidate");
    assert_eq!(parsed.schema(), SCHEMA);
    assert_eq!(parsed.component_id(), id);
    assert!(parsed.parents().is_none());

    for field in ["schema", "component_id", "resources", "metadata"] {
        let mut invalid = valid.clone();
        invalid.as_object_mut().expect("object body").remove(field);
        assert!(
            serde_json::from_value::<ComponentStateCandidate>(invalid).is_err(),
            "omitted required field {field} was accepted"
        );
    }

    for invalid_id in [
        "not-an-id",
        &hash_resource_bytes(b"resource id").to_string(),
    ] {
        let invalid = body_json(SCHEMA, invalid_id, json!([]), json!({}), None);
        assert!(
            serde_json::from_value::<ComponentStateCandidate>(invalid).is_err(),
            "invalid or wrongly namespaced component_id {invalid_id} was accepted"
        );
    }

    for invalid in [
        json!([]),
        json!([id.to_string()]),
        json!(id.to_string()),
        json!(null),
        json!(true),
    ] {
        assert!(
            serde_json::from_value::<ComponentStateCandidate>(invalid).is_err(),
            "non-object candidate was accepted"
        );
    }
}

#[test]
fn component_state_rejects_unknown_top_level_members() {
    let id = component_id();
    for field in [
        "validation_evidence",
        "validator",
        "timestamp",
        "presentation",
        "storage",
        "transport",
        "signature",
        "credential",
        "platform",
        "extension",
        "future_member",
    ] {
        let mut body = body_json(SCHEMA, &id.to_string(), json!([]), json!({}), None);
        body[field] = json!("excluded");
        assert!(
            serde_json::from_value::<ComponentStateCandidate>(body).is_err(),
            "unknown top-level field {field} was accepted"
        );
    }
}

#[test]
fn component_state_admission_accepts_required_empty_resources_and_metadata() {
    let schema = TestSchema::new(SCHEMA);
    let state = admit(
        candidate(SCHEMA, component_id(), Vec::new(), empty_metadata()),
        &schema,
        &[],
    );
    assert!(state.is_ok());
    let state = state.expect("empty collections are valid");
    assert_eq!(state.resources(), []);
    assert!(state.metadata().is_empty());
    assert_eq!(
        serde_json::from_slice::<Value>(state.canonical_body()).ok(),
        Some(body_json(
            SCHEMA,
            &state.component_id().to_string(),
            json!([]),
            json!({}),
            None
        ))
    );
}

#[test]
fn component_state_preserves_unknown_schema_candidates_but_does_not_admit_them() {
    let parsed = serde_json::from_value::<ComponentStateCandidate>(body_json(
        "test.component-state/2",
        &component_id().to_string(),
        json!([]),
        json!({}),
        None,
    ));
    assert!(parsed.is_ok());
    assert_eq!(
        parsed
            .expect("unknown schema candidate remains representable")
            .admit(&[], &[]),
        Err(ComponentStateAdmissionError::UnavailableSchema)
    );

    let unavailable = candidate(SCHEMA, component_id(), Vec::new(), empty_metadata());
    assert_eq!(
        unavailable.admit(&[], &[]),
        Err(ComponentStateAdmissionError::UnavailableSchema)
    );
}

#[test]
fn component_state_requires_one_exact_schema_authority() {
    let schema = TestSchema::new(SCHEMA);
    let duplicate = TestSchema::new(SCHEMA);
    let candidate = candidate(SCHEMA, component_id(), Vec::new(), empty_metadata());

    assert_eq!(
        candidate.admit(&[&schema, &duplicate], &[]),
        Err(ComponentStateAdmissionError::NonUniqueSchemaAuthority)
    );
}

#[test]
fn component_state_exact_metadata_schema_checks_keys_shapes_and_nested_arrays() {
    let schema = TestSchema::new(SCHEMA);
    let mut metadata = BTreeMap::new();
    metadata.insert("tags".to_owned(), json!(["z", "a"]));
    metadata.insert("sequence".to_owned(), json!(["second", "first"]));
    metadata.insert(
        "nested".to_owned(),
        json!({
            "set": ["b", "a"],
            "sequence": [2, 1]
        }),
    );
    let state = admit(
        candidate(SCHEMA, component_id(), Vec::new(), metadata),
        &schema,
        &[],
    )
    .expect("metadata matches the exact fixture schema");
    let body: Value = serde_json::from_slice(state.canonical_body()).expect("canonical JSON body");
    assert_eq!(
        body["metadata"],
        json!({
            "tags": ["a", "z"],
            "sequence": ["second", "first"],
            "nested": {
                "set": ["a", "b"],
                "sequence": [2, 1]
            }
        })
    );

    let mut unknown_key = BTreeMap::new();
    unknown_key.insert("not_permitted".to_owned(), json!(true));
    assert!(matches!(
        admit(
            candidate(SCHEMA, component_id(), Vec::new(), unknown_key),
            &schema,
            &[]
        ),
        Err(ComponentStateAdmissionError::Canonical(
            CanonicalMetadataError::SchemaMismatch { .. }
        ))
    ));

    let invalid_shape = BTreeMap::from([("title".to_owned(), json!([]))]);
    assert!(matches!(
        admit(
            candidate(SCHEMA, component_id(), Vec::new(), invalid_shape),
            &schema,
            &[]
        ),
        Err(ComponentStateAdmissionError::Canonical(
            CanonicalMetadataError::SchemaMismatch { .. }
        ))
    ));

    let unclassified = TestSchema::new(SCHEMA).with_metadata_schema(MetadataSchema::structure([(
        "values",
        MetadataSchema::unclassified_array(MetadataSchema::Scalar),
    )]));
    assert!(matches!(
        admit(
            candidate(
                SCHEMA,
                component_id(),
                Vec::new(),
                BTreeMap::from([("values".to_owned(), json!([]))])
            ),
            &unclassified,
            &[]
        ),
        Err(ComponentStateAdmissionError::Canonical(
            CanonicalMetadataError::UnclassifiedArray { .. }
        ))
    ));
}

#[test]
fn component_state_schema_owns_required_metadata_keys_and_semantics() {
    let schema = TestSchema::new(SCHEMA).requiring(&["title"]);
    assert!(matches!(
        admit(
            candidate(SCHEMA, component_id(), Vec::new(), empty_metadata()),
            &schema,
            &[]
        ),
        Err(ComponentStateAdmissionError::MetadataRejected(_))
    ));
    let metadata = BTreeMap::from([("title".to_owned(), json!("Bass"))]);
    assert!(
        admit(
            candidate(SCHEMA, component_id(), Vec::new(), metadata),
            &schema,
            &[]
        )
        .is_ok()
    );
}

#[test]
fn component_state_distinguishes_unknown_parentage_from_known_initial_state() {
    let schema = TestSchema::new(SCHEMA);
    let unknown = admit(
        candidate(SCHEMA, component_id(), Vec::new(), empty_metadata()),
        &schema,
        &[],
    )
    .expect("unknown parentage is valid");
    let initial = admit(
        candidate(SCHEMA, unknown.component_id(), Vec::new(), empty_metadata())
            .with_parents(Vec::new()),
        &schema,
        &[],
    )
    .expect("known zero-parent initial state is valid");
    let derived = admit(
        candidate(SCHEMA, unknown.component_id(), Vec::new(), empty_metadata())
            .with_parents(vec![ComponentStateId::from_digest([0x12; 32])]),
        &schema,
        &[],
    )
    .expect("known parentage is valid");

    assert!(unknown.parents().is_none());
    assert_eq!(initial.parents(), Some([].as_slice()));
    assert_eq!(derived.parents().map(<[_]>::len), Some(1));
    assert_ne!(unknown.component_state_id(), initial.component_state_id());
    assert_ne!(initial.component_state_id(), derived.component_state_id());
}

#[test]
fn component_state_parent_and_resource_sets_are_permutation_invariant_and_unique() {
    let schema = TestSchema::new(SCHEMA);
    let parent_a = ComponentStateId::from_digest([0x11; 32]);
    let parent_b = ComponentStateId::from_digest([0x22; 32]);
    let resource_a = reference_candidate("a", 1);
    let resource_b = reference_candidate("b", 2);
    let left = admit(
        candidate(
            SCHEMA,
            component_id(),
            vec![resource_a.clone(), resource_b.clone()],
            empty_metadata(),
        )
        .with_parents(vec![parent_a, parent_b]),
        &schema,
        &[],
    )
    .expect("unique sets are valid");
    let right = admit(
        candidate(
            SCHEMA,
            left.component_id(),
            vec![resource_b, resource_a.clone()],
            empty_metadata(),
        )
        .with_parents(vec![parent_b, parent_a]),
        &schema,
        &[],
    )
    .expect("permuted sets are valid");
    assert_eq!(left.canonical_body(), right.canonical_body());
    assert_eq!(left.component_state_id(), right.component_state_id());
    assert_eq!(left, right);

    assert_eq!(
        admit(
            candidate(SCHEMA, component_id(), Vec::new(), empty_metadata())
                .with_parents(vec![parent_a, parent_a]),
            &schema,
            &[]
        ),
        Err(ComponentStateAdmissionError::DuplicateParent)
    );
    assert_eq!(
        admit(
            candidate(
                SCHEMA,
                component_id(),
                vec![resource_a.clone(), resource_a],
                empty_metadata()
            ),
            &schema,
            &[]
        ),
        Err(ComponentStateAdmissionError::DuplicateResource)
    );
}

#[test]
fn component_state_metadata_map_order_is_irrelevant_and_duplicate_names_are_rejected() {
    let schema = TestSchema::new(SCHEMA);
    let left = br#"{"schema":"test.component-state/1","component_id":"019cc17d-1b22-7a41-9fe9-c345c468f82c","resources":[],"metadata":{"title":"Bass","tags":["b","a"]}}"#;
    let right = br#"{"metadata":{"tags":["a","b"],"title":"Bass"},"resources":[],"component_id":"019cc17d-1b22-7a41-9fe9-c345c468f82c","schema":"test.component-state/1"}"#;
    let left: ComponentStateCandidate =
        serde_json::from_slice(left).expect("valid body with schema-owned keys");
    let right: ComponentStateCandidate =
        serde_json::from_slice(right).expect("permuted valid body");
    let left = admit(left, &schema, &[]).expect("valid body");
    let right = admit(right, &schema, &[]).expect("permuted valid body");
    assert_eq!(left.canonical_body(), right.canonical_body());
    assert_eq!(left.component_state_id(), right.component_state_id());

    for duplicate in [
        format!(
            r#"{{"schema":"{SCHEMA}","component_id":"019cc17d-1b22-7a41-9fe9-c345c468f82c","resources":[],"metadata":{{"title":"first","title":"second"}}}}"#
        ),
        format!(
            r#"{{"schema":"{SCHEMA}","component_id":"019cc17d-1b22-7a41-9fe9-c345c468f82c","resources":[],"metadata":{{"nested":{{"key":1,"key":2}}}}}}"#
        ),
        format!(
            r#"{{"schema":"{SCHEMA}","component_id":"019cc17d-1b22-7a41-9fe9-c345c468f82c","resources":[],"metadata":{{"title":"first","\u0074itle":"second"}}}}"#
        ),
    ] {
        assert!(
            serde_json::from_str::<ComponentStateCandidate>(&duplicate).is_err(),
            "duplicate names must be rejected before map decoding"
        );
    }
}

#[test]
fn component_state_property_bearing_resources_need_the_exact_unique_authority() {
    let schema = TestSchema::new(SCHEMA);
    let authority = TestResourceValidator::new(context_for_test_schema());
    let state = admit(
        candidate(
            SCHEMA,
            component_id(),
            vec![properties_candidate()],
            empty_metadata(),
        ),
        &schema,
        &[&authority],
    )
    .expect("exact property authority admits the reference");
    assert_eq!(authority.calls.get(), 1);
    assert_eq!(state.resources().len(), 1);

    let missing = TestSchema::new(SCHEMA).without_resource_binding();
    assert!(matches!(
        admit(
            candidate(
                SCHEMA,
                component_id(),
                vec![properties_candidate()],
                empty_metadata()
            ),
            &missing,
            &[&authority]
        ),
        Err(ComponentStateAdmissionError::Resource(
            ResourceAdmissionError::UnknownContext
        ))
    ));

    let duplicate_authority = TestResourceValidator::new(context_for_test_schema());
    assert!(matches!(
        admit(
            candidate(
                SCHEMA,
                component_id(),
                vec![properties_candidate()],
                empty_metadata()
            ),
            &schema,
            &[&authority, &duplicate_authority]
        ),
        Err(ComponentStateAdmissionError::Resource(
            ResourceAdmissionError::NonUniqueAuthority
        ))
    ));

    let wrong_version = TestResourceValidator::new(ResourceValidationContext {
        containing_schema: SCHEMA.to_owned(),
        adapter: Some(("test.adapter".to_owned(), "test.adapter-state/5".to_owned())),
    });
    assert!(matches!(
        admit(
            candidate(
                SCHEMA,
                component_id(),
                vec![properties_candidate()],
                empty_metadata()
            ),
            &schema,
            &[&wrong_version]
        ),
        Err(ComponentStateAdmissionError::Resource(
            ResourceAdmissionError::UnavailableContext
        ))
    ));

    let semantic_rejection = TestResourceValidator::new(context_for_test_schema()).rejecting();
    assert!(matches!(
        admit(
            candidate(
                SCHEMA,
                component_id(),
                vec![properties_candidate()],
                empty_metadata()
            ),
            &schema,
            &[&semantic_rejection]
        ),
        Err(ComponentStateAdmissionError::Resource(
            ResourceAdmissionError::SemanticRejection(_)
        ))
    ));
}

#[test]
fn empty_resource_properties_still_require_contextual_validation() {
    let schema = TestSchema::new(SCHEMA);
    let reference = reference_candidate("empty-properties", 0).with_properties(BTreeMap::new());
    assert!(matches!(
        admit(
            candidate(SCHEMA, component_id(), vec![reference], empty_metadata()),
            &schema,
            &[]
        ),
        Err(ComponentStateAdmissionError::Resource(
            ResourceAdmissionError::UnavailableContext
        ))
    ));

    let authority = TestResourceValidator::new(context_for_test_schema());
    assert!(
        admit(
            candidate(
                SCHEMA,
                component_id(),
                vec![reference_candidate("empty-properties", 0).with_properties(BTreeMap::new())],
                empty_metadata()
            ),
            &schema,
            &[&authority]
        )
        .is_ok()
    );
    assert_eq!(authority.calls.get(), 1);
}

#[test]
fn component_state_resource_property_arrays_follow_the_exact_context_schema() {
    let schema = TestSchema::new(SCHEMA);
    let authority = TestResourceValidator::new(context_for_test_schema());
    let state = admit(
        candidate(
            SCHEMA,
            component_id(),
            vec![properties_candidate()],
            empty_metadata(),
        ),
        &schema,
        &[&authority],
    )
    .expect("properties match the exact authority schema");
    let reference = &state.resources()[0];
    let value = reference
        .historical_value(Some(&context_for_test_schema()))
        .expect("same context");
    assert_eq!(value["properties"]["tags"], json!(["a", "z"]));
    assert_eq!(value["properties"]["sequence"], json!(["last", "first"]));
}

#[test]
fn component_state_embedded_resource_byte_length_uses_the_approved_exact_range() {
    let schema = TestSchema::new(SCHEMA);
    let accepted = ["0", "1", "9007199254740991"];
    for length in accepted {
        let json = format!(
            r#"{{"schema":"{SCHEMA}","component_id":"{}","resources":[{{"resource_id":"{}","byte_length":{length}}}],"metadata":{{}}}}"#,
            component_id(),
            hash_resource_bytes(b"length-vector")
        );
        let candidate = serde_json::from_str::<ComponentStateCandidate>(&json);
        assert!(candidate.is_ok(), "accepted byte_length {length}");
        assert!(
            admit(candidate.expect("accepted vector"), &schema, &[]).is_ok(),
            "admitted byte_length {length}"
        );
    }

    for length in ["-1", "1.5", "9007199254740992", r#""1""#] {
        let json = format!(
            r#"{{"schema":"{SCHEMA}","component_id":"{}","resources":[{{"resource_id":"{}","byte_length":{length}}}],"metadata":{{}}}}"#,
            component_id(),
            hash_resource_bytes(b"length-vector")
        );
        assert!(
            serde_json::from_str::<ComponentStateCandidate>(&json).is_err(),
            "rejected byte_length {length}"
        );
    }
}

#[test]
fn every_present_historical_body_field_changes_component_state_identity() {
    let schema = TestSchema::new(SCHEMA);
    let component = component_id();
    let base = admit(
        candidate(SCHEMA, component, Vec::new(), empty_metadata()),
        &schema,
        &[],
    )
    .expect("base state");

    let changed_component = admit(
        candidate(SCHEMA, component_id(), Vec::new(), empty_metadata()),
        &schema,
        &[],
    )
    .expect("changed component");
    let changed_resources = admit(
        candidate(
            SCHEMA,
            component,
            vec![reference_candidate("new-resource", 3)],
            empty_metadata(),
        ),
        &schema,
        &[],
    )
    .expect("changed resources");
    let changed_metadata = admit(
        candidate(
            SCHEMA,
            component,
            Vec::new(),
            BTreeMap::from([("title".to_owned(), json!("Bass"))]),
        ),
        &schema,
        &[],
    )
    .expect("changed metadata");
    let changed_parents = admit(
        candidate(SCHEMA, component, Vec::new(), empty_metadata())
            .with_parents(vec![ComponentStateId::from_digest([0x44; 32])]),
        &schema,
        &[],
    )
    .expect("changed parents");

    for changed in [
        changed_component.component_state_id(),
        changed_resources.component_state_id(),
        changed_metadata.component_state_id(),
        changed_parents.component_state_id(),
    ] {
        assert_ne!(base.component_state_id(), changed);
    }

    let new_schema = TestSchema::new("test.component-state/2");
    let changed_schema = admit(
        candidate(
            "test.component-state/2",
            component,
            Vec::new(),
            empty_metadata(),
        ),
        &new_schema,
        &[],
    )
    .expect("new exact schema is available");
    assert_ne!(
        base.component_state_id(),
        changed_schema.component_state_id()
    );
}

#[test]
fn changed_resource_reference_fields_change_component_state_identity() {
    let schema = TestSchema::new(SCHEMA);
    let component = component_id();
    let base_ref = reference_candidate("same-resource", 12)
        .with_role("primary")
        .with_media_type("audio/wav");
    let base = admit(
        candidate(SCHEMA, component, vec![base_ref], empty_metadata()),
        &schema,
        &[],
    )
    .expect("base reference");

    let changed_id = admit(
        candidate(
            SCHEMA,
            component,
            vec![
                reference_candidate("different-resource", 12)
                    .with_role("primary")
                    .with_media_type("audio/wav"),
            ],
            empty_metadata(),
        ),
        &schema,
        &[],
    )
    .expect("changed resource id");
    let changed_length = admit(
        candidate(
            SCHEMA,
            component,
            vec![
                reference_candidate("same-resource", 13)
                    .with_role("primary")
                    .with_media_type("audio/wav"),
            ],
            empty_metadata(),
        ),
        &schema,
        &[],
    )
    .expect("changed length");
    let changed_role = admit(
        candidate(
            SCHEMA,
            component,
            vec![
                reference_candidate("same-resource", 12)
                    .with_role("secondary")
                    .with_media_type("audio/wav"),
            ],
            empty_metadata(),
        ),
        &schema,
        &[],
    )
    .expect("changed role");
    let changed_media = admit(
        candidate(
            SCHEMA,
            component,
            vec![
                reference_candidate("same-resource", 12)
                    .with_role("primary")
                    .with_media_type("audio/flac"),
            ],
            empty_metadata(),
        ),
        &schema,
        &[],
    )
    .expect("changed media type");

    for changed in [
        changed_id.component_state_id(),
        changed_length.component_state_id(),
        changed_role.component_state_id(),
        changed_media.component_state_id(),
    ] {
        assert_ne!(base.component_state_id(), changed);
    }
}

#[test]
fn changing_resource_properties_changes_component_state_identity() {
    let schema = TestSchema::new(SCHEMA);
    let component = component_id();
    let base_properties = properties_candidate();
    let base_property_state = admit(
        candidate(SCHEMA, component, vec![base_properties], empty_metadata()),
        &schema,
        &[&TestResourceValidator::new(context_for_test_schema())],
    )
    .expect("base properties");
    let mut changed_properties = BTreeMap::new();
    changed_properties.insert("approved".to_owned(), json!(false));
    changed_properties.insert("tags".to_owned(), json!(["a", "z"]));
    changed_properties.insert("sequence".to_owned(), json!(["last", "first"]));
    let property_authority = TestResourceValidator::new(context_for_test_schema());
    let changed_property_state = admit(
        candidate(
            SCHEMA,
            component,
            vec![reference_candidate("property-bearing", 17).with_properties(changed_properties)],
            empty_metadata(),
        ),
        &schema,
        &[&property_authority],
    )
    .expect("property change is semantically valid in this test authority");
    assert_ne!(
        base_property_state.component_state_id(),
        changed_property_state.component_state_id()
    );
}

#[test]
fn state_hash_is_exact_body_only_and_excludes_admission_evidence_and_wrappers() {
    let schema = TestSchema::new(SCHEMA);
    let authority = TestResourceValidator::new(context_for_test_schema());
    let state = admit(
        candidate(
            SCHEMA,
            component_id(),
            vec![properties_candidate()],
            empty_metadata(),
        ),
        &schema,
        &[&authority],
    )
    .expect("admitted state");
    let body: Value = serde_json::from_slice(state.canonical_body()).expect("canonical body");
    let keys: Vec<&str> = body
        .as_object()
        .expect("body object")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(keys, ["component_id", "metadata", "resources", "schema"]);
    let encoded = String::from_utf8(state.canonical_body().to_vec()).expect("canonical UTF-8");
    for excluded in [
        "validation_evidence",
        "validator",
        "timestamp",
        "presentation",
        "storage",
        "transport",
        "signature",
        "credential",
        "platform",
        "extension",
        "adapter-state/4",
    ] {
        assert!(
            !encoded.contains(excluded),
            "{excluded} entered historical bytes"
        );
    }
    assert_eq!(
        state.component_state_id(),
        hash_component_state_metadata(state.canonical_body())
    );

    let wrapped = json!({
        "body": body,
        "signature": "not historical"
    });
    assert_ne!(
        serde_json::to_vec(&wrapped).ok().as_deref(),
        Some(state.canonical_body())
    );
}

#[test]
fn component_and_component_state_identifier_namespaces_remain_distinct() {
    let digest = [0xa5; 32];
    let component_state = ComponentStateId::from_digest(digest);
    let content_component_state = hash_component_state_metadata(br#"{"schema":"same"}"#);
    let resource = hash_resource_bytes(br#"{"schema":"same"}"#);
    let creative_component = component_id();
    assert_eq!(component_state.digest(), &digest);
    assert!(
        creative_component
            .to_string()
            .parse::<ComponentStateId>()
            .is_err()
    );
    assert!(
        component_state
            .to_string()
            .parse::<CreativeComponentId>()
            .is_err()
    );
    assert_ne!(component_state.to_string(), resource.to_string());
    assert_eq!(content_component_state.digest(), resource.digest());
    assert_ne!(content_component_state.to_string(), resource.to_string());
}

#[test]
fn resource_property_map_insertion_order_does_not_change_component_state_identity() {
    let schema = TestSchema::new(SCHEMA);
    let component = component_id();
    let authority = TestResourceValidator::new(context_for_test_schema());
    let resource_id = hash_resource_bytes(b"same-resource");
    let left = format!(
        r#"{{"schema":"{SCHEMA}","component_id":"{component}","resources":[{{"resource_id":"{resource_id}","byte_length":12,"properties":{{"approved":true,"tags":["z","a"],"sequence":["second","first"]}}}}],"metadata":{{}}}}"#
    );
    let right = format!(
        r#"{{"metadata":{{}},"resources":[{{"properties":{{"sequence":["second","first"],"tags":["a","z"],"approved":true}},"byte_length":12,"resource_id":"{resource_id}"}}],"component_id":"{component}","schema":"{SCHEMA}"}}"#
    );
    let left: ComponentStateCandidate =
        serde_json::from_str(&left).expect("valid property-bearing state");
    let right: ComponentStateCandidate =
        serde_json::from_str(&right).expect("permuted property-bearing state");
    let left = admit(left, &schema, &[&authority]).expect("left state admitted");
    let right = admit(right, &schema, &[&authority]).expect("right state admitted");
    assert_eq!(left.canonical_body(), right.canonical_body());
    assert_eq!(left.component_state_id(), right.component_state_id());
}

#[test]
fn validated_resource_reference_properties_are_immutable_and_context_bound() {
    let schema = TestSchema::new(SCHEMA);
    let authority = TestResourceValidator::new(context_for_test_schema());
    let state = admit(
        candidate(
            SCHEMA,
            component_id(),
            vec![properties_candidate()],
            empty_metadata(),
        ),
        &schema,
        &[&authority],
    )
    .expect("admitted state");
    let reference: &ResourceReference = &state.resources()[0];
    assert!(
        reference
            .historical_value(Some(&context_for_test_schema()))
            .is_ok()
    );
    let wrong_context = ResourceValidationContext {
        containing_schema: SCHEMA.to_owned(),
        adapter: Some(("test.adapter".to_owned(), "test.adapter-state/5".to_owned())),
    };
    assert_eq!(
        reference.historical_value(Some(&wrong_context)),
        Err(ResourceAdmissionError::ContextMismatch)
    );
}
