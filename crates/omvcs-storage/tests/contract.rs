//! WORK-0015 conformance cases. Minimal fakes only, not the official mock Adapter.
//! Storage §§9, 12, 16–25, 29, 35–36, 85–86, 181–182, 185;
//! Core §§6–8.1; INV-RES-001–007, INV-STOR-001–003, INV-INT-001–003.

#![allow(clippy::unwrap_used)]

use std::{
    cell::RefCell,
    collections::HashMap,
    io::{self, Cursor, Read},
    rc::Rc,
};

use omvcs_storage::{
    AdapterInfo, ByteStorage, Capabilities, ChunkId, ErrorClass, LogicalKey, ObjectId,
    ObjectRequest, Operation, OperationId, ProviderDownload, ProviderMetadata, PutRequest,
    ResourceId, StatOutcome, Storage, StorageEndpointId, StorageError,
};
use sha2::{Digest, Sha256};

const ALL: Capabilities = Capabilities {
    put_resource: true,
    get_resource: true,
    stat_resource: true,
    put_chunk: true,
    get_chunk: true,
    stat_chunk: true,
};
const BYTES: &[u8] = b"\0RIFF\r\n\xff\x80no normalization\n";

#[derive(Default)]
#[allow(clippy::struct_excessive_bools)] // Independent test-only fault switches.
struct State {
    caps: HashMap<StorageEndpointId, Capabilities>,
    objects: HashMap<(StorageEndpointId, LogicalKey), Vec<u8>>,
    calls: Vec<(Operation, ObjectRequest<ObjectId>)>,
    failure: Option<ErrorClass>,
    capability_failure: bool,
    skip_input: bool,
    swallow_input_failure: bool,
    broken_download: bool,
    // Sentinel external state: never provided to storage operations.
    historical_ids: Vec<ResourceId>,
    replica_records: Vec<String>,
}

struct Fake(Rc<RefCell<State>>);

fn failure(class: ErrorClass, operation: Operation, endpoint: StorageEndpointId) -> StorageError {
    StorageError {
        code: "FAKE_INJECTED".into(),
        class,
        operation,
        endpoint,
        recoverable: matches!(class, ErrorClass::Network | ErrorClass::ProviderUnavailable),
        provider_context: Some("sanitized test context".into()),
        summary: "Injected failure".into(),
    }
}

const fn operation(id: ObjectId, put: bool, stat: bool) -> Operation {
    match (id, put, stat) {
        (ObjectId::Resource(_), true, _) => Operation::PutResource,
        (ObjectId::Resource(_), false, true) => Operation::StatResource,
        (ObjectId::Resource(_), false, false) => Operation::GetResource,
        (ObjectId::Chunk(_), true, _) => Operation::PutChunk,
        (ObjectId::Chunk(_), false, true) => Operation::StatChunk,
        (ObjectId::Chunk(_), false, false) => Operation::GetChunk,
    }
}

impl ByteStorage for Fake {
    fn adapter_info(&self) -> AdapterInfo {
        AdapterInfo {
            adapter_id: "org.omvcs.test.byte-fake".into(),
            adapter_version: "test".into(),
            storage_contract_version: "omvcs.storage-adapter/0.1".into(),
            provider_family: "test-only".into(),
            supported_provider_versions: vec![],
        }
    }

    fn capabilities(
        &self,
        endpoint: StorageEndpointId,
        operation: Operation,
    ) -> Result<Capabilities, StorageError> {
        if self.0.borrow().capability_failure {
            return Err(failure(ErrorClass::Configuration, operation, endpoint));
        }
        Ok(self.0.borrow().caps.get(&endpoint).copied().unwrap_or(ALL))
    }

    fn put(
        &mut self,
        request: &PutRequest<ObjectId>,
        input: &mut dyn Read,
    ) -> Result<ProviderMetadata, StorageError> {
        let op = operation(request.object.id, true, false);
        let mut state = self.0.borrow_mut();
        state.calls.push((op, request.object.clone()));
        if let Some(class) = state.failure {
            return Err(failure(class, op, request.object.endpoint));
        }
        if state.skip_input {
            return Ok(ProviderMetadata::default());
        }
        let mut bytes = vec![];
        if let Err(error) = input.read_to_end(&mut bytes) {
            if state.swallow_input_failure {
                return Ok(ProviderMetadata::default());
            }
            return Err(error
                .get_ref()
                .and_then(|e| e.downcast_ref::<StorageError>())
                .cloned()
                .unwrap_or_else(|| {
                    failure(ErrorClass::Precondition, op, request.object.endpoint)
                }));
        }
        let key = (request.object.endpoint, request.object.logical_key.clone());
        // §20: compare complete existing bytes, not stat hints. §21: never overwrite.
        if let Some(existing) = state.objects.get(&key) {
            if *existing != bytes {
                return Err(failure(ErrorClass::Integrity, op, request.object.endpoint));
            }
        } else {
            state.objects.insert(key, bytes.clone());
        }
        Ok(metadata(bytes.len()))
    }

