use model::{AttributeId, AttributeMode};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct ProfileAttributeValueUpdate {
    /// Attribute ID
    pub id: AttributeId,
    /// Empty list removes the attribute.
    ///
    /// For bitflag filters the list only has one u16 value.
    ///
    /// For one level attributes the values are u16 attribute value
    /// IDs.
    ///
    /// For two level attributes the values are u32 values
    /// with most significant u16 containing attribute value ID and
    /// least significant u16 containing group value ID.
    pub v: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, ToSchema, PartialEq, Eq)]
pub struct ProfileAttributeValue {
    /// Attribute ID
    id: AttributeId,
    /// For bitflag filters the list only has one u16 value.
    ///
    /// For one level attributes the values are u16 attribute value
    /// IDs.
    ///
    /// For two level attributes the values are u32 values
    /// with most significant u16 containing attribute value ID and
    /// least significant u16 containing group value ID.
    ///
    /// Values are in ascending order.
    v: Vec<u32>,
}

impl ProfileAttributeValue {
    /// None is returned if [ProfileAttributeValueUpdate] disables
    /// the attribute.
    pub fn from_update(mut value: ProfileAttributeValueUpdate) -> Option<Self> {
        value.v.sort();
        match value.v.first() {
            Some(_) => Some(Self {
                id: value.id,
                v: value.v,
            }),
            None => None,
        }
    }

    pub fn new(id: AttributeId, mut values: Vec<u32>) -> Self {
        values.sort();
        Self { id, v: values }
    }

    pub fn id(&self) -> AttributeId {
        self.id
    }

    pub fn raw_values(&self) -> &[u32] {
        &self.v
    }

    pub fn raw_values_count_u8(&self) -> u8 {
        // Max selected attribute values count is NonZeroU8 value so this is safe
        self.v.len() as u8
    }
}

impl From<ProfileAttributeValue> for ProfileAttributeValueUpdate {
    fn from(value: ProfileAttributeValue) -> Self {
        Self {
            id: value.id,
            v: value.v,
        }
    }
}

/// The profile attributes and attribute values are sorted.
#[derive(Debug, Clone, PartialEq)]
pub struct SortedProfileAttributes {
    attributes: Vec<ProfileAttributeValue>,
}

impl SortedProfileAttributes {
    pub fn new(attributes: Vec<ProfileAttributeValue>) -> Self {
        let mut attributes = attributes;
        attributes.sort_by_key(|a| a.id);
        Self { attributes }
    }

    pub fn attributes(&self) -> &Vec<ProfileAttributeValue> {
        &self.attributes
    }

    pub fn find_id(&self, id: AttributeId) -> Option<&ProfileAttributeValue> {
        self.attributes
            .binary_search_by(|a| a.id.cmp(&id))
            .ok()
            .and_then(|i| self.attributes.get(i))
    }

    pub fn set_attributes(&mut self, mut value: Vec<ProfileAttributeValue>) {
        value.sort_by_key(|a| a.id());
        self.attributes = value;
    }
}

pub struct AttributeValueReader;

impl AttributeValueReader {
    /// The filter_data must not be empty
    pub fn wanted_is_match(
        mode: AttributeMode,
        filter_data: &[u32],
        attribute_data: &[u32],
        logical_and: bool,
    ) -> bool {
        match mode {
            AttributeMode::Bitflag => {
                let filter = filter_data.first().copied().unwrap_or_default() as u16;
                let attribute = attribute_data.first().copied().unwrap_or_default() as u16;
                if logical_and {
                    filter & attribute == filter
                } else {
                    filter & attribute != 0
                }
            }
            AttributeMode::OneLevel => Self::wanted_is_number_lists_match(
                filter_data,
                attribute_data,
                logical_and,
                Self::one_level_value_exists,
            ),
            AttributeMode::TwoLevel => Self::wanted_is_number_lists_match(
                filter_data,
                attribute_data,
                logical_and,
                Self::two_level_value_exists,
            ),
            AttributeMode::UnsignedInteger => {
                let value = attribute_data.first().copied().unwrap_or_default();
                let min = filter_data.first().copied().unwrap_or_default();
                let max = filter_data.get(1).copied().unwrap_or_default();
                value >= min && value <= max
            }
        }
    }

