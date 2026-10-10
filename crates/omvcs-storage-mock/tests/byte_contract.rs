//! Storage §§9–25, 29, 35–36, 85–86, 181–182, 185, 199–200;
//! INV-RES-001–007, INV-STOR-001–003, INV-INT-001–003.
#![allow(clippy::unwrap_used)]

use std::io::{Cursor, Read};

use omvcs_model::hashing::hash_resource_bytes;
use omvcs_storage::{
    ByteStorage, ChunkId, ErrorClass, LogicalKey, ObjectId, ObjectRequest, Operation, OperationId,
    PutRequest, StatOutcome, Storage, StorageEndpointId,
};
use omvcs_storage_mock::{
    ALL_BYTE_OPERATIONS, Effect, EndpointConfiguration, MockStorage, ScriptStep,
};
use sha2::{Digest, Sha256};

const BYTES: &[u8] = b"\0RIFF\r\n\xff\x80no normalization\n";

fn endpoint() -> StorageEndpointId {
    "019cc17d-1b22-7a41-9fe9-c345c468f82c".parse().unwrap()
}

fn setup() -> MockStorage {
    let mock = MockStorage::new();
    mock.configure_endpoint(endpoint(), EndpointConfiguration::default());
    mock
}

fn request<I>(id: I) -> ObjectRequest<I> {
    ObjectRequest {
        endpoint: endpoint(),
        id,
        logical_key: LogicalKey("caller-selected-key".into()),
        operation_id: OperationId("explicit-test-operation".into()),
    }
}

fn put<I: Copy + Into<ObjectId>>(object: &ObjectRequest<I>, bytes: &[u8]) -> PutRequest<I> {
    PutRequest {
        object: object.clone(),
        byte_length: bytes.len() as u64,
        expected_hash: *object.id.into().digest(),
    }
}

fn inject(mock: &MockStorage, op: Operation, effect: Effect) {
    mock.enqueue(ScriptStep {
        endpoint: endpoint(),
        operation: op,
        logical_key: "caller-selected-key".into(),
        effect,
    })
    .unwrap();
}

fn chunk(bytes: &[u8]) -> ChunkId {
    ChunkId::from_digest(Sha256::digest(bytes).into())
}

