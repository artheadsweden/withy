use std::collections::BTreeMap;
use std::error::Error;

use omvcs_model::canonical::{MetadataSchema, canonicalize_metadata_body};
use omvcs_model::hashing::{hash_component_state_metadata, hash_resource_bytes};
use omvcs_model::resource::{
    MAX_RESOURCE_BYTE_LENGTH, ResourceByteLength, ResourceObject, ResourceReference,
};
use serde_json::Value;

const RESOURCE_ID: &str = "omvcs:resource:sha256:\
    ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

fn reference_json(byte_length: &str) -> String {
    format!(r#"{{"resource_id":"{RESOURCE_ID}","byte_length":{byte_length}}}"#)
}

fn resource_reference_schema() -> MetadataSchema {
    MetadataSchema::structure([
        ("resource_id", MetadataSchema::Scalar),
        ("byte_length", MetadataSchema::Scalar),
        ("role", MetadataSchema::Scalar),
        ("media_type", MetadataSchema::Scalar),
        ("properties", MetadataSchema::map(MetadataSchema::Scalar)),
    ])
}

fn reference_identity(
    reference: &ResourceReference,
) -> Result<omvcs_model::ComponentStateId, Box<dyn Error>> {
    let body = serde_json::to_vec(reference)?;
    let canonical = canonicalize_metadata_body(&body, &resource_reference_schema())?;
    Ok(hash_component_state_metadata(&canonical))
}

#[test]
fn resource_object_identity_is_derived_from_complete_immutable_bytes() {
    let first = ResourceObject::new(b"complete Resource bytes".to_vec());
    let second = ResourceObject::new(b"complete Resource bytes".to_vec());
    let changed = ResourceObject::new(b"changed Resource bytes".to_vec());

    assert_eq!(first.bytes(), b"complete Resource bytes");
    assert_eq!(
        first.resource_id(),
        hash_resource_bytes(b"complete Resource bytes")
    );
    assert_eq!(first.resource_id(), second.resource_id());
    assert_ne!(first.resource_id(), changed.resource_id());
}

#[test]
fn resource_byte_length_accepts_exact_safe_integer_domain_and_integer_values() {
    for (json_number, expected) in [
        ("0", 0),
        ("1", 1),
        ("9007199254740991", MAX_RESOURCE_BYTE_LENGTH),
        ("1.0", 1),
        ("1e0", 1),
        ("100e-2", 1),
        ("90071992547409910e-1", MAX_RESOURCE_BYTE_LENGTH),
    ] {
        let parsed = serde_json::from_str::<ResourceByteLength>(json_number)
            .ok()
            .map(ResourceByteLength::get);
        assert_eq!(parsed, Some(expected), "{json_number}");
    }
    assert_eq!(ResourceByteLength::MAX.get(), MAX_RESOURCE_BYTE_LENGTH);
    assert_eq!(
        ResourceByteLength::new(MAX_RESOURCE_BYTE_LENGTH + 1),
        Err(omvcs_model::resource::ResourceByteLengthError::OutOfRange)
    );
}

#[test]
fn resource_byte_length_rejects_normative_invalid_vectors_and_rounded_fractions() {
    for json_number in [
        "-1",
        "1.5",
        "9007199254740992",
        r#""1""#,
        "1e-4000",
        "9007199254740990.9999999999",
        "9007199254740992.0",
    ] {
        assert!(
            serde_json::from_str::<ResourceByteLength>(json_number).is_err(),
            "{json_number}"
        );
    }
}

#[test]
fn resource_reference_requires_typed_id_and_complete_resource_length() {
    let object = ResourceObject::new(b"abc".to_vec());
    let reference = object.reference();

    assert!(reference.is_ok());
    let Ok(reference) = reference else {
        return;
    };
    assert_eq!(reference.resource_id(), object.resource_id());
    assert_eq!(reference.byte_length().get(), 3);
    assert!(reference.matches_resource(&object));
    assert_eq!(
        serde_json::from_str::<ResourceReference>(&reference_json("3"))
            .ok()
            .map(|value| (value.resource_id(), value.byte_length().get())),
        Some((hash_resource_bytes(b"abc"), 3))
    );

    for invalid in [
        "{}".to_owned(),
        r#"{"byte_length":3}"#.to_owned(),
        format!(r#"{{"resource_id":"{RESOURCE_ID}"}}"#),
        r#"{"resource_id":"not-a-resource-id","byte_length":3}"#.to_owned(),
    ] {
        assert!(
            serde_json::from_str::<ResourceReference>(&invalid).is_err(),
            "{invalid}"
        );
    }
}

#[test]
fn resource_reference_contains_only_approved_generic_fields() {
    let forbidden_fields = [
        "logical_name",
        "friendly_name",
        "filename",
        "path",
        "url",
        "chunk",
        "chunk_id",
        "chunk_manifest",
        "chunks",
        "storage",
        "storage_endpoint",
        "storage_location",
        "location",
        "replica",
        "provider",
        "credentials",
        "resource_manifest_id",
    ];

    for field in forbidden_fields {
        let json =
            format!(r#"{{"resource_id":"{RESOURCE_ID}","byte_length":0,"{field}":"excluded"}}"#);
        assert!(
            serde_json::from_str::<ResourceReference>(&json).is_err(),
            "{field}"
        );
    }

    let minimal = serde_json::from_str::<ResourceReference>(&reference_json("0"));
    assert!(minimal.is_ok());
    if let Ok(minimal) = minimal {
        let serialized = serde_json::to_value(minimal);
        assert_eq!(
            serialized.ok(),
            Some(serde_json::json!({
                "resource_id": RESOURCE_ID,
                "byte_length": 0
            }))
        );
    }
}

#[test]
fn resource_reference_deserialization_rejects_duplicate_member_names_recursively() {
    let duplicate_members = [
        format!(
            r#"{{"resource_id":"{RESOURCE_ID}","byte_length":0,"properties":{{"sample_rate":44100,"sample_rate":48000}}}}"#
        ),
        format!(
            r#"{{"resource_id":"{RESOURCE_ID}","byte_length":0,"properties":{{"sample_rate":44100,"\u0073ample_rate":48000}}}}"#
        ),
        format!(
            r#"{{"resource_id":"{RESOURCE_ID}","byte_length":0,"properties":{{"nested":{{"value":1,"value":2}}}}}}"#
        ),
    ];

    for json in duplicate_members {
        assert!(
            serde_json::from_str::<ResourceReference>(&json).is_err(),
            "{json}"
        );
    }
}

#[test]
fn descriptive_reference_fields_change_containing_identity_not_resource_identity()
-> Result<(), Box<dyn Error>> {
    let object = ResourceObject::new(b"same complete bytes".to_vec());
    let base = object.reference()?;
    let with_role = base.clone().with_role("primary-audio");
    let with_media_type = base.clone().with_media_type("audio/wav");
    let properties = BTreeMap::from([("sample_rate".to_owned(), Value::from(48_000))]);
    let with_properties = base.clone().with_properties(properties);

    assert_eq!(
        object.resource_id(),
        hash_resource_bytes(b"same complete bytes")
    );
    assert_eq!(base.resource_id(), with_role.resource_id());
    assert_eq!(base.resource_id(), with_media_type.resource_id());
    assert_eq!(base.resource_id(), with_properties.resource_id());
    let base_identity = reference_identity(&base)?;
    assert_ne!(base_identity, reference_identity(&with_role)?);
    assert_ne!(base_identity, reference_identity(&with_media_type)?);
    assert_ne!(base_identity, reference_identity(&with_properties)?);
    assert_eq!(with_role.role(), Some("primary-audio"));
    assert_eq!(with_media_type.media_type(), Some("audio/wav"));
    assert!(with_properties.properties().is_some());

    let different_length =
        ResourceReference::new(object.resource_id(), ResourceByteLength::new(1)?);
    assert!(!different_length.matches_resource(&object));
    assert_ne!(base_identity, reference_identity(&different_length)?);
    assert_eq!(object.resource_id(), base.resource_id());

    let changed_object = ResourceObject::new(b"different complete bytes".to_vec());
    let changed_resource_reference = changed_object.reference()?;
    assert_ne!(object.resource_id(), changed_object.resource_id());
    assert_ne!(
        base_identity,
        reference_identity(&changed_resource_reference)?
    );
    Ok(())
}

#[test]
fn properties_object_map_insertion_order_does_not_change_canonical_identity()
-> Result<(), Box<dyn Error>> {
    let schema = resource_reference_schema();
    let left = format!(
        r#"{{"resource_id":"{RESOURCE_ID}","byte_length":3,"properties":{{"beta":2,"alpha":1}}}}"#
    );
    let right = format!(
        r#"{{"resource_id":"{RESOURCE_ID}","byte_length":3,"properties":{{"alpha":1,"beta":2}}}}"#
    );
    let left_canonical = canonicalize_metadata_body(left.as_bytes(), &schema)?;
    let right_canonical = canonicalize_metadata_body(right.as_bytes(), &schema)?;

    assert_eq!(left_canonical, right_canonical);
    assert_eq!(
        hash_component_state_metadata(&left_canonical),
        hash_component_state_metadata(&right_canonical)
    );
    Ok(())
}

#[test]
fn resource_reference_properties_reject_duplicate_json_member_names() {
    let body = format!(
        r#"{{"resource_id":"{RESOURCE_ID}","byte_length":3,"properties":{{"sample_rate":44100,"sample_rate":48000}}}}"#
    );

    assert_eq!(
        canonicalize_metadata_body(body.as_bytes(), &resource_reference_schema()),
        Err(omvcs_model::canonical::CanonicalMetadataError::DuplicateMemberName)
    );
}

#[test]
fn external_names_and_physical_storage_variations_are_not_resource_reference_inputs()
-> Result<(), Box<dyn Error>> {
    let object = ResourceObject::new(b"identical bytes".to_vec());
    let reference = object.reference()?;
    let reference_id = reference_identity(&reference)?;
    let physical_contexts = [
        (
            "take.wav",
            "endpoint-a/key-a",
            "provider-a",
            "replica-a",
            "layout-a",
        ),
        (
            "renamed.wav",
            "endpoint-b/key-b",
            "provider-b",
            "replica-b",
            "layout-b",
        ),
    ];

    assert_ne!(physical_contexts[0], physical_contexts[1]);
    assert_eq!(
        object.resource_id(),
        hash_resource_bytes(b"identical bytes")
    );
    assert_eq!(reference_identity(&reference)?, reference_id);
    Ok(())
}
