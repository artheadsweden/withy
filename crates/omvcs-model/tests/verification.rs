#![allow(clippy::expect_used)]

use std::io::{self, Cursor, Read};

use omvcs_model::hashing::hash_resource_bytes;
use omvcs_model::replica::{
    ChunkManifest, ChunkManifestEntry, ChunkManifestError, ProviderLocator, ReplicaCandidate,
    ReplicaRepresentation,
};
use omvcs_model::resource::{ResourceByteLength, ResourceObject};
use omvcs_model::verification::{
    CHUNK_TARGET_SIZE, ProviderChecksumAlgorithm, ProviderChecksumScope, VerificationMethod,
    VerificationOutcome, VerificationStrength, VerificationSubject, chunk_resource,
    verify_candidate_chunk_reader, verify_candidate_resource_reader, verify_chunked_destination,
    verify_resource_reader, verify_resource_reader_with_manifest,
};
use omvcs_model::{ChunkId, ResourceId, StorageEndpointId};
use serde_json::json;

const ENDPOINT_SOURCE: &str = "019cc17d-1b22-7a41-9fe9-c345c468f82c";
const ENDPOINT_DESTINATION: &str = "019cc17d-1b22-7a41-9fe9-c345c468f82d";

fn length(value: usize) -> ResourceByteLength {
    ResourceByteLength::new(value as u64).expect("test length is in range")
}

fn endpoint(text: &str) -> StorageEndpointId {
    text.parse().expect("valid Endpoint ID")
}

fn locator(key: &str) -> ProviderLocator {
    ProviderLocator::new("test.locator/1", json!({"key": key})).expect("valid locator")
}

fn chunked_candidate(
    resource_id: ResourceId,
    manifest: ChunkManifest,
    endpoint_id: StorageEndpointId,
    key: &str,
) -> ReplicaCandidate {
    ReplicaCandidate::new(
        resource_id,
        endpoint_id,
        ReplicaRepresentation::Chunked { manifest },
        locator(key),
    )
    .expect("candidate representation matches Resource")
}

struct Unavailable;

impl Read for Unavailable {
    fn read(&mut self, _buffer: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::new(io::ErrorKind::NotConnected, "unavailable"))
    }
}

struct PartialUnavailable {
    prefix: Cursor<&'static [u8]>,
}

impl Read for PartialUnavailable {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if self.prefix.position()
            < u64::try_from(self.prefix.get_ref().len()).expect("test slice fits u64")
        {
            self.prefix.read(buffer)
        } else {
            Err(io::Error::new(io::ErrorKind::NotConnected, "unavailable"))
        }
    }
}

#[test]
fn chunk_boundaries_are_deterministic_and_resource_identity_is_layout_independent() {
    for size in [
        0,
        1,
        CHUNK_TARGET_SIZE - 1,
        CHUNK_TARGET_SIZE,
        CHUNK_TARGET_SIZE + 1,
        CHUNK_TARGET_SIZE * 2 + 17,
    ] {
        let bytes = vec![0x5a; size];
        let manifest = chunk_resource(&bytes).expect("Resource length is valid");
        if size == 1 {
            let repeated = chunk_resource(&bytes).expect("same bytes have same manifest");
            assert_eq!(manifest, repeated);
        }
        assert_eq!(manifest.total_length(), length(size));
        let expected_count = if size == 0 {
            1
        } else {
            size.div_ceil(CHUNK_TARGET_SIZE)
        };
        assert_eq!(manifest.chunks().len(), expected_count);

        let mut expected_offset = 0;
        for (index, entry) in manifest.chunks().iter().enumerate() {
            assert_eq!(entry.offset(), length(expected_offset));
            let expected_length = if size == 0 {
                0
            } else {
                (size - expected_offset).min(CHUNK_TARGET_SIZE)
            };
            assert_eq!(entry.length(), length(expected_length));
            if index + 1 != expected_count {
                assert_eq!(entry.length(), length(CHUNK_TARGET_SIZE));
            }
            expected_offset += expected_length;
        }
        assert_eq!(expected_offset, size);
        assert_eq!(
            ResourceObject::new(bytes).resource_id(),
            manifest.resource_id()
        );
    }
}

#[test]
fn zero_byte_chunked_resource_has_one_empty_chunk() {
    let manifest = chunk_resource(&[]).expect("empty Resource is valid");
    assert_eq!(manifest.chunks().len(), 1);
    assert_eq!(manifest.chunks()[0].offset(), length(0));
    assert_eq!(manifest.chunks()[0].length(), length(0));
    assert_eq!(
        manifest.chunks()[0].chunk_id(),
        ChunkId::from_digest(sha256(&[]))
    );
}

