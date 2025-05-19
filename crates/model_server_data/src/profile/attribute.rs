use model::AttributeId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::AttributeDataType;

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
    pub fn try_from_update(
        mut value: ProfileAttributeValueUpdate,
    ) -> Result<Self, String> {
        value.v.sort();
        match value.v.first() {
            Some(_) => Ok(Self {
                id: value.id,
                v: value.v,
            }),
            None => Err("Value part1 missing".to_string()),
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
    pub fn new(
        attributes: Vec<ProfileAttributeValue>,
    ) -> Self {
        let mut attributes = attributes;
        attributes.sort_by(|a, b| a.id.cmp(&b.id));
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
    pub fn is_match(
        data_type: AttributeDataType,
        filter_data: &[u32],
        attribute_data: &[u32],
        logical_and: bool,
    ) -> bool {
        match data_type {
            AttributeDataType::Bitflag => {
                let filter = filter_data.first().copied().unwrap_or_default() as u16;
                let attribute = attribute_data.first().copied().unwrap_or_default() as u16;
                if logical_and {
                    filter & attribute == filter
                } else {
                    filter & attribute != 0
                }
            }
            AttributeDataType::OneLevel =>
                Self::is_number_lists_match(
                    filter_data,
                    attribute_data,
                    logical_and,
                    NumberExistence::one_level_attribute_find_from_sorted,
                ),
            AttributeDataType::TwoLevel =>
                Self::is_number_lists_match(
                    filter_data,
                    attribute_data,
                    logical_and,
                    NumberExistence::two_level_attribute_find_from_sorted,
                ),
        }
    }

    /// The filter_data must not be empty
    fn is_number_lists_match<'a, F: Fn(u32, &mut std::iter::Copied<std::slice::Iter<'a, u32>>) -> NumberExistence>(
        filter_data: &[u32],
        attribute_data: &'a [u32],
        logical_and: bool,
        existence_check: F,
    ) -> bool {
        // Assume that both number lists are sorted
        let mut value_iter = attribute_data.iter().copied();
        if logical_and {
            for filter_number in filter_data {
                match existence_check(*filter_number, &mut value_iter) {
                    NumberExistence::Found => continue,
                    NumberExistence::NotFound => return false,
                }
            }
            true
        } else {
            for filter_number in filter_data {
                match existence_check(*filter_number, &mut value_iter) {
                    NumberExistence::Found => return true,
                    NumberExistence::NotFound => continue,
                }
            }
            false
        }
    }

    /// The filter_data must not be empty
    pub fn is_match_nonselected(
        data_type: AttributeDataType,
        filter_data: &[u32],
        attribute_data: &[u32],
    ) -> bool {
        match data_type {
            AttributeDataType::Bitflag => {
                let filter = filter_data.first().copied().unwrap_or_default() as u16;
                let attribute = !(attribute_data.first().copied().unwrap_or_default() as u16);
                filter & attribute == filter
            }
            AttributeDataType::OneLevel =>
                Self::is_number_lists_match_nonselected(
                    filter_data,
                    attribute_data,
                    NumberExistence::one_level_attribute_find_from_sorted,
                ),
            AttributeDataType::TwoLevel =>
                Self::is_number_lists_match_nonselected(
                    filter_data,
                    attribute_data,
                    NumberExistence::two_level_attribute_find_from_sorted,
                ),
        }
    }

    /// The filter_data must not be empty
    fn is_number_lists_match_nonselected<'a, F: Fn(u32, &mut std::iter::Copied<std::slice::Iter<'a, u32>>) -> NumberExistence>(
        filter_data: &[u32],
        attribute_data: &'a [u32],
        existence_check: F,
    ) -> bool {
        // Assume that both number lists are sorted
        let mut value_iter = attribute_data.iter().copied();
        for filter_number in filter_data {
            match existence_check(*filter_number, &mut value_iter) {
                NumberExistence::Found => return false,
                NumberExistence::NotFound => continue,
            }
        }
        true
    }
}

enum NumberExistence {
    Found,
    NotFound,
}

impl NumberExistence {
    fn one_level_attribute_find_from_sorted<T: Iterator<Item=u32>>(filter_number: u32, value_iter: &mut T) -> Self {
        for value_number in value_iter {
            if value_number < filter_number {
                // Can be found still
                continue;
            } else if value_number == filter_number {
                // Found
                return NumberExistence::Found;
            } else {
                // Not found
                return NumberExistence::NotFound;
            }
        }
        NumberExistence::NotFound
    }

    fn two_level_attribute_find_from_sorted<T: Iterator<Item=u32>>(filter_number: u32, value_iter: &mut T) -> Self {
        let filter_group_value = (filter_number & 0xFFFF) as u16;
        if filter_group_value == 0 {
            // Compare only with attribute value
            let filter_number = filter_number & 0xFFFF0000;
            Self::one_level_attribute_find_from_sorted(filter_number, &mut value_iter.by_ref().map(|v| v & 0xFFFF0000))
        } else {
            Self::one_level_attribute_find_from_sorted(filter_number, value_iter)
        }
    }
}
