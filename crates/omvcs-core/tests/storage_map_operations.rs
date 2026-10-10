#![allow(clippy::expect_used)]

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier, Mutex};
use std::thread;

use omvcs_core::storage_map::{
    ConditionalWriteOutcome, PersistedStorageMapData, StorageMapGeneration,
    StorageMapGenerationError, StorageMapMutation, StorageMapMutationOutcome,
    StorageMapPersistence, StorageMapPersistenceFailure, StorageMapSnapshot,
    apply_storage_map_mutation,
};
use omvcs_model::hashing::{hash_resource_bytes, hash_revision_metadata};
use omvcs_model::replica::{ProviderLocator, Replica, ReplicaAvailability};
use omvcs_model::resource::ResourceObject;
use omvcs_model::{ProjectId, ReplicaId, ResourceId, StorageEndpointId};
use serde_json::json;

#[derive(Debug, Default)]
struct TestRepository {
    projects: Mutex<BTreeMap<ProjectId, PersistedStorageMapData>>,
}

impl TestRepository {
    fn new() -> Self {
        Self::default()
    }

    fn initialize_empty(&self, project_id: ProjectId) -> Result<(), &'static str> {
        let mut projects = self
            .projects
            .lock()
            .map_err(|_| "test repository lock poisoned")?;
        if projects.contains_key(&project_id) {
            return Err("test project already initialized");
        }
        projects.insert(
            project_id,
            PersistedStorageMapData::new([], StorageMapGeneration::ZERO),
        );
        drop(projects);
        Ok(())
    }

    fn restore_persisted(
        &self,
        project_id: ProjectId,
        persisted: PersistedStorageMapData,
    ) -> Result<(), &'static str> {
        let mut projects = self
            .projects
            .lock()
            .map_err(|_| "test repository lock poisoned")?;
        if projects.contains_key(&project_id) {
            return Err("test project already initialized");
        }
        projects.insert(project_id, persisted);
        drop(projects);
        Ok(())
    }
}

impl StorageMapPersistence for TestRepository {
    fn load(
        &self,
        project_id: ProjectId,
    ) -> Result<PersistedStorageMapData, StorageMapPersistenceFailure> {
        self.projects
            .lock()
            .map_err(|_| StorageMapPersistenceFailure::RepositoryUnavailable)?
            .get(&project_id)
            .cloned()
            .ok_or(StorageMapPersistenceFailure::MissingOperationalMetadata)
    }

    fn compare_and_swap(
        &self,
        project_id: ProjectId,
        expected_generation: StorageMapGeneration,
        replacement: StorageMapSnapshot,
    ) -> Result<ConditionalWriteOutcome, StorageMapPersistenceFailure> {
        let mut projects = self
            .projects
            .lock()
            .map_err(|_| StorageMapPersistenceFailure::RepositoryUnavailable)?;
        let Some(current) = projects.get(&project_id) else {
            return Err(StorageMapPersistenceFailure::MissingOperationalMetadata);
        };
        if current.generation() != expected_generation {
            return Ok(ConditionalWriteOutcome::Conflict {
                observed_generation: Some(current.generation()),
            });
        }
        let Some(expected_next) = expected_generation
            .get()
            .checked_add(1)
            .and_then(|value| StorageMapGeneration::try_from(value).ok())
        else {
            return Err(StorageMapPersistenceFailure::ProviderFailure);
        };
        if replacement.generation() != expected_next {
            return Err(StorageMapPersistenceFailure::ProviderFailure);
        }
        projects.insert(
            project_id,
            PersistedStorageMapData::new(
                replacement.map().replicas().cloned(),
                replacement.generation(),
            ),
        );
        drop(projects);
        Ok(ConditionalWriteOutcome::Committed)
    }
}

struct SynchronizedReads {
    repository: Arc<TestRepository>,
    readers: Barrier,
}

impl StorageMapPersistence for SynchronizedReads {
    fn load(
        &self,
        project_id: ProjectId,
    ) -> Result<PersistedStorageMapData, StorageMapPersistenceFailure> {
        let snapshot = self.repository.load(project_id)?;
        self.readers.wait();
        Ok(snapshot)
    }

