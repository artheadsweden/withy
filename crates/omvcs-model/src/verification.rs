//! Operational Resource and Chunk verification.
//!
//! Verification records are deliberately not serializable and are not
//! accepted by historical-object constructors. They describe operational
//! evidence only; they never participate in content or history identity.

use std::io::{self, Read};

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::replica::{
    ChunkManifest, ChunkManifestEntry, ProviderLocator, ReplicaCandidate, ReplicaRepresentation,
};
use crate::resource::{MAX_RESOURCE_BYTE_LENGTH, ResourceByteLength};
use crate::{ChunkId, ResourceId};

/// Exact OMVCS 0.1 sequential Chunk target, in bytes.
pub const CHUNK_TARGET_SIZE: usize = 8_388_608;
const READ_BUFFER_SIZE: usize = 64 * 1024;

/// What immutable-content proposition a verification result establishes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationStrength {
    /// The exact bytes of one Chunk match its `ChunkId`.
    ChunkIdentity,
    /// The complete Resource byte sequence matches its `ResourceId`.
    ResourceIdentity,
}

/// Result of the check against the requested immutable identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationOutcome {
    /// The checked bytes or approved equivalent evidence matched.
    Verified,
    /// Checked bytes or an applicable exact-scope checksum did not match.
    Failed,
    /// Required bytes/evidence were missing, unavailable, or inapplicable.
    Indeterminate,
}

/// How the claimed verification proposition was established.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationMethod {
    /// SHA-256 was calculated from a direct byte stream.
    DirectByteReadHash,
    /// A complete, ordered Chunk representation was bound to a verified
    /// Resource identity and each destination Chunk was independently checked.
    DeterministicChunkReconstruction,
    /// An Adapter supplied a checksum with exact OMVCS SHA-256 semantics.
    ProviderEquivalentChecksum,
}

/// The immutable subject addressed by a verification result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationSubject {
    /// A complete Resource Object.
    Resource(ResourceId),
    /// One exact Chunk.
    Chunk(ChunkId),
}

/// Algorithm and byte-scope claims reported by an Adapter for a checksum.
///
/// An Adapter may claim `OmvcsSha256` only when its documented and enforced
/// semantics are exactly SHA-256 over the complete byte scope named by
/// `ProviderChecksumScope`. A checksum value by itself is not such a claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderChecksumAlgorithm {
    /// Exact SHA-256 as required for OMVCS Resource and Chunk identities.
    OmvcsSha256,
    /// Any algorithm or encoding not proven equivalent to OMVCS SHA-256.
    Other,
}

/// Exact object scope covered by an Adapter-reported checksum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderChecksumScope {
    /// The complete byte sequence of one Chunk.
    ExactChunkBytes,
    /// The complete reconstructed Resource byte sequence.
    CompleteResourceBytes,
    /// Any narrower, broader, composite, or otherwise unproven byte scope.
    Other,
}

/// Operational support data for a verification result.
///
/// This intentionally contains no provider locator, credentials, URL,
/// timestamp, or provider-specific payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationEvidence {
    subject: VerificationSubject,
    outcome: VerificationOutcome,
    strength: VerificationStrength,
    method: VerificationMethod,
    target_binding: Option<[u8; 32]>,
    chunk_entry: Option<ChunkManifestEntry>,
    manifest_binding: Option<[u8; 32]>,
}

impl VerificationEvidence {
    /// Returns the immutable identity checked by this evidence.
    #[must_use]
    pub const fn subject(&self) -> VerificationSubject {
        self.subject
    }

    /// Returns the observed verification outcome.
    #[must_use]
    pub const fn outcome(&self) -> VerificationOutcome {
        self.outcome
    }

    /// Returns the requested identity proposition.
    #[must_use]
    pub const fn strength(&self) -> VerificationStrength {
        self.strength
    }

    /// Returns the method used or requested.
    #[must_use]
    pub const fn method(&self) -> VerificationMethod {
        self.method
    }

    /// Returns the canonical binding to a complete ordered manifest, when
    /// this evidence covers one.
    #[must_use]
    pub const fn manifest_binding(&self) -> Option<[u8; 32]> {
        self.manifest_binding
    }
}

