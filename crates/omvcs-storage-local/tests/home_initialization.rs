#![allow(clippy::unwrap_used)]

use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

#[cfg(target_os = "linux")]
use omvcs_core::storage_map::{StorageMapGeneration, StorageMapPersistence};
use omvcs_model::ProjectId;
#[cfg(target_os = "linux")]
use omvcs_storage_local::{FilesystemHomeBootstrapOutcome, FilesystemStorageMapPersistence};
use omvcs_storage_local::{
    FilesystemHomeDiscovery, FilesystemHomeError, FilesystemHomeInitialization,
};

static ROOT_ID: AtomicU64 = AtomicU64::new(0);

struct TestRoot(PathBuf);

impl TestRoot {
    fn new() -> Self {
        let unique = ROOT_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "omvcs-home-local-test-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn metadata_dir(&self) -> PathBuf {
        let path = self.0.join(".omvcs");
        fs::create_dir_all(&path).unwrap();
        path
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

const fn marker() -> &'static [u8] {
    br#"{"layout":"omvcs.storage-layout/0.1","project_id":"019cc17d-1b22-7a41-9fe9-c345c468f82e","schema":"omvcs.repository-home/0.1"}"#
}

const fn empty_map() -> &'static [u8] {
    br#"{"generation":0,"project_id":"019cc17d-1b22-7a41-9fe9-c345c468f82e","replicas":[]}"#
}

#[test]
fn discovery_uses_only_explicit_root_and_classifies_marker_and_map_states() {
    let root = TestRoot::new();
    assert_eq!(
        FilesystemHomeInitialization::discover(&root.0, Some(project())).unwrap(),
        FilesystemHomeDiscovery::NotRepository
    );

    let metadata = root.metadata_dir();
    fs::write(metadata.join("repository.json"), b"{").unwrap();
    assert_eq!(
        FilesystemHomeInitialization::discover(&root.0, Some(project())),
        Err(FilesystemHomeError::InvalidRepositoryMetadata)
    );

    fs::write(metadata.join("storage-map.json"), empty_map()).unwrap();
    assert_eq!(
        FilesystemHomeInitialization::discover(&root.0, Some(project())).unwrap(),
        FilesystemHomeDiscovery::IncompleteInitialization
    );

    fs::write(metadata.join("repository.json"), marker()).unwrap();
    #[cfg(target_os = "linux")]
    assert_eq!(
        FilesystemHomeInitialization::discover(&root.0, Some(project())).unwrap(),
        FilesystemHomeDiscovery::Initialized
    );
    #[cfg(not(target_os = "linux"))]
    assert_eq!(
        FilesystemHomeInitialization::discover(&root.0, Some(project())),
        Err(FilesystemHomeError::UnsupportedFilesystemGuarantee)
    );
    let other_project: ProjectId = "019cc17d-1b22-7a41-9fe9-c345c468f82d".parse().unwrap();
    assert_eq!(
        FilesystemHomeInitialization::discover(&root.0, Some(other_project)),
        Err(FilesystemHomeError::ProjectIdentityMismatch)
    );
}

#[test]
fn unsupported_marker_version_is_never_interpreted_as_a_home() {
    let root = TestRoot::new();
    let metadata = root.metadata_dir();
    fs::write(
        metadata.join("repository.json"),
        br#"{"layout":"omvcs.storage-layout/9.9","project_id":"019cc17d-1b22-7a41-9fe9-c345c468f82e","schema":"omvcs.repository-home/0.1"}"#,
    )
    .unwrap();
    assert_eq!(
        FilesystemHomeInitialization::discover(&root.0, Some(project())),
        Err(FilesystemHomeError::UnsupportedVersion)
    );
}

#[test]
fn marker_only_bootstrap_evidence_is_incomplete() {
    let root = TestRoot::new();
    fs::write(root.metadata_dir().join("repository.json"), marker()).unwrap();
    assert_eq!(
        FilesystemHomeInitialization::discover(&root.0, Some(project())).unwrap(),
        FilesystemHomeDiscovery::IncompleteInitialization
    );
}

#[test]
fn operational_state_without_marker_is_incomplete() {
    let root = TestRoot::new();
    let metadata = root.metadata_dir();
    fs::write(metadata.join("storage-map.json"), empty_map()).unwrap();
    assert_eq!(
        FilesystemHomeInitialization::discover(&root.0, Some(project())).unwrap(),
        FilesystemHomeDiscovery::IncompleteInitialization
    );
}

