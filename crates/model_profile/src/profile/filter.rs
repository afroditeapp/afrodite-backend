use std::collections::HashSet;

use model_server_data::ProfileAttributeFilterValue;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::NUMBER_LIST_ATTRIBUTE_MAX_VALUES;
use crate::{LastSeenTimeFilter, ProfileAttributes};

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct ProfileAttributeFilterListUpdate {
    filters: Vec<ProfileAttributeFilterValueUpdate>,
    last_seen_time_filter: Option<LastSeenTimeFilter>,
    unlimited_likes_filter: Option<bool>,
}

impl ProfileAttributeFilterListUpdate {
    pub fn validate(
        self,
        attribute_info: Option<&ProfileAttributes>,
    ) -> Result<ProfileAttributeFilterListUpdateValidated, String> {
        let mut hash_set = HashSet::new();
        for a in &self.filters {
            if !hash_set.insert(a.id) {
                return Err("Duplicate attribute ID".to_string());
            }

            if let Some(info) = attribute_info {
                let attribute_info = info.attributes.get(a.id as usize);
                match attribute_info {
                    None => return Err("Unknown attribute ID".to_string()),
                    Some(info) => {
                        if info.mode.is_number_list()
                            && a.filter_values.len() > NUMBER_LIST_ATTRIBUTE_MAX_VALUES
                        {
                            return Err(format!(
                                "Number list attribute supports max {} filters",
                                NUMBER_LIST_ATTRIBUTE_MAX_VALUES
                            ));
                        }
                    }
                }
            } else {
                return Err("Profile attributes are disabled".to_string());
            }
        }

        if let Some(value) = self.last_seen_time_filter {
            if value.value < LastSeenTimeFilter::MIN_VALUE {
                return Err("Invalid LastSeenTimeFilter value".to_string());
            }
        }

        Ok(ProfileAttributeFilterListUpdateValidated {
            filters: self.filters,
            last_seen_time_filter: self.last_seen_time_filter,
            unlimited_likes_filter: self.unlimited_likes_filter,
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct ProfileAttributeFilterListUpdateValidated {
    pub filters: Vec<ProfileAttributeFilterValueUpdate>,
    pub last_seen_time_filter: Option<LastSeenTimeFilter>,
    pub unlimited_likes_filter: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct ProfileAttributeFilterValueUpdate {
    /// Attribute ID
    pub id: u16,
    /// - First value is bitflags value or top level attribute value ID or first number list value.
    /// - Second value is sub level attribute value ID or second number list value.
    /// - Third and rest are number list values.
    pub filter_values: Vec<u16>,
    /// Defines should missing attribute be accepted.
    ///
    /// Setting this to `None` disables the filter.
    pub accept_missing_attribute: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct ProfileAttributeFilterList {
    pub filters: Vec<ProfileAttributeFilterValue>,
    pub last_seen_time_filter: Option<LastSeenTimeFilter>,
    pub unlimited_likes_filter: Option<bool>,
}
