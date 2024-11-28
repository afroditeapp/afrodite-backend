use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{Attribute, ProfileAttributeValue};


#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct ProfileAttributeFilterValue {
    /// Attribute ID
    id: u16,
    /// - First value is bitflags value or top level attribute value ID or first number list value.
    /// - Second value is sub level attribute value ID or second number list value.
    /// - Third and rest are number list values.
    ///
    /// The number list values are in ascending order.
    filter_values: Vec<u16>,
    accept_missing_attribute: bool,
}

impl ProfileAttributeFilterValue {
    pub fn new_not_number_list(
        id: u16,
        filter_values: Vec<u16>,
        accept_missing_attribute: bool,
    ) -> Self {
        Self {
            id,
            filter_values,
            accept_missing_attribute,
        }
    }

    pub fn id(&self) -> u16 {
        self.id
    }

    pub fn accept_missing_attribute_enabled(&self) -> bool {
        self.accept_missing_attribute
    }

    /// Bitflag filter value
    pub fn as_bitflags(&self) -> u16 {
        self.filter_values.first().copied().unwrap_or(0)
    }

    /// ID number for top level AttributeValue ID.
    pub fn as_top_level_id(&self) -> Option<u16> {
        self.filter_values.first().copied()
    }

    /// ID number for sub level AttributeValue ID.
    pub fn as_sub_level_id(&self) -> Option<u16> {
        self.filter_values.get(1).copied()
    }

    pub fn as_number_list(&self) -> &[u16] {
        &self.filter_values
    }

    pub fn set_number_list_filter_value(&mut self, mut values: Vec<u16>) {
        values.sort();
        self.filter_values = values;
    }

    #[allow(clippy::comparison_chain)]
    pub fn is_match_with_attribute_value(
        &self,
        value: &ProfileAttributeValue,
        attribute_info: &Attribute,
    ) -> bool {
        if self.id != value.id() {
            return false;
        }

        if attribute_info.mode.is_bitflag_mode() {
            self.as_bitflags() & value.as_bitflags() != 0
        } else if attribute_info.mode.is_number_list() {
            // Assume that both number lists are sorted
            let mut value_iter = value.as_number_list().iter();
            let mut found = !self.as_number_list().is_empty();

            for filter_number in self.as_number_list() {
                while found {
                    match value_iter.next() {
                        Some(value_number) => {
                            if value_number < filter_number {
                                // Can be found still
                                continue;
                            } else if value_number == filter_number {
                                // Found
                                break;
                            } else {
                                // Not found
                                found = false;
                                break;
                            }
                        }
                        None => {
                            found = false;
                            break;
                        }
                    }
                }
            }

            found
        } else {
            if let Some(top_level_id) = self.as_top_level_id() {
                if top_level_id == value.as_top_level_id() {
                    match self.as_sub_level_id() {
                        wanted @ Some(_) => wanted == value.as_sub_level_id(),
                        None => true,
                    }
                } else {
                    false
                }
            } else {
                false
            }
        }
    }
}
