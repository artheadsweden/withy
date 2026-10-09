//! Content-derived OMVCS identifiers.
//!
//! Resource identifiers hash the complete raw Resource byte sequence.
//! Metadata identifiers hash the canonical object-body bytes supplied by
//! [`crate::canonical::canonicalize_metadata_body`]. Object type is represented
//! by each returned typed identifier and is not added to the SHA-256 input.

use sha2::{Digest, Sha256};

use crate::{AdapterStateId, ComponentStateId, ProjectStateId, ReleaseId, ResourceId, RevisionId};

fn sha256(bytes: &[u8]) -> [u8; 32] {
    let digest = Sha256::digest(bytes);
    let mut output = [0_u8; 32];
    output.copy_from_slice(&digest);
    output
}

/// Calculates a Resource Identifier from the complete raw Resource bytes.
///
/// The bytes are hashed directly; they are not JSON-canonicalized and no
/// filename, location, storage, or replica information is included.
#[must_use]
pub fn hash_resource_bytes(resource_bytes: &[u8]) -> ResourceId {
    ResourceId::from_digest(sha256(resource_bytes))
}

/// Calculates a Component State Identifier from canonical metadata body bytes.
///
/// `canonical_metadata` must be the output of the model's canonical metadata
/// API for the historical object body. The bytes are hashed as-is.
#[must_use]
pub fn hash_component_state_metadata(canonical_metadata: &[u8]) -> ComponentStateId {
    ComponentStateId::from_digest(sha256(canonical_metadata))
}

/// Calculates an Adapter State Identifier from canonical metadata body bytes.
///
/// `canonical_metadata` must be the output of the model's canonical metadata
/// API for the historical object body. The bytes are hashed as-is.
#[must_use]
pub fn hash_adapter_state_metadata(canonical_metadata: &[u8]) -> AdapterStateId {
    AdapterStateId::from_digest(sha256(canonical_metadata))
}

/// Calculates a Project State Identifier from canonical metadata body bytes.
///
/// `canonical_metadata` must be the output of the model's canonical metadata
/// API for the historical object body. The bytes are hashed as-is.
#[must_use]
pub fn hash_project_state_metadata(canonical_metadata: &[u8]) -> ProjectStateId {
    ProjectStateId::from_digest(sha256(canonical_metadata))
}

/// Calculates a Revision Identifier from canonical metadata body bytes.
///
/// `canonical_metadata` must be the output of the model's canonical metadata
/// API for the historical object body. The bytes are hashed as-is.
#[must_use]
pub fn hash_revision_metadata(canonical_metadata: &[u8]) -> RevisionId {
    RevisionId::from_digest(sha256(canonical_metadata))
}

/// Calculates a Release Identifier from its canonical metadata body bytes.
///
/// `canonical_metadata` must be the RFC 8785 serialization of the complete
/// admitted Release body. The bytes are hashed as-is, without an object-type
/// prefix.
#[must_use]
pub fn hash_release_metadata(canonical_metadata: &[u8]) -> ReleaseId {
    ReleaseId::from_digest(sha256(canonical_metadata))
}
