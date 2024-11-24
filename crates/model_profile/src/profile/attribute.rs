use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    Attribute, ProfileAttributes, ProfileUpdateValidated
};

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct ProfileAttributeValueUpdate {
    /// Attribute ID
    pub id: u16,
    /// Empty list removes the attribute.
    ///
    /// - First value is bitflags value or top level attribute value ID or first number list value.
    /// - Second value is sub level attribute value ID or second number list value.
    /// - Third and rest are number list values.
    pub v: Vec<u16>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct ProfileAttributeValue {
    /// Attribute ID
    id: u16,
    /// - First value is bitflags value or top level attribute value ID or first number list value.
    /// - Second value is sub level attribute value ID or second number list value.
    /// - Third and rest are number list values.
    ///
    /// The number list values are in ascending order.
    v: Vec<u16>,
}

impl ProfileAttributeValue {
    pub fn try_from_update_and_sort(mut value: ProfileAttributeValueUpdate, attribute: &Attribute) -> Result<Self, String> {
        if attribute.mode.is_number_list() {
            value.v.sort();
        }
        Self::try_from_update(value)
    }

    pub fn try_from_update(value: ProfileAttributeValueUpdate) -> Result<Self, String> {
        match value.v.first() {
            Some(_) => Ok(Self { id: value.id, v: value.v }),
            None => Err("Value part1 missing".to_string()),
        }
    }

    pub fn new_not_number_list(id: u16, values: Vec<u16>) -> Self {
        Self { id, v: values }
    }

    pub fn new_number_list(id: u16, mut values: Vec<u16>) -> Self {
        values.sort();
        Self { id, v: values }
    }

    pub fn id(&self) -> u16 {
        self.id
    }

    pub fn as_bitflags(&self) -> u16 {
        self.v.first().copied().unwrap_or(0)
    }

    /// ID number for top level AttributeValue ID.
    pub fn as_top_level_id(&self) -> u16 {
        self.v.first().copied().unwrap_or(0)
    }

    /// ID number for sub level AttributeValue ID.
    pub fn as_sub_level_id(&self) -> Option<u16> {
        self.v.get(1).copied()
    }

    pub fn as_number_list(&self) -> &[u16] {
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

/// The profile attributes and possible number list values are sorted.
#[derive(Debug, Clone, PartialEq)]
pub struct SortedProfileAttributes {
    attributes: Vec<ProfileAttributeValue>,
}

impl SortedProfileAttributes {
    pub fn new(attributes: Vec<ProfileAttributeValue>, all_attributes: Option<&ProfileAttributes>) -> Self {
        let mut attributes = attributes;
        attributes.sort_by(|a, b| a.id.cmp(&b.id));

        for a in &mut attributes {
            let id_usize: usize = a.id.into();
            if let Some(info) = all_attributes.and_then(|attributes| attributes.attributes.get(id_usize)) {
                if info.mode.is_number_list() {
                    a.v.sort();
                }
            }
        }

        Self { attributes }
    }

    pub fn attributes(&self) -> &Vec<ProfileAttributeValue> {
        &self.attributes
    }

    pub fn find_id(&self, id: u16) -> Option<&ProfileAttributeValue> {
        self.attributes
            .binary_search_by(|a| a.id.cmp(&id))
            .ok()
            .and_then(|i| self.attributes.get(i))
    }

    pub fn update_from(&mut self, update: &ProfileUpdateValidated) {
        let mut attributes = update
            .attributes
            .iter()
            .filter_map(|v| ProfileAttributeValue::try_from_update(v.clone()).ok())
            .collect::<Vec<_>>();
        attributes.sort_by(|a, b| a.id.cmp(&b.id));
        self.attributes = attributes;
    }
}
