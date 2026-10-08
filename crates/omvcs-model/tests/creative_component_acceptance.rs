use omvcs_model::{CreativeComponent, CreativeComponentId};
use serde_json::json;

// Core §9 and ADR-0010 require an object with the assigned component_id field,
// not a positional sequence containing an identifier.
#[test]
fn creative_component_requires_a_json_object_with_a_named_component_id() {
    let id = "019cc17d-1b22-7a41-9fe9-c345c468f82c";
    let positional = format!(r#"["{id}"]"#);

    assert!(
        serde_json::from_str::<CreativeComponent>(&positional).is_err(),
        "a positional array has no required component_id field"
    );
}

// Core §§4, 9: exercise the model's actual decoding boundary rather than
// testing only the assigned-identifier parser used by WORK-0001.
#[test]
fn creative_component_decode_rejects_invalid_assigned_identity_encodings() {
    for invalid in [
        json!(null),
        json!(true),
        json!(7),
        json!([]),
        json!({}),
        json!(""),
        json!("019CC17D-1B22-7A41-9FE9-C345C468F82C"),
        json!("019cc17d1b227a419fe9c345c468f82c"),
        json!("019cc17d-1b22-4a41-9fe9-c345c468f82c"),
        json!("019cc17d-1b22-7a41-0fe9-c345c468f82c"),
        json!(
            "omvcs:resource:sha256:0000000000000000000000000000000000000000000000000000000000000000"
        ),
    ] {
        let object = json!({"component_id": invalid});
        assert!(
            serde_json::from_value::<CreativeComponent>(object.clone()).is_err(),
            "invalid assigned identity was accepted: {object}"
        );
    }
}

// The one-field boundary must survive actual JSON text encode/decode and
// repeated decoding without generating a fresh assigned identifier.
#[test]
fn creative_component_json_roundtrip_preserves_exactly_the_assigned_identity()
-> Result<(), serde_json::Error> {
    for _ in 0..256 {
        let id = CreativeComponentId::new();
        let component = CreativeComponent::new(id);
        let encoded = serde_json::to_string(&component)?;
        assert_eq!(encoded, format!(r#"{{"component_id":"{id}"}}"#));

        let decoded: CreativeComponent = serde_json::from_str(&encoded)?;
        assert_eq!(decoded, component);
        assert_eq!(decoded.component_id(), id);
        assert_eq!(serde_json::to_string(&decoded)?, encoded);
    }
    Ok(())
}