/// Typed result separating outcome, strength, and method/evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationResult {
    evidence: VerificationEvidence,
}

impl VerificationResult {
    /// Returns the exact Resource or Chunk subject.
    #[must_use]
    pub const fn subject(&self) -> VerificationSubject {
        self.evidence.subject
    }

    /// Returns whether verification succeeded, failed, or was indeterminate.
    #[must_use]
    pub const fn outcome(&self) -> VerificationOutcome {
        self.evidence.outcome
    }

    /// Returns the proposition that was requested.
    #[must_use]
    pub const fn strength(&self) -> VerificationStrength {
        self.evidence.strength
    }

    /// Returns the method or evidence class used.
    #[must_use]
    pub const fn method(&self) -> VerificationMethod {
        self.evidence.method
    }

    /// Returns the established strength only for a successful result.
    #[must_use]
    pub const fn established_strength(&self) -> Option<VerificationStrength> {
        if matches!(self.evidence.outcome, VerificationOutcome::Verified) {
            Some(self.evidence.strength)
        } else {
            None
        }
    }

    /// Returns the operational evidence without exposing mutable state.
    #[must_use]
    pub const fn evidence(&self) -> &VerificationEvidence {
        &self.evidence
    }

    /// Makes verified, candidate-specific Resource assurance consumable by
    /// Replica promotion. General or source-only verification is insufficient.
    #[must_use]
    pub fn promotion_eligibility(
        &self,
        candidate: &ReplicaCandidate,
    ) -> Option<PromotionEligibility> {
        let binding = candidate_binding(candidate)?;
        (self.evidence.outcome == VerificationOutcome::Verified
            && self.evidence.strength == VerificationStrength::ResourceIdentity
            && self.evidence.subject == VerificationSubject::Resource(candidate.resource_id())
            && self.evidence.target_binding == Some(binding))
        .then_some(PromotionEligibility {
            resource_id: candidate.resource_id(),
            target_binding: binding,
        })
    }
}

/// Opaque proof that destination-applicable `resource_identity` assurance
/// succeeded for one exact candidate representation.
///
/// Only a verified, candidate-bound `VerificationResult` can produce this
/// value. It is operational and cannot be serialized into historical state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromotionEligibility {
    resource_id: ResourceId,
    target_binding: [u8; 32],
}

impl PromotionEligibility {
    /// Returns whether this assurance applies to the complete candidate as
    /// currently described, including Endpoint, representation, and locator.
    #[must_use]
    pub fn applies_to(&self, candidate: &ReplicaCandidate) -> bool {
        self.resource_id == candidate.resource_id()
            && candidate_binding(candidate) == Some(self.target_binding)
    }

    /// Returns the Resource identity for which promotion was established.
    #[must_use]
    pub const fn resource_id(&self) -> ResourceId {
        self.resource_id
    }
}

/// A complete ordered manifest derived while hashing and verifying a
/// Resource's complete bytes. Construction is restricted to successful
/// complete Resource verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedResourceManifest {
    resource_id: ResourceId,
    manifest: ChunkManifest,
}

impl VerifiedResourceManifest {
    /// Returns the Resource identity established by the complete byte check.
    #[must_use]
    pub const fn resource_id(&self) -> ResourceId {
        self.resource_id
    }

    /// Returns the deterministic manifest derived from those exact bytes.
    #[must_use]
    pub const fn manifest(&self) -> &ChunkManifest {
        &self.manifest
    }
}

/// Complete Resource verification and optional verified-manifest proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceVerification {
    result: VerificationResult,
    verified_manifest: Option<VerifiedResourceManifest>,
}

impl ResourceVerification {
    /// Returns the verification result.
    #[must_use]
    pub const fn result(&self) -> &VerificationResult {
        &self.result
    }

    /// Returns a deterministic manifest proof only when the complete Resource
    /// bytes were read successfully and matched the expected `ResourceId`.
    #[must_use]
    pub const fn verified_manifest(&self) -> Option<&VerifiedResourceManifest> {
        self.verified_manifest.as_ref()
    }
}

/// Invalid Resource length observed while verifying a stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationInputError {
    /// Resource bytes exceed the exact OMVCS 0.1 Resource length domain.
    ResourceLengthOutOfRange,
}

