#![allow(clippy::unwrap_used)]

#[cfg(target_os = "linux")]
use std::io::Read;
use std::{
    fs,
    io::Cursor,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

use omvcs_model::{ResourceId, StorageEndpointId, hashing::hash_resource_bytes};
#[cfg(target_os = "linux")]
use omvcs_storage::ChunkId;
#[cfg(target_os = "linux")]
use omvcs_storage::StatOutcome;
use omvcs_storage::{
    ErrorClass, LogicalKey, ObjectId, ObjectRequest, Operation, OperationId, PutRequest, Storage,
};
use omvcs_storage_local::FilesystemStorage;
#[cfg(target_os = "linux")]
use sha2::{Digest, Sha256};

static ROOT_ID: AtomicU64 = AtomicU64::new(0);

struct TestRoot(PathBuf);

impl TestRoot {
    fn new() -> Self {
        let unique = ROOT_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "omvcs-storage-local-test-{}-{unique}",
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

fn endpoint() -> StorageEndpointId {
    "019cc17d-1b22-7a41-9fe9-c345c468f82c".parse().unwrap()
}

fn logical_key(id: ObjectId) -> String {
    let digest = id.digest();
    let mut digest_hex = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        write!(digest_hex, "{byte:02x}").unwrap();
    }
    let namespace = match id {
        ObjectId::Resource(_) => "resources",
        ObjectId::Chunk(_) => "chunks",
    };
    format!(
        "{namespace}/sha256/{}/{}/{}",
        &digest_hex[..2],
        &digest_hex[2..4],
        digest_hex
    )
}

fn resource_request(id: ResourceId) -> ObjectRequest<ResourceId> {
    let object_id = ObjectId::Resource(id);
    ObjectRequest {
        endpoint: endpoint(),
        id,
        logical_key: LogicalKey(logical_key(object_id)),
        operation_id: OperationId("test-op".into()),
    }
}

#[cfg(target_os = "linux")]
fn chunk_request(id: ChunkId) -> ObjectRequest<ChunkId> {
    let object_id = ObjectId::Chunk(id);
    ObjectRequest {
        endpoint: endpoint(),
        id,
        logical_key: LogicalKey(logical_key(object_id)),
        operation_id: OperationId("test-op".into()),
    }
}

#[cfg(target_os = "linux")]
fn physical_path(id: ObjectId) -> PathBuf {
    let kind = match id {
        ObjectId::Resource(_) => '0',
        ObjectId::Chunk(_) => '1',
    };
    let mut digits = String::with_capacity(128);
    for byte in id.digest() {
        use std::fmt::Write as _;
        let _ = write!(digits, "{:02}{:02}", byte >> 4, byte & 0x0f);
    }
    PathBuf::from(format!("{kind}/{}/{digits}", &digits[..4]))
}

#[cfg(target_os = "linux")]
#[test]
fn explicit_root_get_and_stat_preserve_exact_resource_bytes() {
    let root = TestRoot::new();
    let bytes = b"exact resource bytes";
    let id = hash_resource_bytes(bytes);
    let object_id = ObjectId::Resource(id);
    let path = root.0.join(physical_path(object_id));
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, bytes).unwrap();

    let provider = FilesystemStorage::open(&root.0, endpoint()).unwrap();
    let mut storage = Storage::new(provider);
    let request = resource_request(id);
    assert!(matches!(
        storage.stat_resource(&request).unwrap(),
        StatOutcome::Present(metadata) if metadata.byte_length == Some(bytes.len() as u64)
    ));

    let mut download = storage.get_resource(&request).unwrap();
    let mut retrieved = Vec::new();
    download.read_to_end(&mut retrieved).unwrap();
    let completion = download.complete().unwrap();
    assert_eq!(retrieved, bytes);
    assert_eq!(completion.id, id);
    assert_eq!(completion.byte_length, bytes.len() as u64);
}

#[cfg(target_os = "linux")]
#[test]
fn hostile_and_alternate_logical_keys_are_rejected_before_file_access() {
    let root = TestRoot::new();
    let id = hash_resource_bytes(b"hostile");
    let mut request = resource_request(id);
    request.logical_key = LogicalKey("../outside".into());
    let provider = FilesystemStorage::open(&root.0, endpoint()).unwrap();
    let mut storage = Storage::new(provider);
    let error = storage.stat_resource(&request).unwrap_err();
    assert_eq!(error.class, ErrorClass::InvalidRequest);

    request.logical_key = LogicalKey(logical_key(ObjectId::Resource(id)).to_uppercase());
    assert!(matches!(
        storage.get_resource(&request),
        Err(error) if error.class == ErrorClass::InvalidRequest
    ));
}

#[cfg(not(target_os = "linux"))]
#[test]
fn unsafe_case_insensitive_resolution_is_reported_unsupported() {
    let root = TestRoot::new();
    let bytes = b"uppercase alias must not satisfy canonical namespace";
    let id = hash_resource_bytes(bytes);
    let key = logical_key(ObjectId::Resource(id));
    let upper = key
        .split('/')
        .map(str::to_uppercase)
        .collect::<Vec<_>>()
        .join("/");
    let path = root.0.join(upper);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();

    let provider = FilesystemStorage::open(&root.0, endpoint()).unwrap();
    let mut storage = Storage::new(provider);
    assert!(matches!(
        storage.stat_resource(&resource_request(id)),
        Err(error) if error.class == ErrorClass::Unsupported
    ));
}

#[cfg(target_os = "linux")]
#[test]
fn immutable_put_publishes_exact_bytes_and_repeats_idempotently() {
    let root = TestRoot::new();
    let bytes = b"new immutable object";
    let id = hash_resource_bytes(bytes);
    let request = resource_request(id);
    let provider = FilesystemStorage::open(&root.0, endpoint()).unwrap();
    let mut storage = Storage::new(provider);
    let put = PutRequest {
        object: request.clone(),
        byte_length: bytes.len() as u64,
        expected_hash: *id.digest(),
    };
    assert_eq!(
        storage
            .put_resource(&put, &mut Cursor::new(bytes))
            .unwrap()
            .byte_length,
        bytes.len() as u64
    );
    assert_eq!(
        storage
            .put_resource(&put, &mut Cursor::new(bytes))
            .unwrap()
            .byte_length,
        bytes.len() as u64
    );
    let error = storage
        .put_resource(&put, &mut Cursor::new(b"bad immutable object"))
        .unwrap_err();
    assert_eq!(error.class, ErrorClass::Integrity);
    assert_eq!(
        fs::read(root.0.join(physical_path(ObjectId::Resource(request.id)))).unwrap(),
        bytes
    );
}

#[cfg(target_os = "linux")]
#[test]
fn chunk_put_and_get_use_the_chunk_namespace_and_exact_bytes() {
    let root = TestRoot::new();
    let bytes = b"exact chunk bytes";
    let digest: [u8; 32] = Sha256::digest(bytes).into();
    let id = ChunkId::from_digest(digest);
    let request = chunk_request(id);
    let provider = FilesystemStorage::open(&root.0, endpoint()).unwrap();
    let mut storage = Storage::new(provider);
    storage
        .put_chunk(
            &PutRequest {
                object: request.clone(),
                byte_length: bytes.len() as u64,
                expected_hash: digest,
            },
            &mut Cursor::new(bytes),
        )
        .unwrap();

    let mut download = storage.get_chunk(&request).unwrap();
    let mut retrieved = Vec::new();
    download.read_to_end(&mut retrieved).unwrap();
    download.complete().unwrap();
    assert_eq!(retrieved, bytes);
    assert!(root.0.join(physical_path(ObjectId::Chunk(id))).is_file());
}

#[cfg(target_os = "linux")]
#[test]
fn failed_identity_check_never_publishes_a_canonical_object() {
    let root = TestRoot::new();
    let id = hash_resource_bytes(b"expected");
    let request = resource_request(id);
    let provider = FilesystemStorage::open(&root.0, endpoint()).unwrap();
    let mut storage = Storage::new(provider);
    let put = PutRequest {
        object: request.clone(),
        byte_length: 8,
        expected_hash: *id.digest(),
    };
    let error = storage
        .put_resource(&put, &mut Cursor::new(b"mismatch"))
        .unwrap_err();
    assert_eq!(error.class, ErrorClass::Integrity);
    assert!(
        !root
            .0
            .join(physical_path(ObjectId::Resource(request.id)))
            .exists()
    );
}

#[cfg(target_os = "linux")]
#[test]
fn conflicting_preexisting_object_is_never_overwritten() {
    let root = TestRoot::new();
    let requested = b"correct content";
    let actual = b"wrong content";
    let id = hash_resource_bytes(requested);
    let request = resource_request(id);
    let path = root.0.join(physical_path(ObjectId::Resource(id)));
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, actual).unwrap();

    let provider = FilesystemStorage::open(&root.0, endpoint()).unwrap();
    let mut storage = Storage::new(provider);
    let error = storage
        .put_resource(
            &PutRequest {
                object: request,
                byte_length: requested.len() as u64,
                expected_hash: *id.digest(),
            },
            &mut Cursor::new(requested),
        )
        .unwrap_err();
    assert_eq!(error.class, ErrorClass::Integrity);
    assert_eq!(fs::read(path).unwrap(), actual);
}

