use omvcs_model::canonical::{
    ArrayOrdering, CanonicalMetadataError, MetadataSchema, canonicalize_metadata_body,
};

fn object_schema(
    fields: impl IntoIterator<Item = (&'static str, MetadataSchema)>,
) -> MetadataSchema {
    MetadataSchema::structure(
        fields
            .into_iter()
            .map(|(name, schema)| (name.to_owned(), schema)),
    )
}

#[test]
fn canonical_metadata_uses_rfc8785_member_order_without_entry_value_sorting() {
    let schema = object_schema([("a", MetadataSchema::Scalar), ("b", MetadataSchema::Scalar)]);
    let expected = br#"{"a":"z","b":"a"}"#;

    assert_eq!(
        canonicalize_metadata_body(br#" { "b" : "a", "a" : "z" } "#, &schema),
        Ok(expected.to_vec())
    );
    assert_eq!(
        canonicalize_metadata_body(br#"{"a":"z","b":"a"}"#, &schema),
        Ok(expected.to_vec())
    );
}

#[test]
fn canonical_metadata_orders_object_member_names_by_rfc8785_utf16_code_units() {
    let schema = MetadataSchema::map(MetadataSchema::Scalar);
    let input = "{\"\":\"bmp\",\"𐀀\":\"supplementary\"}".as_bytes();
    let expected = "{\"𐀀\":\"supplementary\",\"\":\"bmp\"}".as_bytes();

    assert_eq!(
        canonicalize_metadata_body(input, &schema),
        Ok(expected.to_vec())
    );
}

#[test]
fn canonical_metadata_rejects_duplicate_raw_member_names_before_canonicalization() {
    let schema = MetadataSchema::map(MetadataSchema::Scalar);
    let nested_schema = MetadataSchema::map(MetadataSchema::map(MetadataSchema::Scalar));

    assert_eq!(
        canonicalize_metadata_body(br#"{"name":"first","name":"second"}"#, &schema),
        Err(CanonicalMetadataError::DuplicateMemberName)
    );
    assert_eq!(
        canonicalize_metadata_body(br#"{"outer":{"name":1,"name":2}}"#, &nested_schema),
        Err(CanonicalMetadataError::DuplicateMemberName)
    );
    assert_eq!(
        canonicalize_metadata_body(br#"{"name":1,"\u006eame":2}"#, &schema),
        Err(CanonicalMetadataError::DuplicateMemberName)
    );
}

#[test]
fn canonical_metadata_normalizes_map_values_by_their_recursive_schemas() {
    let record = object_schema([(
        "tags",
        MetadataSchema::array(ArrayOrdering::SetLike, MetadataSchema::Scalar),
    )]);
    let schema = object_schema([("metadata", MetadataSchema::map(record))]);
    let left = br#"{"metadata":{"beta":{"tags":["z","a"]},"alpha":{"tags":["y","b"]}}}"#;
    let right = br#"{"metadata":{"alpha":{"tags":["b","y"]},"beta":{"tags":["a","z"]}}}"#;
    let expected = br#"{"metadata":{"alpha":{"tags":["b","y"]},"beta":{"tags":["a","z"]}}}"#;

    assert_eq!(
        canonicalize_metadata_body(left, &schema),
        Ok(expected.to_vec())
    );
    assert_eq!(
        canonicalize_metadata_body(right, &schema),
        Ok(expected.to_vec())
    );
}

#[test]
fn canonical_metadata_normalizes_nested_set_like_arrays_recursively() {
    let item = object_schema([(
        "members",
        MetadataSchema::array(ArrayOrdering::SetLike, MetadataSchema::Scalar),
    )]);
    let schema = object_schema([(
        "groups",
        MetadataSchema::array(ArrayOrdering::SetLike, item),
    )]);
    let left = br#"{"groups":[{"members":["z","a"]},{"members":["y","b"]}]}"#;
    let right = br#"{"groups":[{"members":["b","y"]},{"members":["a","z"]}]}"#;
    let expected = br#"{"groups":[{"members":["a","z"]},{"members":["b","y"]}]}"#;

    assert_eq!(
        canonicalize_metadata_body(left, &schema),
        Ok(expected.to_vec())
    );
    assert_eq!(
        canonicalize_metadata_body(right, &schema),
        Ok(expected.to_vec())
    );
}

#[test]
fn canonical_metadata_rejects_set_like_elements_equal_after_canonicalization() {
    let element = object_schema([("a", MetadataSchema::Scalar), ("b", MetadataSchema::Scalar)]);
    let schema = object_schema([(
        "members",
        MetadataSchema::array(ArrayOrdering::SetLike, element),
    )]);
    let input = br#"{"members":[{"b":2,"a":"x"},{"a":"x","b":2}]}"#;

    assert_eq!(
        canonicalize_metadata_body(input, &schema),
        Err(CanonicalMetadataError::DuplicateSetLikeElement {
            path: "/members".to_owned()
        })
    );
}

#[test]
fn canonical_metadata_preserves_ordered_sequence_element_order() {
    let schema = object_schema([(
        "sequence",
        MetadataSchema::array(ArrayOrdering::Ordered, MetadataSchema::Scalar),
    )]);
    let forward = canonicalize_metadata_body(br#"{"sequence":["first","second"]}"#, &schema);
    let reverse = canonicalize_metadata_body(br#"{"sequence":["second","first"]}"#, &schema);

    assert_eq!(forward, Ok(br#"{"sequence":["first","second"]}"#.to_vec()));
    assert_eq!(reverse, Ok(br#"{"sequence":["second","first"]}"#.to_vec()));
    assert_ne!(forward, reverse);
}

#[test]
fn canonical_metadata_rejects_arrays_without_schema_classification_recursively() {
    let schema = object_schema([(
        "nested",
        MetadataSchema::map(MetadataSchema::unclassified_array(MetadataSchema::Scalar)),
    )]);

    assert_eq!(
        canonicalize_metadata_body(br#"{"nested":{"items":[1,2]}}"#, &schema),
        Err(CanonicalMetadataError::UnclassifiedArray {
            path: "/nested/*".to_owned()
        })
    );
}

#[test]
fn canonical_metadata_rejects_unclassified_arrays_even_without_present_values() {
    let empty_map_schema =
        MetadataSchema::map(MetadataSchema::unclassified_array(MetadataSchema::Scalar));
    let absent_field_schema = object_schema([(
        "optional",
        MetadataSchema::unclassified_array(MetadataSchema::Scalar),
    )]);
    let array_element_schema = object_schema([(
        "members",
        MetadataSchema::unclassified_array(MetadataSchema::Scalar),
    )]);
    let empty_array_schema = object_schema([(
        "groups",
        MetadataSchema::array(ArrayOrdering::Ordered, array_element_schema),
    )]);

    assert_eq!(
        canonicalize_metadata_body(b"{}", &empty_map_schema),
        Err(CanonicalMetadataError::UnclassifiedArray {
            path: "/*".to_owned()
        })
    );
    assert_eq!(
        canonicalize_metadata_body(b"{}", &absent_field_schema),
        Err(CanonicalMetadataError::UnclassifiedArray {
            path: "/optional".to_owned()
        })
    );
    assert_eq!(
        canonicalize_metadata_body(br#"{"groups":[]}"#, &empty_array_schema),
        Err(CanonicalMetadataError::UnclassifiedArray {
            path: "/groups/*/members".to_owned()
        })
    );
}

#[test]
fn canonical_metadata_matches_rfc8785_utf8_string_and_number_vectors() {
    let schema = object_schema([
        ("label", MetadataSchema::Scalar),
        (
            "numbers",
            MetadataSchema::array(ArrayOrdering::Ordered, MetadataSchema::Scalar),
        ),
    ]);
    let input = r#"{"label":"é\/\"\\\n\u000f","numbers":[333333333.33333329,1E30,4.50,2e-3,0.000000000000000000000000001,-0,1e-6,1e-7,1e21,1e20]}"#.as_bytes();
    let expected = r#"{"label":"é/\"\\\n\u000f","numbers":[333333333.3333333,1e+30,4.5,0.002,1e-27,0,0.000001,1e-7,1e+21,100000000000000000000]}"#.as_bytes();

    assert_eq!(
        canonicalize_metadata_body(input, &schema),
        Ok(expected.to_vec())
    );
}

#[test]
fn canonical_metadata_rejects_numbers_not_exactly_representable_as_binary64_integers() {
    let schema = object_schema([("value", MetadataSchema::Scalar)]);

    assert_eq!(
        canonicalize_metadata_body(br#"{"value":9007199254740992}"#, &schema),
        Ok(br#"{"value":9007199254740992}"#.to_vec())
    );
    assert_eq!(
        canonicalize_metadata_body(br#"{"value":9007199254740993}"#, &schema),
        Err(CanonicalMetadataError::InvalidJson)
    );
    assert_eq!(
        canonicalize_metadata_body(br#"{"value":1e400}"#, &schema),
        Err(CanonicalMetadataError::InvalidJson)
    );
}

#[test]
fn canonical_metadata_rejects_non_utf8_input() {
    let schema = object_schema([("value", MetadataSchema::Scalar)]);

    assert_eq!(
        canonicalize_metadata_body(b"{\"value\":\"\xff\"}", &schema),
        Err(CanonicalMetadataError::InvalidUtf8)
    );
}

#[test]
fn canonical_metadata_body_must_be_a_json_object() {
    assert_eq!(
        canonicalize_metadata_body(b"null", &MetadataSchema::Scalar),
        Err(CanonicalMetadataError::SchemaMismatch {
            path: "/".to_owned()
        })
    );
}

#[test]
fn canonical_metadata_body_api_does_not_include_external_wrappers() {
    let schema = object_schema([
        ("schema", MetadataSchema::Scalar),
        ("description", MetadataSchema::Scalar),
    ]);
    let body = br#"{"schema":"omvcs.example/1","description":"take"}"#;
    let wrapped = br#"{"body":{"schema":"omvcs.example/1","description":"take"},"database_key":"record-7","signature":"external","timestamp":"external"}"#;

    assert_eq!(
        canonicalize_metadata_body(body, &schema),
        Ok(br#"{"description":"take","schema":"omvcs.example/1"}"#.to_vec())
    );
    assert!(matches!(
        canonicalize_metadata_body(wrapped, &schema),
        Err(CanonicalMetadataError::SchemaMismatch { .. })
    ));
}

#[test]
fn canonical_metadata_preserves_unicode_without_normalization() {
    let schema = object_schema([("value", MetadataSchema::Scalar)]);
    let composed = canonicalize_metadata_body(r#"{"value":"é"}"#.as_bytes(), &schema);
    let decomposed = canonicalize_metadata_body(br#"{"value":"e\u0301"}"#, &schema);

    assert_eq!(composed, Ok(r#"{"value":"é"}"#.as_bytes().to_vec()));
    assert_eq!(decomposed, Ok(b"{\"value\":\"e\xCC\x81\"}".to_vec()));
    assert_ne!(composed, decomposed);
}

#[test]
fn canonical_metadata_serialization_is_deterministic_for_identical_bodies() {
    let schema = object_schema([
        ("name", MetadataSchema::Scalar),
        (
            "members",
            MetadataSchema::array(ArrayOrdering::SetLike, MetadataSchema::Scalar),
        ),
    ]);
    let body = br#"{"members":["third","first","second"],"name":"same"}"#;

    let first = canonicalize_metadata_body(body, &schema);
    let second = canonicalize_metadata_body(body, &schema);

    assert_eq!(first, second);
}