impl std::fmt::Display for VerificationInputError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("verified Resource exceeds the OMVCS 0.1 byte-length domain")
    }
}

impl std::error::Error for VerificationInputError {}

/// Divides complete Resource bytes into the OMVCS 0.1 ordered Chunk manifest.
///
/// No byte data is copied or retained. The caller remains responsible for
/// retaining or transferring the original bytes.
///
/// # Errors
///
/// Returns [`VerificationInputError::ResourceLengthOutOfRange`] if the
/// Resource exceeds the exact OMVCS 0.1 byte-length domain.
pub fn chunk_resource(bytes: &[u8]) -> Result<ChunkManifest, VerificationInputError> {
    if bytes.len() as u128 > u128::from(MAX_RESOURCE_BYTE_LENGTH) {
        return Err(VerificationInputError::ResourceLengthOutOfRange);
    }
    let resource_id = crate::hashing::hash_resource_bytes(bytes);
    let mut entries = Vec::new();
    if bytes.is_empty() {
        entries.push(ChunkManifestEntry::new(
            ChunkId::from_digest(sha256(&[])),
            ResourceByteLength::new(0)
                .map_err(|_| VerificationInputError::ResourceLengthOutOfRange)?,
            ResourceByteLength::new(0)
                .map_err(|_| VerificationInputError::ResourceLengthOutOfRange)?,
        ));
    } else {
        for (index, chunk) in bytes.chunks(CHUNK_TARGET_SIZE).enumerate() {
            let offset = (index * CHUNK_TARGET_SIZE) as u64;
            entries.push(ChunkManifestEntry::new(
                ChunkId::from_digest(sha256(chunk)),
                ResourceByteLength::new(offset)
                    .map_err(|_| VerificationInputError::ResourceLengthOutOfRange)?,
                ResourceByteLength::new(chunk.len() as u64)
                    .map_err(|_| VerificationInputError::ResourceLengthOutOfRange)?,
            ));
        }
    }
    ChunkManifest::new(
        resource_id,
        ResourceByteLength::new(bytes.len() as u64)
            .map_err(|_| VerificationInputError::ResourceLengthOutOfRange)?,
        entries,
    )
    .map_err(|_| VerificationInputError::ResourceLengthOutOfRange)
}

/// Verifies a complete Resource from a byte stream using incremental SHA-256.
///
/// A read failure is `indeterminate`, not corruption. The reader is consumed
/// only as far as the failure/EOF and is never buffered in full.
///
/// # Errors
///
/// Returns [`VerificationInputError::ResourceLengthOutOfRange`] if observed
/// bytes exceed the OMVCS 0.1 Resource length domain.
pub fn verify_resource_reader<R: Read>(
    expected: ResourceId,
    reader: &mut R,
) -> Result<VerificationResult, VerificationInputError> {
    let (observed, outcome) = digest_reader(reader, None)?;
    Ok(make_result(
        VerificationSubject::Resource(expected),
        VerificationStrength::ResourceIdentity,
        VerificationMethod::DirectByteReadHash,
        outcome_for_digest(observed, outcome, expected.digest()),
        None,
        None,
        None,
    ))
}

/// Verifies a Resource stream read from one exact candidate representation.
///
/// The result can authorize promotion only while the candidate's Endpoint,
/// representation, and locator binding remain unchanged.
///
/// # Errors
///
/// Returns [`VerificationInputError::ResourceLengthOutOfRange`] if observed
/// bytes exceed the OMVCS 0.1 Resource length domain.
pub fn verify_candidate_resource_reader<R: Read>(
    candidate: &ReplicaCandidate,
    reader: &mut R,
) -> Result<VerificationResult, VerificationInputError> {
    let (observed, outcome) = digest_reader(reader, None)?;
    Ok(make_result(
        VerificationSubject::Resource(candidate.resource_id()),
        VerificationStrength::ResourceIdentity,
        VerificationMethod::DirectByteReadHash,
        outcome_for_digest(observed, outcome, candidate.resource_id().digest()),
        candidate_binding(candidate),
        None,
        None,
    ))
}

