#![allow(clippy::expect_used)]

use omvcs_model::canonical::{MetadataSchema, canonicalize_metadata_body};
use omvcs_model::hashing::{hash_resource_bytes, hash_revision_metadata};
use omvcs_model::replica::{
    ChunkManifest, ChunkManifestEntry, ChunkManifestError, ProviderLocator, ProviderLocatorError,
    Replica, ReplicaAvailability, ReplicaCandidate, ReplicaRepresentation,
};
use omvcs_model::resource::{ResourceByteLength, ResourceObject};
use omvcs_model::{ChunkId, ReplicaId, ResourceId, StorageEndpointId};
use serde_json::{Value, json};

const RESOURCE_ONE: &str =
    "omvcs:resource:sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const RESOURCE_TWO: &str =
    "omvcs:resource:sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const CHUNK_ONE: &str =
    "omvcs:chunk:sha256:1111111111111111111111111111111111111111111111111111111111111111";
const CHUNK_TWO: &str =
    "omvcs:chunk:sha256:2222222222222222222222222222222222222222222222222222222222222222";
const ENDPOINT: &str = "019cc17d-1b22-7a41-9fe9-c345c468f82c";
const REPLICA_ONE: &str = "019cc17d-1b22-7a41-9fe9-c345c468f82d";
const REPLICA_TWO: &str = "019cc17d-1b22-7a41-9fe9-c345c468f82e";

fn length(value: u64) -> ResourceByteLength {
    ResourceByteLength::new(value).expect("test byte length is in range")
}

fn locator(value: Value) -> ProviderLocator {
    ProviderLocator::new("provider.object-locator/1", value).expect("valid JCS locator")
}

fn resource_id() -> ResourceId {
    RESOURCE_ONE.parse().expect("valid ResourceId")
}

fn endpoint_id() -> StorageEndpointId {
    ENDPOINT.parse().expect("valid StorageEndpointId")
}

fn manifest() -> ChunkManifest {
    ChunkManifest::new(
        resource_id(),
        length(5),
        vec![
            ChunkManifestEntry::new(CHUNK_ONE.parse().expect("ChunkId"), length(0), length(2)),
            ChunkManifestEntry::new(CHUNK_TWO.parse().expect("ChunkId"), length(2), length(3)),
        ],
    )
    .expect("ordered complete manifest")
}

fn persisted_replica(
    id: &str,
    resource_id: ResourceId,
    endpoint_id: StorageEndpointId,
    representation: &ReplicaRepresentation,
    locator: &ProviderLocator,
    availability: ReplicaAvailability,
) -> Replica {
    serde_json::from_value(json!({
        "replica_id": id,
        "resource_id": resource_id.to_string(),
        "endpoint_id": endpoint_id.to_string(),
        "representation": representation,
        "locator": locator,
        "availability": availability,
    }))
    .expect("decode an already-registered persisted record")
}

#[test]
fn replica_ids_are_canonical_uuidv7_and_round_trip() {
    for value in [ReplicaId::new(), REPLICA_ONE.parse().expect("ReplicaId")] {
        let text = value.to_string();
        assert_eq!(text.parse::<ReplicaId>(), Ok(value));
        assert_eq!(text.len(), 36);
        assert!(text.bytes().all(|byte| !byte.is_ascii_uppercase()));
    }
    assert!(REPLICA_ONE.to_uppercase().parse::<ReplicaId>().is_err());
    assert!(
        "019cc17d-1b22-4a41-9fe9-c345c468f82d"
            .parse::<ReplicaId>()
            .is_err()
    );
}

#[test]
fn complete_and_chunked_representations_bind_complete_resource_data() {
    let complete = ReplicaRepresentation::CompleteObject;
    let chunked = ReplicaRepresentation::Chunked {
        manifest: manifest(),
    };
    let complete_json = serde_json::to_value(&complete).expect("serialize complete representation");
    let chunked_json = serde_json::to_value(&chunked).expect("serialize chunked representation");
    assert_eq!(complete_json, json!({"kind": "complete-object"}));
    assert_eq!(
        chunked_json["kind"], "chunked",
        "the representation kind is explicit"
    );
    let decoded: ReplicaRepresentation = serde_json::from_str(
        &serde_json::to_string(&chunked).expect("serialize chunked representation"),
    )
    .expect("decode chunked representation");
    assert_eq!(decoded, chunked);

    let expected = resource_id();
    let other = RESOURCE_TWO.parse().expect("ResourceId");
    assert!(matches!(
        ReplicaCandidate::new(
            other,
            endpoint_id(),
            decoded,
            locator(json!({"key": "opaque"})),
        ),
        Err(omvcs_model::replica::ReplicaError::ManifestResourceMismatch)
    ));
    assert_eq!(manifest().resource_id(), expected);
}