#[test]
fn marker_and_initial_operational_state_must_agree() {
    let root = TestRoot::new();
    let metadata = root.metadata_dir();
    fs::write(metadata.join("repository.json"), marker()).unwrap();
    fs::write(
        metadata.join("storage-map.json"),
        br#"{"generation":0,"project_id":"019cc17d-1b22-7a41-9fe9-c345c468f82d","replicas":[]}"#,
    )
    .unwrap();
    assert_eq!(
        FilesystemHomeInitialization::discover(&root.0, Some(project())).unwrap(),
        FilesystemHomeDiscovery::IncompleteInitialization
    );
}

#[test]
fn abandoned_bootstrap_staging_is_incomplete_until_commit() {
    let root = TestRoot::new();
    fs::write(
        root.metadata_dir().join(".omvcs-stage-abandoned"),
        b"partial",
    )
    .unwrap();
    assert_eq!(
        FilesystemHomeInitialization::discover(&root.0, Some(project())).unwrap(),
        FilesystemHomeDiscovery::IncompleteInitialization
    );
}

#[cfg(target_os = "linux")]
#[test]
fn bootstrap_is_idempotent_and_keeps_initial_generation_zero() {
    let root = TestRoot::new();
    assert_eq!(
        FilesystemHomeInitialization::bootstrap(&root.0, project()).unwrap(),
        FilesystemHomeBootstrapOutcome::Initialized
    );
    assert_eq!(
        FilesystemHomeInitialization::bootstrap(&root.0, project()).unwrap(),
        FilesystemHomeBootstrapOutcome::AlreadyInitialized
    );
    assert_eq!(
        FilesystemHomeInitialization::discover(&root.0, Some(project())).unwrap(),
        FilesystemHomeDiscovery::Initialized
    );

    let persistence = FilesystemStorageMapPersistence::open(&root.0).unwrap();
    let map = persistence.load(project()).unwrap();
    assert_eq!(map.generation(), StorageMapGeneration::ZERO);
    assert_eq!(map.replicas().len(), 0);
}

#[cfg(target_os = "linux")]
#[test]
fn bootstrap_retries_only_from_valid_same_project_marker_or_empty_map() {
    let marker_root = TestRoot::new();
    fs::write(marker_root.metadata_dir().join("repository.json"), marker()).unwrap();
    assert_eq!(
        FilesystemHomeInitialization::bootstrap(&marker_root.0, project()).unwrap(),
        FilesystemHomeBootstrapOutcome::Initialized
    );

    let map_root = TestRoot::new();
    fs::write(
        map_root.metadata_dir().join("storage-map.json"),
        empty_map(),
    )
    .unwrap();
    assert_eq!(
        FilesystemHomeInitialization::bootstrap(&map_root.0, project()).unwrap(),
        FilesystemHomeBootstrapOutcome::Initialized
    );
}

#[cfg(target_os = "linux")]
#[test]
fn populated_unmarked_root_is_not_adopted() {
    let root = TestRoot::new();
    fs::write(root.0.join("unrelated.txt"), b"preserve me").unwrap();
    assert_eq!(
        FilesystemHomeInitialization::bootstrap(&root.0, project()),
        Err(FilesystemHomeError::IncompleteInitialization)
    );
    assert_eq!(
        fs::read(root.0.join("unrelated.txt")).unwrap(),
        b"preserve me"
    );
}

#[cfg(target_os = "linux")]
#[test]
fn unrecognized_operational_artifacts_are_not_adopted_during_retry() {
    let root = TestRoot::new();
    let metadata = root.metadata_dir();
    fs::write(metadata.join("storage-map.json"), empty_map()).unwrap();
    fs::write(metadata.join("unrelated.txt"), b"preserve me").unwrap();

    assert_eq!(
        FilesystemHomeInitialization::discover(&root.0, Some(project())).unwrap(),
        FilesystemHomeDiscovery::IncompleteInitialization
    );
    assert_eq!(
        FilesystemHomeInitialization::bootstrap(&root.0, project()),
        Err(FilesystemHomeError::IncompleteInitialization)
    );
    assert!(!metadata.join("repository.json").exists());
    assert_eq!(
        fs::read(metadata.join("unrelated.txt")).unwrap(),
        b"preserve me"
    );
}
