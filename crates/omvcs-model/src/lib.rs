//! OMVCS domain-model reference crate.
//!
//! Normative types are added only through approved work packages.

#![forbid(unsafe_code)]

pub mod canonical;
pub mod component_state;
pub mod creative_component;
pub mod hashing;
pub mod project_state;
pub mod release;
pub mod replica;
pub mod resource;
pub mod revision;
mod timestamp;

pub use component_state::ComponentState;
pub use creative_component::CreativeComponent;
pub use project_state::ProjectState;
pub use release::Release;
pub use revision::Revision;

use std::fmt;
use std::str::FromStr;

use uuid::{Uuid, Variant};

/// Errors returned when parsing an assigned `UUIDv7` identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignedIdentifierParseError {
    /// The input is not a UUID in canonical lowercase hyphenated form.
    InvalidUuid,
    /// The input is a UUID, but not in canonical lowercase hyphenated form.
    NonCanonical,
    /// Assigned identifiers in OMVCS 0.1 must use UUID version 7.
    NotUuidV7,
    /// Assigned identifiers must use the RFC UUID variant.
    NotRfcUuidVariant,
}

impl fmt::Display for AssignedIdentifierParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidUuid => "invalid UUID",
            Self::NonCanonical => "UUID is not canonical lowercase text",
            Self::NotUuidV7 => "assigned identifier is not UUID version 7",
            Self::NotRfcUuidVariant => "assigned identifier does not use the RFC UUID variant",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for AssignedIdentifierParseError {}

fn parse_assigned_uuid(input: &str) -> Result<Uuid, AssignedIdentifierParseError> {
    let uuid = Uuid::parse_str(input).map_err(|_| AssignedIdentifierParseError::InvalidUuid)?;
    if uuid.to_string() != input {
        return Err(AssignedIdentifierParseError::NonCanonical);
    }
    if uuid.get_version_num() != 7 {
        return Err(AssignedIdentifierParseError::NotUuidV7);
    }
    if uuid.get_variant() != Variant::RFC4122 {
        return Err(AssignedIdentifierParseError::NotRfcUuidVariant);
    }
    Ok(uuid)
}

macro_rules! assigned_identifier {
    ($name:ident) => {
        #[doc = "A distinct OMVCS assigned `UUIDv7` identifier."]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(Uuid);

        impl $name {
            /// Generates a new UUID version 7 assigned identifier.
            #[must_use]
            pub fn new() -> Self {
                Self(Uuid::now_v7())
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(&self.0, formatter)
            }
        }

        impl FromStr for $name {
            type Err = AssignedIdentifierParseError;

            fn from_str(input: &str) -> Result<Self, Self::Err> {
                parse_assigned_uuid(input).map(Self)
            }
        }
    };
}

assigned_identifier!(ProjectId);
assigned_identifier!(CreativeComponentId);
assigned_identifier!(LineId);
assigned_identifier!(StorageEndpointId);
assigned_identifier!(ReplicaId);
assigned_identifier!(ContributionId);
assigned_identifier!(ActorId);

/// Errors returned when parsing a typed content identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentIdentifierParseError {
    /// The value does not match the expected OMVCS identifier format.
    InvalidFormat,
    /// The digest is not exactly 64 lowercase hexadecimal characters.
    InvalidDigest,
}

impl fmt::Display for ContentIdentifierParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidFormat => "invalid typed content identifier format",
            Self::InvalidDigest => "digest must be 64 lowercase hexadecimal characters",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for ContentIdentifierParseError {}

fn parse_digest(input: &str, prefix: &str) -> Result<[u8; 32], ContentIdentifierParseError> {
    let digest = input
        .strip_prefix(prefix)
        .ok_or(ContentIdentifierParseError::InvalidFormat)?;
    let bytes = digest.as_bytes();
    if bytes.len() != 64 {
        return Err(ContentIdentifierParseError::InvalidDigest);
    }

    let mut output = [0_u8; 32];
    for (index, pair) in bytes.chunks_exact(2).enumerate() {
        let high = decode_lower_hex(pair[0]).ok_or(ContentIdentifierParseError::InvalidDigest)?;
        let low = decode_lower_hex(pair[1]).ok_or(ContentIdentifierParseError::InvalidDigest)?;
        output[index] = (high << 4) | low;
    }
    Ok(output)
}

const fn decode_lower_hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

fn write_content_identifier(
    formatter: &mut fmt::Formatter<'_>,
    prefix: &str,
    digest: &[u8; 32],
) -> fmt::Result {
    formatter.write_str(prefix)?;
    for byte in digest {
        write!(formatter, "{byte:02x}")?;
    }
    Ok(())
}

