#![allow(clippy::expect_used)]

use std::collections::BTreeMap;
use std::error::Error;

use omvcs_model::canonical::{ArrayOrdering, MetadataSchema};
use omvcs_model::component_state::{
    ComponentStateAdmissionError, ComponentStateCandidate, ComponentStateSchemaValidator,
};
use omvcs_model::resource::{
    ResourcePropertiesValidator, ResourceReferenceCandidate, ResourceValidationContext,
};
use omvcs_model::{ComponentState, ComponentStateId};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const SCHEMA: &str = "test.verifier-state/1";
const COMPONENT: &str = "019cc17d-1b22-7a41-9fe9-c345c468f82c";
const RESOURCE: &str = "omvcs:resource:sha256:\
    ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

// Test-only schema authorities. These are NOT a Core metadata vocabulary.
struct Authority {
    shape: MetadataSchema,
    context: ResourceValidationContext,
}

impl Authority {
    fn scalar_map() -> Self {
        Self {
            shape: MetadataSchema::map(MetadataSchema::Scalar),
            context: ResourceValidationContext {
                containing_schema: SCHEMA.to_owned(),
                adapter: None,
            },
        }
    }
}

impl ComponentStateSchemaValidator for Authority {
    fn schema(&self) -> &str {
        SCHEMA
    }

    fn metadata_schema(&self) -> MetadataSchema {
        self.shape.clone()
    }

    fn validate_metadata(&self, _: &BTreeMap<String, Value>) -> Result<(), String> {
        Ok(())
    }

    fn resource_validation_context(
        &self,
        _: &ResourceReferenceCandidate,
    ) -> Option<ResourceValidationContext> {
        Some(self.context.clone())
    }
}

impl ResourcePropertiesValidator for Authority {
    fn context(&self) -> &ResourceValidationContext {
        &self.context
    }

    fn properties_schema(&self) -> MetadataSchema {
        MetadataSchema::structure([(
            "values",
            MetadataSchema::array(ArrayOrdering::SetLike, MetadataSchema::Scalar),
        )])
    }

    fn validate_properties(&self, _: &BTreeMap<String, Value>) -> Result<(), String> {
        Ok(())
    }
}

fn body(resources: Value, metadata: Value) -> Value {
    let mut value = json!({
        "schema": SCHEMA,
        "component_id": COMPONENT
    });
    value["resources"] = resources;
    value["metadata"] = metadata;
    value
}

fn admit(value: Value, authority: &Authority) -> Result<ComponentState, Box<dyn Error>> {
    let candidate: ComponentStateCandidate = serde_json::from_value(value)?;
    Ok(candidate.admit(&[authority], &[authority])?)
}

// Core §§7, 10; ADR-0007/0011: Resource References are named-field JSON
// objects, not serde positional structs. Report whether invalid input reached
// actual historical admission, not merely whether decoding succeeded.
#[test]
fn embedded_resource_references_reject_positional_arrays_before_history() {
    reject_positional_reference(&json!([RESOURCE, 3]));
}

#[test]
fn property_bearing_resource_references_reject_positional_arrays_before_history() {
    reject_positional_reference(&json!([RESOURCE, 3, "primary", "audio/wav", {}]));
}

fn reject_positional_reference(reference: &Value) {
    let authority = Authority::scalar_map();
    let input = body(json!([reference]), json!({}));
    let text = serde_json::to_string(&input).expect("test JSON");
    let text_rejected = serde_json::from_str::<ComponentStateCandidate>(&text).is_err();
    let value_rejected = serde_json::from_value::<ComponentStateCandidate>(input.clone()).is_err();
    let admitted = admit(input, &authority).is_ok();
    assert!(
        text_rejected && value_rejected,
        "positional Resource Reference: text_rejected={text_rejected}, value_rejected={value_rejected}, historical_admission={admitted}; input={text}"
    );
}

// Core §§7, 10 and ADR-0008: raw mathematical byte counts must not be
// rounded into valid history, including when embedded in a Component State.
#[test]
fn embedded_byte_lengths_reject_lossy_numbers_and_alternate_types() {
    for length in [
        "-1",
        "1.5",
        "9007199254740992",
        "9007199254740991.0000000001",
        "9007199254740990.9999999999",
        "1e-4000",
        "18446744073709551615",
        "\"1\"",
        "true",
        "null",
        "[1]",
        "{\"integer\":1}",
    ] {
        let raw = format!(
            r#"{{"schema":"{SCHEMA}","component_id":"{COMPONENT}","resources":[{{"resource_id":"{RESOURCE}","byte_length":{length}}}],"metadata":{{}}}}"#
        );
        assert!(
            serde_json::from_str::<ComponentStateCandidate>(&raw).is_err(),
            "invalid embedded byte_length {length} accepted"
        );
    }
}

