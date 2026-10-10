//! WORK-0018 uses WORK-0017 verification and WORK-0016 CAS unchanged.
//! Core §§8, 32–35, 48–55; Storage §§30–36, 199–205;
//! ADR-0032/0033/0034/0036/0037/0038; INV-RES-001–007,
//! INV-STOR-001–005, INV-INT-001–003.
#![allow(clippy::unwrap_used)]

use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    io::Cursor,
};

use omvcs_core::storage_map::{
    ConditionalWriteOutcome, PersistedStorageMapData, StorageMapGeneration, StorageMapMutation,
    StorageMapMutationOutcome, StorageMapPersistence, StorageMapPersistenceFailure,
    StorageMapSnapshot, apply_storage_map_mutation,
};
use omvcs_model::{
    ProjectId, ReplicaId, StorageEndpointId,
    hashing::{
        hash_component_state_metadata, hash_project_state_metadata, hash_resource_bytes,
        hash_revision_metadata,
    },
    replica::{ProviderLocator, ReplicaAvailability, ReplicaCandidate, ReplicaRepresentation},
    verification::{
        CHUNK_TARGET_SIZE, ProviderChecksumAlgorithm, ProviderChecksumScope, VerificationMethod,
        VerificationOutcome, VerificationResult, VerificationStrength, chunk_resource,
        verify_candidate_chunk_reader, verify_candidate_resource_reader,
        verify_chunked_destination, verify_resource_reader_with_manifest,
    },
};
use omvcs_storage::{
    ByteStorage, ErrorClass, LogicalKey, ObjectId, ObjectRequest, Operation, OperationId,
    ProviderChecksumReport, PutRequest, verify_chunk_checksum, verify_resource_checksum,
};
use omvcs_storage_mock::{Effect, EndpointConfiguration, MockStorage, ScriptStep};
use serde_json::json;

fn endpoint(destination: bool) -> StorageEndpointId {
    if destination {
        "019cc17d-1b22-7a41-9fe9-c345c468f82d"
    } else {
        "019cc17d-1b22-7a41-9fe9-c345c468f82c"
    }
    .parse()
    .unwrap()
}

fn project() -> ProjectId {
    "019cc17d-1b22-7a41-9fe9-c345c468f82e".parse().unwrap()
}

// The integrated WORK-0015 byte seam and WORK-0017 model expose distinct
// ChunkId Rust types with the same normative digest. Bridge in fixtures only.
const fn storage_chunk(id: omvcs_model::ChunkId) -> ObjectId {
    ObjectId::Chunk(omvcs_storage::ChunkId::from_digest(*id.digest()))
}

fn setup() -> MockStorage {
    let mock = MockStorage::new();
    for id in [endpoint(false), endpoint(true)] {
        mock.configure_endpoint(id, EndpointConfiguration::default());
    }
    mock
}

fn object(id: ObjectId, destination: bool, key: &str) -> ObjectRequest<ObjectId> {
    ObjectRequest {
        endpoint: endpoint(destination),
        id,
        logical_key: LogicalKey(key.into()),
        operation_id: OperationId("fixture-call".into()),
    }
}

fn upload(
    mock: &mut MockStorage,
    request: &ObjectRequest<ObjectId>,
    bytes: &[u8],
) -> Result<omvcs_storage::ProviderMetadata, omvcs_storage::StorageError> {
    mock.put(
        &PutRequest {
            object: request.clone(),
            byte_length: bytes.len() as u64,
            expected_hash: *request.id.digest(),
        },
        &mut Cursor::new(bytes),
    )
}

fn candidate(
    bytes: &[u8],
    destination: bool,
    key: &str,
    representation: ReplicaRepresentation,
) -> ReplicaCandidate {
    ReplicaCandidate::new(
        hash_resource_bytes(bytes),
        endpoint(destination),
        representation,
        ProviderLocator::new("test.controlled-locator/1", json!({"key": key})).unwrap(),
    )
    .unwrap()
}