    fn compare_and_swap(
        &self,
        project_id: ProjectId,
        expected_generation: StorageMapGeneration,
        replacement: StorageMapSnapshot,
    ) -> Result<ConditionalWriteOutcome, StorageMapPersistenceFailure> {
        self.repository
            .compare_and_swap(project_id, expected_generation, replacement)
    }
}

const PROJECT_A: &str = "019cc17d-1b22-7a41-9fe9-c345c468f82c";
const PROJECT_B: &str = "019cc17d-1b22-7a41-9fe9-c345c468f82d";
const ENDPOINT_A: &str = "019cc17d-1b22-7a41-9fe9-c345c468f82e";
const ENDPOINT_B: &str = "019cc17d-1b22-7a41-9fe9-c345c468f82f";
const REPLICA_A: &str = "019cc17d-1b22-7a41-9fe9-c345c468f820";
const REPLICA_B: &str = "019cc17d-1b22-7a41-9fe9-c345c468f821";
const REPLICA_C: &str = "019cc17d-1b22-7a41-9fe9-c345c468f822";

fn project_a() -> ProjectId {
    PROJECT_A.parse().expect("ProjectId")
}

fn project_b() -> ProjectId {
    PROJECT_B.parse().expect("ProjectId")
}

fn resource_id() -> ResourceId {
    hash_resource_bytes(b"storage-map-test-resource")
}

fn locator(key: &str) -> ProviderLocator {
    ProviderLocator::new("test.provider.locator/1", json!({"opaque": key}))
        .expect("valid generic locator")
}

fn replica(id: &str, endpoint: &str, locator_key: &str) -> Replica {
    serde_json::from_value(json!({
        "replica_id": id,
        "resource_id": resource_id().to_string(),
        "endpoint_id": endpoint,
        "representation": {"kind": "complete-object"},
        "locator": {
            "schema": "test.provider.locator/1",
            "value": {"opaque": locator_key}
        },
        "availability": "available",
    }))
    .expect("decode an already-registered persisted record")
}

fn snapshot(
    records: impl IntoIterator<Item = Replica>,
    generation: u64,
) -> PersistedStorageMapData {
    PersistedStorageMapData::new(
        records,
        StorageMapGeneration::try_from(generation).expect("valid test generation"),
    )
}

fn loaded(repository: &TestRepository, project_id: ProjectId) -> PersistedStorageMapData {
    repository
        .load(project_id)
        .expect("initialized project map")
}

fn persisted_replica(state: &PersistedStorageMapData, id: ReplicaId) -> Option<&Replica> {
    state.replicas().iter().find(|replica| replica.id() == id)
}

#[test]
fn generation_accepts_only_canonical_exact_json_integer_domain() {
    for (text, value) in [
        ("0", 0),
        ("1", 1),
        ("123456789", 123_456_789),
        ("9007199254740991", 9_007_199_254_740_991),
    ] {
        let generation: StorageMapGeneration =
            serde_json::from_str(text).expect("canonical generation");
        assert_eq!(generation.get(), value);
        assert_eq!(
            serde_json::to_string(&generation).expect("serialize generation"),
            text
        );
    }
    for text in [
        "-0",
        "-1",
        "1.0",
        "1e0",
        "\"1\"",
        "9007199254740992",
        "18446744073709551616",
    ] {
        assert!(
            serde_json::from_str::<StorageMapGeneration>(text).is_err(),
            "accepted invalid generation {text}"
        );
    }
    assert!(serde_json::from_str::<StorageMapGeneration>("01").is_err());
    assert_eq!(
        StorageMapGeneration::try_from(u64::MAX),
        Err(StorageMapGenerationError::OutOfRange)
    );
}

#[test]
fn initialized_empty_map_is_generation_zero_but_absent_metadata_is_not() {
    let repository = TestRepository::new();
    assert_eq!(
        repository.load(project_a()),
        Err(StorageMapPersistenceFailure::MissingOperationalMetadata)
    );
    repository
        .initialize_empty(project_a())
        .expect("initialize empty map");
    let state = loaded(&repository, project_a());
    assert_eq!(state.replicas(), []);
    assert_eq!(state.generation(), StorageMapGeneration::ZERO);
    assert!(repository.initialize_empty(project_a()).is_err());
}