#[cfg(target_os = "linux")]
#[test]
fn interrupted_input_does_not_publish_a_complete_canonical_object() {
    struct FailingReader;
    impl Read for FailingReader {
        fn read(&mut self, _buffer: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("injected read failure"))
        }
    }

    let root = TestRoot::new();
    let bytes = b"expected";
    let id = hash_resource_bytes(bytes);
    let request = resource_request(id);
    let provider = FilesystemStorage::open(&root.0, endpoint()).unwrap();
    let mut storage = Storage::new(provider);
    let error = storage
        .put_resource(
            &PutRequest {
                object: request,
                byte_length: bytes.len() as u64,
                expected_hash: *id.digest(),
            },
            &mut FailingReader,
        )
        .unwrap_err();
    assert_eq!(error.class, ErrorClass::Precondition);
    assert!(!root.0.join(physical_path(ObjectId::Resource(id))).exists());
}

#[cfg(target_os = "linux")]
#[test]
fn invalid_data_reader_errors_are_not_reported_as_content_mismatches() {
    struct InvalidDataReader;
    impl Read for InvalidDataReader {
        fn read(&mut self, _buffer: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "injected caller stream failure",
            ))
        }
    }

    let root = TestRoot::new();
    let bytes = b"existing immutable content";
    let id = hash_resource_bytes(bytes);
    let request = resource_request(id);
    let provider = FilesystemStorage::open(&root.0, endpoint()).unwrap();
    let mut storage = Storage::new(provider);
    let put = PutRequest {
        object: request,
        byte_length: bytes.len() as u64,
        expected_hash: *id.digest(),
    };
    storage.put_resource(&put, &mut Cursor::new(bytes)).unwrap();

    let error = storage
        .put_resource(&put, &mut InvalidDataReader)
        .unwrap_err();
    assert_eq!(error.class, ErrorClass::Precondition);
    assert!(error.code.ends_with("STREAM_FAILURE"));
    assert!(!error.code.ends_with("CONTENT_MISMATCH"));
    assert_eq!(
        fs::read(root.0.join(physical_path(ObjectId::Resource(id)))).unwrap(),
        bytes
    );
}