    fn get(
        &mut self,
        request: &ObjectRequest<ObjectId>,
    ) -> Result<ProviderDownload<'_>, StorageError> {
        let op = operation(request.id, false, false);
        let mut state = self.0.borrow_mut();
        state.calls.push((op, request.clone()));
        if let Some(class) = state.failure {
            return Err(failure(class, op, request.endpoint));
        }
        let bytes = state
            .objects
            .get(&(request.endpoint, request.logical_key.clone()))
            .ok_or_else(|| failure(ErrorClass::NotFound, op, request.endpoint))?
            .clone();
        let length = bytes.len();
        let stream: Box<dyn Read> = if state.broken_download {
            Box::new(BrokenStream {
                prefix: Cursor::new(bytes),
                failed: false,
                error: failure(ErrorClass::Network, op, request.endpoint),
            })
        } else {
            Box::new(Cursor::new(bytes))
        };
        Ok(ProviderDownload {
            stream,
            metadata: metadata(length),
        })
    }

    fn stat(&mut self, request: &ObjectRequest<ObjectId>) -> Result<StatOutcome, StorageError> {
        let op = operation(request.id, false, true);
        let mut state = self.0.borrow_mut();
        state.calls.push((op, request.clone()));
        if let Some(class) = state.failure {
            return Err(failure(class, op, request.endpoint));
        }
        Ok(state
            .objects
            .get(&(request.endpoint, request.logical_key.clone()))
            .map_or(StatOutcome::Absent, |bytes| {
                StatOutcome::Present(metadata(bytes.len()))
            }))
    }
}

fn metadata(length: usize) -> ProviderMetadata {
    ProviderMetadata {
        byte_length: Some(length as u64),
        checksum: Some("not-an-OMVCS-hash".into()),
        version_id: Some("provider-id".into()),
        last_modified: Some("mutable-provider-time".into()),
    }
}

// Delivers a prefix then fails, even when the prefix is all expected bytes:
// no EOF means no completed transfer.
struct BrokenStream {
    prefix: Cursor<Vec<u8>>,
    failed: bool,
    error: StorageError,
}

impl Read for BrokenStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.failed || self.prefix.position() == self.prefix.get_ref().len() as u64 {
            self.failed = true;
            return Err(io::Error::other(self.error.clone()));
        }
        let length = buf.len().min(3);
        self.prefix.read(&mut buf[..length])
    }
}

fn setup() -> (Storage<Fake>, Rc<RefCell<State>>) {
    let state = Rc::new(RefCell::new(State::default()));
    (Storage::new(Fake(state.clone())), state)
}

fn resource(bytes: &[u8]) -> ResourceId {
    omvcs_model::hashing::hash_resource_bytes(bytes)
}

fn chunk(bytes: &[u8]) -> ChunkId {
    ChunkId::from_digest(Sha256::digest(bytes).into())
}

fn request<I>(id: I) -> ObjectRequest<I> {
    ObjectRequest {
        endpoint: StorageEndpointId::new(),
        id,
        logical_key: LogicalKey("opaque caller key".into()),
        operation_id: OperationId("caller operation".into()),
    }
}

fn put<I: Copy + Into<ObjectId>>(object: &ObjectRequest<I>, bytes: &[u8]) -> PutRequest<I> {
    PutRequest {
        object: object.clone(),
        byte_length: bytes.len() as u64,
        expected_hash: *object.id.into().digest(),
    }
}