#[test]
fn decoding_a_replica_record_alone_does_not_register_or_initialize_a_map() {
    let repository = TestRepository::new();
    let decoded = replica(REPLICA_A, ENDPOINT_A, "decoded-only");
    assert_eq!(decoded.id(), REPLICA_A.parse().expect("ReplicaId"));
    assert_eq!(
        repository.load(project_a()),
        Err(StorageMapPersistenceFailure::MissingOperationalMetadata)
    );
}

#[test]
fn persisted_generation_zero_rejects_a_nonempty_storage_map() {
    let repository = TestRepository::new();
    repository
        .restore_persisted(
            project_a(),
            snapshot([replica(REPLICA_A, ENDPOINT_A, "record")], 0),
        )
        .expect("persist malformed state");
    let mut mutation = StorageMapMutation::new();
    mutation
        .remove_replica(REPLICA_A.parse().expect("ReplicaId"))
        .expect("request removal");
    assert_eq!(
        apply_storage_map_mutation(
            &repository,
            project_a(),
            StorageMapGeneration::ZERO,
            &mutation,
        ),
        StorageMapMutationOutcome::PersistenceFailure(
            StorageMapPersistenceFailure::InvalidPersistedState
        )
    );
}

#[test]
fn map_supports_zero_one_and_multiple_records_at_one_endpoint() {
    let empty = snapshot([], 0);
    assert_eq!(empty.replicas(), []);

    let one = snapshot([replica(REPLICA_A, ENDPOINT_A, "one")], 1);
    assert_eq!(one.replicas().len(), 1);
    let many = snapshot(
        [
            replica(REPLICA_A, ENDPOINT_A, "object"),
            replica(REPLICA_B, ENDPOINT_A, "chunk-layout"),
            replica(REPLICA_C, ENDPOINT_B, "other-endpoint"),
        ],
        2,
    );
    let same_endpoint = many
        .replicas()
        .iter()
        .filter(|record| record.endpoint_id() == ENDPOINT_A.parse().expect("EndpointId"))
        .count();
    assert_eq!(same_endpoint, 2);
    assert_eq!(many.replicas().len(), 3);
}

#[test]
fn multi_entry_update_and_metadata_removal_commit_atomically_once() {
    let repository = TestRepository::new();
    repository
        .restore_persisted(
            project_a(),
            snapshot(
                [
                    replica(REPLICA_A, ENDPOINT_A, "before"),
                    replica(REPLICA_B, ENDPOINT_A, "remove"),
                    replica(REPLICA_C, ENDPOINT_B, "unchanged"),
                ],
                8,
            ),
        )
        .expect("restore complete state");

    let mut mutation = StorageMapMutation::new();
    mutation
        .update_replica(
            REPLICA_A.parse().expect("ReplicaId"),
            Some(locator("after")),
            Some(ReplicaAvailability::TemporarilyUnavailable),
        )
        .expect("update one record");
    mutation
        .remove_replica(REPLICA_B.parse().expect("ReplicaId"))
        .expect("remove another record");

    assert_eq!(
        apply_storage_map_mutation(
            &repository,
            project_a(),
            StorageMapGeneration::try_from(8).expect("generation"),
            &mutation,
        ),
        StorageMapMutationOutcome::Applied {
            generation: StorageMapGeneration::try_from(9).expect("generation"),
        }
    );
    let changed = loaded(&repository, project_a());
    assert_eq!(changed.generation().get(), 9);
    assert_eq!(changed.replicas().len(), 2);
    assert_eq!(
        persisted_replica(&changed, REPLICA_A.parse().expect("ReplicaId"))
            .expect("retained record")
            .locator(),
        &locator("after")
    );
    assert_eq!(
        persisted_replica(&changed, REPLICA_A.parse().expect("ReplicaId"))
            .expect("retained record")
            .availability(),
        ReplicaAvailability::TemporarilyUnavailable
    );
    assert!(persisted_replica(&changed, REPLICA_B.parse().expect("ReplicaId")).is_none());
    assert!(persisted_replica(&changed, REPLICA_C.parse().expect("ReplicaId")).is_some());
}