fn inject(
    mock: &MockStorage,
    request: &ObjectRequest<ObjectId>,
    operation: Operation,
    effect: Effect,
) {
    mock.enqueue(ScriptStep {
        endpoint: request.endpoint,
        operation,
        logical_key: request.logical_key.0.clone(),
        effect,
    })
    .unwrap();
}

fn direct(
    mock: &mut MockStorage,
    request: &ObjectRequest<ObjectId>,
    candidate: &ReplicaCandidate,
) -> VerificationResult {
    // Raw ByteStorage streams deliberately expose unchecked adversarial bytes.
    // WORK-0017 performs the identity check; no invented verification taxonomy.
    let mut download = mock.get(request).unwrap();
    verify_candidate_resource_reader(candidate, &mut download.stream).unwrap()
}

/// Test-local coherent persistence boundary, NOT an Adapter implementation or
/// Repository Home conformance claim. Core owns validation/generation changes.
/// This fixture only compares and replaces supplied complete snapshots.
struct RepositoryFixture {
    state: RefCell<PersistedStorageMapData>,
    writes: Cell<usize>,
    failures: RefCell<VecDeque<Result<ConditionalWriteOutcome, StorageMapPersistenceFailure>>>,
    load_failure: Cell<Option<StorageMapPersistenceFailure>>,
}

impl RepositoryFixture {
    fn new() -> Self {
        Self {
            state: RefCell::new(PersistedStorageMapData::new([], StorageMapGeneration::ZERO)),
            writes: Cell::new(0),
            failures: RefCell::new(VecDeque::new()),
            load_failure: Cell::new(None),
        }
    }
    fn snapshot(&self) -> PersistedStorageMapData {
        self.state.borrow().clone()
    }
}

impl StorageMapPersistence for RepositoryFixture {
    fn load(&self, id: ProjectId) -> Result<PersistedStorageMapData, StorageMapPersistenceFailure> {
        assert_eq!(id, project());
        if let Some(failure) = self.load_failure.get() {
            return Err(failure);
        }
        Ok(self.snapshot())
    }
    fn compare_and_swap(
        &self,
        id: ProjectId,
        expected: StorageMapGeneration,
        replacement: StorageMapSnapshot,
    ) -> Result<ConditionalWriteOutcome, StorageMapPersistenceFailure> {
        assert_eq!(id, project());
        self.writes.set(self.writes.get() + 1);
        if let Some(result) = self.failures.borrow_mut().pop_front() {
            return result;
        }
        let mut state = self.state.borrow_mut();
        if state.generation() != expected {
            return Ok(ConditionalWriteOutcome::Conflict {
                observed_generation: Some(state.generation()),
            });
        }
        // The generation increment must have been decided by existing Core.
        assert_eq!(replacement.generation().get(), expected.get() + 1);
        *state = PersistedStorageMapData::new(
            replacement.map().replicas().cloned(),
            replacement.generation(),
        );
        Ok(ConditionalWriteOutcome::Committed)
    }
}

fn stage(
    candidate: ReplicaCandidate,
    result: &VerificationResult,
    mutation: &mut StorageMapMutation,
) -> ReplicaId {
    let eligibility = result.promotion_eligibility(&candidate).unwrap();
    mutation
        .add_replica(candidate, &eligibility, ReplicaAvailability::Available)
        .unwrap()
}