macro_rules! content_identifier {
    ($name:ident, $object_type:literal) => {
        #[doc = concat!("A typed SHA-256 identifier for an OMVCS ", $object_type, " object.")]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name([u8; 32]);

        impl $name {
            const PREFIX: &'static str = concat!("omvcs:", $object_type, ":sha256:");

            /// Constructs the typed identifier from its 32-byte SHA-256 digest.
            #[must_use]
            pub const fn from_digest(digest: [u8; 32]) -> Self {
                Self(digest)
            }

            /// Returns the identifier's 32-byte SHA-256 digest.
            #[must_use]
            pub const fn digest(&self) -> &[u8; 32] {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write_content_identifier(formatter, Self::PREFIX, &self.0)
            }
        }

        impl FromStr for $name {
            type Err = ContentIdentifierParseError;

            fn from_str(input: &str) -> Result<Self, Self::Err> {
                parse_digest(input, Self::PREFIX).map(Self)
            }
        }
    };
}

content_identifier!(ResourceId, "resource");
content_identifier!(ChunkId, "chunk");
content_identifier!(ComponentStateId, "component-state");
content_identifier!(AdapterStateId, "adapter-state");
content_identifier!(ProjectStateId, "project-state");
content_identifier!(RevisionId, "revision");
content_identifier!(ReleaseId, "release");

#[cfg(test)]
mod tests {
    use super::*;

    struct ActorFixture {
        id: ActorId,
        display_name: &'static str,
        email: &'static str,
        username: &'static str,
        platform_account: &'static str,
        signing_key: &'static str,
    }

    struct AssignedEntityFixture {
        project_id: ProjectId,
        component_id: CreativeComponentId,
        filename: &'static str,
        path: &'static str,
        storage_endpoint: &'static str,
        platform_url: &'static str,
        credential_reference: &'static str,
    }