macro_rules! byte_cases {
    ($module:ident, $id:ident, $put:ident, $get:ident, $stat:ident, $put_op:ident, $get_op:ident, $stat_op:ident) => {
        mod $module {
            use super::*;

            #[test]
            fn exact_round_trip_and_stat_preserve_bytes_and_identity() {
                let mock = setup();
                let mut storage = Storage::new(mock.clone());
                for bytes in [b"".as_slice(), BYTES, &[0; 4097]] {
                    let mut object = request($id(bytes));
                    object.logical_key.0 = format!("size-{}", bytes.len());
                    let written = storage
                        .$put(&put(&object, bytes), &mut Cursor::new(bytes))
                        .unwrap();
                    assert_eq!(written.id, object.id);
                    assert_eq!(written.byte_length, bytes.len() as u64);
                    let mut download = storage.$get(&object).unwrap();
                    let mut output = vec![];
                    download.read_to_end(&mut output).unwrap();
                    assert_eq!(download.complete().unwrap().id, object.id);
                    assert_eq!(output, bytes);
                    let StatOutcome::Present(metadata) = storage.$stat(&object).unwrap() else {
                        panic!("present")
                    };
                    assert_eq!(metadata.byte_length, Some(bytes.len() as u64));
                    assert_eq!(metadata.checksum, None);
                    assert_eq!(metadata.last_modified, None);
                }
            }

            #[test]
            fn absent_is_distinct_from_unavailable_auth_quota_and_other_failures() {
                let mock = setup();
                let mut storage = Storage::new(mock.clone());
                let object = request($id(BYTES));
                assert_eq!(storage.$stat(&object).unwrap(), StatOutcome::Absent);
                assert_eq!(
                    storage.$get(&object).err().unwrap().class,
                    ErrorClass::NotFound
                );
                for class in [
                    ErrorClass::Authentication,
                    ErrorClass::Authorization,
                    ErrorClass::Network,
                    ErrorClass::ProviderUnavailable,
                    ErrorClass::Quota,
                    ErrorClass::RateLimit,
                    ErrorClass::Cancelled,
                    ErrorClass::Integrity,
                    ErrorClass::Conflict,
                    ErrorClass::Internal,
                ] {
                    for op in [Operation::$put_op, Operation::$get_op, Operation::$stat_op] {
                        inject(
                            &mock,
                            op,
                            Effect::Fail {
                                class,
                                recoverable: true,
                            },
                        );
                        let err = match op {
                            Operation::$put_op => storage
                                .$put(&put(&object, BYTES), &mut Cursor::new(BYTES))
                                .err()
                                .unwrap(),
                            Operation::$get_op => storage.$get(&object).err().unwrap(),
                            _ => storage.$stat(&object).err().unwrap(),
                        };
                        assert_eq!(err.class, class);
                        assert_eq!(err.endpoint, object.endpoint);
                        assert_eq!(err.operation, op);
                        assert!(err.recoverable);
                        assert!(!err.code.is_empty() && !err.summary.is_empty());
                        assert_eq!(err.provider_context, None);
                    }
                }
                assert_eq!(mock.stored_bytes(endpoint(), &object.logical_key.0), None);
                assert!(mock.pending_script().is_empty());
            }

            #[test]
            fn immutable_repeats_are_idempotent_and_conflicts_never_rewrite_bytes() {
                let mock = setup();
                let mut storage = Storage::new(mock.clone());
                let object = request($id(BYTES));
                let first = storage
                    .$put(&put(&object, BYTES), &mut Cursor::new(BYTES))
                    .unwrap();
                assert_eq!(
                    storage
                        .$put(&put(&object, BYTES), &mut Cursor::new(BYTES))
                        .unwrap(),
                    first
                );
                let other = request($id(b"different bytes"));
                assert_eq!(
                    storage
                        .$put(
                            &put(&other, b"different bytes"),
                            &mut Cursor::new(b"different bytes")
                        )
                        .err()
                        .unwrap()
                        .class,
                    ErrorClass::Integrity
                );
                assert_eq!(
                    storage
                        .$put(&put(&object, b"wrong"), &mut Cursor::new(b"wrong"))
                        .err()
                        .unwrap()
                        .class,
                    ErrorClass::Integrity
                );
                assert_eq!(
                    mock.stored_bytes(endpoint(), &object.logical_key.0)
                        .unwrap(),
                    BYTES
                );
            }

            #[test]
            fn interrupted_upload_at_zero_middle_or_final_byte_never_commits() {
                for after in [0, 3, BYTES.len() as u64] {
                    let mock = setup();
                    let object = request($id(BYTES));
                    inject(
                        &mock,
                        Operation::$put_op,
                        Effect::FailAfter {
                            bytes: after,
                            class: ErrorClass::Authentication,
                            recoverable: true,
                        },
                    );
                    let mut storage = Storage::new(mock.clone());
                    assert_eq!(
                        storage
                            .$put(&put(&object, BYTES), &mut Cursor::new(BYTES))
                            .err()
                            .unwrap()
                            .class,
                        ErrorClass::Authentication
                    );
                    assert_eq!(storage.$stat(&object).unwrap(), StatOutcome::Absent);
                    storage
                        .$put(&put(&object, BYTES), &mut Cursor::new(BYTES))
                        .unwrap();
                    inject(
                        &mock,
                        Operation::$put_op,
                        Effect::FailAfter {
                            bytes: after,
                            class: ErrorClass::Quota,
                            recoverable: false,
                        },
                    );
                    assert_eq!(
                        storage
                            .$put(&put(&object, BYTES), &mut Cursor::new(BYTES))
                            .err()
                            .unwrap()
                            .class,
                        ErrorClass::Quota
                    );
                    assert_eq!(
                        mock.stored_bytes(endpoint(), &object.logical_key.0)
                            .unwrap(),
                        BYTES
                    );
                }
            }

            #[test]
            fn interrupted_download_including_after_all_bytes_cannot_complete() {
                for after in [0, 2, BYTES.len() as u64] {
                    let mock = setup();
                    let object = request($id(BYTES));
                    let mut storage = Storage::new(mock.clone());
                    storage
                        .$put(&put(&object, BYTES), &mut Cursor::new(BYTES))
                        .unwrap();
                    inject(
                        &mock,
                        Operation::$get_op,
                        Effect::FailAfter {
                            bytes: after,
                            class: ErrorClass::Network,
                            recoverable: true,
                        },
                    );
                    let mut download = storage.$get(&object).unwrap();
                    let mut output = vec![];
                    assert!(download.read_to_end(&mut output).is_err());
                    assert_eq!(output.len() as u64, after);
                    assert_eq!(
                        download.complete().err().unwrap().class,
                        ErrorClass::Network
                    );
                }
            }

            #[test]
            fn wrong_truncated_and_corrupt_returns_cannot_be_successful_retrieval() {
                let mock = setup();
                let object = request($id(BYTES));
                let mut storage = Storage::new(mock.clone());
                storage
                    .$put(&put(&object, BYTES), &mut Cursor::new(BYTES))
                    .unwrap();
                for effect in [
                    Effect::ReturnBytes(vec![b'X'; BYTES.len()]),
                    Effect::Truncate(2),
                    Effect::Corrupt { offset: 0, xor: 1 },
                ] {
                    inject(&mock, Operation::$get_op, effect);
                    let mut download = storage.$get(&object).unwrap();
                    assert!(download.read_to_end(&mut vec![]).is_err());
                    assert_eq!(
                        download.complete().err().unwrap().class,
                        ErrorClass::Integrity
                    );
                    assert_eq!(
                        mock.stored_bytes(endpoint(), &object.logical_key.0)
                            .unwrap(),
                        BYTES
                    );
                }
                assert!(
                    storage.$get(&object).unwrap().complete().is_err(),
                    "abandonment is not success"
                );
            }

            #[test]
            fn unsupported_operations_are_explicit_and_do_not_consume_scripts() {
                for op in [Operation::$put_op, Operation::$get_op, Operation::$stat_op] {
                    let mock = setup();
                    let mut caps = ALL_BYTE_OPERATIONS;
                    match op {
                        Operation::$put_op => caps.$put = false,
                        Operation::$get_op => caps.$get = false,
                        _ => caps.$stat = false,
                    }
                    mock.configure_endpoint(
                        endpoint(),
                        EndpointConfiguration {
                            capabilities: caps,
                            discovery_failure: None,
                        },
                    );
                    inject(
                        &mock,
                        op,
                        Effect::Fail {
                            class: ErrorClass::Network,
                            recoverable: false,
                        },
                    );
                    let mut storage = Storage::new(mock.clone());
                    let object = request($id(BYTES));
                    let err = match op {
                        Operation::$put_op => storage
                            .$put(&put(&object, BYTES), &mut Cursor::new(BYTES))
                            .err()
                            .unwrap(),
                        Operation::$get_op => storage.$get(&object).err().unwrap(),
                        _ => storage.$stat(&object).err().unwrap(),
                    };
                    assert_eq!(err.class, ErrorClass::Unsupported);
                    assert_eq!(mock.calls(), []);
                    assert_eq!(mock.pending_script().len(), 1);
                }
            }

            #[test]
            fn scripted_visibility_is_call_controlled_not_time_or_deletion() {
                let mock = setup();
                let mut storage = Storage::new(mock.clone());
                let object = request($id(BYTES));
                storage
                    .$put(&put(&object, BYTES), &mut Cursor::new(BYTES))
                    .unwrap();
                inject(&mock, Operation::$stat_op, Effect::Absent);
                inject(&mock, Operation::$get_op, Effect::Absent);
                assert_eq!(storage.$stat(&object).unwrap(), StatOutcome::Absent);
                assert_eq!(
                    storage.$get(&object).err().unwrap().class,
                    ErrorClass::NotFound
                );
                assert!(matches!(
                    storage.$stat(&object).unwrap(),
                    StatOutcome::Present(_)
                ));
                assert_eq!(
                    mock.stored_bytes(endpoint(), &object.logical_key.0)
                        .unwrap(),
                    BYTES
                );
            }
        }
    };
}

