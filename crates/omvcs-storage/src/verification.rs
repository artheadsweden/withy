//! Generic Adapter checksum evidence boundary.
//!
//! An Adapter may report OMVCS-equivalent checksum semantics only when they
//! are documented and enforced for the exact object bytes. Other provider
//! checksums remain operational hints and yield indeterminate verification.

use omvcs_model::replica::{ChunkManifestEntry, ReplicaCandidate};
use omvcs_model::verification::{
    ProviderChecksumAlgorithm, ProviderChecksumScope, VerificationResult,
    verify_candidate_chunk_checksum, verify_candidate_resource_checksum,
};

/// Provider-reported checksum facts for an exact immutable byte scope.
///
/// `OmvcsSha256` is an Adapter attestation that the provider's documented and
/// enforced algorithm is exact SHA-256. The scope must be either the complete
/// bytes of one Chunk or the complete reconstructed Resource. `ETags`,
/// multipart checksums, CRCs, versions, lengths, and generic success do not
/// satisfy this contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderChecksumReport {
    algorithm: ProviderChecksumAlgorithm,
    scope: ProviderChecksumScope,
    digest: [u8; 32],
}

impl ProviderChecksumReport {
    /// Creates a report from Adapter-established algorithm and byte-scope
    /// semantics. Use `Other` unless exact OMVCS equivalence is proven.
    #[must_use]
    pub const fn new(
        algorithm: ProviderChecksumAlgorithm,
        scope: ProviderChecksumScope,
        digest: [u8; 32],
    ) -> Self {
        Self {
            algorithm,
            scope,
            digest,
        }
    }

    /// Returns the reported checksum algorithm class.
    #[must_use]
    pub const fn algorithm(self) -> ProviderChecksumAlgorithm {
        self.algorithm
    }

    /// Returns the reported byte scope.
    #[must_use]
    pub const fn scope(self) -> ProviderChecksumScope {
        self.scope
    }

    /// Returns the reported digest bytes.
    #[must_use]
    pub const fn digest(self) -> [u8; 32] {
        self.digest
    }
}

/// Evaluates a provider checksum for destination Resource assurance.
#[must_use]
pub fn verify_resource_checksum(
    candidate: &ReplicaCandidate,
    report: ProviderChecksumReport,
) -> VerificationResult {
    verify_candidate_resource_checksum(candidate, report.algorithm, report.scope, report.digest)
}

/// Evaluates a provider checksum for one destination Chunk.
#[must_use]
pub fn verify_chunk_checksum(
    candidate: &ReplicaCandidate,
    entry: ChunkManifestEntry,
    report: ProviderChecksumReport,
) -> VerificationResult {
    verify_candidate_chunk_checksum(
        candidate,
        entry,
        report.algorithm,
        report.scope,
        report.digest,
    )
}