/// Verifies a Chunk from a byte stream against one manifest entry at a
/// destination candidate.
///
/// If `entry` is absent from the candidate's chunked manifest, the result is
/// indeterminate; no bytes were established to be corrupt.
pub fn verify_candidate_chunk_reader<R: Read>(
    candidate: &ReplicaCandidate,
    entry: ChunkManifestEntry,
    reader: &mut R,
) -> VerificationResult {
    let Some(manifest) = chunked_manifest(candidate) else {
        return make_result(
            VerificationSubject::Chunk(entry.chunk_id()),
            VerificationStrength::ChunkIdentity,
            VerificationMethod::DirectByteReadHash,
            VerificationOutcome::Indeterminate,
            candidate_binding(candidate),
            Some(entry),
            None,
        );
    };
    if !manifest.chunks().contains(&entry) {
        return make_result(
            VerificationSubject::Chunk(entry.chunk_id()),
            VerificationStrength::ChunkIdentity,
            VerificationMethod::DirectByteReadHash,
            VerificationOutcome::Indeterminate,
            candidate_binding(candidate),
            Some(entry),
            None,
        );
    }
    let (observed, outcome) = digest_reader(reader, Some(entry.length().get()))
        .unwrap_or((None, VerificationOutcome::Indeterminate));
    make_result(
        VerificationSubject::Chunk(entry.chunk_id()),
        VerificationStrength::ChunkIdentity,
        VerificationMethod::DirectByteReadHash,
        outcome_for_digest(observed, outcome, entry.chunk_id().digest()),
        candidate_binding(candidate),
        Some(entry),
        None,
    )
}

/// Verifies a complete Resource and derives its deterministic Chunk manifest
/// in a bounded-memory stream pass.
///
/// At most one 8,388,608-byte Chunk plus the fixed read buffer is retained.
/// Missing/unavailable stream bytes yield an indeterminate result and no
/// verified-manifest proof.
///
/// # Errors
///
/// Returns [`VerificationInputError::ResourceLengthOutOfRange`] if observed
/// bytes exceed the OMVCS 0.1 Resource length domain.
pub fn verify_resource_reader_with_manifest<R: Read>(
    expected: ResourceId,
    reader: &mut R,
) -> Result<ResourceVerification, VerificationInputError> {
    let mut resource_hash = Sha256::new();
    let mut read_buffer = vec![0_u8; READ_BUFFER_SIZE].into_boxed_slice();
    let mut pending = Vec::with_capacity(CHUNK_TARGET_SIZE);
    let mut entries = Vec::new();
    let mut total_length = 0_u64;
    let mut next_offset = 0_u64;
    let mut read_failed = false;

    loop {
        let count = loop {
            match reader.read(&mut read_buffer) {
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(_) => {
                    read_failed = true;
                    break 0;
                }
                Ok(count) => break count,
            }
        };
        if count == 0 {
            break;
        }
        total_length = total_length
            .checked_add(count as u64)
            .ok_or(VerificationInputError::ResourceLengthOutOfRange)?;
        if total_length > MAX_RESOURCE_BYTE_LENGTH {
            return Err(VerificationInputError::ResourceLengthOutOfRange);
        }
        resource_hash.update(&read_buffer[..count]);

        let mut remaining = &read_buffer[..count];
        while !remaining.is_empty() {
            let available = CHUNK_TARGET_SIZE - pending.len();
            let take = available.min(remaining.len());
            pending.extend_from_slice(&remaining[..take]);
            remaining = &remaining[take..];
            if pending.len() == CHUNK_TARGET_SIZE {
                entries.push(manifest_entry(next_offset, &pending)?);
                next_offset += CHUNK_TARGET_SIZE as u64;
                pending.clear();
            }
        }
    }

    let observed = if read_failed {
        None
    } else {
        Some(finalize_digest(resource_hash))
    };
    let outcome = if read_failed {
        VerificationOutcome::Indeterminate
    } else {
        outcome_for_digest(observed, VerificationOutcome::Verified, expected.digest())
    };
    let mut proof = None;
    let manifest_binding = if outcome == VerificationOutcome::Verified {
        if !pending.is_empty() || entries.is_empty() {
            entries.push(manifest_entry(next_offset, &pending)?);
        }
        let manifest = ChunkManifest::new(
            expected,
            ResourceByteLength::new(total_length)
                .map_err(|_| VerificationInputError::ResourceLengthOutOfRange)?,
            entries,
        )
        .map_err(|_| VerificationInputError::ResourceLengthOutOfRange)?;
        let binding = manifest_binding(&manifest);
        proof = Some(VerifiedResourceManifest {
            resource_id: expected,
            manifest,
        });
        Some(binding)
    } else {
        None
    };
    let result = make_result(
        VerificationSubject::Resource(expected),
        VerificationStrength::ResourceIdentity,
        VerificationMethod::DirectByteReadHash,
        outcome,
        None,
        None,
        manifest_binding,
    );
    Ok(ResourceVerification {
        result,
        verified_manifest: proof,
    })
}