#[test]
fn chunk_manifests_reject_missing_empty_chunk_and_nonconforming_splits() {
    let empty_id = hash_resource_bytes(b"");
    assert!(matches!(
        ChunkManifest::new(empty_id, length(0), vec![]),
        Err(ChunkManifestError::InvalidChunkingPolicy)
    ));
    assert!(
        serde_json::from_value::<ChunkManifest>(json!({
            "resource_id": empty_id.to_string(),
            "total_length": 0,
            "chunks": [],
        }))
        .is_err(),
        "deserialization must apply the same chunking-policy validation"
    );

    let resource_id = ResourceId::from_digest([0x43; 32]);
    assert!(matches!(
        ChunkManifest::new(
            resource_id,
            length(CHUNK_TARGET_SIZE + 1),
            vec![ChunkManifestEntry::new(
                ChunkId::from_digest([0x42; 32]),
                length(0),
                length(CHUNK_TARGET_SIZE + 1),
            )],
        ),
        Err(ChunkManifestError::InvalidChunkingPolicy)
    ));
    assert!(
        serde_json::from_value::<ChunkManifest>(json!({
            "resource_id": resource_id.to_string(),
            "total_length": CHUNK_TARGET_SIZE + 1,
            "chunks": [{
                "chunk_id": ChunkId::from_digest([0x42; 32]).to_string(),
                "offset": 0,
                "length": CHUNK_TARGET_SIZE + 1,
            }],
        }))
        .is_err(),
        "deserialization cannot admit an oversized Chunk"
    );
    assert!(matches!(
        ChunkManifest::new(
            empty_id,
            length(0),
            vec![
                ChunkManifestEntry::new(ChunkId::from_digest([0x41; 32]), length(0), length(0),),
                ChunkManifestEntry::new(ChunkId::from_digest([0x42; 32]), length(0), length(0),),
            ],
        ),
        Err(ChunkManifestError::InvalidChunkingPolicy)
    ));

    let complete_empty = ReplicaCandidate::new(
        empty_id,
        endpoint(ENDPOINT_DESTINATION),
        ReplicaRepresentation::CompleteObject,
        locator("complete-empty"),
    )
    .expect("complete-object empty Resource remains valid");
    assert_eq!(complete_empty.resource_id(), empty_id);
}

#[test]
fn direct_resource_and_chunk_checks_distinguish_verified_failed_and_indeterminate() {
    let bytes = b"complete resource";
    let resource_id = hash_resource_bytes(bytes);
    let mut correct = Cursor::new(bytes.as_slice());
    let result = verify_resource_reader(resource_id, &mut correct).expect("valid size");
    assert_eq!(result.outcome(), VerificationOutcome::Verified);
    assert_eq!(
        result.established_strength(),
        Some(VerificationStrength::ResourceIdentity)
    );

    let mut wrong = Cursor::new(bytes.as_slice());
    let mismatch =
        verify_resource_reader(hash_resource_bytes(b"other"), &mut wrong).expect("valid size");
    assert_eq!(mismatch.outcome(), VerificationOutcome::Failed);
    assert_eq!(mismatch.established_strength(), None);

    let unavailable = verify_resource_reader(resource_id, &mut Unavailable).expect("valid size");
    assert_eq!(unavailable.outcome(), VerificationOutcome::Indeterminate);
    let mut partial = PartialUnavailable {
        prefix: Cursor::new(b"partial"),
    };
    assert_eq!(
        verify_resource_reader(resource_id, &mut partial)
            .expect("valid byte-length domain")
            .outcome(),
        VerificationOutcome::Indeterminate
    );

    let manifest = chunk_resource(bytes).expect("valid Resource");
    let entry = manifest.chunks()[0];
    let candidate = chunked_candidate(
        resource_id,
        manifest,
        endpoint(ENDPOINT_DESTINATION),
        "destination",
    );
    let mut correct_chunk = Cursor::new(bytes.as_slice());
    let result = verify_candidate_chunk_reader(&candidate, entry, &mut correct_chunk);
    assert_eq!(result.outcome(), VerificationOutcome::Verified);
    assert_eq!(result.strength(), VerificationStrength::ChunkIdentity);

    let mut corrupt_chunk = Cursor::new(b"corrupted bytes".as_slice());
    let result = verify_candidate_chunk_reader(&candidate, entry, &mut corrupt_chunk);
    assert_eq!(result.outcome(), VerificationOutcome::Failed);
    assert_eq!(
        result.subject(),
        VerificationSubject::Chunk(entry.chunk_id())
    );

    let result = verify_candidate_chunk_reader(&candidate, entry, &mut Unavailable);
    assert_eq!(result.outcome(), VerificationOutcome::Indeterminate);
}

