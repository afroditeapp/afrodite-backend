use model::AttributeId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{Attribute, AttributeValueReader, ProfileAttributeValue};

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct ProfileAttributeFilterValue {
    /// Attribute ID
    id: AttributeId,
    /// Wanted attribute values.
    ///
    /// For bitflag filters the list only has one u16 value.
    ///
    /// For one level attributes the values are u16 attribute value
    /// IDs.
    ///
    /// For two level attributes the values are u32 values
    /// with most significant u16 containing attribute value ID and
    /// least significant u16 containing group value ID.
    ///
    /// The values are stored in ascending order.
    wanted: Vec<u32>,
    /// Same as [Self::wanted] but for unwanted values.
    ///
    /// The unwanted values are checked always with AND operator.
    unwanted: Vec<u32>,
    accept_missing_attribute: bool,
    use_logical_operator_and: bool,
}

impl ProfileAttributeFilterValue {
    pub fn new(
        id: AttributeId,
        mut wanted: Vec<u32>,
        mut unwanted: Vec<u32>,
        accept_missing_attribute: bool,
        use_logical_operator_and: bool,
    ) -> Self {
        wanted.sort();
        unwanted.sort();
        Self {
            id,
            wanted,
            unwanted,
            accept_missing_attribute,
            use_logical_operator_and,
        }
    }

    pub fn id(&self) -> AttributeId {
        self.id
    }

    pub fn accept_missing_attribute_enabled(&self) -> bool {
        self.accept_missing_attribute
    }

    #[allow(clippy::comparison_chain)]
    pub fn is_match_with_attribute_value(
        &self,
        value: &ProfileAttributeValue,
        attribute_info: &Attribute,
    ) -> bool {
        let wanted_match = if self.wanted.is_empty() {
            true
        } else {
            AttributeValueReader::wanted_is_match(
                attribute_info.mode,
                &self.wanted,
                value.raw_values(),
                self.use_logical_operator_and,
            )
        };

        let unwanted_match = if self.unwanted.is_empty() {
            true
        } else {
            AttributeValueReader::unwanted_is_match(
                attribute_info.mode,
                &self.wanted,
                value.raw_values(),
            )
        };

        wanted_match && unwanted_match
    }
}