/// Checks destination Chunk evidence against a previously verified complete
/// Resource manifest and produces destination-applicable Resource assurance.
///
/// The evidence slice must contain exactly one result per manifest entry, in
/// reconstruction order. Missing, stale, unavailable, or wrong-subject
/// evidence is indeterminate. Checked mismatches or a changed ordering fail.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn verify_chunked_destination(
    candidate: &ReplicaCandidate,
    verified_source: Option<&VerifiedResourceManifest>,
    chunk_results: &[VerificationResult],
) -> VerificationResult {
    let subject = VerificationSubject::Resource(candidate.resource_id());
    let binding = candidate_binding(candidate);
    let Some(destination_manifest) = chunked_manifest(candidate) else {
        return make_result(
            subject,
            VerificationStrength::ResourceIdentity,
            VerificationMethod::DeterministicChunkReconstruction,
            VerificationOutcome::Indeterminate,
            binding,
            None,
            None,
        );
    };
    let Some(verified_source) = verified_source else {
        return make_result(
            subject,
            VerificationStrength::ResourceIdentity,
            VerificationMethod::DeterministicChunkReconstruction,
            VerificationOutcome::Indeterminate,
            binding,
            None,
            None,
        );
    };
    if chunk_results.len() != destination_manifest.chunks().len() {
        return make_result(
            subject,
            VerificationStrength::ResourceIdentity,
            VerificationMethod::DeterministicChunkReconstruction,
            VerificationOutcome::Indeterminate,
            binding,
            None,
            Some(manifest_binding(destination_manifest)),
        );
    }
    let expected_entries = destination_manifest.chunks();
    let reported_entries = chunk_results
        .iter()
        .map(|result| result.evidence.chunk_entry)
        .collect::<Option<Vec<_>>>();
    if let Some(reported_entries) = reported_entries
        && reported_entries != expected_entries
        && chunk_results.iter().all(|result| {
            let Some(entry) = result.evidence.chunk_entry else {
                return false;
            };
            result.evidence.outcome == VerificationOutcome::Verified
                && result.evidence.strength == VerificationStrength::ChunkIdentity
                && result.evidence.target_binding == binding
                && result.evidence.subject == VerificationSubject::Chunk(entry.chunk_id())
        })
        && expected_entries.iter().all(|entry| {
            reported_entries
                .iter()
                .filter(|reported| **reported == *entry)
                .count()
                == expected_entries
                    .iter()
                    .filter(|expected| **expected == *entry)
                    .count()
        })
    {
        return make_result(
            subject,
            VerificationStrength::ResourceIdentity,
            VerificationMethod::DeterministicChunkReconstruction,
            VerificationOutcome::Failed,
            binding,
            None,
            Some(manifest_binding(destination_manifest)),
        );
    }
    for (entry, result) in destination_manifest.chunks().iter().zip(chunk_results) {
        let evidence = &result.evidence;
        if evidence.outcome == VerificationOutcome::Failed
            && evidence.subject == VerificationSubject::Chunk(entry.chunk_id())
            && evidence.chunk_entry == Some(*entry)
            && evidence.target_binding == binding
        {
            return make_result(
                subject,
                VerificationStrength::ResourceIdentity,
                VerificationMethod::DeterministicChunkReconstruction,
                VerificationOutcome::Failed,
                binding,
                None,
                Some(manifest_binding(destination_manifest)),
            );
        }
        if evidence.outcome != VerificationOutcome::Verified
            || evidence.strength != VerificationStrength::ChunkIdentity
            || evidence.subject != VerificationSubject::Chunk(entry.chunk_id())
            || evidence.chunk_entry != Some(*entry)
            || evidence.target_binding != binding
        {
            return make_result(
                subject,
                VerificationStrength::ResourceIdentity,
                VerificationMethod::DeterministicChunkReconstruction,
                VerificationOutcome::Indeterminate,
                binding,
                None,
                Some(manifest_binding(destination_manifest)),
            );
        }
    }
    let source_manifest_matches = verified_source.resource_id == candidate.resource_id()
        && verified_source.manifest == *destination_manifest;
    if !source_manifest_matches {
        return make_result(
            subject,
            VerificationStrength::ResourceIdentity,
            VerificationMethod::DeterministicChunkReconstruction,
            VerificationOutcome::Failed,
            binding,
            None,
            Some(manifest_binding(destination_manifest)),
        );
    }
    make_result(
        subject,
        VerificationStrength::ResourceIdentity,
        VerificationMethod::DeterministicChunkReconstruction,
        VerificationOutcome::Verified,
        binding,
        None,
        Some(manifest_binding(destination_manifest)),
    )
}