#[test]
fn direct_resource_verification_is_bound_to_the_exact_replica_candidate() {
    let bytes = b"bound destination";
    let resource_id = hash_resource_bytes(bytes);
    let candidate = ReplicaCandidate::new(
        resource_id,
        endpoint(ENDPOINT_DESTINATION),
        ReplicaRepresentation::CompleteObject,
        locator("destination"),
    )
    .expect("complete candidate");
    let mut stream = Cursor::new(bytes.as_slice());
    let result = verify_candidate_resource_reader(&candidate, &mut stream).expect("valid size");
    assert_eq!(result.outcome(), VerificationOutcome::Verified);
    assert_eq!(result.method(), VerificationMethod::DirectByteReadHash);
    assert!(result.promotion_eligibility(&candidate).is_some());

    let moved_candidate = ReplicaCandidate::new(
        resource_id,
        endpoint(ENDPOINT_SOURCE),
        ReplicaRepresentation::CompleteObject,
        locator("source"),
    )
    .expect("source candidate");
    assert!(result.promotion_eligibility(&moved_candidate).is_none());
}

#[test]
fn verified_stream_builds_complete_ordered_manifest_and_destination_reconstruction() {
    let size = CHUNK_TARGET_SIZE + 29;
    let bytes = vec![0x31; size];
    let resource_id = hash_resource_bytes(&bytes);
    let mut source = Cursor::new(bytes.as_slice());
    let source_verification =
        verify_resource_reader_with_manifest(resource_id, &mut source).expect("valid length");
    assert_eq!(
        source_verification.result().outcome(),
        VerificationOutcome::Verified
    );
    let proof = source_verification
        .verified_manifest()
        .expect("verified Resource yields complete manifest proof");
    assert_eq!(proof.manifest().chunks().len(), 2);
    assert_eq!(
        proof.manifest().chunks()[0].length(),
        length(CHUNK_TARGET_SIZE)
    );
    assert_eq!(proof.manifest().chunks()[1].length(), length(29));
    assert!(
        source_verification
            .result()
            .evidence()
            .manifest_binding()
            .is_some()
    );

    let candidate = chunked_candidate(
        resource_id,
        proof.manifest().clone(),
        endpoint(ENDPOINT_DESTINATION),
        "destination",
    );
    let mut chunk_results = Vec::new();
    for entry in proof.manifest().chunks() {
        let start = usize::try_from(entry.offset().get()).expect("test offset fits platform");
        let end = start + usize::try_from(entry.length().get()).expect("test length fits platform");
        let mut chunk = Cursor::new(&bytes[start..end]);
        chunk_results.push(verify_candidate_chunk_reader(
            &candidate, *entry, &mut chunk,
        ));
    }
    let reconstructed = verify_chunked_destination(&candidate, Some(proof), &chunk_results);
    assert_eq!(reconstructed.outcome(), VerificationOutcome::Verified);
    assert_eq!(
        reconstructed.method(),
        VerificationMethod::DeterministicChunkReconstruction
    );
    assert!(reconstructed.promotion_eligibility(&candidate).is_some());

    assert_eq!(
        verify_chunked_destination(&candidate, Some(proof), &chunk_results[..1]).outcome(),
        VerificationOutcome::Indeterminate
    );
}