macro_rules! content_cases {
    ($module:ident, $identify:ident, $put:ident, $get:ident, $stat:ident, $get_op:ident, $stat_op:ident) => {
        mod $module {
            use super::*;

            #[test]
            fn exact_bytes_round_trip_and_caller_identity_is_preserved() {
                for bytes in [BYTES, b"".as_slice(), b"\r\n\0\xff".as_slice()] {
                    let (mut storage, state) = setup();
                    let object = request($identify(bytes));
                    let outcome = storage.$put(&put(&object, bytes), &mut Cursor::new(bytes)).unwrap();
                    assert_eq!(outcome.id, object.id);
                    let mut download = storage.$get(&object).unwrap();
                    let mut actual = vec![];
                    download.read_to_end(&mut actual).unwrap();
                    assert_eq!(actual, bytes);
                    let outcome = download.complete().unwrap();
                    assert_eq!(outcome.id, object.id);
                    assert_eq!(outcome.byte_length, bytes.len() as u64);
                    for (_, seen) in &state.borrow().calls {
                        assert_eq!(seen, &erase_for_test(&object));
                    }
                }
            }

            #[test]
            fn absent_provider_failure_and_unsupported_are_distinct() {
                let (mut storage, state) = setup();
                let object = request($identify(BYTES));
                assert_eq!(storage.$stat(&object).unwrap(), StatOutcome::Absent);
                assert_eq!(storage.$get(&object).err().unwrap().class, ErrorClass::NotFound);
                for class in [ErrorClass::Authentication, ErrorClass::Authorization,
                    ErrorClass::Network, ErrorClass::ProviderUnavailable] {
                    state.borrow_mut().failure = Some(class);
                    let error = storage.$stat(&object).unwrap_err();
                    assert_eq!(error.class, class);
                    assert_eq!(error.operation, Operation::$stat_op);
                    assert_eq!(error.endpoint, object.endpoint);
                    assert_eq!(error.provider_context.as_deref(), Some("sanitized test context"));
                    let error = storage.$get(&object).err().unwrap();
                    assert_eq!(error.class, class);
                    assert_eq!(error.operation, Operation::$get_op);
                }
                state.borrow_mut().caps.insert(object.endpoint, Capabilities::default());
                assert_eq!(storage.$stat(&object).unwrap_err().class, ErrorClass::Unsupported);
                assert_eq!(storage.$get(&object).err().unwrap().class, ErrorClass::Unsupported);
                assert_eq!(storage.$put(&put(&object, BYTES), &mut Cursor::new(BYTES))
                    .unwrap_err().class, ErrorClass::Unsupported);
            }

            #[test]
            fn wrong_bytes_and_matching_stat_length_are_not_success() {
                let (mut storage, state) = setup();
                let object = request($identify(BYTES));
                let wrong = vec![42; BYTES.len()];
                state.borrow_mut().objects.insert(
                    (object.endpoint, object.logical_key.clone()), wrong);
                assert!(matches!(storage.$stat(&object).unwrap(),
                    StatOutcome::Present(ProviderMetadata { byte_length: Some(n), .. })
                        if n == BYTES.len() as u64));
                let mut download = storage.$get(&object).unwrap();
                assert!(download.read_to_end(&mut vec![]).is_err());
                assert_eq!(download.complete().unwrap_err().class, ErrorClass::Integrity);
            }

            #[test]
            fn wrong_input_bytes_and_false_provider_acknowledgement_fail() {
                let (mut storage, state) = setup();
                let object = request($identify(BYTES));
                let wrong = vec![42; BYTES.len()];
                state.borrow_mut().swallow_input_failure = true;
                assert_eq!(storage.$put(&put(&object, BYTES), &mut Cursor::new(wrong))
                    .unwrap_err().class, ErrorClass::Integrity);
                assert!(state.borrow().objects.is_empty());
                state.borrow_mut().skip_input = true;
                assert_eq!(storage.$put(&put(&object, BYTES), &mut Cursor::new(BYTES))
                    .unwrap_err().class, ErrorClass::Precondition);
            }

            #[test]
            fn input_length_and_expected_hash_are_checked() {
                let (mut storage, state) = setup();
                let object = request($identify(BYTES));
                let mut input = put(&object, BYTES);
                input.expected_hash = [0; 32];
                assert_eq!(storage.$put(&input, &mut Cursor::new(BYTES))
                    .unwrap_err().class, ErrorClass::InvalidRequest);
                assert!(state.borrow().calls.is_empty());
                for length in [0, BYTES.len() as u64 + 1] {
                    let mut input = put(&object, BYTES);
                    input.byte_length = length;
                    assert_eq!(storage.$put(&input, &mut Cursor::new(BYTES))
                        .unwrap_err().class, ErrorClass::Integrity);
                }
            }

            #[test]
            fn existing_content_requires_correspondence_and_conflicts_do_not_overwrite() {
                let (mut storage, state) = setup();
                let object = request($identify(BYTES));
                storage.$put(&put(&object, BYTES), &mut Cursor::new(BYTES)).unwrap();
                storage.$put(&put(&object, BYTES), &mut Cursor::new(BYTES)).unwrap();
                let different = b"different immutable content";
                let conflicting = ObjectRequest { id: $identify(different), ..object.clone() };
                assert_eq!(storage.$put(&put(&conflicting, different), &mut Cursor::new(different))
                    .unwrap_err().class, ErrorClass::Integrity);
                assert_eq!(state.borrow().objects[&(object.endpoint, object.logical_key.clone())], BYTES);
                // Same ID, existing corrupted bytes and even matching length:
                state.borrow_mut().objects.insert((object.endpoint, object.logical_key.clone()),
                    vec![42; BYTES.len()]);
                assert_eq!(storage.$put(&put(&object, BYTES), &mut Cursor::new(BYTES))
                    .unwrap_err().class, ErrorClass::Integrity);
            }

            #[test]
            fn partial_and_failed_transfers_never_complete() {
                let (mut storage, state) = setup();
                let object = request($identify(BYTES));
                let mut broken = BrokenStream { prefix: Cursor::new(BYTES[..3].to_vec()),
                    failed: false, error: failure(ErrorClass::Cancelled,
                        operation(object.id.into(), true, false), object.endpoint) };
                assert_eq!(storage.$put(&put(&object, BYTES), &mut broken)
                    .unwrap_err().class, ErrorClass::Cancelled);
                assert!(state.borrow().objects.is_empty());
                // Retry succeeds without duplicate physical test content.
                storage.$put(&put(&object, BYTES), &mut Cursor::new(BYTES)).unwrap();
                state.borrow_mut().broken_download = true;
                let mut download = storage.$get(&object).unwrap();
                assert!(download.read_to_end(&mut vec![]).is_err());
                let error = download.complete().unwrap_err();
                assert_eq!(error.class, ErrorClass::Network);
                assert!(error.recoverable);
                state.borrow_mut().broken_download = false;
                let mut download = storage.$get(&object).unwrap();
                let mut prefix = [0; 1];
                download.read_exact(&mut prefix).unwrap();
                assert_eq!(download.complete().unwrap_err().class, ErrorClass::Precondition);
            }

            #[test]
            fn endpoint_and_logical_key_changes_do_not_change_content_identity() {
                let (mut storage, state) = setup();
                let object = request($identify(BYTES));
                for index in 0..4 {
                    let address = ObjectRequest { endpoint: StorageEndpointId::new(),
                        logical_key: LogicalKey(format!("caller-key-{index}")),
                        operation_id: OperationId(format!("operation-{index}")), ..object.clone() };
                    assert_eq!(storage.$put(&put(&address, BYTES), &mut Cursor::new(BYTES))
                        .unwrap().id, object.id);
                    let mut download = storage.$get(&address).unwrap();
                    download.read_to_end(&mut vec![]).unwrap();
                    assert_eq!(download.complete().unwrap().id, object.id);
                }
                assert_eq!(state.borrow().objects.len(), 4);
            }
        }
    };
}