#[test]
fn corrupt_or_interrupted_destination_never_promotes_and_source_is_retained() {
    let bytes = b"only valid source";
    let mut mock = setup();
    let repository = RepositoryFixture::new();
    let source = object(
        ObjectId::Resource(hash_resource_bytes(bytes)),
        false,
        "source",
    );
    let destination = object(source.id, true, "destination");
    upload(&mut mock, &source, bytes).unwrap();
    let source_candidate = candidate(
        bytes,
        false,
        "source",
        ReplicaRepresentation::CompleteObject,
    );
    let destination_candidate = candidate(
        bytes,
        true,
        "destination",
        ReplicaRepresentation::CompleteObject,
    );
    let source_result = direct(&mut mock, &source, &source_candidate);
    let mut add_source = StorageMapMutation::new();
    stage(source_candidate.clone(), &source_result, &mut add_source);
    assert!(matches!(
        apply_storage_map_mutation(
            &repository,
            project(),
            StorageMapGeneration::ZERO,
            &add_source
        ),
        StorageMapMutationOutcome::Applied { .. }
    ));
    let before = repository.snapshot();
    assert!(
        source_result
            .promotion_eligibility(&destination_candidate)
            .is_none()
    );

    // Failure during upload is not a candidate that can be verified/promoted.
    inject(
        &mock,
        &destination,
        Operation::PutResource,
        Effect::FailAfter {
            bytes: 3,
            class: ErrorClass::Quota,
            recoverable: false,
        },
    );
    assert_eq!(
        upload(&mut mock, &destination, bytes).unwrap_err().class,
        ErrorClass::Quota
    );
    assert_eq!(mock.stored_bytes(destination.endpoint, "destination"), None);
    upload(&mut mock, &destination, bytes).unwrap();
    for (effect, outcome) in [
        (
            Effect::Corrupt { offset: 0, xor: 1 },
            VerificationOutcome::Failed,
        ),
        (Effect::Truncate(2), VerificationOutcome::Failed),
        (
            Effect::ReturnBytes(vec![0; bytes.len()]),
            VerificationOutcome::Failed,
        ),
        (
            Effect::FailAfter {
                bytes: bytes.len() as u64,
                class: ErrorClass::Authentication,
                recoverable: true,
            },
            VerificationOutcome::Indeterminate,
        ),
    ] {
        inject(&mock, &destination, Operation::GetResource, effect);
        let result = direct(&mut mock, &destination, &destination_candidate);
        assert_eq!(result.outcome(), outcome);
        assert!(
            result
                .promotion_eligibility(&destination_candidate)
                .is_none()
        );
        assert_eq!(repository.snapshot(), before);
        assert_eq!(mock.stored_bytes(source.endpoint, "source").unwrap(), bytes);
    }
    assert_eq!(
        direct(&mut mock, &source, &source_candidate).outcome(),
        VerificationOutcome::Verified
    );
    assert_eq!(
        direct(&mut mock, &destination, &destination_candidate).outcome(),
        VerificationOutcome::Verified
    );
}

#[test]
fn exact_checksum_scope_verifies_actual_bytes_not_requested_identity_or_stat() {
    let bytes = b"checksum scope";
    let mut mock = setup();
    let resource = object(
        ObjectId::Resource(hash_resource_bytes(bytes)),
        true,
        "complete",
    );
    upload(&mut mock, &resource, bytes).unwrap();
    let complete = candidate(
        bytes,
        true,
        "complete",
        ReplicaRepresentation::CompleteObject,
    );
    let report = mock.checksum_report(&resource).unwrap();
    let verified = verify_resource_checksum(&complete, report);
    assert_eq!(verified.outcome(), VerificationOutcome::Verified);
    assert_eq!(
        verified.method(),
        VerificationMethod::ProviderEquivalentChecksum
    );
    assert!(verified.promotion_eligibility(&complete).is_some());
    for invalid in [
        ProviderChecksumReport::new(
            ProviderChecksumAlgorithm::Other,
            report.scope(),
            report.digest(),
        ),
        ProviderChecksumReport::new(
            report.algorithm(),
            ProviderChecksumScope::Other,
            report.digest(),
        ),
        ProviderChecksumReport::new(
            report.algorithm(),
            ProviderChecksumScope::ExactChunkBytes,
            report.digest(),
        ),
    ] {
        assert_eq!(
            verify_resource_checksum(&complete, invalid).outcome(),
            VerificationOutcome::Indeterminate
        );
    }
    let wrong_candidate = candidate(
        b"different identity",
        true,
        "complete",
        ReplicaRepresentation::CompleteObject,
    );
    let mut wrong_request = resource.clone();
    wrong_request.id = ObjectId::Resource(wrong_candidate.resource_id());
    assert_eq!(
        verify_resource_checksum(
            &wrong_candidate,
            mock.checksum_report(&wrong_request).unwrap()
        )
        .outcome(),
        VerificationOutcome::Failed
    );

    let manifest = chunk_resource(bytes).unwrap();
    let entry = manifest.chunks()[0];
    let chunked = candidate(
        bytes,
        true,
        "chunks",
        ReplicaRepresentation::Chunked { manifest },
    );
    let chunk = object(storage_chunk(entry.chunk_id()), true, "chunk");
    upload(&mut mock, &chunk, bytes).unwrap();
    let chunk_report = mock.checksum_report(&chunk).unwrap();
    let chunk_result = verify_chunk_checksum(&chunked, entry, chunk_report);
    assert_eq!(chunk_result.strength(), VerificationStrength::ChunkIdentity);
    assert_eq!(chunk_result.outcome(), VerificationOutcome::Verified);
    assert!(chunk_result.promotion_eligibility(&chunked).is_none());
    assert_eq!(
        verify_resource_checksum(&complete, chunk_report).outcome(),
        VerificationOutcome::Indeterminate
    );
    inject(&mock, &resource, Operation::StatResource, Effect::Absent);
    assert_eq!(
        mock.checksum_report(&resource).unwrap_err().class,
        ErrorClass::NotFound
    );
    inject(
        &mock,
        &resource,
        Operation::StatResource,
        Effect::Fail {
            class: ErrorClass::Authentication,
            recoverable: true,
        },
    );
    assert_eq!(
        mock.checksum_report(&resource).unwrap_err().class,
        ErrorClass::Authentication
    );
}