#[test]
fn reconstruction_rejects_unavailable_stale_wrong_subject_and_reordered_evidence() {
    let size = CHUNK_TARGET_SIZE + 3;
    let bytes = vec![0x72; size];
    let resource_id = hash_resource_bytes(&bytes);
    let mut source = Cursor::new(bytes.as_slice());
    let proof = verify_resource_reader_with_manifest(resource_id, &mut source)
        .expect("valid length")
        .verified_manifest()
        .cloned()
        .expect("verified proof");
    let destination = chunked_candidate(
        resource_id,
        proof.manifest().clone(),
        endpoint(ENDPOINT_DESTINATION),
        "destination",
    );
    let source_candidate = chunked_candidate(
        resource_id,
        proof.manifest().clone(),
        endpoint(ENDPOINT_SOURCE),
        "source",
    );
    let mut source_results = Vec::new();
    let mut destination_results = Vec::new();
    for entry in proof.manifest().chunks() {
        let start = usize::try_from(entry.offset().get()).expect("test offset fits platform");
        let end = start + usize::try_from(entry.length().get()).expect("test length fits platform");
        let mut source_chunk = Cursor::new(&bytes[start..end]);
        source_results.push(verify_candidate_chunk_reader(
            &source_candidate,
            *entry,
            &mut source_chunk,
        ));
        let mut dest_chunk = Cursor::new(&bytes[start..end]);
        destination_results.push(verify_candidate_chunk_reader(
            &destination,
            *entry,
            &mut dest_chunk,
        ));
    }
    assert_eq!(
        verify_chunked_destination(&destination, Some(&proof), &source_results).outcome(),
        VerificationOutcome::Indeterminate
    );
    assert_eq!(
        verify_chunked_destination(&destination, None, &destination_results).outcome(),
        VerificationOutcome::Indeterminate
    );
    let mut unavailable = destination_results.clone();
    unavailable[1] =
        verify_candidate_chunk_reader(&destination, proof.manifest().chunks()[1], &mut Unavailable);
    assert_eq!(
        verify_chunked_destination(&destination, Some(&proof), &unavailable).outcome(),
        VerificationOutcome::Indeterminate
    );
    let mut corrupt = destination_results.clone();
    let mut wrong_bytes = Cursor::new(b"bad".as_slice());
    corrupt[1] =
        verify_candidate_chunk_reader(&destination, proof.manifest().chunks()[1], &mut wrong_bytes);
    let corrupt_result = verify_chunked_destination(&destination, Some(&proof), &corrupt);
    assert_eq!(corrupt_result.outcome(), VerificationOutcome::Failed);
    assert!(corrupt_result.promotion_eligibility(&destination).is_none());

    let mut wrong_subject = destination_results.clone();
    wrong_subject[1] = wrong_subject[0].clone();
    assert_eq!(
        verify_chunked_destination(&destination, Some(&proof), &wrong_subject).outcome(),
        VerificationOutcome::Indeterminate
    );

    let mut reordered = destination_results.clone();
    reordered.swap(0, 1);
    assert_eq!(
        verify_chunked_destination(&destination, Some(&proof), &reordered).outcome(),
        VerificationOutcome::Failed
    );
}

#[test]
fn provider_checksum_strength_and_exact_scope_must_match() {
    let bytes = b"provider scoped resource";
    let resource_id = hash_resource_bytes(bytes);
    let candidate = ReplicaCandidate::new(
        resource_id,
        endpoint(ENDPOINT_DESTINATION),
        ReplicaRepresentation::CompleteObject,
        locator("checksum-target"),
    )
    .expect("candidate");
    let exact = omvcs_model::verification::verify_candidate_resource_checksum(
        &candidate,
        ProviderChecksumAlgorithm::OmvcsSha256,
        ProviderChecksumScope::CompleteResourceBytes,
        *resource_id.digest(),
    );
    assert_eq!(exact.outcome(), VerificationOutcome::Verified);
    assert_eq!(
        exact.method(),
        VerificationMethod::ProviderEquivalentChecksum
    );
    assert_eq!(
        omvcs_model::verification::verify_candidate_resource_checksum(
            &candidate,
            ProviderChecksumAlgorithm::OmvcsSha256,
            ProviderChecksumScope::ExactChunkBytes,
            *resource_id.digest(),
        )
        .outcome(),
        VerificationOutcome::Indeterminate
    );
    assert_eq!(
        omvcs_model::verification::verify_candidate_resource_checksum(
            &candidate,
            ProviderChecksumAlgorithm::Other,
            ProviderChecksumScope::CompleteResourceBytes,
            *resource_id.digest(),
        )
        .outcome(),
        VerificationOutcome::Indeterminate
    );
    assert_eq!(
        omvcs_model::verification::verify_candidate_resource_checksum(
            &candidate,
            ProviderChecksumAlgorithm::OmvcsSha256,
            ProviderChecksumScope::CompleteResourceBytes,
            *hash_resource_bytes(b"wrong").digest(),
        )
        .outcome(),
        VerificationOutcome::Failed
    );
}

#[test]
fn verification_metadata_is_not_part_of_historical_resource_identity() {
    let bytes = b"identity excludes all verification operations";
    let original = ResourceObject::new(bytes.to_vec());
    let id_before = original.resource_id();
    let mut stream = Cursor::new(bytes.as_slice());
    let evidence = verify_resource_reader(id_before, &mut stream).expect("valid length");
    assert_eq!(evidence.outcome(), VerificationOutcome::Verified);
    assert_eq!(ResourceObject::new(bytes.to_vec()).resource_id(), id_before);
    assert_eq!(hash_resource_bytes(bytes), id_before);
}

fn sha256(bytes: &[u8]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes).into()
}
