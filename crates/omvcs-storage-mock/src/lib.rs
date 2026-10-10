//! Deterministic mock Storage Adapter for conformance and failure injection.
//!
//! Only the six WORK-0015 byte operations are implemented. No persistence,
//! Repository Home, Replica registration, deletion, retries, clocks, or
//! background work is provided. Streams are accepted incrementally, but this
//! deliberately in-memory backend retains object bytes in memory.
//!
//! Clones share one explicitly controlled fixture state. Downloads own a byte
//! snapshot taken when opened; subsequent fixture changes do not alter it.
//! Script steps match the next byte operation exactly. Capability discovery
//! does not consume steps. An unexpected call fails without consuming a step.
//! These are test controls, not provider consistency or transaction guarantees.

#![forbid(unsafe_code)]

use std::{
    cell::RefCell,
    collections::{BTreeMap, VecDeque},
    io::{self, Cursor, Read},
    rc::Rc,
};

use omvcs_model::verification::{ProviderChecksumAlgorithm, ProviderChecksumScope};
use omvcs_storage::{
    AdapterInfo, ByteStorage, Capabilities, ErrorClass, ObjectId, ObjectRequest, Operation,
    ProviderChecksumReport, ProviderDownload, ProviderMetadata, PutRequest, StatOutcome,
    StorageEndpointId, StorageError,
};
use sha2::{Digest, Sha256};

/// All six byte operations; not a conformance-class designation.
pub const ALL_BYTE_OPERATIONS: Capabilities = Capabilities {
    put_resource: true,
    get_resource: true,
    stat_resource: true,
    put_chunk: true,
    get_chunk: true,
    stat_chunk: true,
};

/// Explicit test-controlled behavior for one byte operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    Pass,
    Fail {
        class: ErrorClass,
        recoverable: bool,
    },
    /// Transfer interruption before EOF, including after all expected bytes.
    FailAfter {
        bytes: u64,
        class: ErrorClass,
        recoverable: bool,
    },
    /// Simulated absence, without removing the stored object. Get/stat only.
    Absent,
    /// Adversarial output only; never rewrites immutable stored bytes.
    ReturnBytes(Vec<u8>),
    /// Adversarial early EOF. Get only.
    Truncate(usize),
    /// Adversarial output corruption. Get only; invalid offsets fail explicitly.
    Corrupt {
        offset: usize,
        xor: u8,
    },
}

/// FIFO expectation. Endpoint and caller key are test selectors, not identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptStep {
    pub endpoint: StorageEndpointId,
    pub operation: Operation,
    pub logical_key: String,
    pub effect: Effect,
}

/// Inspectable record of an attempted byte operation and selected effect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Call {
    pub request: ObjectRequest<ObjectId>,
    pub operation: Operation,
    pub effect: Effect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndpointConfiguration {
    pub capabilities: Capabilities,
    /// Explicit discovery failure; not proof of object absence.
    pub discovery_failure: Option<ErrorClass>,
}

impl Default for EndpointConfiguration {
    fn default() -> Self {
        Self {
            capabilities: ALL_BYTE_OPERATIONS,
            discovery_failure: None,
        }
    }
}

#[derive(Debug, Default)]
struct State {
    endpoints: BTreeMap<StorageEndpointId, EndpointConfiguration>,
    objects: BTreeMap<(StorageEndpointId, String), Vec<u8>>,
    script: VecDeque<ScriptStep>,
    calls: Vec<Call>,
}

/// No endpoints are implicit: callers configure stable, caller-assigned IDs.
#[derive(Debug, Clone, Default)]
pub struct MockStorage(Rc<RefCell<State>>);

impl MockStorage {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Changes fixture capabilities only; does not change stored bytes or IDs.
    pub fn configure_endpoint(&self, endpoint: StorageEndpointId, config: EndpointConfiguration) {
        self.0.borrow_mut().endpoints.insert(endpoint, config);
    }

    /// Adds one explicit expectation. Invalid effect/operation combinations are
    /// rejected before execution, rather than ignored.
    ///
    /// # Errors
    /// Returns `InvalidRequest` for an inapplicable effect.
    pub fn enqueue(&self, step: ScriptStep) -> Result<(), StorageError> {
        let get = matches!(step.operation, Operation::GetResource | Operation::GetChunk);
        let transfer =
            get || matches!(step.operation, Operation::PutResource | Operation::PutChunk);
        let valid = match step.effect {
            Effect::Pass | Effect::Fail { .. } => true,
            Effect::FailAfter { .. } => transfer,
            Effect::Absent => {
                get || matches!(
                    step.operation,
                    Operation::StatResource | Operation::StatChunk
                )
            }
            Effect::ReturnBytes(_) | Effect::Truncate(_) | Effect::Corrupt { .. } => get,
        };
        if !valid {
            return Err(failure(
                step.endpoint,
                step.operation,
                ErrorClass::InvalidRequest,
                false,
            ));
        }
        self.0.borrow_mut().script.push_back(step);
        Ok(())
    }