#[test]
fn embedded_byte_lengths_accept_exact_integral_number_forms() -> Result<(), Box<dyn Error>> {
    let authority = Authority::scalar_map();
    for (length, expected) in [
        ("0", 0),
        ("1", 1),
        ("9007199254740991", 9_007_199_254_740_991),
        ("1.0", 1),
        ("100e-2", 1),
        ("90071992547409910e-1", 9_007_199_254_740_991),
    ] {
        let raw = format!(
            r#"{{"schema":"{SCHEMA}","component_id":"{COMPONENT}","resources":[{{"resource_id":"{RESOURCE}","byte_length":{length}}}],"metadata":{{}}}}"#
        );
        let candidate: ComponentStateCandidate = serde_json::from_str(&raw)?;
        let state = candidate.admit(&[&authority], &[])?;
        assert_eq!(state.resources()[0].byte_length().get(), expected);
    }
    Ok(())
}

// Core §5.1/10, ADR-0001: duplicate equality is canonical bytes, including
// recursively normalized properties, not raw JSON or Resource IDs alone.
#[test]
fn resource_sets_reject_equivalent_canonical_elements() -> Result<(), Box<dyn Error>> {
    let authority = Authority::scalar_map();
    let resources = json!([
        {"resource_id": RESOURCE, "byte_length": 3,
         "properties": {"values": [2, 1]}},
        {"properties": {"values": [1.0, 2.0]},
         "byte_length": 3, "resource_id": RESOURCE}
    ]);
    let candidate: ComponentStateCandidate = serde_json::from_value(body(resources, json!({})))?;
    assert_eq!(
        candidate.admit(&[&authority], &[&authority]),
        Err(ComponentStateAdmissionError::DuplicateResource)
    );
    // Same Resource ID but different canonical reference fields are not duplicates.
    assert!(
        admit(
            body(
                json!([
                    {"resource_id": RESOURCE, "byte_length": 3, "role": "a"},
                    {"resource_id": RESOURCE, "byte_length": 3, "role": "b"}
                ]),
                json!({})
            ),
            &authority
        )
        .is_ok()
    );
    Ok(())
}