#[test]
fn approved_chunking_and_empty_resource_support_multiple_representations_and_replicas() {
    for size in [
        0,
        1,
        CHUNK_TARGET_SIZE - 1,
        CHUNK_TARGET_SIZE,
        CHUNK_TARGET_SIZE + 1,
        2 * CHUNK_TARGET_SIZE + 17,
    ] {
        let bytes = vec![0x31; size];
        let mut mock = setup();
        let resource = object(
            ObjectId::Resource(hash_resource_bytes(&bytes)),
            false,
            "complete",
        );
        upload(&mut mock, &resource, &bytes).unwrap();
        let complete = candidate(
            &bytes,
            false,
            "complete",
            ReplicaRepresentation::CompleteObject,
        );
        let complete_result = direct(&mut mock, &resource, &complete);
        let mut source_stream = mock.get(&resource).unwrap();
        let proof_result =
            verify_resource_reader_with_manifest(complete.resource_id(), &mut source_stream.stream)
                .unwrap();
        let proof = proof_result.verified_manifest().unwrap();
        assert_eq!(*proof.manifest(), chunk_resource(&bytes).unwrap());
        drop(source_stream);
        let chunked = candidate(
            &bytes,
            false,
            "chunks",
            ReplicaRepresentation::Chunked {
                manifest: proof.manifest().clone(),
            },
        );
        let mut results = vec![];
        for (index, entry) in proof.manifest().chunks().iter().enumerate() {
            let start = usize::try_from(entry.offset().get()).unwrap();
            let end = start + usize::try_from(entry.length().get()).unwrap();
            let request = object(
                storage_chunk(entry.chunk_id()),
                false,
                &format!("chunk-{index}"),
            );
            upload(&mut mock, &request, &bytes[start..end]).unwrap();
            let mut stream = mock.get(&request).unwrap();
            results.push(verify_candidate_chunk_reader(
                &chunked,
                *entry,
                &mut stream.stream,
            ));
        }
        if size == 0 {
            assert_eq!(proof.manifest().chunks().len(), 1);
            assert_eq!(proof.manifest().chunks()[0].length().get(), 0);
        }
        let result = verify_chunked_destination(&chunked, Some(proof), &results);
        assert_eq!(result.outcome(), VerificationOutcome::Verified);
        assert_eq!(result.strength(), VerificationStrength::ResourceIdentity);
        let repository = RepositoryFixture::new();
        let mut mutation = StorageMapMutation::new();
        let first = stage(complete, &complete_result, &mut mutation);
        let second = stage(chunked, &result, &mut mutation);
        assert_ne!(first, second);
        assert_eq!(
            apply_storage_map_mutation(
                &repository,
                project(),
                StorageMapGeneration::ZERO,
                &mutation
            ),
            StorageMapMutationOutcome::Applied {
                generation: StorageMapGeneration::try_from(1).unwrap()
            }
        );
        assert_eq!(repository.snapshot().replicas().len(), 2);
        for replica in repository.snapshot().replicas() {
            assert_eq!(replica.resource_id(), hash_resource_bytes(&bytes));
            assert_eq!(replica.endpoint_id(), endpoint(false));
        }
    }
}