    #[must_use]
    pub fn pending_script(&self) -> Vec<ScriptStep> {
        self.0.borrow().script.iter().cloned().collect()
    }

    #[must_use]
    pub fn calls(&self) -> Vec<Call> {
        self.0.borrow().calls.clone()
    }

    /// Exact stored bytes, for fixture assertions only. Does not verify content.
    #[must_use]
    pub fn stored_bytes(&self, endpoint: StorageEndpointId, key: &str) -> Option<Vec<u8>> {
        self.0
            .borrow()
            .objects
            .get(&(endpoint, key.into()))
            .cloned()
    }

    /// Mock-only checksum facility: performs the scripted stat first, then
    /// computes SHA-256 of the actual complete stored Resource or Chunk bytes.
    /// This is not a checksum of a manifest, `ETag`, or supplied expected ID.
    ///
    /// # Errors
    /// Propagates stat failures/unsupported/absence. No report on failure.
    pub fn checksum_report(
        &mut self,
        request: &ObjectRequest<ObjectId>,
    ) -> Result<ProviderChecksumReport, StorageError> {
        let op = operation(request.id, Access::Stat);
        if self.stat(request)? == StatOutcome::Absent {
            return Err(failure(request.endpoint, op, ErrorClass::NotFound, false));
        }
        let state = self.0.borrow();
        let bytes = state
            .objects
            .get(&(request.endpoint, request.logical_key.0.clone()))
            .ok_or_else(|| failure(request.endpoint, op, ErrorClass::NotFound, false))?;
        Ok(ProviderChecksumReport::new(
            ProviderChecksumAlgorithm::OmvcsSha256,
            match request.id {
                ObjectId::Resource(_) => ProviderChecksumScope::CompleteResourceBytes,
                ObjectId::Chunk(_) => ProviderChecksumScope::ExactChunkBytes,
            },
            Sha256::digest(bytes).into(),
        ))
    }

    fn begin(
        &self,
        request: &ObjectRequest<ObjectId>,
        op: Operation,
    ) -> Result<Effect, StorageError> {
        if !self.capabilities(request.endpoint, op)?.supports(op) {
            return Err(failure(
                request.endpoint,
                op,
                ErrorClass::Unsupported,
                false,
            ));
        }
        let mut state = self.0.borrow_mut();
        let effect = if let Some(step) = state.script.front() {
            if step.endpoint != request.endpoint
                || step.operation != op
                || step.logical_key != request.logical_key.0
            {
                return Err(failure(request.endpoint, op, ErrorClass::Internal, false));
            }
            let effect = step.effect.clone();
            state.script.pop_front();
            effect
        } else {
            Effect::Pass
        };
        state.calls.push(Call {
            request: request.clone(),
            operation: op,
            effect: effect.clone(),
        });
        if let Effect::Fail { class, recoverable } = effect {
            return Err(failure(request.endpoint, op, class, recoverable));
        }
        Ok(effect)
    }
}

impl ByteStorage for MockStorage {
    fn adapter_info(&self) -> AdapterInfo {
        AdapterInfo {
            adapter_id: "org.openmusic.storage.mock".into(),
            adapter_version: env!("CARGO_PKG_VERSION").into(),
            storage_contract_version: "omvcs.storage-adapter/0.1".into(),
            provider_family: "in-memory-test-only".into(),
            supported_provider_versions: vec![],
        }
    }

    fn capabilities(
        &self,
        endpoint: StorageEndpointId,
        op: Operation,
    ) -> Result<Capabilities, StorageError> {
        let state = self.0.borrow();
        let config = state
            .endpoints
            .get(&endpoint)
            .ok_or_else(|| failure(endpoint, op, ErrorClass::Configuration, false))?;
        if let Some(class) = config.discovery_failure {
            return Err(failure(endpoint, op, class, false));
        }
        Ok(config.capabilities)
    }