    fn valid_v7_text() -> &'static str {
        "019cc17d-1b22-7a41-9fe9-c345c468f82c"
    }

    #[test]
    fn generated_assigned_identifiers_are_canonical_uuidv7() {
        let generated = [
            ProjectId::new().to_string(),
            CreativeComponentId::new().to_string(),
            StorageEndpointId::new().to_string(),
            ContributionId::new().to_string(),
            ActorId::new().to_string(),
        ];

        for text in generated {
            assert_eq!(
                Uuid::parse_str(&text).map(|parsed| (parsed.get_version_num(), parsed.to_string())),
                Ok((7, text))
            );
        }
    }

    #[test]
    fn assigned_identifier_parsing_requires_canonical_uuidv7_text() {
        assert_eq!(
            valid_v7_text().parse::<ActorId>().map(|id| id.to_string()),
            Ok(valid_v7_text().to_owned())
        );
        assert_eq!(
            valid_v7_text().to_uppercase().parse::<ActorId>(),
            Err(AssignedIdentifierParseError::NonCanonical)
        );
        assert_eq!(
            "019cc17d1b227a419fe9c345c468f82c".parse::<ProjectId>(),
            Err(AssignedIdentifierParseError::NonCanonical)
        );
        assert_eq!(
            "019cc17d-1b22-4a41-9fe9-c345c468f82c".parse::<CreativeComponentId>(),
            Err(AssignedIdentifierParseError::NotUuidV7)
        );
        assert_eq!(
            "019cc17d-1b22-7a41-1fe9-c345c468f82c".parse::<ActorId>(),
            Err(AssignedIdentifierParseError::NotRfcUuidVariant)
        );
        assert_eq!(
            "not-a-uuid".parse::<StorageEndpointId>(),
            Err(AssignedIdentifierParseError::InvalidUuid)
        );
    }

    #[test]
    fn actor_id_is_independent_of_profile_account_and_signing_key_values() {
        let mut actor = ActorFixture {
            id: ActorId::new(),
            display_name: "Display name v1",
            email: "email-v1@example.invalid",
            username: "username-v1",
            platform_account: "account-v1",
            signing_key: "key-v1",
        };
        let original_id = actor.id;
        let old_profile = (
            actor.display_name,
            actor.email,
            actor.username,
            actor.platform_account,
            actor.signing_key,
        );

        actor.display_name = "Display name v2";
        actor.email = "email-v2@example.invalid";
        actor.username = "username-v2";
        actor.platform_account = "account-v2";
        actor.signing_key = "key-v2";
        let updated_profile = (
            actor.display_name,
            actor.email,
            actor.username,
            actor.platform_account,
            actor.signing_key,
        );

        assert_ne!(old_profile, updated_profile);
        assert_eq!(actor.id, original_id);
        assert_eq!(actor.id.to_string().parse::<ActorId>(), Ok(original_id));

        actor.signing_key = "key-v3-after-rotation";
        assert_ne!(actor.signing_key, "key-v2");
        assert_eq!(actor.id, original_id);
    }

    #[test]
    fn assigned_project_and_component_ids_are_generated_without_location_inputs() {
        let mut entity = AssignedEntityFixture {
            project_id: ProjectId::new(),
            component_id: CreativeComponentId::new(),
            filename: "project-file-v1",
            path: "project-path-v1",
            storage_endpoint: "storage-v1",
            platform_url: "platform-v1",
            credential_reference: "credential-v1",
        };
        let original_project_id = entity.project_id;
        let original_component_id = entity.component_id;
        let old_locations = (
            entity.filename,
            entity.path,
            entity.storage_endpoint,
            entity.platform_url,
            entity.credential_reference,
        );

        entity.filename = "project-file-v2";
        entity.path = "project-path-v2";
        entity.storage_endpoint = "storage-v2";
        entity.platform_url = "platform-v2";
        entity.credential_reference = "credential-v2";
        let updated_locations = (
            entity.filename,
            entity.path,
            entity.storage_endpoint,
            entity.platform_url,
            entity.credential_reference,
        );

        assert_ne!(old_locations, updated_locations);
        assert_eq!(entity.project_id, original_project_id);
        assert_eq!(entity.component_id, original_component_id);
        assert_eq!(
            entity.project_id.to_string().parse::<ProjectId>(),
            Ok(original_project_id)
        );
        assert_eq!(
            entity
                .component_id
                .to_string()
                .parse::<CreativeComponentId>(),
            Ok(original_component_id)
        );
    }

    #[test]
    fn typed_content_identifiers_round_trip_and_keep_object_namespaces_distinct() {
        let digest = [0xa5; 32];
        let resource = ResourceId::from_digest(digest);
        let component_state = ComponentStateId::from_digest(digest);
        let adapter_state = AdapterStateId::from_digest(digest);
        let project_state = ProjectStateId::from_digest(digest);
        let revision = RevisionId::from_digest(digest);

        assert_eq!(
            resource.to_string(),
            format!("omvcs:resource:sha256:{}", "a5".repeat(32))
        );
        assert_eq!(resource.to_string().parse::<ResourceId>(), Ok(resource));
        assert_eq!(
            component_state.to_string().parse::<ComponentStateId>(),
            Ok(component_state)
        );
        assert_eq!(
            adapter_state.to_string().parse::<AdapterStateId>(),
            Ok(adapter_state)
        );
        assert_eq!(
            project_state.to_string().parse::<ProjectStateId>(),
            Ok(project_state)
        );
        assert_eq!(revision.to_string().parse::<RevisionId>(), Ok(revision));
        assert_ne!(resource.to_string(), revision.to_string());
        assert_ne!(component_state.to_string(), project_state.to_string());
        assert!(
            ("omvcs:revision:sha256:".to_owned() + &"a5".repeat(32))
                .parse::<ResourceId>()
                .is_err()
        );
        assert!(
            ("omvcs:resource:sha256:".to_owned() + &"a5".repeat(32))
                .parse::<RevisionId>()
                .is_err()
        );
    }

    #[test]
    fn content_identifier_parser_rejects_invalid_prefix_digest_length_and_encoding() {
        let good_digest = "a5".repeat(32);
        for malformed in [
            format!("omvcs:Resource:sha256:{good_digest}"),
            format!("omvcs:resource:sha512:{good_digest}"),
            format!("omvcs:resource:sha256:{}", "a5".repeat(31)),
            format!("omvcs:resource:sha256:{}a", "a5".repeat(31)),
            format!("omvcs:resource:sha256:{}a", "a5".repeat(32)),
            format!("omvcs:resource:sha256:{}", "a5".repeat(32) + "a5"),
            format!("omvcs:resource:sha256:{}", "A5".repeat(32)),
            format!("omvcs:resource:sha256:{}", "g5".repeat(32)),
        ] {
            assert!(malformed.parse::<ResourceId>().is_err(), "{malformed}");
        }
    }

    #[test]
    fn content_identifier_digest_access_preserves_all_32_bytes() {
        let digest = [0xa5; 32];
        let resource = ResourceId::from_digest(digest);

        assert_eq!(resource.digest(), &digest);
    }
}
