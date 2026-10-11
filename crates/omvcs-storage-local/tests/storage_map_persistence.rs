#![cfg(target_os = "linux")]
#![allow(clippy::unwrap_used)]

use std::{
    fs,
    io::Cursor,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    sync::{Arc, Barrier},
};

use omvcs_core::storage_map::{
    ConditionalWriteOutcome, StorageMapGeneration, StorageMapMutation, StorageMapMutationOutcome,
    StorageMapPersistence, StorageMapPersistenceFailure, apply_storage_map_mutation,
};
use omvcs_model::{
    ProjectId, StorageEndpointId,
    hashing::hash_resource_bytes,
    replica::{ProviderLocator, ReplicaAvailability, ReplicaCandidate, ReplicaRepresentation},
    verification::verify_candidate_resource_reader,
};
use omvcs_storage_local::{
    FilesystemHomeDiscovery, FilesystemHomeInitialization, FilesystemRepositoryHome,
    FilesystemStorageMapPersistence,
};
use serde_json::{Value, json};

static ROOT_ID: AtomicU64 = AtomicU64::new(0);

struct TestRoot(PathBuf);

impl TestRoot {
    fn new() -> Self {
        let unique = ROOT_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "omvcs-storage-map-local-test-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(path.join(".omvcs")).unwrap();
        Self(path)
    }
}

impl Drop for TestRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn project() -> ProjectId {
    "019cc17d-1b22-7a41-9fe9-c345c468f82e".parse().unwrap()
}

fn endpoint() -> StorageEndpointId {
    "019cc17d-1b22-7a41-9fe9-c345c468f82c".parse().unwrap()
}

#[test]
fn initialization_persists_existing_empty_map_at_generation_zero() {
    let root = TestRoot::new();
    let persistence = FilesystemStorageMapPersistence::open(&root.0).unwrap();
    persistence.initialize_empty_map(project()).unwrap();

    let loaded = persistence.load(project()).unwrap();
    assert_eq!(loaded.replicas().len(), 0);
    assert_eq!(loaded.generation(), StorageMapGeneration::ZERO);

    let bytes = fs::read(root.0.join(".omvcs/storage-map.json")).unwrap();
    assert!(bytes.is_ascii());
    assert!(
        String::from_utf8(bytes)
            .unwrap()
            .contains("\"generation\":0")
    );
}

#[test]
fn storage_map_mutation_commits_complete_snapshot_and_generation_once() {
    let root = TestRoot::new();
    let persistence = FilesystemStorageMapPersistence::open(&root.0).unwrap();
    persistence.initialize_empty_map(project()).unwrap();
    FilesystemHomeInitialization::bootstrap(&root.0, project()).unwrap();

    let bytes = b"verified destination bytes";
    let resource_id = hash_resource_bytes(bytes);
    let candidate = ReplicaCandidate::new(
        resource_id,
        endpoint(),
        ReplicaRepresentation::CompleteObject,
        ProviderLocator::new("fixture.locator/1", json!({"key": "resource"})).unwrap(),
    )
    .unwrap();
    let result = verify_candidate_resource_reader(&candidate, &mut Cursor::new(bytes)).unwrap();
    let eligibility = result.promotion_eligibility(&candidate).unwrap();
    let mut mutation = StorageMapMutation::new();
    mutation
        .add_replica(candidate, &eligibility, ReplicaAvailability::Available)
        .unwrap();

    assert!(matches!(
        apply_storage_map_mutation(
            &persistence,
            project(),
            StorageMapGeneration::ZERO,
            &mutation,
        ),
        StorageMapMutationOutcome::Applied { generation }
            if generation.get() == 1
    ));
    let loaded = persistence.load(project()).unwrap();
    assert_eq!(loaded.generation().get(), 1);
    assert_eq!(loaded.replicas().len(), 1);
    assert_eq!(
        FilesystemHomeInitialization::discover(&root.0, Some(project())).unwrap(),
        FilesystemHomeDiscovery::Initialized
    );
    assert!(FilesystemRepositoryHome::open(&root.0, project()).is_ok());

    let map_path = root.0.join(".omvcs/storage-map.json");
    let mut persisted: Value = serde_json::from_slice(&fs::read(&map_path).unwrap()).unwrap();
    persisted["generation"] = json!(0);
    fs::write(&map_path, serde_jcs::to_vec(&persisted).unwrap()).unwrap();
    assert_eq!(
        FilesystemHomeInitialization::discover(&root.0, Some(project())).unwrap(),
        FilesystemHomeDiscovery::IncompleteInitialization
    );
}

#[test]
fn stale_map_mutation_conflicts_without_partial_state_or_generation_change() {
    let root = TestRoot::new();
    let persistence = FilesystemStorageMapPersistence::open(&root.0).unwrap();
    persistence.initialize_empty_map(project()).unwrap();

    let bytes = b"replica";
    let candidate = ReplicaCandidate::new(
        hash_resource_bytes(bytes),
        endpoint(),
        ReplicaRepresentation::CompleteObject,
        ProviderLocator::new("fixture.locator/1", json!({})).unwrap(),
    )
    .unwrap();
    let assurance = verify_candidate_resource_reader(&candidate, &mut Cursor::new(bytes)).unwrap();
    let eligibility = assurance.promotion_eligibility(&candidate).unwrap();
    let mut add = StorageMapMutation::new();
    add.add_replica(candidate, &eligibility, ReplicaAvailability::Available)
        .unwrap();
    let first =
        apply_storage_map_mutation(&persistence, project(), StorageMapGeneration::ZERO, &add);
    assert!(matches!(
        first,
        StorageMapMutationOutcome::Applied { generation } if generation.get() == 1
    ));

    let stale_noop = StorageMapMutation::new();
    // A stale generation conflicts before any mutation can be considered.
    let outcome = apply_storage_map_mutation(
        &persistence,
        project(),
        StorageMapGeneration::ZERO,
        &stale_noop,
    );
    assert!(matches!(
        outcome,
        StorageMapMutationOutcome::Conflict {
            observed_generation: Some(generation)
        } if generation.get() == 1
    ));
    let state = persistence.load(project()).unwrap();
    assert_eq!(state.generation().get(), 1);
    assert_eq!(state.replicas().len(), 1);
    assert!(stale_noop.is_empty());
}