#[test]
fn stale_request_conflicts_without_retry_or_partial_change() {
    let repository = TestRepository::new();
    repository
        .restore_persisted(
            project_a(),
            snapshot([replica(REPLICA_A, ENDPOINT_A, "before")], 3),
        )
        .expect("restore");
    let before = loaded(&repository, project_a());
    let mut mutation = StorageMapMutation::new();
    mutation
        .update_replica(
            REPLICA_A.parse().expect("ReplicaId"),
            Some(locator("after")),
            None,
        )
        .expect("update");
    let counting = CountingConflictRepository {
        snapshot: before.clone(),
        compare_calls: AtomicUsize::new(0),
    };
    assert_eq!(
        apply_storage_map_mutation(
            &counting,
            project_a(),
            StorageMapGeneration::try_from(2).expect("generation"),
            &mutation,
        ),
        StorageMapMutationOutcome::Conflict {
            observed_generation: Some(StorageMapGeneration::try_from(3).expect("generation")),
        }
    );
    assert_eq!(counting.compare_calls.load(Ordering::SeqCst), 0);
    assert_eq!(loaded(&repository, project_a()), before);
}

#[test]
fn invalid_multi_entry_request_leaves_every_record_and_generation_unchanged() {
    let repository = TestRepository::new();
    repository
        .restore_persisted(
            project_a(),
            snapshot([replica(REPLICA_A, ENDPOINT_A, "before")], 4),
        )
        .expect("restore");
    let before = loaded(&repository, project_a());
    let missing = REPLICA_B.parse::<ReplicaId>().expect("ReplicaId");
    let mut mutation = StorageMapMutation::new();
    mutation
        .update_replica(
            REPLICA_A.parse().expect("ReplicaId"),
            Some(locator("changed")),
            None,
        )
        .expect("valid change");
    mutation
        .remove_replica(missing)
        .expect("request removal of an absent record");
    assert_eq!(
        apply_storage_map_mutation(
            &repository,
            project_a(),
            StorageMapGeneration::try_from(4).expect("generation"),
            &mutation,
        ),
        StorageMapMutationOutcome::Invalid {
            replica_id: missing
        }
    );
    assert_eq!(loaded(&repository, project_a()), before);
}

#[test]
fn concurrent_mutations_from_one_snapshot_have_one_commit_and_one_conflict() {
    let repository = Arc::new(TestRepository::new());
    let project_id = project_a();
    repository
        .restore_persisted(
            project_id,
            snapshot([replica(REPLICA_A, ENDPOINT_A, "initial")], 7),
        )
        .expect("persist initial snapshot");
    let synchronized = Arc::new(SynchronizedReads {
        repository: Arc::clone(&repository),
        readers: Barrier::new(2),
    });

    let first_persistence = Arc::clone(&synchronized);
    let first = thread::spawn(move || {
        let mut mutation = StorageMapMutation::new();
        mutation
            .update_replica(
                REPLICA_A.parse().expect("ReplicaId"),
                Some(locator("first")),
                None,
            )
            .expect("valid first update");
        apply_storage_map_mutation(
            &*first_persistence,
            project_id,
            StorageMapGeneration::try_from(7).expect("generation"),
            &mutation,
        )
    });

    let second_persistence = Arc::clone(&synchronized);
    let second = thread::spawn(move || {
        let mut mutation = StorageMapMutation::new();
        mutation
            .update_replica(
                REPLICA_A.parse().expect("ReplicaId"),
                None,
                Some(ReplicaAvailability::TemporarilyUnavailable),
            )
            .expect("valid second update");
        apply_storage_map_mutation(
            &*second_persistence,
            project_id,
            StorageMapGeneration::try_from(7).expect("generation"),
            &mutation,
        )
    });

    let outcomes = [
        first.join().expect("first mutation thread"),
        second.join().expect("second mutation thread"),
    ];
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| matches!(outcome, StorageMapMutationOutcome::Applied { .. }))
            .count(),
        1
    );
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| matches!(outcome, StorageMapMutationOutcome::Conflict { .. }))
            .count(),
        1
    );
    let final_snapshot = loaded(&repository, project_id);
    assert_eq!(final_snapshot.generation().get(), 8);
    assert_eq!(final_snapshot.replicas().len(), 1);
    let final_record = persisted_replica(&final_snapshot, REPLICA_A.parse().expect("ReplicaId"))
        .expect("the existing record remains");
    let first_update_won = final_record.locator() == &locator("first")
        && final_record.availability() == ReplicaAvailability::Available;
    let second_update_won = final_record.locator() == &locator("initial")
        && final_record.availability() == ReplicaAvailability::TemporarilyUnavailable;
    assert!(
        first_update_won || second_update_won,
        "the final snapshot must equal exactly one committed update"
    );
}