// ADR-0005: escaped aliases at every raw object depth fail before overwrite.
#[test]
fn raw_duplicate_names_fail_at_envelope_reference_and_property_depths() {
    for (prefix, duplicate) in [
        (
            "",
            r#""schema":"test.verifier-state/1","\u0073chema":"other""#,
        ),
        ("resource", r#""byte_length":3,"\u0062yte_length":4"#),
        ("properties", r#""values":[],"\u0076alues":[] "#),
    ] {
        let raw = match prefix {
            "" => format!(
                r#"{{{duplicate},"component_id":"{COMPONENT}","resources":[],"metadata":{{}}}}"#
            ),
            "resource" => format!(
                r#"{{"schema":"{SCHEMA}","component_id":"{COMPONENT}","resources":[{{"resource_id":"{RESOURCE}",{duplicate}}}],"metadata":{{}}}}"#
            ),
            _ => format!(
                r#"{{"schema":"{SCHEMA}","component_id":"{COMPONENT}","resources":[{{"resource_id":"{RESOURCE}","byte_length":3,"properties":{{{duplicate}}}}}],"metadata":{{}}}}"#
            ),
        };
        assert!(serde_json::from_str::<ComponentStateCandidate>(&raw).is_err());
    }
}

// ADR-0011: assert bytes independently, then SHA-256 directly rather than
// using the model's hashing helper as both implementation and oracle.
#[test]
fn exact_body_preimage_uses_jcs_utf16_map_order_and_no_hash_prefix() -> Result<(), Box<dyn Error>> {
    let authority = Authority::scalar_map();
    let state = admit(
        body(json!([]), json!({"\u{e000}": "a", "\u{10000}": "z"})),
        &authority,
    )?;
    let expected = format!(
        "{{\"component_id\":\"{COMPONENT}\",\"metadata\":{{\"\u{10000}\":\"z\",\"\u{e000}\":\"a\"}},\"resources\":[],\"schema\":\"{SCHEMA}\"}}"
    );
    assert_eq!(state.canonical_body(), expected.as_bytes());
    assert_eq!(
        state.component_state_id().digest().as_slice(),
        Sha256::digest(expected.as_bytes()).as_slice()
    );
    Ok(())
}

// Exhaust all 24 parent permutations x 24 resource permutations. This bounded
// property checks both collections together and asserts canonical-byte order
// (reference role order deliberately disagrees with resource_id order).
#[test]
fn all_parent_and_resource_permutations_preserve_identity_and_known_lineage()
-> Result<(), Box<dyn Error>> {
    let authority = Authority::scalar_map();
    let parents: Vec<_> = (1..=4)
        .map(|byte| ComponentStateId::from_digest([byte; 32]).to_string())
        .collect();
    let resources: Vec<_> = (1..=4)
        .map(|byte| {
            json!({
                "resource_id": format!("omvcs:resource:sha256:{}", format!("{byte:02x}").repeat(32)),
                "byte_length": 3,
                "role": format!("{}", 5 - byte)
            })
        })
        .collect();
    let orders = permutations(&[0, 1, 2, 3]);
    let mut canonical = None;
    for parent_order in &orders {
        for resource_order in &orders {
            let mut input = body(
                json!(
                    resource_order
                        .iter()
                        .map(|&i| &resources[i])
                        .collect::<Vec<_>>()
                ),
                json!({}),
            );
            input["parents"] = json!(
                parent_order
                    .iter()
                    .map(|&i| &parents[i])
                    .collect::<Vec<_>>()
            );
            let state = admit(input, &authority)?;
            let signature = (state.canonical_body().to_vec(), state.component_state_id());
            if let Some(expected) = &canonical {
                assert_eq!(&signature, expected);
            } else {
                canonical = Some(signature);
            }
            let known: Vec<_> = state
                .parents()
                .expect("known lineage stays present")
                .iter()
                .map(ToString::to_string)
                .collect();
            assert_eq!(known, parents);
            let normalized: Value = serde_json::from_slice(state.canonical_body())?;
            let element_bytes: Vec<_> = normalized["resources"]
                .as_array()
                .expect("resource array")
                .iter()
                .map(|v| serde_jcs::to_vec(v).expect("canonical reference"))
                .collect();
            assert!(element_bytes.windows(2).all(|pair| pair[0] < pair[1]));
        }
    }
    Ok(())
}

fn permutations(items: &[usize]) -> Vec<Vec<usize>> {
    if items.is_empty() {
        return vec![vec![]];
    }
    let mut result = Vec::new();
    for (position, &first) in items.iter().enumerate() {
        let mut rest = items.to_vec();
        rest.remove(position);
        for mut suffix in permutations(&rest) {
            suffix.insert(0, first);
            result.push(suffix);
        }
    }
    result
}

// Core §10: presence must never disappear through null/shape coercion.
#[test]
fn present_component_state_fields_reject_null_and_wrong_shapes() {
    for (field, invalid) in [
        ("schema", json!(null)),
        ("schema", json!(1)),
        ("component_id", json!(null)),
        ("resources", json!(null)),
        ("resources", json!({})),
        ("metadata", json!(null)),
        ("metadata", json!([])),
        ("parents", json!(null)),
        ("parents", json!({})),
        ("parents", json!([RESOURCE])),
    ] {
        let mut input = body(json!([]), json!({}));
        input[field] = invalid;
        assert!(serde_json::from_value::<ComponentStateCandidate>(input).is_err());
    }
}

// Core §§10, 76–77: availability of an older schema must not permit guessed
// interpretation of an unknown version.
#[test]
fn unknown_schema_does_not_fall_back_to_an_available_authority() -> Result<(), Box<dyn Error>> {
    let authority = Authority::scalar_map();
    let mut input = body(json!([]), json!({}));
    input["schema"] = json!("test.verifier-state/2");
    let candidate: ComponentStateCandidate = serde_json::from_value(input)?;
    assert_eq!(
        candidate.admit(&[&authority], &[&authority]),
        Err(ComponentStateAdmissionError::UnavailableSchema)
    );
    Ok(())
}

// ADR-0011/0009: a matching property validator for the wrong containing schema
// still cannot establish validity in this Component State.
#[test]
fn resource_binding_cannot_name_a_different_containing_schema() -> Result<(), Box<dyn Error>> {
    let mut authority = Authority::scalar_map();
    authority.context.containing_schema = "test.verifier-state/2".to_owned();
    let candidate: ComponentStateCandidate = serde_json::from_value(body(
        json!([{"resource_id": RESOURCE, "byte_length": 3, "properties": {}}]),
        json!({}),
    ))?;
    assert_eq!(
        candidate.admit(&[&authority], &[&authority]),
        Err(ComponentStateAdmissionError::InvalidResourceBinding)
    );
    Ok(())
}

// Core §5.1/10: incomplete collection declarations are not excused by
// absent values, and the metadata root is always an object map.
#[test]
fn invalid_metadata_schema_definitions_cannot_admit_empty_maps() -> Result<(), Box<dyn Error>> {
    use omvcs_model::canonical::CanonicalMetadataError;

    let mut authority = Authority::scalar_map();
    authority.shape = MetadataSchema::structure([(
        "absent",
        MetadataSchema::map(MetadataSchema::unclassified_array(MetadataSchema::Scalar)),
    )]);
    let candidate: ComponentStateCandidate = serde_json::from_value(body(json!([]), json!({})))?;
    assert!(matches!(
        candidate.admit(&[&authority], &[]),
        Err(ComponentStateAdmissionError::Canonical(
            CanonicalMetadataError::UnclassifiedArray { .. }
        ))
    ));
    authority.shape = MetadataSchema::Scalar;
    let candidate: ComponentStateCandidate = serde_json::from_value(body(json!([]), json!({})))?;
    assert_eq!(
        candidate.admit(&[&authority], &[]),
        Err(ComponentStateAdmissionError::InvalidSchemaDefinition)
    );
    Ok(())
}
