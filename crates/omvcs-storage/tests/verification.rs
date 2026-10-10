#![allow(clippy::expect_used)]

use omvcs_model::StorageEndpointId;
use omvcs_model::hashing::hash_resource_bytes;
use omvcs_model::replica::{ProviderLocator, ReplicaCandidate, ReplicaRepresentation};
use omvcs_model::verification::{
    ProviderChecksumAlgorithm, ProviderChecksumScope, VerificationMethod, VerificationOutcome,
    VerificationStrength, chunk_resource,
};
use omvcs_storage::{ProviderChecksumReport, verify_chunk_checksum, verify_resource_checksum};
use serde_json::json;

fn candidate() -> ReplicaCandidate {
    let bytes = b"storage checksum boundary";
    let manifest = chunk_resource(bytes).expect("valid chunk manifest");
    let resource_id = hash_resource_bytes(bytes);
    ReplicaCandidate::new(
        resource_id,
        "019cc17d-1b22-7a41-9fe9-c345c468f82c"
            .parse::<StorageEndpointId>()
            .expect("EndpointId"),
        ReplicaRepresentation::Chunked { manifest },
        ProviderLocator::new("provider.locator/1", json!({"opaque": "key"})).expect("locator"),
    )
    .expect("candidate")
}

#[test]
fn only_exact_omvcs_sha256_byte_scopes_establish_checksum_verification() {
    let candidate = candidate();
    let resource_digest = *candidate.resource_id().digest();
    let resource_result = verify_resource_checksum(
        &candidate,
        ProviderChecksumReport::new(
            ProviderChecksumAlgorithm::OmvcsSha256,
            ProviderChecksumScope::CompleteResourceBytes,
            resource_digest,
        ),
    );
    assert_eq!(resource_result.outcome(), VerificationOutcome::Verified);
    assert_eq!(
        resource_result.strength(),
        VerificationStrength::ResourceIdentity
    );
    assert_eq!(
        resource_result.method(),
        VerificationMethod::ProviderEquivalentChecksum
    );

    assert!(matches!(
        candidate.representation(),
        ReplicaRepresentation::Chunked { .. }
    ));
    let entry = if let ReplicaRepresentation::Chunked { manifest } = candidate.representation() {
        manifest.chunks()[0]
    } else {
        return;
    };
    let chunk_result = verify_chunk_checksum(
        &candidate,
        entry,
        ProviderChecksumReport::new(
            ProviderChecksumAlgorithm::OmvcsSha256,
            ProviderChecksumScope::ExactChunkBytes,
            *entry.chunk_id().digest(),
        ),
    );
    assert_eq!(chunk_result.outcome(), VerificationOutcome::Verified);
    assert_eq!(chunk_result.strength(), VerificationStrength::ChunkIdentity);

    for (algorithm, scope) in [
        (
            ProviderChecksumAlgorithm::Other,
            ProviderChecksumScope::CompleteResourceBytes,
        ),
        (
            ProviderChecksumAlgorithm::OmvcsSha256,
            ProviderChecksumScope::ExactChunkBytes,
        ),
        (
            ProviderChecksumAlgorithm::OmvcsSha256,
            ProviderChecksumScope::Other,
        ),
    ] {
        let rejected = verify_resource_checksum(
            &candidate,
            ProviderChecksumReport::new(algorithm, scope, resource_digest),
        );
        assert_eq!(rejected.outcome(), VerificationOutcome::Indeterminate);
        assert_eq!(rejected.established_strength(), None);
    }

    let false_checksum = verify_chunk_checksum(
        &candidate,
        entry,
        ProviderChecksumReport::new(
            ProviderChecksumAlgorithm::OmvcsSha256,
            ProviderChecksumScope::ExactChunkBytes,
            [0; 32],
        ),
    );
    assert_eq!(false_checksum.outcome(), VerificationOutcome::Failed);
}
