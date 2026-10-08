use omvcs_model::{
    CreativeComponent, CreativeComponentId, ResourceId, hashing::hash_resource_bytes,
};
use serde_json::{Value, json};

fn component_id() -> CreativeComponentId {
    CreativeComponentId::new()
}

#[test]
fn creative_component_requires_a_typed_assigned_component_id() -> Result<(), serde_json::Error> {
    let id = component_id();
    let component = CreativeComponent::new(id);
    let encoded = serde_json::to_value(&component)?;

    assert_eq!(component.component_id(), id);
    assert_eq!(encoded, json!({"component_id": id.to_string()}));

    for invalid in [
        json!({}),
        json!({"component_id": null}),
        json!({"component_id": "not-an-identifier"}),
    ] {
        assert!(
            serde_json::from_value::<CreativeComponent>(invalid).is_err(),
            "invalid component object was accepted"
        );
    }

    Ok(())
}

#[test]
fn creative_component_decoding_rejects_fields_outside_the_generic_object() {
    let excluded_fields = [
        "project_id",
        "name",
        "kind",
        "created_at",
        "resource",
        "resources",
        "component_state",
        "parents",
        "daw_native_id",
        "platform",
        "storage",
        "location",
    ];
    let id = component_id();

    for field in excluded_fields {
        let mut fields = serde_json::Map::new();
        fields.insert("component_id".to_owned(), json!(id.to_string()));
        fields.insert(field.to_owned(), Value::Null);
        let object = Value::Object(fields);

        assert!(
            serde_json::from_value::<CreativeComponent>(object).is_err(),
            "generic Component decoder accepted {field}"
        );
    }
}

#[test]
fn creative_component_identity_is_unchanged_by_external_context_values()
-> Result<(), serde_json::Error> {
    let assigned_id = component_id();
    let component = CreativeComponent::new(assigned_id);
    let canonical_object = serde_json::to_value(&component)?;
    let resource_before_replacement = hash_resource_bytes(b"original recording");
    let resource_after_rerecording = hash_resource_bytes(b"new recording");
    assert_ne!(resource_before_replacement, resource_after_rerecording);
    let external_contexts = [
        json!({
            "resource": resource_before_replacement.to_string(),
            "parentage": "first parent",
            "project_membership": "first membership",
            "daw_native_id": "first native ID",
            "presentation": "Bass",
            "timestamp": "first timestamp",
            "filename": "bass.wav",
            "storage": "first storage",
            "platform_account": "first account",
            "location": "first location"
        }),
        json!({
            "resource": resource_after_rerecording.to_string(),
            "parentage": "different parent",
            "project_membership": "different membership",
            "daw_native_id": "different native ID",
            "presentation": "Electric Bass",
            "timestamp": "different timestamp",
            "filename": "bass-take-2.wav",
            "storage": "different storage",
            "platform_account": "different account",
            "location": "different location"
        }),
    ];

    for external_context in external_contexts {
        // This synthetic context remains outside the generic Component object;
        // it does not implement any of the referenced object models.
        let envelope = json!({
            "external_context": external_context,
            "creative_component": canonical_object
        });
        let component_value = envelope["creative_component"].clone();
        let decoded: CreativeComponent = serde_json::from_value(component_value)?;

        assert_eq!(decoded.component_id(), assigned_id);
        assert_eq!(serde_json::to_value(&decoded)?, canonical_object);
    }

    Ok(())
}

#[test]
fn presentation_rename_does_not_create_a_new_component_identifier() -> Result<(), serde_json::Error>
{
    let component = CreativeComponent::new(component_id());
    let component_id_before_rename = component.component_id();
    let presentation_name_before = "Bass";
    let presentation_name_after = "Electric Bass";

    assert_ne!(presentation_name_before, presentation_name_after);
    assert_eq!(component.component_id(), component_id_before_rename);
    assert_eq!(
        serde_json::to_value(&component)?,
        json!({"component_id": component_id_before_rename.to_string()})
    );

    Ok(())
}

#[test]
fn creative_component_and_resource_identifiers_are_distinct_types() {
    fn accepts_component_id(_: CreativeComponentId) {}
    fn accepts_resource_id(_: ResourceId) {}

    let component = CreativeComponent::new(component_id());
    let resource_id = hash_resource_bytes(b"resource bytes");
    accepts_component_id(component.component_id());
    accepts_resource_id(resource_id);

    assert!(
        component
            .component_id()
            .to_string()
            .parse::<ResourceId>()
            .is_err()
    );
    assert!(
        resource_id
            .to_string()
            .parse::<CreativeComponentId>()
            .is_err()
    );
}
