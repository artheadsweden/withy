use std::io::{self, Read};

use sha2::{Digest, Sha256};

use crate::{
    ErrorClass, ObjectId, ObjectRequest, Operation, ProviderDownload, ProviderMetadata,
    StorageEndpointId, StorageError, TransferOutcome, error,
};

/// A bounded-memory provisional download (Storage §§23–25).
///
/// `Read` exposes bytes before the whole digest can be known. Do not accept a
/// prefix or even a full buffer on its own: only `complete()` after EOF yields
/// successful retrieval. Dropping the stream is abandonment, not completion.
/// Provider metadata is returned only as non-authoritative operational hints.
pub struct Download<'a, I> {
    stream: CheckedStream<Box<dyn Read + 'a>>,
    id: I,
    metadata: ProviderMetadata,
}

impl<'a, I> Download<'a, I> {
    pub(crate) fn new(
        download: ProviderDownload<'a>,
        request: &ObjectRequest<ObjectId>,
        id: I,
        operation: Operation,
    ) -> Self {
        Self {
            stream: CheckedStream::new(download.stream, request, operation, None),
            id,
            metadata: download.metadata,
        }
    }

    /// Completes only an EOF-observed exact-content transfer.
    /// This does not register or promote a Replica.
    ///
    /// # Errors
    /// Returns structured Integrity, stream failure, or incomplete Precondition.
    pub fn complete(self) -> Result<TransferOutcome<I>, StorageError> {
        let byte_length = self.stream.complete()?;
        Ok(TransferOutcome {
            id: self.id,
            byte_length,
            metadata: self.metadata,
        })
    }
}

impl<I> Read for Download<'_, I> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buf)
    }
}

pub struct CheckedStream<R> {
    inner: R,
    hash: Sha256,
    expected: [u8; 32],
    expected_length: Option<u64>,
    length: u64,
    eof: bool,
    failure: Option<StorageError>,
    endpoint: StorageEndpointId,
    operation: Operation,
}

impl<R> CheckedStream<R> {
    pub(crate) fn new(
        inner: R,
        request: &ObjectRequest<ObjectId>,
        operation: Operation,
        expected_length: Option<u64>,
    ) -> Self {
        Self {
            inner,
            hash: Sha256::new(),
            expected: *request.id.digest(),
            expected_length,
            length: 0,
            eof: false,
            failure: None,
            endpoint: request.endpoint,
            operation,
        }
    }

    pub(crate) fn complete(self) -> Result<u64, StorageError> {
        if let Some(failure) = self.failure {
            return Err(failure);
        }
        if !self.eof {
            return Err(error(
                self.endpoint,
                self.operation,
                ErrorClass::Precondition,
                "INCOMPLETE_STREAM",
                "Content stream has not reached EOF",
            ));
        }
        Ok(self.length)
    }

    fn fail(&mut self, failure: StorageError) -> io::Error {
        self.failure = Some(failure.clone());
        io::Error::other(failure)
    }
}

impl<R: Read> Read for CheckedStream<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if let Some(failure) = &self.failure {
            return Err(io::Error::other(failure.clone()));
        }
        if buf.is_empty() || self.eof {
            return Ok(0);
        }
        let count = match self.inner.read(buf) {
            Ok(count) => count,
            Err(failure) if failure.kind() == io::ErrorKind::Interrupted => return Err(failure),
            Err(failure) => {
                // Preserve structured provider stream failures where supplied.
                let contextual = failure
                    .get_ref()
                    .and_then(|source| source.downcast_ref::<StorageError>())
                    .cloned()
                    .unwrap_or_else(|| {
                        error(
                            self.endpoint,
                            self.operation,
                            if matches!(
                                self.operation,
                                Operation::PutResource | Operation::PutChunk
                            ) {
                                ErrorClass::Precondition
                            } else {
                                ErrorClass::ProviderUnavailable
                            },
                            "STREAM_FAILURE",
                            "Content stream failed",
                        )
                    });
                return Err(self.fail(contextual));
            }
        };
        if count == 0 {
            if self
                .expected_length
                .is_some_and(|expected| expected != self.length)
                || self.hash.clone().finalize().as_slice() != self.expected
            {
                return Err(self.fail(error(
                    self.endpoint,
                    self.operation,
                    ErrorClass::Integrity,
                    "CONTENT_MISMATCH",
                    "Stream bytes or length do not match the requested immutable content",
                )));
            }
            self.eof = true;
            return Ok(0);
        }
        let Some(length) = self.length.checked_add(count as u64) else {
            return Err(self.fail(error(
                self.endpoint,
                self.operation,
                ErrorClass::Integrity,
                "LENGTH_OVERFLOW",
                "Content length exceeds the byte-I/O domain",
            )));
        };
        self.length = length;
        self.hash.update(&buf[..count]);
        if self
            .expected_length
            .is_some_and(|expected| length > expected)
        {
            return Err(self.fail(error(
                self.endpoint,
                self.operation,
                ErrorClass::Integrity,
                "LENGTH_MISMATCH",
                "Input stream exceeds the supplied length",
            )));
        }
        Ok(count)
    }
}
