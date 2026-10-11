#![cfg(target_os = "linux")]
#![allow(clippy::unwrap_used)]

use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

use omvcs_model::ProjectId;
use omvcs_storage::LogicalKey;
use omvcs_storage_local::{
    FilesystemHomeInitialization, FilesystemHomeMetadataError, FilesystemRepositoryHome,
    ImmutableMetadataWriteOutcome, OperationalMetadataWriteOutcome,
};

static ROOT_ID: AtomicU64 = AtomicU64::new(0);

struct TestRoot(PathBuf);

impl TestRoot {
    fn new() -> Self {
        let unique = ROOT_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "omvcs-home-metadata-test-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&path).unwrap();
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

fn initialized_home(root: &TestRoot) -> FilesystemRepositoryHome {
    FilesystemHomeInitialization::bootstrap(&root.0, project()).unwrap();
    FilesystemRepositoryHome::open(&root.0, project()).unwrap()
}

#[test]
fn immutable_metadata_round_trips_exact_bytes_and_rejects_key_conflicts() {
    let root = TestRoot::new();
    let home = initialized_home(&root);
    let key = LogicalKey("omvcs:revision:sha256:fixture".into());
    let bytes = br#"{"schema":"test.metadata/1","value":"exact bytes"}"#;

    assert_eq!(
        home.write_immutable_metadata_object(&key, bytes).unwrap(),
        ImmutableMetadataWriteOutcome::Created
    );
    assert_eq!(home.read_metadata_object(&key).unwrap(), bytes);
    assert_eq!(
        home.write_immutable_metadata_object(&key, bytes).unwrap(),
        ImmutableMetadataWriteOutcome::AlreadyPresent
    );
    assert_eq!(
        home.write_immutable_metadata_object(&key, b"different bytes"),
        Err(FilesystemHomeMetadataError::Integrity)
    );
}

#[test]
fn opened_home_declares_the_complete_minimum_home_profile() {
    let root = TestRoot::new();
    let home = initialized_home(&root);
    let capabilities = home.capabilities();

    assert!(capabilities.immutable_historical_metadata);
    assert!(capabilities.operational_metadata);
    assert!(capabilities.guarded_operational_updates);
    assert!(capabilities.project_storage_map_cas);
    assert!(capabilities.marker_and_discovery);
    assert!(capabilities.durable_write_acknowledgement);
}

#[test]
fn operational_metadata_updates_are_guarded_and_durable() {
    let root = TestRoot::new();
    let home = initialized_home(&root);
    let key = LogicalKey("project.default-line".into());

    assert_eq!(
        home.compare_and_swap_operational_metadata(&key, None, b"line-a")
            .unwrap(),
        OperationalMetadataWriteOutcome::Updated
    );
    assert_eq!(
        home.read_operational_metadata(&key).unwrap().as_deref(),
        Some(&b"line-a"[..])
    );
    assert_eq!(
        home.compare_and_swap_operational_metadata(&key, None, b"line-b")
            .unwrap(),
        OperationalMetadataWriteOutcome::Conflict
    );
    assert_eq!(
        home.compare_and_swap_operational_metadata(&key, Some(b"line-a"), b"line-b")
            .unwrap(),
        OperationalMetadataWriteOutcome::Updated
    );
    assert_eq!(
        home.read_operational_metadata(&key).unwrap().as_deref(),
        Some(&b"line-b"[..])
    );
}

#[test]
fn concurrent_operational_updates_with_one_expected_value_have_one_winner() {
    let root = TestRoot::new();
    let home = initialized_home(&root);
    let key = LogicalKey("project.default-line".into());
    home.compare_and_swap_operational_metadata(&key, None, b"line-a")
        .unwrap();

    let root_path = root.0.clone();
    let key_for_thread = key.clone();
    let project_id = project();
    let thread = std::thread::spawn(move || {
        let other = FilesystemRepositoryHome::open(root_path, project_id).unwrap();
        other
            .compare_and_swap_operational_metadata(&key_for_thread, Some(b"line-a"), b"line-thread")
            .unwrap()
    });
    let main_result = home
        .compare_and_swap_operational_metadata(&key, Some(b"line-a"), b"line-main")
        .unwrap();
    let thread_result = thread.join().unwrap();
    assert_ne!(main_result, thread_result);
    assert!(matches!(
        home.read_operational_metadata(&key).unwrap().as_deref(),
        Some(b"line-main" | b"line-thread")
    ));
}

#[test]
fn metadata_keys_are_never_used_as_filesystem_paths() {
    let root = TestRoot::new();
    let home = initialized_home(&root);
    let key = LogicalKey("../../outside\\arbitrary".into());
    let bytes = b"confined";
    home.write_immutable_metadata_object(&key, bytes).unwrap();
    assert_eq!(home.read_metadata_object(&key).unwrap(), bytes);
    assert!(!root.0.parent().unwrap().join("outside").exists());
}