fn erase_for_test<I: Copy + Into<ObjectId>>(r: &ObjectRequest<I>) -> ObjectRequest<ObjectId> {
    ObjectRequest {
        endpoint: r.endpoint,
        id: r.id.into(),
        logical_key: r.logical_key.clone(),
        operation_id: r.operation_id.clone(),
    }
}

content_cases!(
    resources,
    resource,
    put_resource,
    get_resource,
    stat_resource,
    GetResource,
    StatResource
);
content_cases!(
    chunks, chunk, put_chunk, get_chunk, stat_chunk, GetChunk, StatChunk
);

#[test]
fn six_capabilities_are_independent_truthful_and_endpoint_specific() {
    let (mut storage, state) = setup();
    let resource = request(resource(BYTES));
    let chunk = ObjectRequest {
        id: chunk(BYTES),
        endpoint: resource.endpoint,
        logical_key: resource.logical_key.clone(),
        operation_id: resource.operation_id.clone(),
    };
    for op in [
        Operation::PutResource,
        Operation::GetResource,
        Operation::StatResource,
        Operation::PutChunk,
        Operation::GetChunk,
        Operation::StatChunk,
    ] {
        let caps = Capabilities {
            put_resource: op == Operation::PutResource,
            get_resource: op == Operation::GetResource,
            stat_resource: op == Operation::StatResource,
            put_chunk: op == Operation::PutChunk,
            get_chunk: op == Operation::GetChunk,
            stat_chunk: op == Operation::StatChunk,
        };
        state.borrow_mut().caps.insert(resource.endpoint, caps);
        assert_eq!(storage.capabilities(resource.endpoint, op).unwrap(), caps);
        assert_eq!(
            storage.capabilities(StorageEndpointId::new(), op).unwrap(),
            ALL
        );
        for candidate in [
            Operation::PutResource,
            Operation::GetResource,
            Operation::StatResource,
            Operation::PutChunk,
            Operation::GetChunk,
            Operation::StatChunk,
        ] {
            state.borrow_mut().calls.clear();
            let result = match candidate {
                Operation::PutResource => storage
                    .put_resource(&put(&resource, BYTES), &mut Cursor::new(BYTES))
                    .map(|_| ()),
                Operation::GetResource => storage.get_resource(&resource).map(|_| ()),
                Operation::StatResource => storage.stat_resource(&resource).map(|_| ()),
                Operation::PutChunk => storage
                    .put_chunk(&put(&chunk, BYTES), &mut Cursor::new(BYTES))
                    .map(|_| ()),
                Operation::GetChunk => storage.get_chunk(&chunk).map(|_| ()),
                Operation::StatChunk => storage.stat_chunk(&chunk).map(|_| ()),
            };
            if candidate == op {
                assert_eq!(state.borrow().calls.len(), 1);
            } else {
                assert_eq!(result.unwrap_err().class, ErrorClass::Unsupported);
                assert_eq!(state.borrow().calls, []);
            }
        }
    }
}