#[test]
fn final_chunk_upload_and_read_failures_cannot_establish_complete_destination_assurance() {
    let bytes = vec![0x72; CHUNK_TARGET_SIZE + 3];
    let resource_id = hash_resource_bytes(&bytes);
    let proof = verify_resource_reader_with_manifest(resource_id, &mut Cursor::new(&bytes))
        .unwrap()
        .verified_manifest()
        .unwrap()
        .clone();
    let destination = candidate(
        &bytes,
        true,
        "chunks",
        ReplicaRepresentation::Chunked {
            manifest: proof.manifest().clone(),
        },
    );
    let mut mock = setup();
    let mut results = vec![];
    let mut last_request = None;
    for (index, entry) in proof.manifest().chunks().iter().enumerate() {
        let start = usize::try_from(entry.offset().get()).unwrap();
        let end = start + usize::try_from(entry.length().get()).unwrap();
        let request = object(
            storage_chunk(entry.chunk_id()),
            true,
            &format!("chunk-{index}"),
        );
        if index == 1 {
            inject(
                &mock,
                &request,
                Operation::PutChunk,
                Effect::FailAfter {
                    bytes: 3,
                    class: ErrorClass::Network,
                    recoverable: true,
                },
            );
            assert!(upload(&mut mock, &request, &bytes[start..end]).is_err());
            assert!(mock.get(&request).is_err());
            assert_eq!(
                verify_chunked_destination(&destination, Some(&proof), &results).outcome(),
                VerificationOutcome::Indeterminate
            );
        }
        upload(&mut mock, &request, &bytes[start..end]).unwrap();
        let mut stream = mock.get(&request).unwrap();
        results.push(verify_candidate_chunk_reader(
            &destination,
            *entry,
            &mut stream.stream,
        ));
        last_request = Some(request);
    }
    let last = last_request.unwrap();
    let entry = proof.manifest().chunks()[1];
    for (effect, outcome) in [
        (
            Effect::FailAfter {
                bytes: 3,
                class: ErrorClass::Network,
                recoverable: true,
            },
            VerificationOutcome::Indeterminate,
        ),
        (
            Effect::Corrupt { offset: 1, xor: 1 },
            VerificationOutcome::Failed,
        ),
    ] {
        inject(&mock, &last, Operation::GetChunk, effect);
        let mut stream = mock.get(&last).unwrap();
        let mut injected_results = results.clone();
        injected_results[1] =
            verify_candidate_chunk_reader(&destination, entry, &mut stream.stream);
        let result = verify_chunked_destination(&destination, Some(&proof), &injected_results);
        assert_eq!(result.outcome(), outcome);
        assert!(result.promotion_eligibility(&destination).is_none());
    }
    let mut reordered = results.clone();
    reordered.reverse();
    assert_ne!(
        verify_chunked_destination(&destination, Some(&proof), &reordered).outcome(),
        VerificationOutcome::Verified
    );
    assert_eq!(
        verify_chunked_destination(&destination, None, &results).outcome(),
        VerificationOutcome::Indeterminate
    );
    assert_eq!(
        verify_chunked_destination(&destination, Some(&proof), &results).outcome(),
        VerificationOutcome::Verified
    );
}