    /// The filter_data must not be empty
    fn wanted_is_number_lists_match(
        filter_data: &[u32],
        attribute_data: &[u32],
        logical_and: bool,
        value_exists: fn(u32, &[u32]) -> bool,
    ) -> bool {
        let mut filter_numbers = filter_data.iter().copied();
        if logical_and {
            filter_numbers.all(|v| value_exists(v, attribute_data))
        } else {
            filter_numbers.any(|v| value_exists(v, attribute_data))
        }
    }

    /// The filter_data must not be empty
    pub fn unwanted_is_match(
        mode: AttributeMode,
        filter_data: &[u32],
        attribute_data: &[u32],
    ) -> bool {
        match mode {
            AttributeMode::Bitflag => {
                let filter = filter_data.first().copied().unwrap_or_default() as u16;
                let attribute = !(attribute_data.first().copied().unwrap_or_default() as u16);
                filter & attribute == filter
            }
            AttributeMode::OneLevel => Self::unwanted_is_number_lists_match(
                filter_data,
                attribute_data,
                Self::one_level_value_exists,
            ),
            AttributeMode::TwoLevel => Self::unwanted_is_number_lists_match(
                filter_data,
                attribute_data,
                Self::two_level_value_exists,
            ),
            // The unsigned integer mode has no unwanted values so no values are
            // excluded.
            AttributeMode::UnsignedInteger => false,
        }
    }

    /// The filter_data must not be empty
    fn unwanted_is_number_lists_match(
        filter_data: &[u32],
        attribute_data: &[u32],
        value_exists: fn(u32, &[u32]) -> bool,
    ) -> bool {
        !filter_data.iter().any(|v| value_exists(*v, attribute_data))
    }

    fn one_level_value_exists(filter_number: u32, attribute_data: &[u32]) -> bool {
        attribute_data.contains(&filter_number)
    }

    fn two_level_value_exists(filter_number: u32, attribute_data: &[u32]) -> bool {
        let filter_group_value = (filter_number & 0xFFFF) as u16;
        if filter_group_value == 0 {
            // Compare only with attribute value
            attribute_data
                .iter()
                .any(|v| v & 0xFFFF0000 == filter_number)
        } else {
            attribute_data.contains(&filter_number)
        }
    }
}

#[cfg(test)]
mod test {
    use model::AttributeMode::{self, Bitflag, OneLevel, TwoLevel};

    use super::AttributeValueReader;

    const fn two_level(value: u32, group_value: u32) -> u32 {
        (value << 16) | group_value
    }

    const V1: u32 = two_level(1, 0);
    const V1_G1: u32 = two_level(1, 1);
    const V1_G2: u32 = two_level(1, 2);
    const V1_G3: u32 = two_level(1, 3);
    const V2: u32 = two_level(2, 0);
    const V2_G1: u32 = two_level(2, 1);
    const V2_G2: u32 = two_level(2, 2);
    const V3: u32 = two_level(3, 0);
    const V3_G1: u32 = two_level(3, 1);

    fn wanted_or(mode: AttributeMode, filter: &[u32], profile: &[u32]) -> bool {
        AttributeValueReader::wanted_is_match(mode, filter, profile, false)
    }

    fn wanted_and(mode: AttributeMode, filter: &[u32], profile: &[u32]) -> bool {
        AttributeValueReader::wanted_is_match(mode, filter, profile, true)
    }

    fn unwanted(mode: AttributeMode, filter: &[u32], profile: &[u32]) -> bool {
        AttributeValueReader::unwanted_is_match(mode, filter, profile)
    }

    #[test]
    fn bitflag_wanted_or() {
        assert!(wanted_or(Bitflag, &[0b101], &[0b100]));
        assert!(wanted_or(Bitflag, &[0b101], &[0b011]));
        assert!(!wanted_or(Bitflag, &[0b101], &[0b010]));
    }

    #[test]
    fn bitflag_wanted_and() {
        assert!(wanted_and(Bitflag, &[0b101], &[0b111]));
        assert!(!wanted_and(Bitflag, &[0b101], &[0b100]));
    }

    #[test]
    fn bitflag_unwanted() {
        assert!(unwanted(Bitflag, &[0b101], &[0b010]));
        assert!(!unwanted(Bitflag, &[0b101], &[0b110]));
    }

