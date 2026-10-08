use std::collections::BTreeMap;
use std::error::Error;

use omvcs_model::canonical::{
    ArrayOrdering, CanonicalMetadataError, MetadataSchema, canonicalize_metadata_body,
};
use omvcs_model::hashing::{hash_component_state_metadata, hash_resource_bytes};
use omvcs_model::resource::{
    MAX_RESOURCE_BYTE_LENGTH, ResourceAdmissionError, ResourceByteLength, ResourceObject,
    ResourcePropertiesValidator, ResourceReference, ResourceReferenceCandidate,
    ResourceValidationContext,
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
    // A synthetic versioned containing body, not a separate Resource Reference ID
    // or an implementation of the future Component State model.
    let body = serde_json::to_vec(&serde_json::json!({
        "schema": context().containing_schema,
        "reference": reference.historical_value(Some(&context()))?,
    }))?;
    let canonical = canonicalize_metadata_body(
        &body,
        &MetadataSchema::structure([
            ("schema", MetadataSchema::Scalar),
            ("reference", resource_reference_schema()),
        ]),
    )?;
    Ok(hash_component_state_metadata(&canonical))
}

fn context() -> ResourceValidationContext {
    ResourceValidationContext {
        containing_schema: "test.containing-state/0.1".to_owned(),
        adapter: Some((
            "test.authority".to_owned(),
            "test.authority.state/0.1".to_owned(),
        )),
    }
}

// Synthetic authority only: no real Adapter vocabulary or semantics.
struct Authority {
    context: ResourceValidationContext,
    schema: MetadataSchema,
    rejection: Option<String>,
    calls: std::cell::Cell<usize>,
    seen: std::cell::RefCell<Option<BTreeMap<String, Value>>>,
}

impl Authority {
    fn new(schema: MetadataSchema) -> Self {
        Self {
            context: context(),
            schema,
            rejection: None,
            calls: std::cell::Cell::new(0),
            seen: std::cell::RefCell::new(None),
        }
    }
}

impl ResourcePropertiesValidator for Authority {
    fn context(&self) -> &ResourceValidationContext {
        &self.context
    }
    fn properties_schema(&self) -> MetadataSchema {
        self.schema.clone()
    }
    fn validate_properties(&self, properties: &BTreeMap<String, Value>) -> Result<(), String> {
        self.calls.set(self.calls.get() + 1);
        *self.seen.borrow_mut() = Some(properties.clone());
        self.rejection.clone().map_or(Ok(()), Err)
    }
}