#[test]
fn persisted_storage_map_rejects_duplicate_replica_ids() {
    let repository = TestRepository::new();
    let first = replica(REPLICA_A, ENDPOINT_A, "first");
    let second = replica(REPLICA_A, ENDPOINT_B, "second");
    repository
        .restore_persisted(project_a(), snapshot([first, second], 1))
        .expect("persist duplicate records");
    assert_eq!(
        apply_storage_map_mutation(
            &repository,
            project_a(),
            StorageMapGeneration::try_from(1).expect("generation"),
            &StorageMapMutation::new(),
        ),
        StorageMapMutationOutcome::PersistenceFailure(
            StorageMapPersistenceFailure::InvalidPersistedState
        )
    );
}

#[test]
fn no_op_request_does_not_advance_generation() {
    let repository = TestRepository::new();
    repository
        .restore_persisted(
            project_a(),
            snapshot([replica(REPLICA_A, ENDPOINT_A, "same")], 12),
        )
        .expect("restore");
    let before = loaded(&repository, project_a());

    let mut same_value = StorageMapMutation::new();
    same_value
        .update_replica(
            REPLICA_A.parse().expect("ReplicaId"),
            Some(locator("same")),
            Some(ReplicaAvailability::Available),
        )
        .expect("no logical state change");
    for mutation in [&same_value, &StorageMapMutation::new()] {
        assert_eq!(
            apply_storage_map_mutation(
                &repository,
                project_a(),
                StorageMapGeneration::try_from(12).expect("generation"),
                mutation,
            ),
            StorageMapMutationOutcome::Unchanged {
                generation: StorageMapGeneration::try_from(12).expect("generation"),
            }
        );
        assert_eq!(loaded(&repository, project_a()), before);
    }
}

#[test]
fn no_op_at_maximum_generation_succeeds_without_exhaustion_or_increment() {
    let repository = TestRepository::new();
    let maximum = StorageMapGeneration::try_from(9_007_199_254_740_991).expect("maximum");
    repository
        .restore_persisted(
            project_a(),
            snapshot([replica(REPLICA_A, ENDPOINT_A, "same")], maximum.get()),
        )
        .expect("restore maximum-generation snapshot");
    let before = loaded(&repository, project_a());
    let mut mutation = StorageMapMutation::new();
    mutation
        .update_replica(
            REPLICA_A.parse().expect("ReplicaId"),
            Some(locator("same")),
            None,
        )
        .expect("valid no-op update");

    assert_eq!(
        apply_storage_map_mutation(&repository, project_a(), maximum, &mutation),
        StorageMapMutationOutcome::Unchanged {
            generation: maximum
        }
    );
    assert_eq!(loaded(&repository, project_a()), before);
}

#[test]
fn exhausted_generation_rejects_state_change_without_partial_removal() {
    let repository = TestRepository::new();
    repository
        .restore_persisted(
            project_a(),
            snapshot(
                [replica(REPLICA_A, ENDPOINT_A, "last-copy-record")],
                StorageMapGeneration::MAX.get(),
            ),
        )
        .expect("restore at maximum");
    let before = loaded(&repository, project_a());
    let mut mutation = StorageMapMutation::new();
    mutation
        .remove_replica(REPLICA_A.parse().expect("ReplicaId"))
        .expect("removal request");
    assert_eq!(
        apply_storage_map_mutation(
            &repository,
            project_a(),
            StorageMapGeneration::MAX,
            &mutation,
        ),
        StorageMapMutationOutcome::GenerationExhausted {
            generation: StorageMapGeneration::MAX,
        }
    );
    assert_eq!(loaded(&repository, project_a()), before);
}

#[test]
fn unsupported_and_provider_failures_leave_generation_and_map_unchanged() {
    for failure in [
        StorageMapPersistenceFailure::UnsupportedConditionalAtomicity,
        StorageMapPersistenceFailure::ProviderFailure,
    ] {
        let repository = TestRepository::new();
        repository
            .restore_persisted(
                project_a(),
                snapshot([replica(REPLICA_A, ENDPOINT_A, "before")], 6),
            )
            .expect("restore");
        let before = loaded(&repository, project_a());
        let mut mutation = StorageMapMutation::new();
        mutation
            .update_replica(
                REPLICA_A.parse().expect("ReplicaId"),
                Some(locator("after")),
                None,
            )
            .expect("change");
        let failing = FailingRepository {
            inner: &repository,
            failure,
        };
        assert_eq!(
            apply_storage_map_mutation(
                &failing,
                project_a(),
                StorageMapGeneration::try_from(6).expect("generation"),
                &mutation,
            ),
            StorageMapMutationOutcome::PersistenceFailure(failure)
        );
        assert_eq!(loaded(&repository, project_a()), before);
    }
}

