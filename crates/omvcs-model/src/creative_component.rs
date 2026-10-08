use serde::{Deserialize, Serialize};

use crate::CreativeComponentId;

/// The generic OMVCS Creative Component identity object.
///
/// Project State membership and Component State history are separate
/// relationships and are not represented by fields on this object.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreativeComponent {
    #[serde(with = "creative_component_id_serde")]
    component_id: CreativeComponentId,
}

impl CreativeComponent {
    /// Creates a Creative Component from its assigned identifier.
    #[must_use]
    pub const fn new(component_id: CreativeComponentId) -> Self {
        Self { component_id }
    }

    /// Returns the assigned identifier for this Creative Component.
    #[must_use]
    pub const fn component_id(&self) -> CreativeComponentId {
        self.component_id
    }
}

mod creative_component_id_serde {
    use serde::{Deserialize, Deserializer, Serializer, de::Error};

    use crate::CreativeComponentId;

    pub(super) fn serialize<S>(
        component_id: &CreativeComponentId,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&component_id.to_string())
    }

    pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<CreativeComponentId, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(D::Error::custom)
    }
}