byte_cases!(
    resource,
    hash_resource_bytes,
    put_resource,
    get_resource,
    stat_resource,
    PutResource,
    GetResource,
    StatResource
);
byte_cases!(
    chunk_content,
    chunk,
    put_chunk,
    get_chunk,
    stat_chunk,
    PutChunk,
    GetChunk,
    StatChunk
);

#[test]
fn identical_input_and_script_produce_identical_results_calls_and_storage() {
    fn run() -> (Vec<omvcs_storage_mock::Call>, Vec<ScriptStep>, Vec<u8>) {
        let mock = setup();
        let object = request(hash_resource_bytes(BYTES));
        inject(
            &mock,
            Operation::PutResource,
            Effect::FailAfter {
                bytes: 4,
                class: ErrorClass::Network,
                recoverable: true,
            },
        );
        inject(&mock, Operation::PutResource, Effect::Pass);
        inject(
            &mock,
            Operation::GetResource,
            Effect::Corrupt {
                offset: 1,
                xor: 0x80,
            },
        );
        let mut storage = Storage::new(mock.clone());
        assert_eq!(
            storage
                .put_resource(&put(&object, BYTES), &mut Cursor::new(BYTES))
                .err()
                .unwrap()
                .class,
            ErrorClass::Network
        );
        storage
            .put_resource(&put(&object, BYTES), &mut Cursor::new(BYTES))
            .unwrap();
        let mut download = storage.get_resource(&object).unwrap();
        assert!(download.read_to_end(&mut vec![]).is_err());
        assert_eq!(
            download.complete().err().unwrap().class,
            ErrorClass::Integrity
        );
        (
            mock.calls(),
            mock.pending_script(),
            mock.stored_bytes(endpoint(), &object.logical_key.0)
                .unwrap(),
        )
    }
    assert_eq!(run(), run());
}