/// Evaluates an Adapter's checksum report for exact candidate-specific
/// Resource verification.
///
/// `OmvcsSha256` plus `CompleteResourceBytes` is an Adapter assertion that its
/// documented and enforced checksum covers the exact complete Resource
/// sequence.
#[must_use]
pub fn verify_candidate_resource_checksum(
    candidate: &ReplicaCandidate,
    algorithm: ProviderChecksumAlgorithm,
    scope: ProviderChecksumScope,
    digest: [u8; 32],
) -> VerificationResult {
    let outcome = if algorithm != ProviderChecksumAlgorithm::OmvcsSha256
        || scope != ProviderChecksumScope::CompleteResourceBytes
    {
        VerificationOutcome::Indeterminate
    } else if digest == *candidate.resource_id().digest() {
        VerificationOutcome::Verified
    } else {
        VerificationOutcome::Failed
    };
    make_result(
        VerificationSubject::Resource(candidate.resource_id()),
        VerificationStrength::ResourceIdentity,
        VerificationMethod::ProviderEquivalentChecksum,
        outcome,
        candidate_binding(candidate),
        None,
        None,
    )
}

/// Evaluates an Adapter's checksum report for one exact destination Chunk.
#[must_use]
pub fn verify_candidate_chunk_checksum(
    candidate: &ReplicaCandidate,
    entry: ChunkManifestEntry,
    algorithm: ProviderChecksumAlgorithm,
    scope: ProviderChecksumScope,
    digest: [u8; 32],
) -> VerificationResult {
    let Some(manifest) = chunked_manifest(candidate) else {
        return make_result(
            VerificationSubject::Chunk(entry.chunk_id()),
            VerificationStrength::ChunkIdentity,
            VerificationMethod::ProviderEquivalentChecksum,
            VerificationOutcome::Indeterminate,
            candidate_binding(candidate),
            Some(entry),
            None,
        );
    };
    let outcome = if !manifest.chunks().contains(&entry)
        || algorithm != ProviderChecksumAlgorithm::OmvcsSha256
        || scope != ProviderChecksumScope::ExactChunkBytes
    {
        VerificationOutcome::Indeterminate
    } else if digest == *entry.chunk_id().digest() {
        VerificationOutcome::Verified
    } else {
        VerificationOutcome::Failed
    };
    make_result(
        VerificationSubject::Chunk(entry.chunk_id()),
        VerificationStrength::ChunkIdentity,
        VerificationMethod::ProviderEquivalentChecksum,
        outcome,
        candidate_binding(candidate),
        Some(entry),
        None,
    )
}

fn manifest_entry(offset: u64, bytes: &[u8]) -> Result<ChunkManifestEntry, VerificationInputError> {
    Ok(ChunkManifestEntry::new(
        ChunkId::from_digest(sha256(bytes)),
        ResourceByteLength::new(offset)
            .map_err(|_| VerificationInputError::ResourceLengthOutOfRange)?,
        ResourceByteLength::new(bytes.len() as u64)
            .map_err(|_| VerificationInputError::ResourceLengthOutOfRange)?,
    ))
}

