#![allow(clippy::expect_used)]

use std::collections::BTreeMap;
use std::error::Error;

use omvcs_model::canonical::MetadataSchema;
use omvcs_model::component_state::{ComponentStateCandidate, ComponentStateSchemaValidator};
use omvcs_model::resource::{
    ResourcePropertiesValidator, ResourceReferenceCandidate, ResourceValidationContext,
};
use serde::Deserialize;
use serde_json::{Value, json};

const SCHEMA: &str = "test.verifier-remediation/1";
const COMPONENT: &str = "019cc17d-1b22-7a41-9fe9-c345c468f82c";
const RESOURCE: &str = "omvcs:resource:sha256:\
    ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

// Synthetic authority, not a concrete metadata vocabulary in Core.
struct Authority(ResourceValidationContext);

impl Authority {
    fn new() -> Self {
        Self(ResourceValidationContext {
            containing_schema: SCHEMA.to_owned(),
            adapter: None,
        })
    }
}

impl ComponentStateSchemaValidator for Authority {
    fn schema(&self) -> &str {
        SCHEMA
    }
    fn metadata_schema(&self) -> MetadataSchema {
        MetadataSchema::map(MetadataSchema::Scalar)
    }
    fn validate_metadata(&self, _: &BTreeMap<String, Value>) -> Result<(), String> {
        Ok(())
    }
    fn resource_validation_context(
        &self,
        _: &ResourceReferenceCandidate,
    ) -> Option<ResourceValidationContext> {
        Some(self.0.clone())
    }
}

impl ResourcePropertiesValidator for Authority {
    fn context(&self) -> &ResourceValidationContext {
        &self.0
    }
    fn properties_schema(&self) -> MetadataSchema {
        MetadataSchema::map(MetadataSchema::Scalar)
    }
    fn validate_properties(&self, _: &BTreeMap<String, Value>) -> Result<(), String> {
        Ok(())
    }
}

fn containing_body(reference: &Value) -> Value {
    json!({
        "schema": SCHEMA, "component_id": COMPONENT,
        "resources": [reference], "metadata": {}
    })
}

// Core §§7/10, ADR-0007/0011: rejected shapes must fail every JSON ingress,
// directly and embedded. If a direct decoder accepts, explicitly try inserting
// its result via the public constructor to detect historical admission too.
fn reject_at_all_boundaries(reference: &Value) {
    let authority = Authority::new();
    let text = reference.to_string();
    for (path, result) in [
        (
            "text",
            serde_json::from_str::<ResourceReferenceCandidate>(&text),
        ),
        ("from_value", serde_json::from_value(reference.clone())),
        (
            "owned Deserialize",
            ResourceReferenceCandidate::deserialize(reference.clone()),
        ),
        (
            "borrowed Deserialize",
            ResourceReferenceCandidate::deserialize(reference),
        ),
    ] {
        let admitted = result.as_ref().is_ok_and(|candidate| {
            ComponentStateCandidate::new(
                SCHEMA,
                COMPONENT.parse().expect("fixture Component ID"),
                vec![candidate.clone()],
                BTreeMap::new(),
            )
            .admit(&[&authority], &[&authority])
            .is_ok()
        });
        assert!(
            result.is_err(),
            "{path} accepted Resource Reference {text}; historical admission={admitted}"
        );
    }
    let body = containing_body(reference);
    let text = body.to_string();
    for (path, result) in [
        (
            "text",
            serde_json::from_str::<ComponentStateCandidate>(&text),
        ),
        ("from_value", serde_json::from_value(body.clone())),
        (
            "owned Deserialize",
            ComponentStateCandidate::deserialize(body.clone()),
        ),
        (
            "borrowed Deserialize",
            ComponentStateCandidate::deserialize(&body),
        ),
    ] {
        let rejected = result.is_err();
        let admitted =
            result.is_ok_and(|candidate| candidate.admit(&[&authority], &[&authority]).is_ok());
        assert!(
            rejected,
            "{path} accepted embedded Resource Reference {reference}; historical admission={admitted}"
        );
    }
}

#[test]
fn positional_prefixes_partial_optionals_and_null_holes_cannot_enter_history() {
    let full = [
        json!(RESOURCE),
        json!(3),
        json!("primary"),
        json!("audio/wav"),
        json!({}),
        json!("extra"),
        json!(null),
        json!(true),
    ];
    for length in 0..=full.len() {
        reject_at_all_boundaries(&Value::Array(full[..length].to_vec()));
    }
    let optional = [json!("primary"), json!("audio/wav"), json!({})];
    // All eight partial/full optional subsequences; no positional alternative
    // can stand in for named optional members.
    for mask in 0..8 {
        let mut fields = vec![json!(RESOURCE), json!(3)];
        for (index, value) in optional.iter().enumerate() {
            if mask & (1 << index) != 0 {
                fields.push(value.clone());
            }
        }
        reject_at_all_boundaries(&Value::Array(fields));
        let mut holes = full[..5].to_vec();
        for index in 0..3 {
            if mask & (1 << index) == 0 {
                holes[index + 2] = Value::Null;
            }
        }
        reject_at_all_boundaries(&Value::Array(holes));
    }
}