#[cfg(target_os = "linux")]
#[test]
fn atomic_create_races_never_replace_or_corrupt_existing_bytes() {
    let root = TestRoot::new();
    let root_path = root.0.clone();
    let bytes = b"shared immutable bytes";
    let id = hash_resource_bytes(bytes);
    let logical = resource_request(id);
    let mut threads = Vec::new();
    for _ in 0..8 {
        let path = root_path.clone();
        let request = logical.clone();
        threads.push(std::thread::spawn(move || {
            let provider = FilesystemStorage::open(path, endpoint()).unwrap();
            let mut storage = Storage::new(provider);
            storage
                .put_resource(
                    &PutRequest {
                        object: request,
                        byte_length: bytes.len() as u64,
                        expected_hash: *id.digest(),
                    },
                    &mut Cursor::new(bytes),
                )
                .unwrap()
        }));
    }
    for thread in threads {
        assert_eq!(thread.join().unwrap().byte_length, bytes.len() as u64);
    }
    assert_eq!(
        fs::read(root.0.join(physical_path(ObjectId::Resource(logical.id)))).unwrap(),
        bytes
    );
}

#[cfg(target_os = "linux")]
#[test]
fn symlinked_namespace_component_cannot_escape_the_selected_root() {
    use std::os::unix::fs::symlink;

    let root = TestRoot::new();
    let outside = TestRoot::new();
    let id = hash_resource_bytes(b"outside");
    let path = physical_path(ObjectId::Resource(id));
    let prefix = path.components().take(2).collect::<PathBuf>();
    let prefix_path = root.0.join(prefix);
    fs::create_dir_all(prefix_path.parent().unwrap()).unwrap();
    symlink(&outside.0, &prefix_path).unwrap();
    let provider = FilesystemStorage::open(&root.0, endpoint()).unwrap();
    let mut storage = Storage::new(provider);
    assert!(matches!(
        storage.get_resource(&resource_request(id)),
        Err(error) if error.class != ErrorClass::NotFound
    ));
}

#[cfg(target_os = "linux")]
#[test]
fn unsupported_endpoint_is_not_reported_as_absence() {
    let root = TestRoot::new();
    let provider = FilesystemStorage::open(&root.0, endpoint()).unwrap();
    let mut storage = Storage::new(provider);
    let request = resource_request(hash_resource_bytes(b"absent"));
    assert_eq!(
        storage.stat_resource(&request).unwrap(),
        StatOutcome::Absent
    );
    assert!(
        storage
            .capabilities(endpoint(), Operation::PutResource)
            .unwrap()
            .put_resource
    );
}

#[cfg(not(target_os = "linux"))]
#[test]
fn platform_without_directory_durability_disables_put_explicitly() {
    let root = TestRoot::new();
    let provider = FilesystemStorage::open(&root.0, endpoint()).unwrap();
    let mut storage = Storage::new(provider);
    let id = hash_resource_bytes(b"absent");
    let request = resource_request(id);
    assert!(
        !storage
            .capabilities(endpoint(), Operation::PutResource)
            .unwrap()
            .put_resource
    );
    assert!(
        !storage
            .capabilities(endpoint(), Operation::GetResource)
            .unwrap()
            .get_resource
    );
    assert!(
        !storage
            .capabilities(endpoint(), Operation::StatResource)
            .unwrap()
            .stat_resource
    );
    assert!(matches!(
        storage.get_resource(&request),
        Err(error) if error.class == ErrorClass::Unsupported
    ));
    assert!(matches!(
        storage.stat_resource(&request),
        Err(error) if error.class == ErrorClass::Unsupported
    ));
    let error = storage
        .put_resource(
            &PutRequest {
                object: request,
                byte_length: 0,
                expected_hash: *id.digest(),
            },
            &mut Cursor::new(b""),
        )
        .unwrap_err();
    assert_eq!(error.class, ErrorClass::Unsupported);
}
