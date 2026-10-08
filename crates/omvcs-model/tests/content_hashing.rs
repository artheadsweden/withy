use omvcs_model::canonical::{
    ArrayOrdering, CanonicalMetadataError, MetadataSchema, canonicalize_metadata_body,
};
use omvcs_model::hashing::{
    hash_adapter_state_metadata, hash_component_state_metadata, hash_project_state_metadata,
    hash_resource_bytes, hash_revision_metadata,
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
fn resource_identifiers_match_standard_sha256_vectors() {
    let empty = hash_resource_bytes(b"");
    let abc = hash_resource_bytes(b"abc");

    assert_eq!(
        empty.to_string(),
        "omvcs:resource:sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        abc.to_string(),
        "omvcs:resource:sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn metadata_identifiers_hash_exact_canonical_body_bytes_without_prefixes() {
    let schema = object_schema([("a", MetadataSchema::Scalar), ("b", MetadataSchema::Scalar)]);
    let canonical = canonicalize_metadata_body(br#"{"b":"a","a":"z"}"#, &schema);
    let expected_body = br#"{"a":"z","b":"a"}"#;

    assert_eq!(canonical, Ok(expected_body.to_vec()));
    let canonical = canonical.unwrap_or_default();
    let expected_digest = "b977da1f0b6307aad6140fa0079d3f363d04a69ef55925461fec496e31e6a865";

    let component_state = hash_component_state_metadata(&canonical);
    let adapter_state = hash_adapter_state_metadata(&canonical);
    let project_state = hash_project_state_metadata(&canonical);
    let revision = hash_revision_metadata(&canonical);
    let resource = hash_resource_bytes(&canonical);

    assert_eq!(component_state.digest(), resource.digest());
    assert_eq!(adapter_state.digest(), resource.digest());
    assert_eq!(project_state.digest(), resource.digest());
    assert_eq!(revision.digest(), resource.digest());
    assert_eq!(
        component_state.to_string(),
        format!("omvcs:component-state:sha256:{expected_digest}")
    );
    assert_eq!(
        adapter_state.to_string(),
        format!("omvcs:adapter-state:sha256:{expected_digest}")
    );
    assert_eq!(
        project_state.to_string(),
        format!("omvcs:project-state:sha256:{expected_digest}")
    );
    assert_eq!(
        revision.to_string(),
        format!("omvcs:revision:sha256:{expected_digest}")
    );
    assert_eq!(
        resource.to_string(),
        format!("omvcs:resource:sha256:{expected_digest}")
    );
    assert_ne!(component_state.to_string(), revision.to_string());
}

#[test]
fn canonical_map_and_set_normalization_preserves_metadata_hash_identity() {
    let schema = object_schema([
        (
            "members",
            MetadataSchema::array(ArrayOrdering::SetLike, MetadataSchema::Scalar),
        ),
        ("properties", MetadataSchema::map(MetadataSchema::Scalar)),
    ]);
    let left = br#"{"members":["z","a"],"properties":{"beta":"two","alpha":"one"}}"#;
    let right = br#"{"properties":{"alpha":"one","beta":"two"},"members":["a","z"]}"#;

    let canonical_left = canonicalize_metadata_body(left, &schema);
    let canonical_right = canonicalize_metadata_body(right, &schema);

    assert_eq!(canonical_left, canonical_right);
    let canonical_left = canonical_left.unwrap_or_default();
    let canonical_right = canonical_right.unwrap_or_default();
    assert_eq!(
        hash_project_state_metadata(&canonical_left),
        hash_project_state_metadata(&canonical_right)
    );
}

#[test]
fn duplicate_raw_member_names_are_rejected_before_metadata_hashing() {
    let schema = MetadataSchema::map(MetadataSchema::Scalar);

    assert_eq!(
        canonicalize_metadata_body(br#"{"name":"first","name":"second"}"#, &schema),
        Err(CanonicalMetadataError::DuplicateMemberName)
    );
}

#[test]
fn changing_external_wrapper_fields_does_not_change_metadata_identity() {
    let schema = object_schema([
        ("schema", MetadataSchema::Scalar),
        ("value", MetadataSchema::Scalar),
    ]);
    let wrapper_one =
        br#"{"body":{"schema":"omvcs.example/1","value":"same"},"signature":"first"}"#;
    let wrapper_two =
        br#"{"body":{"schema":"omvcs.example/1","value":"same"},"timestamp":"second"}"#;
    let body_one = serde_json::from_slice::<serde_json::Value>(wrapper_one)
        .ok()
        .and_then(|wrapper| wrapper.get("body").cloned())
        .and_then(|body| serde_json::to_vec(&body).ok());
    let body_two = serde_json::from_slice::<serde_json::Value>(wrapper_two)
        .ok()
        .and_then(|wrapper| wrapper.get("body").cloned())
        .and_then(|body| serde_json::to_vec(&body).ok());

    assert_ne!(wrapper_one.as_slice(), wrapper_two.as_slice());
    assert_eq!(
        body_one.as_deref(),
        Some(br#"{"schema":"omvcs.example/1","value":"same"}"#.as_slice())
    );
    assert_eq!(body_one, body_two);
    let body = body_one.unwrap_or_default();
    let canonical_one = canonicalize_metadata_body(&body, &schema);
    let canonical_two = canonicalize_metadata_body(&body, &schema);

    assert_eq!(canonical_one, canonical_two);
    let canonical = canonical_one.unwrap_or_default();
    assert_eq!(
        hash_revision_metadata(&canonical),
        hash_revision_metadata(&body)
    );
}

#[test]
fn resource_identity_depends_on_raw_bytes_not_location_or_replica_state() {
    let bytes = b"identical complete Resource bytes";
    let first = ResourceFixture {
        bytes,
        filename: "take.wav",
        path: "endpoint-a/path/one",
        provider: "provider-a",
        platform_url: "https://platform-a.invalid/resource",
        credential: "credential-a",
        replica_available: true,
    };
    let second = ResourceFixture {
        bytes,
        filename: "renamed.wav",
        path: "endpoint-b/path/two",
        provider: "provider-b",
        platform_url: "https://platform-b.invalid/resource",
        credential: "credential-b",
        replica_available: false,
    };

    assert_ne!(first.filename, second.filename);
    assert_ne!(first.path, second.path);
    assert_ne!(first.provider, second.provider);
    assert_ne!(first.platform_url, second.platform_url);
    assert_ne!(first.credential, second.credential);
    assert_ne!(first.replica_available, second.replica_available);
    assert_eq!(first.bytes, second.bytes);
    assert_eq!(
        hash_resource_bytes(first.bytes),
        hash_resource_bytes(second.bytes)
    );
}

#[test]
fn a_single_resource_byte_change_changes_the_identifier() {
    assert_ne!(
        hash_resource_bytes(b"resource"),
        hash_resource_bytes(b"Resource")
    );
}

#[test]
fn resource_hashing_is_deterministic_for_broad_byte_inputs() {
    let mut state = 0x9e37_79b9_7f4a_7c15_u64;

    for length in 0..=1024 {
        let mut bytes = Vec::with_capacity(length);
        for _ in 0..length {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            bytes.push(state.to_le_bytes()[3]);
        }

        assert_eq!(
            hash_resource_bytes(&bytes),
            hash_resource_bytes(&bytes),
            "hash differs for repeated input of length {length}"
        );
    }
}

struct ResourceFixture<'a> {
    bytes: &'a [u8],
    filename: &'a str,
    path: &'a str,
    provider: &'a str,
    platform_url: &'a str,
    credential: &'a str,
    replica_available: bool,
}
