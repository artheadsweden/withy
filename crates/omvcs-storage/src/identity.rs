use std::{fmt, str::FromStr};

use omvcs_model::ContentIdentifierParseError;

/// Narrow storage primitive from Core §8.1 / Storage §22.
///
/// The digest is SHA-256 over caller-selected Chunk bytes, not over its
/// Endpoint, key or a Resource ID. This type selects no chunking policy and
/// defines no Chunk Manifest. Like model content IDs, construction accepts a
/// supplied digest; streaming I/O checks correspondence to actual bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChunkId([u8; 32]);

impl ChunkId {
    #[must_use]
    pub const fn from_digest(digest: [u8; 32]) -> Self {
        Self(digest)
    }

    #[must_use]
    pub const fn digest(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Display for ChunkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("omvcs:chunk:sha256:")?;
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl FromStr for ChunkId {
    type Err = ContentIdentifierParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        // Reuse model's canonical SHA-256 parsing without conflating namespaces.
        let digest = input
            .strip_prefix("omvcs:chunk:sha256:")
            .ok_or(ContentIdentifierParseError::InvalidFormat)?;
        let resource: omvcs_model::ResourceId =
            format!("omvcs:resource:sha256:{digest}").parse()?;
        Ok(Self(*resource.digest()))
    }
}