#[test]
fn unknown_endpoint_and_discovery_failure_are_not_absence() {
    let mut mock = MockStorage::new();
    let object = request(ObjectId::Resource(hash_resource_bytes(BYTES)));
    assert_eq!(
        mock.stat(&object).err().unwrap().class,
        ErrorClass::Configuration
    );
    for class in [ErrorClass::Authentication, ErrorClass::ProviderUnavailable] {
        mock.configure_endpoint(
            endpoint(),
            EndpointConfiguration {
                discovery_failure: Some(class),
                ..EndpointConfiguration::default()
            },
        );
        assert_eq!(mock.stat(&object).err().unwrap().class, class);
        assert_eq!(mock.calls(), []);
    }
}

#[test]
fn script_mismatch_invalid_effect_and_invalid_corruption_offset_are_explicit() {
    let mut mock = setup();
    let object = request(ObjectId::Resource(hash_resource_bytes(BYTES)));
    assert!(
        mock.enqueue(ScriptStep {
            endpoint: endpoint(),
            operation: Operation::StatResource,
            logical_key: object.logical_key.0.clone(),
            effect: Effect::Truncate(1)
        })
        .is_err()
    );
    inject(&mock, Operation::GetResource, Effect::Pass);
    assert_eq!(
        mock.stat(&object).err().unwrap().class,
        ErrorClass::Internal
    );
    assert_eq!(mock.pending_script().len(), 1);
    assert_eq!(mock.get(&object).err().unwrap().class, ErrorClass::NotFound);
    mock.put(&put(&object, BYTES), &mut Cursor::new(BYTES))
        .unwrap();
    inject(
        &mock,
        Operation::GetResource,
        Effect::Corrupt {
            offset: BYTES.len(),
            xor: 1,
        },
    );
    assert_eq!(
        mock.get(&object).err().unwrap().class,
        ErrorClass::InvalidRequest
    );
}

#[test]
fn direct_byte_boundary_rejects_wrong_hash_length_and_input_bytes() {
    let mut mock = setup();
    let object = request(ObjectId::Resource(hash_resource_bytes(BYTES)));
    let mut input = put(&object, BYTES);
    input.expected_hash = [0; 32];
    assert_eq!(
        mock.put(&input, &mut Cursor::new(BYTES))
            .err()
            .unwrap()
            .class,
        ErrorClass::InvalidRequest
    );
    input = put(&object, BYTES);
    input.byte_length += 1;
    assert_eq!(
        mock.put(&input, &mut Cursor::new(BYTES))
            .err()
            .unwrap()
            .class,
        ErrorClass::Integrity
    );
    assert_eq!(
        mock.put(&put(&object, BYTES), &mut Cursor::new(b"wrong"))
            .err()
            .unwrap()
            .class,
        ErrorClass::Integrity
    );
    assert_eq!(mock.stat(&object).unwrap(), StatOutcome::Absent);
}

#[test]
fn endpoint_and_key_changes_do_not_define_content_identity_or_shared_objects() {
    let mut mock = setup();
    let other: StorageEndpointId = "019cc17d-1b22-7a41-9fe9-c345c468f82d".parse().unwrap();
    mock.configure_endpoint(other, EndpointConfiguration::default());
    let a = request(ObjectId::Resource(hash_resource_bytes(BYTES)));
    let mut b = a.clone();
    b.endpoint = other;
    mock.put(&put(&a, BYTES), &mut Cursor::new(BYTES)).unwrap();
    assert_eq!(mock.stat(&b).unwrap(), StatOutcome::Absent);
    mock.put(&put(&b, b"different"), &mut Cursor::new(b"different"))
        .unwrap_err();
    b.logical_key.0 = "another-opaque-key".into();
    mock.put(&put(&b, BYTES), &mut Cursor::new(BYTES)).unwrap();
    assert_eq!(a.id, b.id);
    assert_eq!(
        mock.stored_bytes(a.endpoint, &a.logical_key.0),
        mock.stored_bytes(b.endpoint, &b.logical_key.0)
    );
    assert_eq!(mock.adapter_info().adapter_id, "org.openmusic.storage.mock");
}