#[test]
#[allow(clippy::too_many_lines)] // One continuous CAS failure/retry/exhaustion scenario.
fn core_cas_stale_noop_invalid_exhausted_and_injected_failures_are_atomic() {
    let bytes = b"verified destination for CAS";
    let mut mock = setup();
    let request = object(
        ObjectId::Resource(hash_resource_bytes(bytes)),
        true,
        "verified",
    );
    upload(&mut mock, &request, bytes).unwrap();
    let candidate = candidate(
        bytes,
        true,
        "verified",
        ReplicaRepresentation::CompleteObject,
    );
    let result = direct(&mut mock, &request, &candidate);
    let repository = RepositoryFixture::new();
    let mut addition = StorageMapMutation::new();
    let id = stage(candidate, &result, &mut addition);
    let second_request = object(request.id, true, "second-addressable-copy");
    upload(&mut mock, &second_request, bytes).unwrap();
    let second_candidate = crate::candidate(
        bytes,
        true,
        "second-addressable-copy",
        ReplicaRepresentation::CompleteObject,
    );
    let second_result = direct(&mut mock, &second_request, &second_candidate);
    stage(second_candidate, &second_result, &mut addition);
    for failure in [
        StorageMapPersistenceFailure::ProviderFailure,
        StorageMapPersistenceFailure::UnsupportedConditionalAtomicity,
    ] {
        repository.failures.borrow_mut().push_back(Err(failure));
        let before = repository.snapshot();
        assert_eq!(
            apply_storage_map_mutation(
                &repository,
                project(),
                StorageMapGeneration::ZERO,
                &addition
            ),
            StorageMapMutationOutcome::PersistenceFailure(failure)
        );
        assert_eq!(repository.snapshot(), before);
    }
    let before = repository.snapshot();
    let writes = repository.writes.get();
    repository
        .failures
        .borrow_mut()
        .push_back(Ok(ConditionalWriteOutcome::Conflict {
            observed_generation: Some(StorageMapGeneration::try_from(4).unwrap()),
        }));
    assert!(matches!(
        apply_storage_map_mutation(
            &repository,
            project(),
            StorageMapGeneration::ZERO,
            &addition
        ),
        StorageMapMutationOutcome::Conflict { .. }
    ));
    assert_eq!(
        repository.writes.get(),
        writes + 1,
        "Core does not implicitly retry"
    );
    assert_eq!(repository.snapshot(), before);
    assert!(matches!(
        apply_storage_map_mutation(
            &repository,
            project(),
            StorageMapGeneration::ZERO,
            &addition
        ),
        StorageMapMutationOutcome::Applied { .. }
    ));
    let before = repository.snapshot();
    let writes = repository.writes.get();
    assert!(matches!(
        apply_storage_map_mutation(
            &repository,
            project(),
            StorageMapGeneration::ZERO,
            &addition
        ),
        StorageMapMutationOutcome::Conflict { .. }
    ));
    assert_eq!(repository.writes.get(), writes);
    assert_eq!(repository.snapshot(), before);

    let one = StorageMapGeneration::try_from(1).unwrap();
    let mut invalid_batch = StorageMapMutation::new();
    invalid_batch
        .update_replica(id, None, Some(ReplicaAvailability::Unknown))
        .unwrap();
    invalid_batch
        .update_replica(
            "019cc17d-1b22-7a41-9fe9-c345c468f82f".parse().unwrap(),
            None,
            None,
        )
        .unwrap();
    assert!(matches!(
        apply_storage_map_mutation(&repository, project(), one, &invalid_batch),
        StorageMapMutationOutcome::Invalid { .. }
    ));
    assert_eq!(repository.snapshot(), before);
    assert_eq!(
        apply_storage_map_mutation(&repository, project(), one, &StorageMapMutation::new()),
        StorageMapMutationOutcome::Unchanged { generation: one }
    );
    repository
        .load_failure
        .set(Some(StorageMapPersistenceFailure::RepositoryUnavailable));
    assert!(matches!(
        apply_storage_map_mutation(&repository, project(), one, &invalid_batch),
        StorageMapMutationOutcome::PersistenceFailure(_)
    ));
    repository.load_failure.set(None);
    assert_eq!(repository.snapshot(), before);

    // Restore prior registered records at the boundary value for exhaustion;
    // no new registration or Adapter generation algorithm is invented.
    *repository.state.borrow_mut() =
        PersistedStorageMapData::new(before.replicas().iter().cloned(), StorageMapGeneration::MAX);
    let maximum = repository.snapshot();
    assert_eq!(
        apply_storage_map_mutation(
            &repository,
            project(),
            StorageMapGeneration::MAX,
            &StorageMapMutation::new()
        ),
        StorageMapMutationOutcome::Unchanged {
            generation: StorageMapGeneration::MAX
        }
    );
    let mut change = StorageMapMutation::new();
    change
        .update_replica(id, None, Some(ReplicaAvailability::Unknown))
        .unwrap();
    assert_eq!(
        apply_storage_map_mutation(&repository, project(), StorageMapGeneration::MAX, &change),
        StorageMapMutationOutcome::GenerationExhausted {
            generation: StorageMapGeneration::MAX
        }
    );
    assert_eq!(repository.snapshot(), maximum);
    assert_eq!(
        mock.stored_bytes(request.endpoint, "verified").unwrap(),
        bytes
    );
}