#[test]
fn concurrent_map_mutations_from_one_generation_have_one_winner() {
    let root = TestRoot::new();
    let persistence = FilesystemStorageMapPersistence::open(&root.0).unwrap();
    persistence.initialize_empty_map(project()).unwrap();

    let barrier = Arc::new(Barrier::new(3));
    let mut threads = Vec::new();
    for bytes in [b"replica one".as_slice(), b"replica two".as_slice()] {
        let root_path = root.0.clone();
        let barrier = Arc::clone(&barrier);
        threads.push(std::thread::spawn(move || {
            let persistence = FilesystemStorageMapPersistence::open(root_path).unwrap();
            let candidate = ReplicaCandidate::new(
                hash_resource_bytes(bytes),
                endpoint(),
                ReplicaRepresentation::CompleteObject,
                ProviderLocator::new("fixture.locator/1", json!({"key": "resource"})).unwrap(),
            )
            .unwrap();
            let verification =
                verify_candidate_resource_reader(&candidate, &mut Cursor::new(bytes)).unwrap();
            let eligibility = verification.promotion_eligibility(&candidate).unwrap();
            let mut mutation = StorageMapMutation::new();
            mutation
                .add_replica(candidate, &eligibility, ReplicaAvailability::Available)
                .unwrap();
            barrier.wait();
            apply_storage_map_mutation(
                &persistence,
                project(),
                StorageMapGeneration::ZERO,
                &mutation,
            )
        }));
    }
    barrier.wait();

    let outcomes = threads
        .into_iter()
        .map(|thread| thread.join().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| matches!(
                outcome,
                StorageMapMutationOutcome::Applied { generation }
                    if generation.get() == 1
            ))
            .count(),
        1
    );
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| matches!(
                outcome,
                StorageMapMutationOutcome::Conflict {
                    observed_generation: Some(generation)
                } if generation.get() == 1
            ))
            .count(),
        1
    );
    let final_map = persistence.load(project()).unwrap();
    assert_eq!(final_map.generation().get(), 1);
    assert_eq!(final_map.replicas().len(), 1);
}

#[test]
fn noncanonical_or_project_mismatched_persistence_is_rejected() {
    let root = TestRoot::new();
    let persistence = FilesystemStorageMapPersistence::open(&root.0).unwrap();
    persistence.initialize_empty_map(project()).unwrap();
    assert_eq!(
        persistence.load("019cc17d-1b22-7a41-9fe9-c345c468f82d".parse().unwrap()),
        Err(StorageMapPersistenceFailure::InvalidPersistedState)
    );
    fs::write(
        root.0.join(".omvcs/storage-map.json"),
        br#"{"generation":0,"project_id":"019cc17d-1b22-7a41-9fe9-c345c468f82e","replicas":[] }"#,
    )
    .unwrap();
    assert_eq!(
        persistence.load(project()),
        Err(StorageMapPersistenceFailure::InvalidPersistedState)
    );
}

#[cfg(target_os = "linux")]
#[test]
fn storage_map_access_distinguishes_missing_home_from_rejected_symlink() {
    use std::os::unix::fs::symlink;

    let missing_root = TestRoot::new();
    fs::remove_dir(missing_root.0.join(".omvcs")).unwrap();
    let missing = FilesystemStorageMapPersistence::open(&missing_root.0).unwrap();
    assert_eq!(
        missing.load(project()),
        Err(StorageMapPersistenceFailure::MissingOperationalMetadata)
    );

    let linked_root = TestRoot::new();
    let external_root = TestRoot::new();
    fs::remove_dir(linked_root.0.join(".omvcs")).unwrap();
    symlink(external_root.0.join(".omvcs"), linked_root.0.join(".omvcs")).unwrap();
    let linked = FilesystemStorageMapPersistence::open(&linked_root.0).unwrap();
    assert_eq!(
        linked.load(project()),
        Err(StorageMapPersistenceFailure::ProviderFailure)
    );
}

#[test]
fn direct_conditional_write_rejects_bad_replacement_and_conflicts_stale() {
    let root = TestRoot::new();
    let persistence = FilesystemStorageMapPersistence::open(&root.0).unwrap();
    persistence.initialize_empty_map(project()).unwrap();
    let empty = persistence.load(project()).unwrap();
    let replacement = omvcs_core::storage_map::StorageMapSnapshot::initialized_empty();
    // A state replacement that attempts to manufacture generation one from an
    // empty map is invalid; the provider rejects it rather than writing.
    assert_eq!(
        persistence.compare_and_swap(project(), StorageMapGeneration::ZERO, replacement),
        Err(StorageMapPersistenceFailure::InvalidPersistedState)
    );
    assert_eq!(empty.replicas().len(), 0);
    assert_eq!(
        persistence.load(project()).unwrap().generation(),
        StorageMapGeneration::ZERO
    );
    assert_eq!(
        persistence.compare_and_swap(
            project(),
            StorageMapGeneration::try_from(1).unwrap(),
            omvcs_core::storage_map::StorageMapSnapshot::initialized_empty(),
        ),
        Ok(ConditionalWriteOutcome::Conflict {
            observed_generation: Some(StorageMapGeneration::ZERO),
        })
    );
}