fn admit_json(json: &str, authority: &Authority) -> Result<ResourceReference, Box<dyn Error>> {
    let candidate: ResourceReferenceCandidate = serde_json::from_str(json)?;
    Ok(candidate.admit(Some(&context()), &[authority])?)
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
        "18446744073709551615",
        "9007199254740991.0000000001",
        "true",
        "null",
        r#"{"integer":"1"}"#,
        "[1]",
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
        serde_json::from_str::<ResourceReferenceCandidate>(&reference_json("3"))
            .ok()
            .map(|value| (value.resource_id(), value.byte_length().get())),
        Some((hash_resource_bytes(b"abc"), 3))
    );

    for invalid in [
        "{}".to_owned(),
        r#"{"byte_length":3}"#.to_owned(),
        format!(r#"{{"resource_id":"{RESOURCE_ID}"}}"#),
        r#"{"resource_id":"not-a-resource-id","byte_length":3}"#.to_owned(),
        format!(
            r#"{{"resource_id":"{}","byte_length":3}}"#,
            RESOURCE_ID.replace(":resource:", ":revision:")
        ),
    ] {
        assert!(
            serde_json::from_str::<ResourceReferenceCandidate>(&invalid).is_err(),
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
        "property_schema_id",
        "validation_context",
        "validation_status",
        "validation_evidence",
    ];

    for field in forbidden_fields {
        let json =
            format!(r#"{{"resource_id":"{RESOURCE_ID}","byte_length":0,"{field}":"excluded"}}"#);
        assert!(
            serde_json::from_str::<ResourceReferenceCandidate>(&json).is_err(),
            "{field}"
        );
    }

    let minimal = serde_json::from_str::<ResourceReferenceCandidate>(&reference_json("0"));
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
        format!(r#"{{"resource_id":"{RESOURCE_ID}","byte_length":0,"byte_length":1}}"#),
        format!(
            r#"{{"resource_id":"{RESOURCE_ID}","byte_length":0,"properties":{{"sample_rate":44100,"sample_rate":48000}}}}"#
        ),
        format!(
            r#"{{"resource_id":"{RESOURCE_ID}","byte_length":0,"properties":{{"sample_rate":44100,"\u0073ample_rate":48000}}}}"#
        ),
        format!(
            r#"{{"resource_id":"{RESOURCE_ID}","byte_length":0,"properties":{{"nested":{{"value":1,"value":2}}}}}}"#
        ),
        format!(r#"{{"resource_id":"{RESOURCE_ID}","byte_length":0,"\u0062yte_length":1}}"#),
        format!(
            r#"{{"resource_id":"{RESOURCE_ID}","byte_length":0,"properties":{{"nested":[{{"value":1,"\u0076alue":2}}]}}}}"#
        ),
    ];

    for json in duplicate_members {
        assert!(
            serde_json::from_str::<ResourceReferenceCandidate>(&json).is_err(),
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
    let authority = Authority::new(MetadataSchema::map(MetadataSchema::Scalar));
    let with_properties = base
        .clone()
        .with_properties(properties)
        .admit(Some(&context()), &[&authority])?;

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
    let changed_properties = base
        .clone()
        .with_properties(BTreeMap::from([(
            "sample_rate".to_owned(),
            Value::from(44_100),
        )]))
        .admit(Some(&context()), &[&authority])?;
    assert_ne!(
        reference_identity(&with_properties)?,
        reference_identity(&changed_properties)?
    );
    assert_eq!(
        with_properties.resource_id(),
        changed_properties.resource_id()
    );
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
    let authority = Authority::new(MetadataSchema::map(MetadataSchema::Scalar));
    let left = format!(
        r#"{{"resource_id":"{RESOURCE_ID}","byte_length":3,"properties":{{"beta":2,"alpha":1}}}}"#
    );
    let right = format!(
        r#"{{"resource_id":"{RESOURCE_ID}","byte_length":3,"properties":{{"alpha":1,"beta":2}}}}"#
    );
    let left = admit_json(&left, &authority)?;
    let right = admit_json(&right, &authority)?;
    let left_canonical = left.canonical_bytes(Some(&context()))?;
    let right_canonical = right.canonical_bytes(Some(&context()))?;

    assert_eq!(left_canonical, right_canonical);
    assert_eq!(reference_identity(&left)?, reference_identity(&right)?);
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

// Core §§5.1, 7, 76–77; INV-RES-004/008; ADR-0009.
#[test]
fn no_properties_pass_generic_validation_but_even_empty_properties_require_context()
-> Result<(), Box<dyn Error>> {
    let candidate: ResourceReferenceCandidate = serde_json::from_str(&reference_json("3"))?;
    let admitted = candidate.admit(None, &[])?;
    assert!(admitted.properties().is_none());
    assert_eq!(
        admitted.canonical_bytes(None)?,
        ResourceReference::new(hash_resource_bytes(b"abc"), ResourceByteLength::new(3)?)
            .canonical_bytes(None)?
    );

    let empty = candidate.with_properties(BTreeMap::new());
    assert_eq!(
        empty.admit(None, &[]),
        Err(ResourceAdmissionError::UnknownContext)
    );
    let authority = Authority::new(MetadataSchema::map(MetadataSchema::Scalar));
    let admitted = empty.admit(Some(&context()), &[&authority])?;
    assert_eq!(authority.calls.get(), 1);
    assert_eq!(admitted.properties(), Some(&BTreeMap::new()));
    assert_ne!(
        admitted.canonical_bytes(Some(&context()))?,
        ResourceReference::new(hash_resource_bytes(b"abc"), ResourceByteLength::new(3)?)
            .canonical_bytes(None)?
    );

    let null_properties =
        format!(r#"{{"resource_id":"{RESOURCE_ID}","byte_length":3,"properties":null}}"#);
    assert!(serde_json::from_str::<ResourceReferenceCandidate>(&null_properties).is_err());
    Ok(())
}

#[test]
fn unavailable_unrelated_and_non_unique_authorities_leave_preservable_candidates_unchecked()
-> Result<(), Box<dyn Error>> {
    let json =
        format!(r#"{{"resource_id":"{RESOURCE_ID}","byte_length":3,"properties":{{"value":1}}}}"#);
    let candidate: ResourceReferenceCandidate = serde_json::from_str(&json)?;
    let mut unrelated = Authority::new(MetadataSchema::map(MetadataSchema::Scalar));
    unrelated.context.containing_schema = "test.containing-state/0.2".to_owned();
    assert_eq!(
        candidate.admit(Some(&context()), &[]),
        Err(ResourceAdmissionError::UnavailableContext)
    );
    assert_eq!(
        candidate.admit(Some(&context()), &[&unrelated]),
        Err(ResourceAdmissionError::UnavailableContext)
    );
    let first = Authority::new(MetadataSchema::map(MetadataSchema::Scalar));
    let second = Authority::new(MetadataSchema::map(MetadataSchema::Scalar));
    assert_eq!(
        candidate.admit(Some(&context()), &[&first, &second]),
        Err(ResourceAdmissionError::NonUniqueAuthority)
    );
    assert_eq!(
        first.calls.get() + second.calls.get() + unrelated.calls.get(),
        0
    );

    // Preservation is an unchecked transport path, never an admission operation.
    let preserved = serde_json::to_vec(&candidate)?;
    let restored: ResourceReferenceCandidate = serde_json::from_slice(&preserved)?;
    assert_eq!(restored, candidate);
    assert_eq!(
        restored.admit(None, &[]),
        Err(ResourceAdmissionError::UnknownContext)
    );
    let admitted = restored.admit(Some(&context()), &[&first])?;
    assert_eq!(first.calls.get(), 1);
    assert_eq!(
        admitted.canonical_bytes(None),
        Err(ResourceAdmissionError::ContextMismatch)
    );
    assert_eq!(
        admitted.historical_value(Some(&unrelated.context)),
        Err(ResourceAdmissionError::ContextMismatch)
    );

    for changed in [
        ResourceValidationContext {
            adapter: Some((
                "another.authority".to_owned(),
                "test.authority.state/0.1".to_owned(),
            )),
            ..context()
        },
        ResourceValidationContext {
            adapter: Some((
                "test.authority".to_owned(),
                "test.authority.state/0.2".to_owned(),
            )),
            ..context()
        },
        ResourceValidationContext {
            adapter: None,
            ..context()
        },
    ] {
        assert_eq!(
            candidate.admit(Some(&changed), &[&first]),
            Err(ResourceAdmissionError::UnavailableContext)
        );
        assert_eq!(
            admitted.canonical_bytes(Some(&changed)),
            Err(ResourceAdmissionError::ContextMismatch)
        );
    }
    // Replacing an already validated map revokes admission, including with {}.
    assert_eq!(
        admitted.with_properties(BTreeMap::new()).admit(None, &[]),
        Err(ResourceAdmissionError::UnknownContext)
    );
    Ok(())
}

#[test]
fn exact_semantic_authority_rejection_propagates_without_core_key_heuristics()
-> Result<(), Box<dyn Error>> {
    let mut authority = Authority::new(MetadataSchema::map(MetadataSchema::map(
        MetadataSchema::Scalar,
    )));
    authority.rejection = Some("synthetic authority: excluded interpretation".to_owned());
    for key in ["filename", "unrelated_alias"] {
        let candidate = ResourceReferenceCandidate::new(
            hash_resource_bytes(b"abc"),
            ResourceByteLength::new(3)?,
        )
        .with_properties(BTreeMap::from([(
            key.to_owned(),
            serde_json::json!({"nested_alias": "excluded synthetic meaning"}),
        )]));
        assert_eq!(
            candidate.admit(Some(&context()), &[&authority]),
            Err(ResourceAdmissionError::SemanticRejection(
                "synthetic authority: excluded interpretation".to_owned(),
            ))
        );
    }
    assert_eq!(authority.calls.get(), 2);
    // Generic Core does not guess meanings even from suggestive key names.
    let authority = Authority::new(MetadataSchema::map(MetadataSchema::Scalar));
    let candidate =
        ResourceReferenceCandidate::new(hash_resource_bytes(b"abc"), ResourceByteLength::new(3)?)
            .with_properties(BTreeMap::from([("filename".to_owned(), Value::from(17))]));
    assert!(candidate.admit(Some(&context()), &[&authority]).is_ok());
    assert_eq!(authority.calls.get(), 1);
    Ok(())
}

fn nested_properties_schema(ordering: Option<ArrayOrdering>) -> MetadataSchema {
    let element = MetadataSchema::structure([
        (
            "sequence",
            MetadataSchema::array(ArrayOrdering::Ordered, MetadataSchema::Scalar),
        ),
        (
            "members",
            MetadataSchema::Array {
                ordering,
                elements: Box::new(MetadataSchema::Scalar),
            },
        ),
    ]);
    MetadataSchema::map(MetadataSchema::array(ArrayOrdering::SetLike, element))
}

#[test]
fn exact_context_supplies_nested_shapes_and_ordered_and_set_like_array_semantics()
-> Result<(), Box<dyn Error>> {
    let authority = Authority::new(nested_properties_schema(Some(ArrayOrdering::SetLike)));
    let base =
        ResourceReferenceCandidate::new(hash_resource_bytes(b"abc"), ResourceByteLength::new(3)?);
    let first = base
        .clone()
        .with_properties(BTreeMap::from([(
            "items".to_owned(),
            serde_json::json!([
                {"sequence": [2, 1], "members": [3, 1, 2]},
                {"sequence": [4, 5], "members": [6, 7]}
            ]),
        )]))
        .admit(Some(&context()), &[&authority])?;
    let permuted = base
        .clone()
        .with_properties(BTreeMap::from([(
            "items".to_owned(),
            serde_json::json!([
                {"members": [7, 6], "sequence": [4, 5]},
                {"members": [2, 3, 1], "sequence": [2, 1]}
            ]),
        )]))
        .admit(Some(&context()), &[&authority])?;
    assert_eq!(
        first.canonical_bytes(Some(&context()))?,
        permuted.canonical_bytes(Some(&context()))?
    );
    let reordered = base
        .clone()
        .with_properties(BTreeMap::from([(
            "items".to_owned(),
            serde_json::json!([
                {"sequence": [1, 2], "members": [3, 1, 2]},
                {"sequence": [4, 5], "members": [6, 7]}
            ]),
        )]))
        .admit(Some(&context()), &[&authority])?;
    assert_ne!(
        first.canonical_bytes(Some(&context()))?,
        reordered.canonical_bytes(Some(&context()))?
    );

    // Different classifications from the exact context change normalization.
    let mut ordered_authority =
        Authority::new(nested_properties_schema(Some(ArrayOrdering::Ordered)));
    // Changed property interpretation uses a different versioned context (§77).
    ordered_authority.context.containing_schema = "test.containing-state/0.2".to_owned();
    let changed_members = base
        .with_properties(BTreeMap::from([(
            "items".to_owned(),
            serde_json::json!([{"sequence": [2, 1], "members": [3, 1, 2]}]),
        )]))
        .admit(Some(&ordered_authority.context), &[&ordered_authority])?;
    assert_eq!(
        changed_members
            .properties()
            .and_then(|p| p.get("items"))
            .and_then(|v| v.get(0))
            .and_then(|v| v.get("members")),
        Some(&serde_json::json!([3, 1, 2]))
    );
    assert_eq!(*authority.seen.borrow(), reordered.properties().cloned());
    assert_eq!(
        *ordered_authority.seen.borrow(),
        changed_members.properties().cloned()
    );
    Ok(())
}

#[test]
fn unclassified_nested_arrays_shape_mismatches_and_duplicate_set_members_block_admission()
-> Result<(), Box<dyn Error>> {
    let base =
        ResourceReferenceCandidate::new(hash_resource_bytes(b"abc"), ResourceByteLength::new(3)?);
    let unclassified = Authority::new(nested_properties_schema(None));
    // Reject incomplete schema even with no nested values.
    assert!(matches!(
        base.clone()
            .with_properties(BTreeMap::new())
            .admit(Some(&context()), &[&unclassified]),
        Err(ResourceAdmissionError::Canonical(
            CanonicalMetadataError::UnclassifiedArray { .. }
        ))
    ));
    assert_eq!(unclassified.calls.get(), 0);
    let scalar_authority = Authority::new(MetadataSchema::map(MetadataSchema::Scalar));
    let candidate = base.clone().with_properties(BTreeMap::from([(
        "array".to_owned(),
        serde_json::json!([1, 2]),
    )]));
    assert!(matches!(
        candidate.admit(Some(&context()), &[&scalar_authority]),
        Err(ResourceAdmissionError::Canonical(
            CanonicalMetadataError::SchemaMismatch { .. }
        ))
    ));
    assert_eq!(scalar_authority.calls.get(), 0);
    let classified = Authority::new(nested_properties_schema(Some(ArrayOrdering::SetLike)));
    let candidate = base.with_properties(BTreeMap::from([(
        "items".to_owned(),
        serde_json::json!([{"sequence": [2, 1], "members": [1, 1]}]),
    )]));
    assert!(matches!(
        candidate.admit(Some(&context()), &[&classified]),
        Err(ResourceAdmissionError::Canonical(
            CanonicalMetadataError::DuplicateSetLikeElement { .. }
        ))
    ));
    assert_eq!(classified.calls.get(), 0);
    Ok(())
}

#[test]
fn operational_validation_evidence_is_absent_from_canonical_historical_fields()
-> Result<(), Box<dyn Error>> {
    let candidate =
        ResourceReferenceCandidate::new(hash_resource_bytes(b"abc"), ResourceByteLength::new(3)?)
            .with_properties(BTreeMap::from([("value".to_owned(), Value::from(17))]));
    let first = Authority::new(MetadataSchema::map(MetadataSchema::Scalar));
    let second = Authority::new(MetadataSchema::map(MetadataSchema::Scalar));
    second.calls.set(100);
    let left = candidate.admit(Some(&context()), &[&first])?;
    let right = candidate.admit(Some(&context()), &[&second])?;
    assert_eq!(
        left.canonical_bytes(Some(&context()))?,
        right.canonical_bytes(Some(&context()))?
    );
    assert_eq!(
        left.historical_value(Some(&context()))?,
        serde_json::json!({
            "resource_id": RESOURCE_ID, "byte_length": 3, "properties": {"value": 17}
        })
    );
    assert_eq!(first.calls.get(), 1);
    assert_eq!(second.calls.get(), 101);
    let mut another_context = Authority::new(MetadataSchema::map(MetadataSchema::Scalar));
    another_context.context.containing_schema = "test.other-containing-state/0.1".to_owned();
    let other = candidate.admit(Some(&another_context.context), &[&another_context])?;
    assert_eq!(
        left.canonical_bytes(Some(&context()))?,
        other.canonical_bytes(Some(&another_context.context))?
    );
    Ok(())
}

#[test]
fn resource_reference_decoding_enforces_integer_boundaries_and_invalid_encodings() {
    for valid in ["0", "1", "9007199254740991"] {
        assert!(serde_json::from_str::<ResourceReferenceCandidate>(&reference_json(valid)).is_ok());
    }

    for invalid in [
        "-1",
        "1.5",
        "9007199254740992",
        r#""1""#,
        "18446744073709551615",
        "null",
        "true",
        "[1]",
        r#"{"integer":1}"#,
        "9007199254740991.0000000001",
    ] {
        assert!(
            serde_json::from_str::<ResourceReferenceCandidate>(&reference_json(invalid)).is_err(),
            "{invalid}"
        );
    }
}

#[test]
fn present_optional_fields_cannot_silently_disappear_during_decoding() {
    for field in ["role", "media_type", "properties"] {
        let json = format!(r#"{{"resource_id":"{RESOURCE_ID}","byte_length":3,"{field}":null}}"#);
        assert!(
            serde_json::from_str::<ResourceReferenceCandidate>(&json).is_err(),
            "{field}"
        );
    }
}