#[test]
fn availability_is_not_corruption_or_verification_and_addition_is_not_exposed() {
    let record = replica(REPLICA_A, ENDPOINT_A, "object");
    assert_eq!(record.availability(), ReplicaAvailability::Available);
    assert!(
        serde_json::to_value(&record)
            .expect("serialize record")
            .get("verified")
            .is_none()
    );
    assert!(serde_json::from_str::<ReplicaAvailability>(r#""corrupt""#).is_err());
    assert!(serde_json::from_str::<ReplicaAvailability>(r#""temporarily_unavailable""#).is_ok());

    // All mutation variants name an already existing Replica; there is no
    // unverified-candidate addition or verified-promotion variant.
    let mut mutation = StorageMapMutation::new();
    mutation
        .update_replica(record.id(), None, Some(ReplicaAvailability::Unknown))
        .expect("operational update only");
    assert!(!mutation.is_empty());
}

#[test]
fn removal_changes_only_map_metadata_and_preserves_immutable_bytes_and_history() {
    let repository = TestRepository::new();
    let bytes = b"immutable resource bytes";
    let resource = ResourceObject::new(bytes.as_slice());
    let resource_identity = resource.resource_id();
    let original_resource_hash = hash_resource_bytes(resource.bytes());
    let revision_identity = hash_revision_metadata(b"unchanged canonical revision bytes");

    repository
        .restore_persisted(
            project_a(),
            snapshot([replica(REPLICA_A, ENDPOINT_A, "object")], 2),
        )
        .expect("restore");
    let mut mutation = StorageMapMutation::new();
    mutation
        .remove_replica(REPLICA_A.parse().expect("ReplicaId"))
        .expect("remove metadata record");
    assert!(matches!(
        apply_storage_map_mutation(
            &repository,
            project_a(),
            StorageMapGeneration::try_from(2).expect("generation"),
            &mutation,
        ),
        StorageMapMutationOutcome::Applied { .. }
    ));
    assert_eq!(loaded(&repository, project_a()).replicas(), []);

    assert_eq!(resource.bytes(), bytes);
    assert_eq!(resource.resource_id(), resource_identity);
    assert_eq!(
        hash_resource_bytes(resource.bytes()),
        original_resource_hash
    );
    assert_eq!(
        hash_revision_metadata(b"unchanged canonical revision bytes"),
        revision_identity
    );
}

#[test]
fn generations_are_project_scoped_and_do_not_encode_provider_tokens() {
    let repository = TestRepository::new();
    repository
        .restore_persisted(
            project_a(),
            snapshot([replica(REPLICA_A, ENDPOINT_A, "A")], 1),
        )
        .expect("restore A");
    repository
        .restore_persisted(
            project_b(),
            snapshot([replica(REPLICA_B, ENDPOINT_A, "B")], 1),
        )
        .expect("restore B");
    let mut mutation = StorageMapMutation::new();
    mutation
        .update_replica(
            REPLICA_A.parse().expect("ReplicaId"),
            Some(locator("A-updated")),
            None,
        )
        .expect("change Project A map");
    assert_eq!(
        apply_storage_map_mutation(
            &repository,
            project_a(),
            StorageMapGeneration::try_from(1).expect("generation"),
            &mutation,
        ),
        StorageMapMutationOutcome::Applied {
            generation: StorageMapGeneration::try_from(2).expect("generation"),
        }
    );
    assert_eq!(loaded(&repository, project_a()).generation().get(), 2);
    assert_eq!(
        loaded(&repository, project_b()).generation(),
        StorageMapGeneration::try_from(1).expect("generation")
    );
}

struct CountingConflictRepository {
    snapshot: PersistedStorageMapData,
    compare_calls: AtomicUsize,
}

impl StorageMapPersistence for CountingConflictRepository {
    fn load(&self, _: ProjectId) -> Result<PersistedStorageMapData, StorageMapPersistenceFailure> {
        Ok(self.snapshot.clone())
    }

    fn compare_and_swap(
        &self,
        _: ProjectId,
        _: StorageMapGeneration,
        _: StorageMapSnapshot,
    ) -> Result<ConditionalWriteOutcome, StorageMapPersistenceFailure> {
        self.compare_calls.fetch_add(1, Ordering::SeqCst);
        Ok(ConditionalWriteOutcome::Conflict {
            observed_generation: Some(StorageMapGeneration::try_from(4).expect("generation")),
        })
    }
}

struct FailingRepository<'a> {
    inner: &'a TestRepository,
    failure: StorageMapPersistenceFailure,
}

impl StorageMapPersistence for FailingRepository<'_> {
    fn load(
        &self,
        project_id: ProjectId,
    ) -> Result<PersistedStorageMapData, StorageMapPersistenceFailure> {
        self.inner.load(project_id)
    }

    fn compare_and_swap(
        &self,
        _: ProjectId,
        _: StorageMapGeneration,
        _: StorageMapSnapshot,
    ) -> Result<ConditionalWriteOutcome, StorageMapPersistenceFailure> {
        Err(self.failure)
    }
}

#[test]
fn conditional_write_conflict_after_read_is_returned_without_retry() {
    let snapshot = snapshot([replica(REPLICA_A, ENDPOINT_A, "before")], 3);
    let racing = CountingConflictRepository {
        snapshot,
        compare_calls: AtomicUsize::new(0),
    };
    let mut mutation = StorageMapMutation::new();
    mutation
        .update_replica(
            REPLICA_A.parse().expect("ReplicaId"),
            Some(locator("after")),
            None,
        )
        .expect("update");
    assert_eq!(
        apply_storage_map_mutation(
            &racing,
            project_a(),
            StorageMapGeneration::try_from(3).expect("generation"),
            &mutation,
        ),
        StorageMapMutationOutcome::Conflict {
            observed_generation: Some(StorageMapGeneration::try_from(4).expect("generation")),
        }
    );
    assert_eq!(racing.compare_calls.load(Ordering::SeqCst), 1);
}

#[test]
fn storage_map_mutation_does_not_reassign_replica_identity() {
    let id = REPLICA_A.parse::<ReplicaId>().expect("ReplicaId");
    let record = replica(REPLICA_A, ENDPOINT_A, "before");
    let updated = record
        .with_locator(locator("rekeyed-within-endpoint"))
        .with_availability(ReplicaAvailability::TemporarilyUnavailable);
    assert_eq!(updated.id(), id);
    assert_eq!(updated.resource_id(), resource_id());
    assert_eq!(
        updated.endpoint_id(),
        ENDPOINT_A.parse().expect("EndpointId")
    );
    let different_endpoint = replica(REPLICA_B, ENDPOINT_B, "copy");
    assert_ne!(different_endpoint.id(), id);
    assert_eq!(different_endpoint.resource_id(), resource_id());
}

#[test]
fn historical_identity_is_independent_of_location_availability_preference_and_generation() {
    let resource = ResourceObject::new(b"fixed historical bytes".as_slice());
    let resource_id_before = resource.resource_id();
    let resource_hash_before = hash_resource_bytes(resource.bytes());
    let revision_hash_before = hash_revision_metadata(b"fixed canonical revision body");

    let before = (
        ENDPOINT_A.parse::<StorageEndpointId>().expect("EndpointId"),
        locator("before"),
        ReplicaAvailability::Available,
        Some(REPLICA_A.parse::<ReplicaId>().expect("ReplicaId")),
        StorageMapGeneration::ZERO,
    );
    let after = (
        ENDPOINT_B.parse::<StorageEndpointId>().expect("EndpointId"),
        locator("after"),
        ReplicaAvailability::TemporarilyUnavailable,
        Some(REPLICA_B.parse::<ReplicaId>().expect("ReplicaId")),
        StorageMapGeneration::try_from(37).expect("generation"),
    );
    assert_ne!(before, after);

    assert_eq!(resource.resource_id(), resource_id_before);
    assert_eq!(hash_resource_bytes(resource.bytes()), resource_hash_before);
    assert_eq!(
        hash_revision_metadata(b"fixed canonical revision body"),
        revision_hash_before
    );
}