#[test]
fn storage_and_operational_map_changes_preserve_external_history_identity_sentinels() {
    // External canonical-byte sentinels test non-mutation, not object admission.
    let component = br#"{"sentinel":"component"}"#.to_vec();
    let project_state = br#"{"sentinel":"project"}"#.to_vec();
    let revision = br#"{"sentinel":"revision"}"#.to_vec();
    let before = (
        hash_component_state_metadata(&component),
        hash_project_state_metadata(&project_state),
        hash_revision_metadata(&revision),
    );
    let bytes = b"historically referenced immutable bytes";
    let manifest = chunk_resource(bytes).unwrap();
    let chunk_id = manifest.chunks()[0].chunk_id();
    let mut mock = setup();
    let request = object(
        ObjectId::Resource(hash_resource_bytes(bytes)),
        false,
        "source",
    );
    upload(&mut mock, &request, bytes).unwrap();
    let source = candidate(
        bytes,
        false,
        "source",
        ReplicaRepresentation::CompleteObject,
    );
    let result = direct(&mut mock, &request, &source);
    let repository = RepositoryFixture::new();
    let mut addition = StorageMapMutation::new();
    let id = stage(source, &result, &mut addition);
    assert!(matches!(
        apply_storage_map_mutation(
            &repository,
            project(),
            StorageMapGeneration::ZERO,
            &addition,
        ),
        StorageMapMutationOutcome::Applied { .. }
    ));
    let mut update = StorageMapMutation::new();
    update
        .update_replica(
            id,
            Some(
                ProviderLocator::new(
                    "test.controlled-locator/1",
                    json!({"key":"in-context-move"}),
                )
                .unwrap(),
            ),
            Some(ReplicaAvailability::TemporarilyUnavailable),
        )
        .unwrap();
    assert!(matches!(
        apply_storage_map_mutation(
            &repository,
            project(),
            StorageMapGeneration::try_from(1).unwrap(),
            &update
        ),
        StorageMapMutationOutcome::Applied { .. }
    ));
    assert_eq!(repository.snapshot().replicas()[0].id(), id);
    mock.configure_endpoint(
        endpoint(false),
        EndpointConfiguration {
            discovery_failure: Some(ErrorClass::Authentication),
            ..EndpointConfiguration::default()
        },
    );
    assert!(mock.get(&request).is_err());
    assert_eq!(
        (
            hash_component_state_metadata(&component),
            hash_project_state_metadata(&project_state),
            hash_revision_metadata(&revision)
        ),
        before
    );
    assert_eq!(hash_resource_bytes(bytes), manifest.resource_id());
    assert_eq!(
        chunk_resource(bytes).unwrap().chunks()[0].chunk_id(),
        chunk_id
    );
    assert_eq!(
        mock.stored_bytes(request.endpoint, "source").unwrap(),
        bytes
    );
}