#[test]
fn resource_length_domain_is_not_widened_by_storage() {
    let (mut storage, state) = setup();
    let object = request(resource(BYTES));
    let mut input = put(&object, BYTES);
    input.byte_length = omvcs_model::resource::MAX_RESOURCE_BYTE_LENGTH + 1;
    assert_eq!(
        storage
            .put_resource(&input, &mut Cursor::new(BYTES))
            .unwrap_err()
            .class,
        ErrorClass::InvalidRequest
    );
    assert_eq!(state.borrow().calls, []);
}

#[test]
fn chunk_namespace_is_typed_canonical_and_hashes_only_chunk_bytes() {
    let id = chunk(b"abc");
    assert_eq!(
        id.to_string(),
        "omvcs:chunk:sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(id.to_string().parse::<ChunkId>().unwrap(), id);
    assert!(resource(b"abc").to_string().parse::<ChunkId>().is_err());
    assert!(id.to_string().parse::<ResourceId>().is_err());
    for malformed in [
        "omvcs:chunk:sha256:abc".into(),
        id.to_string().to_uppercase(),
        format!("{id} "),
        id.to_string().replace("sha256", "sha512"),
    ] {
        assert!(malformed.parse::<ChunkId>().is_err());
    }
    assert_eq!(id.digest(), resource(b"abc").digest());
}

#[test]
fn byte_io_has_no_historical_or_replica_effects_and_no_implicit_chunking() {
    let (mut storage, state) = setup();
    let bytes = vec![23; 32_769];
    let object = request(resource(&bytes));
    state.borrow_mut().historical_ids.push(object.id);
    state
        .borrow_mut()
        .replica_records
        .push("existing external sentinel".into());
    let history = state.borrow().historical_ids.clone();
    let replicas = state.borrow().replica_records.clone();
    storage
        .put_resource(&put(&object, &bytes), &mut Cursor::new(&bytes))
        .unwrap();
    let mut download = storage.get_resource(&object).unwrap();
    download.read_to_end(&mut vec![]).unwrap();
    download.complete().unwrap();
    state.borrow_mut().failure = Some(ErrorClass::Quota);
    assert_eq!(
        storage
            .put_resource(&put(&object, &bytes), &mut Cursor::new(&bytes))
            .unwrap_err()
            .class,
        ErrorClass::Quota
    );
    assert_eq!(state.borrow().historical_ids, history);
    assert_eq!(state.borrow().replica_records, replicas);
    assert!(
        state
            .borrow()
            .calls
            .iter()
            .all(|(op, _)| matches!(op, Operation::PutResource | Operation::GetResource))
    );
}

// Generates bytes on demand: no object-sized input/output buffer.
struct RepeatedBytes(u64);
impl Read for RepeatedBytes {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let count = usize::try_from(self.0.min(buf.len() as u64)).unwrap();
        buf[..count].fill(17);
        self.0 -= count as u64;
        Ok(count)
    }
}