    fn put(
        &mut self,
        request: &PutRequest<ObjectId>,
        content: &mut dyn Read,
    ) -> Result<ProviderMetadata, StorageError> {
        let object = &request.object;
        let op = operation(object.id, Access::Put);
        let effect = self.begin(object, op)?;
        if request.expected_hash != *object.id.digest()
            || (matches!(object.id, ObjectId::Resource(_))
                && request.byte_length > omvcs_model::resource::MAX_RESOURCE_BYTE_LENGTH)
        {
            return Err(failure(
                object.endpoint,
                op,
                ErrorClass::InvalidRequest,
                false,
            ));
        }
        let interruption = match effect {
            Effect::FailAfter {
                bytes,
                class,
                recoverable,
            } => Some((bytes, failure(object.endpoint, op, class, recoverable))),
            _ => None,
        };
        let mut input = InterruptedReader {
            inner: content,
            position: 0,
            interruption,
        };
        let mut bytes = Vec::new();
        input.read_to_end(&mut bytes).map_err(|err| {
            err.get_ref()
                .and_then(|source| source.downcast_ref::<StorageError>())
                .cloned()
                .unwrap_or_else(|| failure(object.endpoint, op, ErrorClass::Precondition, false))
        })?;
        if bytes.len() as u64 != request.byte_length
            || Sha256::digest(&bytes).as_slice() != object.id.digest()
        {
            return Err(failure(object.endpoint, op, ErrorClass::Integrity, false));
        }
        let mut state = self.0.borrow_mut();
        let key = (object.endpoint, object.logical_key.0.clone());
        if let Some(existing) = state.objects.get(&key) {
            if *existing != bytes {
                return Err(failure(object.endpoint, op, ErrorClass::Integrity, false));
            }
        } else {
            state.objects.insert(key, bytes);
        }
        Ok(metadata(request.byte_length))
    }

    fn get(
        &mut self,
        request: &ObjectRequest<ObjectId>,
    ) -> Result<ProviderDownload<'_>, StorageError> {
        let op = operation(request.id, Access::Get);
        let effect = self.begin(request, op)?;
        if effect == Effect::Absent {
            return Err(failure(request.endpoint, op, ErrorClass::NotFound, false));
        }
        let mut bytes = self
            .stored_bytes(request.endpoint, &request.logical_key.0)
            .ok_or_else(|| failure(request.endpoint, op, ErrorClass::NotFound, false))?;
        let metadata = metadata(bytes.len() as u64);
        let interruption = match effect {
            Effect::ReturnBytes(replacement) => {
                bytes = replacement;
                None
            }
            Effect::Truncate(length) => {
                bytes.truncate(length);
                None
            }
            Effect::Corrupt { offset, xor } => {
                let byte = bytes.get_mut(offset).ok_or_else(|| {
                    failure(request.endpoint, op, ErrorClass::InvalidRequest, false)
                })?;
                *byte ^= xor;
                None
            }
            Effect::FailAfter {
                bytes,
                class,
                recoverable,
            } => Some((bytes, failure(request.endpoint, op, class, recoverable))),
            _ => None,
        };
        Ok(ProviderDownload {
            stream: Box::new(InterruptedReader {
                inner: Cursor::new(bytes),
                position: 0,
                interruption,
            }),
            metadata,
        })
    }

    fn stat(&mut self, request: &ObjectRequest<ObjectId>) -> Result<StatOutcome, StorageError> {
        let op = operation(request.id, Access::Stat);
        if self.begin(request, op)? == Effect::Absent {
            return Ok(StatOutcome::Absent);
        }
        Ok(self
            .stored_bytes(request.endpoint, &request.logical_key.0)
            .map_or(StatOutcome::Absent, |bytes| {
                StatOutcome::Present(metadata(bytes.len() as u64))
            }))
    }
}

#[derive(Clone, Copy)]
enum Access {
    Put,
    Get,
    Stat,
}

const fn operation(id: ObjectId, access: Access) -> Operation {
    match (id, access) {
        (ObjectId::Resource(_), Access::Put) => Operation::PutResource,
        (ObjectId::Resource(_), Access::Get) => Operation::GetResource,
        (ObjectId::Resource(_), Access::Stat) => Operation::StatResource,
        (ObjectId::Chunk(_), Access::Put) => Operation::PutChunk,
        (ObjectId::Chunk(_), Access::Get) => Operation::GetChunk,
        (ObjectId::Chunk(_), Access::Stat) => Operation::StatChunk,
    }
}

fn metadata(length: u64) -> ProviderMetadata {
    ProviderMetadata {
        byte_length: Some(length),
        ..ProviderMetadata::default()
    }
}

fn failure(
    endpoint: StorageEndpointId,
    operation: Operation,
    class: ErrorClass,
    recoverable: bool,
) -> StorageError {
    StorageError {
        code: format!("OMVCS_MOCK_{class:?}"),
        class,
        operation,
        endpoint,
        recoverable,
        provider_context: None,
        summary: format!("Explicit mock {class:?} outcome"),
    }
}

struct InterruptedReader<R> {
    inner: R,
    position: u64,
    interruption: Option<(u64, StorageError)>,
}

impl<R: Read> Read for InterruptedReader<R> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }
        let limit = if let Some((after, failure)) = &self.interruption {
            if self.position >= *after {
                return Err(io::Error::other(failure.clone()));
            }
            usize::try_from(after - self.position)
                .unwrap_or(usize::MAX)
                .min(buffer.len())
        } else {
            buffer.len()
        };
        let count = self.inner.read(&mut buffer[..limit])?;
        self.position += count as u64;
        Ok(count)
    }
}
