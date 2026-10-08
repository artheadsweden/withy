use serde::{
    Deserialize, Deserializer, Serialize,
    de::{MapAccess, Visitor, value::MapAccessDeserializer},
};

use crate::CreativeComponentId;

/// The generic OMVCS Creative Component identity object.
///
/// Project State membership and Component State history are separate
/// relationships and are not represented by fields on this object.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct CreativeComponent {
    #[serde(with = "creative_component_id_serde")]
    component_id: CreativeComponentId,
}

impl<'de> Deserialize<'de> for CreativeComponent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Fields {
            #[serde(with = "creative_component_id_serde")]
            component_id: CreativeComponentId,
        }

        struct ComponentVisitor;

        impl<'de> Visitor<'de> for ComponentVisitor {
            type Value = CreativeComponent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a Creative Component object with a named component_id")
            }

            fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let fields = Fields::deserialize(MapAccessDeserializer::new(map))?;
                Ok(CreativeComponent::new(fields.component_id))
            }
        }

        // Core §9 and ADR-0010 require an object, not serde's positional
        // struct representation. Only map access reaches the typed fields.
        deserializer.deserialize_map(ComponentVisitor)
    }
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