struct StreamingOnly;
impl ByteStorage for StreamingOnly {
    fn adapter_info(&self) -> AdapterInfo {
        Fake(Rc::default()).adapter_info()
    }
    fn capabilities(
        &self,
        _: StorageEndpointId,
        _: Operation,
    ) -> Result<Capabilities, StorageError> {
        Ok(ALL)
    }
    fn put(
        &mut self,
        r: &PutRequest<ObjectId>,
        input: &mut dyn Read,
    ) -> Result<ProviderMetadata, StorageError> {
        io::copy(input, &mut io::sink()).map_err(|_| {
            failure(
                ErrorClass::Internal,
                Operation::PutResource,
                r.object.endpoint,
            )
        })?;
        Ok(ProviderMetadata::default())
    }
    fn get(&mut self, _: &ObjectRequest<ObjectId>) -> Result<ProviderDownload<'_>, StorageError> {
        Ok(ProviderDownload {
            stream: Box::new(RepeatedBytes(1_048_577)),
            metadata: ProviderMetadata::default(),
        })
    }
    fn stat(&mut self, _: &ObjectRequest<ObjectId>) -> Result<StatOutcome, StorageError> {
        Ok(StatOutcome::Present(ProviderMetadata::default()))
    }
}

#[test]
fn streams_can_transfer_without_an_object_sized_buffer() {
    let mut hash = Sha256::new();
    for _ in 0..256 {
        hash.update([17; 4096]);
    }
    hash.update([17]);
    let id = ResourceId::from_digest(hash.finalize().into());
    let object = request(id);
    let input = PutRequest {
        object: object.clone(),
        byte_length: 1_048_577,
        expected_hash: *id.digest(),
    };
    let mut storage = Storage::new(StreamingOnly);
    assert_eq!(
        storage
            .put_resource(&input, &mut RepeatedBytes(input.byte_length))
            .unwrap()
            .byte_length,
        input.byte_length
    );
    let mut download = storage.get_resource(&object).unwrap();
    assert_eq!(
        io::copy(&mut download, &mut io::sink()).unwrap(),
        input.byte_length
    );
    assert_eq!(download.complete().unwrap().id, id);
}

#[test]
fn empty_read_is_not_eof_or_success() {
    let (mut storage, _) = setup();
    let object = request(resource(b""));
    storage
        .put_resource(&put(&object, b""), &mut Cursor::new(b""))
        .unwrap();
    let mut download = storage.get_resource(&object).unwrap();
    assert_eq!(download.read(&mut []).unwrap(), 0);
    assert_eq!(
        download.complete().unwrap_err().class,
        ErrorClass::Precondition
    );
}

#[test]
fn failed_capability_discovery_is_not_support_or_absence() {
    let (mut storage, state) = setup();
    state.borrow_mut().capability_failure = true;
    let object = request(resource(BYTES));
    let error = storage.stat_resource(&object).unwrap_err();
    assert_eq!(error.class, ErrorClass::Configuration);
    assert_eq!(error.operation, Operation::StatResource);
    assert_eq!(error.endpoint, object.endpoint);
    assert_eq!(state.borrow().calls, []);
}

struct InterruptedOnce<R> {
    stream: R,
    interrupted: bool,
}

impl<R: Read> Read for InterruptedOnce<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.interrupted {
            self.stream.read(buf)
        } else {
            self.interrupted = true;
            Err(io::ErrorKind::Interrupted.into())
        }
    }
}

#[test]
fn retryable_read_interruption_does_not_hash_or_duplicate_bytes() {
    let (mut storage, _) = setup();
    let object = request(resource(BYTES));
    let mut input = InterruptedOnce {
        stream: Cursor::new(BYTES),
        interrupted: false,
    };
    storage
        .put_resource(&put(&object, BYTES), &mut input)
        .unwrap();
    let mut download = storage.get_resource(&object).unwrap();
    let mut actual = vec![];
    download.read_to_end(&mut actual).unwrap();
    download.complete().unwrap();
    assert_eq!(actual, BYTES);
}