    #[test]
    fn one_level_wanted_or() {
        assert!(wanted_or(OneLevel, &[1, 3], &[1]));
        assert!(wanted_or(OneLevel, &[1, 3], &[3]));
        assert!(wanted_or(OneLevel, &[1, 3], &[2, 3]));
        assert!(wanted_or(OneLevel, &[1, 2, 3], &[3, 4]));
        assert!(!wanted_or(OneLevel, &[1, 3], &[2]));
        assert!(!wanted_or(OneLevel, &[1, 3], &[2, 4]));
    }

    #[test]
    fn one_level_wanted_and() {
        assert!(wanted_and(OneLevel, &[1, 3], &[1, 3]));
        assert!(wanted_and(OneLevel, &[1, 3], &[1, 2, 3, 4]));
        assert!(!wanted_and(OneLevel, &[1, 3], &[1]));
        assert!(!wanted_and(OneLevel, &[1, 3], &[3]));
    }

    #[test]
    fn one_level_unwanted() {
        assert!(unwanted(OneLevel, &[1, 3], &[2]));
        assert!(!unwanted(OneLevel, &[1, 3], &[1]));
        assert!(!unwanted(OneLevel, &[1, 3], &[3]));
        assert!(!unwanted(OneLevel, &[1, 3], &[2, 3]));
    }

    #[test]
    fn two_level_wanted_or() {
        assert!(wanted_or(TwoLevel, &[V1_G1, V1_G3], &[V1_G3]));
        assert!(wanted_or(TwoLevel, &[V1_G1, V2_G1], &[V2_G1]));
        assert!(!wanted_or(TwoLevel, &[V1_G1, V2_G1], &[V1_G2]));
        assert!(!wanted_or(TwoLevel, &[V1_G1], &[V1]));
    }

    #[test]
    fn two_level_wanted_or_top_level_value() {
        assert!(wanted_or(TwoLevel, &[V1], &[V1]));
        assert!(wanted_or(TwoLevel, &[V1], &[V1_G2]));
        assert!(wanted_or(TwoLevel, &[V1, V3], &[V3_G1]));
        assert!(wanted_or(TwoLevel, &[V1, V2_G1], &[V2_G1]));
        assert!(!wanted_or(TwoLevel, &[V1, V3], &[V2_G1]));
    }

    #[test]
    fn two_level_wanted_and() {
        assert!(wanted_and(TwoLevel, &[V1_G1, V2_G1], &[V1_G1, V2_G1]));
        assert!(wanted_and(
            TwoLevel,
            &[V1_G1, V2_G1],
            &[V1_G1, V1_G2, V2_G1]
        ));
        assert!(!wanted_and(TwoLevel, &[V1_G1, V2_G1], &[V1_G1]));
        assert!(!wanted_and(TwoLevel, &[V1_G1, V2_G1], &[V2_G1]));
    }

    #[test]
    fn two_level_wanted_and_top_level_value() {
        assert!(wanted_and(TwoLevel, &[V1, V2], &[V1_G1, V2_G2]));
        assert!(wanted_and(TwoLevel, &[V1, V2_G1], &[V1_G2, V2_G1]));
        assert!(!wanted_and(TwoLevel, &[V1, V2], &[V1_G1, V1_G2]));
        assert!(!wanted_and(TwoLevel, &[V1, V2_G1], &[V1_G2, V2_G2]));
    }

    #[test]
    fn two_level_wanted_and_top_level_value_with_its_group_value() {
        assert!(wanted_and(TwoLevel, &[V1, V1_G2], &[V1_G2]));
        assert!(wanted_and(TwoLevel, &[V1, V1_G2], &[V1_G1, V1_G2]));
        assert!(!wanted_and(TwoLevel, &[V1, V1_G2], &[V1_G1]));
    }

    #[test]
    fn two_level_unwanted() {
        assert!(unwanted(TwoLevel, &[V1_G1, V2_G1], &[V1_G2]));
        assert!(!unwanted(TwoLevel, &[V1_G1, V2_G1], &[V1_G1]));
        assert!(!unwanted(TwoLevel, &[V1_G1, V2_G1], &[V2_G1]));
    }

    #[test]
    fn two_level_unwanted_top_level_value() {
        assert!(unwanted(TwoLevel, &[V1, V3], &[V2_G1]));
        assert!(!unwanted(TwoLevel, &[V1, V3], &[V1_G1]));
        assert!(!unwanted(TwoLevel, &[V1, V3], &[V3_G1]));
    }
}