#[test]
fn chunk_manifest_preserves_order_and_rejects_inconsistent_offsets_or_total() {
    let value = manifest();
    assert_eq!(
        value
            .chunks()
            .iter()
            .map(|chunk| chunk.chunk_id())
            .collect::<Vec<_>>(),
        [
            CHUNK_ONE.parse::<ChunkId>().expect("ChunkId"),
            CHUNK_TWO.parse::<ChunkId>().expect("ChunkId"),
        ]
    );
    let encoded = serde_json::to_string(&value).expect("serialize manifest");
    let decoded: ChunkManifest = serde_json::from_str(&encoded).expect("decode manifest");
    assert_eq!(decoded, value);
    assert!(matches!(
        ChunkManifest::new(
            resource_id(),
            length(5),
            vec![ChunkManifestEntry::new(
                CHUNK_ONE.parse().expect("ChunkId"),
                length(1),
                length(5),
            )],
        ),
        Err(ChunkManifestError::InvalidOffset)
    ));
    assert!(matches!(
        ChunkManifest::new(
            resource_id(),
            length(6),
            vec![ChunkManifestEntry::new(
                CHUNK_ONE.parse().expect("ChunkId"),
                length(0),
                length(5),
            )],
        ),
        Err(ChunkManifestError::TotalLengthMismatch)
    ));
}

#[test]
fn chunked_replica_is_fully_available_only_when_every_manifest_entry_is_available() {
    let manifest = manifest();
    assert!(manifest.is_fully_available(&[
        ReplicaAvailability::Available,
        ReplicaAvailability::Available,
    ]));
    assert!(!manifest.is_fully_available(&[
        ReplicaAvailability::Available,
        ReplicaAvailability::TemporarilyUnavailable,
    ]));
    assert!(
        !manifest
            .is_fully_available(&[ReplicaAvailability::Available, ReplicaAvailability::Unknown,])
    );
    assert!(!manifest.is_fully_available(&[ReplicaAvailability::Available]));
    assert!(!manifest.is_fully_available(&[
        ReplicaAvailability::Available,
        ReplicaAvailability::Available,
        ReplicaAvailability::Available,
    ]));
}

#[test]
fn provider_locator_is_opaque_and_round_trips_as_canonical_json() {
    let value = json!({"z": [true, null], "a": {"y": 2, "x": 1}});
    let locator = ProviderLocator::new("provider.chunk-locator/7", value.clone())
        .expect("provider-owned JSON value");
    assert_eq!(locator.schema(), "provider.chunk-locator/7");
    assert_eq!(locator.value(), &value);
    assert_eq!(
        locator.canonical_bytes().expect("JCS bytes"),
        br#"{"schema":"provider.chunk-locator/7","value":{"a":{"x":1,"y":2},"z":[true,null]}}"#
    );

    let encoded = locator.canonical_bytes().expect("canonical locator");
    let decoded: ProviderLocator =
        serde_json::from_slice(&encoded).expect("decode canonical locator");
    assert_eq!(decoded, locator);
    assert_eq!(decoded.canonical_bytes().expect("re-encode"), encoded);
    assert!(matches!(
        ProviderLocator::new("", json!({})),
        Err(ProviderLocatorError::EmptySchema)
    ));
}

#[test]
fn provider_locator_deserialization_rejects_duplicate_members_and_extensions() {
    assert!(
        serde_json::from_str::<ProviderLocator>(
            r#"{"schema":"provider.locator/1","value":{"k":1,"k":2}}"#
        )
        .is_err()
    );
    assert!(
        serde_json::from_str::<ProviderLocator>(
            r#"{"schema":"provider.locator/1","value":{},"extension":true}"#
        )
        .is_err()
    );
}

#[test]
fn multiple_independently_addressable_representations_share_resource_and_endpoint() {
    let id = resource_id();
    let first = persisted_replica(
        REPLICA_ONE,
        id,
        endpoint_id(),
        &ReplicaRepresentation::CompleteObject,
        &locator(json!({"object": "one"})),
        ReplicaAvailability::Unknown,
    );
    let second = persisted_replica(
        REPLICA_TWO,
        id,
        endpoint_id(),
        &ReplicaRepresentation::Chunked {
            manifest: manifest(),
        },
        &locator(json!({"chunks": ["one", "two"]})),
        ReplicaAvailability::Unknown,
    );
    assert_ne!(first.id(), second.id());
    assert_eq!(first.resource_id(), second.resource_id());
    assert_eq!(first.endpoint_id(), second.endpoint_id());
    assert_eq!(first.availability(), ReplicaAvailability::Unknown);
    let decoded: Replica =
        serde_json::from_str(&serde_json::to_string(&second).expect("serialize Replica"))
            .expect("decode Replica");
    assert_eq!(decoded, second);
}

