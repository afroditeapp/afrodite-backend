use model::{Attribute, AttributeId};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{AttributeValueReader, ProfileAttributeValue};

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
                &self.unwanted,
                value.raw_values(),
            )
        };

        wanted_match && unwanted_match
    }
}

#[cfg(test)]
mod test {
    use std::num::NonZeroU8;

    use model::{
        Attribute, AttributeId,
        AttributeMode::{self, Bitflag, OneLevel, TwoLevel},
        AttributeValueOrderMode,
    };

    use super::{ProfileAttributeFilterValue, ProfileAttributeValue};

    const V1: u32 = 1 << 16;
    const V1_G1: u32 = V1 | 1;
    const V1_G2: u32 = V1 | 2;
    const V2: u32 = 2 << 16;
    const V2_G1: u32 = V2 | 1;

    fn is_match(mode: AttributeMode, wanted: &[u32], unwanted: &[u32], profile: &[u32]) -> bool {
        let id = AttributeId::new(0);
        let attribute = Attribute {
            key: String::new(),
            name: String::new(),
            mode,
            unsigned_integer_config: None,
            max_selected: NonZeroU8::MAX,
            max_filters: NonZeroU8::MAX,
            editable: true,
            visible: true,
            required: false,
            icon: None,
            id,
            order_number: 0,
            value_order: AttributeValueOrderMode::OrderNumber,
            values: vec![],
            translations: vec![],
        };
        ProfileAttributeFilterValue::new(id, wanted.to_vec(), unwanted.to_vec(), false, false)
            .is_match_with_attribute_value(
                &ProfileAttributeValue::new(id, profile.to_vec()),
                &attribute,
            )
    }

    #[test]
    fn bitflag_unwanted() {
        assert!(is_match(Bitflag, &[], &[0b10], &[0b01]));
        assert!(!is_match(Bitflag, &[], &[0b10], &[0b10]));
        assert!(!is_match(Bitflag, &[], &[0b10], &[0b11]));
    }

    #[test]
    fn bitflag_wanted_and_unwanted() {
        assert!(is_match(Bitflag, &[0b01], &[0b10], &[0b01]));
        assert!(!is_match(Bitflag, &[0b01], &[0b10], &[0b11]));
        assert!(!is_match(Bitflag, &[0b01], &[0b10], &[0b100]));
    }

    #[test]
    fn one_level_unwanted() {
        assert!(is_match(OneLevel, &[], &[2], &[1]));
        assert!(!is_match(OneLevel, &[], &[2], &[2]));
        assert!(!is_match(OneLevel, &[], &[2], &[1, 2]));
    }

    #[test]
    fn one_level_wanted_and_unwanted() {
        assert!(is_match(OneLevel, &[1], &[2], &[1]));
        assert!(!is_match(OneLevel, &[1], &[2], &[1, 2]));
        assert!(!is_match(OneLevel, &[1], &[2], &[3]));
    }

    #[test]
    fn two_level_unwanted() {
        assert!(is_match(TwoLevel, &[], &[V1_G2], &[V1_G1]));
        assert!(!is_match(TwoLevel, &[], &[V1_G2], &[V1_G2]));
        assert!(is_match(TwoLevel, &[], &[V1], &[V2_G1]));
        assert!(!is_match(TwoLevel, &[], &[V1], &[V1_G2]));
    }

    #[test]
    fn two_level_wanted_and_unwanted() {
        assert!(is_match(TwoLevel, &[V1], &[V1_G2], &[V1_G1]));
        assert!(!is_match(TwoLevel, &[V1], &[V1_G2], &[V1_G1, V1_G2]));
        assert!(!is_match(TwoLevel, &[V1], &[V1_G2], &[V2_G1]));
    }
}