fn digest_reader<R: Read>(
    reader: &mut R,
    expected_length: Option<u64>,
) -> Result<(Option<[u8; 32]>, VerificationOutcome), VerificationInputError> {
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; READ_BUFFER_SIZE].into_boxed_slice();
    let mut length = 0_u64;
    loop {
        let count = match reader.read(&mut buffer) {
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(_) => return Ok((None, VerificationOutcome::Indeterminate)),
            Ok(count) => count,
        };
        if count == 0 {
            let digest = finalize_digest(hasher);
            let outcome = if expected_length.is_some_and(|expected| expected != length) {
                VerificationOutcome::Failed
            } else {
                VerificationOutcome::Verified
            };
            return Ok((Some(digest), outcome));
        }
        length = length
            .checked_add(count as u64)
            .ok_or(VerificationInputError::ResourceLengthOutOfRange)?;
        if length > MAX_RESOURCE_BYTE_LENGTH {
            return Err(VerificationInputError::ResourceLengthOutOfRange);
        }
        hasher.update(&buffer[..count]);
    }
}

fn outcome_for_digest(
    observed: Option<[u8; 32]>,
    read_outcome: VerificationOutcome,
    expected: &[u8; 32],
) -> VerificationOutcome {
    match (observed, read_outcome) {
        (None, _) => VerificationOutcome::Indeterminate,
        (Some(_), VerificationOutcome::Failed) => VerificationOutcome::Failed,
        (Some(digest), VerificationOutcome::Verified) if digest == *expected => {
            VerificationOutcome::Verified
        }
        (Some(_), VerificationOutcome::Verified | VerificationOutcome::Indeterminate) => {
            VerificationOutcome::Failed
        }
    }
}

const fn make_result(
    subject: VerificationSubject,
    strength: VerificationStrength,
    method: VerificationMethod,
    outcome: VerificationOutcome,
    target_binding: Option<[u8; 32]>,
    chunk_entry: Option<ChunkManifestEntry>,
    manifest_binding: Option<[u8; 32]>,
) -> VerificationResult {
    VerificationResult {
        evidence: VerificationEvidence {
            subject,
            outcome,
            strength,
            method,
            target_binding,
            chunk_entry,
            manifest_binding,
        },
    }
}

fn candidate_binding(candidate: &ReplicaCandidate) -> Option<[u8; 32]> {
    #[derive(Serialize)]
    struct Binding<'a> {
        resource_id: String,
        endpoint_id: String,
        representation: &'a ReplicaRepresentation,
        locator: &'a ProviderLocator,
    }

    serde_jcs::to_vec(&Binding {
        resource_id: candidate.resource_id().to_string(),
        endpoint_id: candidate.endpoint_id().to_string(),
        representation: candidate.representation(),
        locator: candidate.locator(),
    })
    .ok()
    .map(|bytes| sha256(&bytes))
}

fn manifest_binding(manifest: &ChunkManifest) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"OMVCS verified manifest binding 0.1\0");
    hasher.update(manifest.resource_id().digest());
    hasher.update(manifest.total_length().get().to_be_bytes());
    hasher.update((manifest.chunks().len() as u64).to_be_bytes());
    for entry in manifest.chunks() {
        hasher.update(entry.chunk_id().digest());
        hasher.update(entry.offset().get().to_be_bytes());
        hasher.update(entry.length().get().to_be_bytes());
    }
    finalize_digest(hasher)
}

const fn chunked_manifest(candidate: &ReplicaCandidate) -> Option<&ChunkManifest> {
    match candidate.representation() {
        ReplicaRepresentation::Chunked { manifest } => Some(manifest),
        ReplicaRepresentation::CompleteObject => None,
    }
}

fn sha256(bytes: &[u8]) -> [u8; 32] {
    let digest = Sha256::digest(bytes);
    let mut output = [0_u8; 32];
    output.copy_from_slice(&digest);
    output
}

fn finalize_digest(hasher: Sha256) -> [u8; 32] {
    let digest = hasher.finalize();
    let mut output = [0_u8; 32];
    output.copy_from_slice(&digest);
    output
}