#[test]
fn candidate_representation_is_not_a_registered_replica() {
    let candidate = ReplicaCandidate::new(
        resource_id(),
        endpoint_id(),
        ReplicaRepresentation::Chunked {
            manifest: manifest(),
        },
        locator(json!({"chunks": ["one", "two"]})),
    )
    .expect("valid candidate");

    assert_eq!(candidate.resource_id(), resource_id());
    assert_eq!(candidate.endpoint_id(), endpoint_id());
    assert_eq!(
        candidate.representation(),
        &ReplicaRepresentation::Chunked {
            manifest: manifest()
        }
    );
    assert_eq!(
        candidate.locator(),
        &locator(json!({"chunks": ["one", "two"]}))
    );
}

#[test]
fn replica_persistence_bytes_canonicalize_nested_provider_locator() {
    let record = persisted_replica(
        REPLICA_ONE,
        resource_id(),
        endpoint_id(),
        &ReplicaRepresentation::CompleteObject,
        &locator(json!({"z": 1, "a": 2})),
        ReplicaAvailability::Available,
    );

    let bytes = record
        .canonical_bytes()
        .expect("canonical persisted record");
    let canonical: Value = serde_json::from_slice(&bytes).expect("canonical JSON record");
    assert_eq!(
        serde_jcs::to_vec(&canonical["locator"]).expect("canonical locator bytes"),
        br#"{"schema":"provider.object-locator/1","value":{"a":2,"z":1}}"#
    );
    let restored: Replica =
        serde_json::from_slice(&bytes).expect("restore canonical persisted record");
    assert_eq!(restored, record);
    assert_eq!(
        bytes,
        serde_jcs::to_vec(&record).expect("JCS serialization is deterministic")
    );
}

#[test]
fn equal_locator_values_at_different_endpoints_do_not_establish_shared_identity() {
    let same_provider_value = locator(json!({"object_key": "item-7"}));
    let first = persisted_replica(
        REPLICA_ONE,
        resource_id(),
        endpoint_id(),
        &ReplicaRepresentation::CompleteObject,
        &same_provider_value,
        ReplicaAvailability::Available,
    );
    let second = persisted_replica(
        REPLICA_TWO,
        resource_id(),
        "019cc17d-1b22-7a41-9fe9-c345c468f82f"
            .parse()
            .expect("EndpointId"),
        &ReplicaRepresentation::CompleteObject,
        &same_provider_value,
        ReplicaAvailability::Available,
    );

    assert_eq!(first.locator(), second.locator());
    assert_ne!(first.endpoint_id(), second.endpoint_id());
    assert_ne!(first.id(), second.id());
}

#[test]
fn operational_location_and_retrieval_state_do_not_enter_historical_identity() {
    let bytes = b"historical resource bytes";
    let resource = ResourceObject::new(bytes.as_slice());
    let resource_identity = resource.resource_id();
    let reference = resource.reference().expect("Resource Reference");
    let historical_schema = MetadataSchema::structure([
        ("resource_id", MetadataSchema::Scalar),
        ("byte_length", MetadataSchema::Scalar),
    ]);
    let historical_body = serde_json::to_vec(&json!({
        "resource_id": reference.resource_id().to_string(),
        "byte_length": reference.byte_length().get(),
    }))
    .expect("reference body");
    let historical_bytes =
        canonicalize_metadata_body(&historical_body, &historical_schema).expect("canonical body");
    let historical_revision_id = hash_revision_metadata(&historical_bytes);
    let initial_hash = hash_resource_bytes(resource.bytes());

    let initial = persisted_replica(
        REPLICA_ONE,
        resource_identity,
        endpoint_id(),
        &ReplicaRepresentation::CompleteObject,
        &locator(json!({"key": "before"})),
        ReplicaAvailability::Unknown,
    );
    let moved = initial
        .with_locator(locator(json!({"key": "after"})))
        .with_availability(ReplicaAvailability::TemporarilyUnavailable);
    let other_endpoint = persisted_replica(
        REPLICA_TWO,
        resource_identity,
        "019cc17d-1b22-7a41-9fe9-c345c468f82f"
            .parse()
            .expect("EndpointId"),
        &ReplicaRepresentation::CompleteObject,
        &locator(json!({"key": "elsewhere"})),
        ReplicaAvailability::Available,
    );

    assert_eq!(initial.id(), moved.id());
    assert_eq!(initial.id(), REPLICA_ONE.parse().expect("ReplicaId"));
    assert_ne!(initial.endpoint_id(), other_endpoint.endpoint_id());
    assert_eq!(moved.resource_id(), resource_identity);
    assert_eq!(other_endpoint.resource_id(), resource_identity);
    assert_eq!(hash_resource_bytes(resource.bytes()), initial_hash);
    assert_eq!(resource.resource_id(), resource_identity);
    assert_eq!(
        canonicalize_metadata_body(&historical_body, &historical_schema).expect("same body"),
        historical_bytes
    );
    assert_eq!(
        hash_revision_metadata(&historical_bytes),
        historical_revision_id
    );

    assert_eq!(hash_resource_bytes(resource.bytes()), initial_hash);
    assert_eq!(
        hash_revision_metadata(&historical_bytes),
        historical_revision_id
    );
}