#[test]
fn wrappers_tagged_enum_forms_and_nonobjects_cannot_enter_history() {
    let object = json!({"resource_id": RESOURCE, "byte_length": 3});
    let sequence = json!([RESOURCE, 3, "primary", "audio/wav", {}]);
    for payload in [object.clone(), sequence.clone()] {
        for wrapper in ["body", "reference", "ResourceReference", "Some", "Ok"] {
            let mut value = json!({});
            value[wrapper] = payload.clone();
            reject_at_all_boundaries(&value);
        }
        reject_at_all_boundaries(&json!({"type": "ResourceReference", "value": payload}));
        reject_at_all_boundaries(&json!({"ResourceReferenceCandidate": payload}));
        reject_at_all_boundaries(&json!(["ResourceReference", payload]));
        reject_at_all_boundaries(&json!([payload]));
    }
    for value in [
        Value::Null,
        json!(false),
        json!(3),
        json!("ResourceReference"),
        json!(RESOURCE),
    ] {
        reject_at_all_boundaries(&value);
    }
    let mut disguised = object;
    disguised["body"] = sequence;
    reject_at_all_boundaries(&disguised);
}

// Positive control: no blanket rejection of borrowed/owned values or valid
// optional-member combinations. Candidate output preserves exact present fields;
// all four Component State paths admit identical canonical bodies and IDs.
#[test]
fn named_objects_with_every_optional_combination_admit_identically_across_ingress()
-> Result<(), Box<dyn Error>> {
    let authority = Authority::new();
    let optional = [
        ("role", json!("primary")),
        ("media_type", json!("audio/wav")),
        ("properties", json!({})),
    ];
    for mask in 0..8 {
        let mut reference = json!({"resource_id": RESOURCE, "byte_length": 3});
        for (index, (name, value)) in optional.iter().enumerate() {
            if mask & (1 << index) != 0 {
                reference[*name] = value.clone();
            }
        }
        let text = reference.to_string();
        for candidate in [
            serde_json::from_str::<ResourceReferenceCandidate>(&text)?,
            serde_json::from_value(reference.clone())?,
            ResourceReferenceCandidate::deserialize(reference.clone())?,
            ResourceReferenceCandidate::deserialize(&reference)?,
        ] {
            assert_eq!(serde_json::to_value(&candidate)?, reference);
            let state = ComponentStateCandidate::new(
                SCHEMA,
                COMPONENT.parse()?,
                vec![candidate],
                BTreeMap::new(),
            )
            .admit(&[&authority], &[&authority])?;
            assert_eq!(
                serde_json::from_slice::<Value>(state.canonical_body())?,
                containing_body(&reference)
            );
        }
        let body = containing_body(&reference);
        let text = body.to_string();
        let mut expected = None;
        for candidate in [
            serde_json::from_str::<ComponentStateCandidate>(&text)?,
            serde_json::from_value(body.clone())?,
            ComponentStateCandidate::deserialize(body.clone())?,
            ComponentStateCandidate::deserialize(&body)?,
        ] {
            let state = candidate.admit(&[&authority], &[&authority])?;
            let actual = (state.canonical_body().to_vec(), state.component_state_id());
            if let Some(expected) = &expected {
                assert_eq!(&actual, expected);
            } else {
                expected = Some(actual);
            }
        }
    }
    Ok(())
}

// Map-only remediation must retain raw duplicate detection and not accept
// multiple JSON documents. Do not preparse raw text into an overwriting map.
#[test]
fn raw_duplicates_and_trailing_documents_still_fail_before_admission() {
    let object = format!(r#"{{"resource_id":"{RESOURCE}","byte_length":3}}"#);
    for raw in [
        format!(r#"{{"resource_id":"{RESOURCE}","byte_length":3,"\u0062yte_length":4}}"#),
        format!(
            r#"{{"resource_id":"{RESOURCE}","byte_length":3,"properties":{{"x":1,"\u0078":2}}}}"#
        ),
        format!("{object} {object}"),
        format!("{object} []"),
    ] {
        assert!(serde_json::from_str::<ResourceReferenceCandidate>(&raw).is_err());
        let body = format!(
            r#"{{"schema":"{SCHEMA}","component_id":"{COMPONENT}","resources":[{raw}],"metadata":{{}}}}"#
        );
        assert!(serde_json::from_str::<ComponentStateCandidate>(&body).is_err());
    }
}
