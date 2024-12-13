use std::collections::HashSet;

use model_server_data::{MaxDistanceKm, ProfileAttributeFilterValue};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::NUMBER_LIST_ATTRIBUTE_MAX_VALUES;
use crate::{LastSeenTimeFilter, ProfileAttributes};

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct ProfileFilteringSettingsUpdate {
    filters: Vec<ProfileAttributeFilterValueUpdate>,
    last_seen_time_filter: Option<LastSeenTimeFilter>,
    unlimited_likes_filter: Option<bool>,
    max_distance_km: Option<MaxDistanceKm>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    random_profile_order: bool,
}

impl ProfileFilteringSettingsUpdate {
    pub fn validate(
        self,
        attribute_info: Option<&ProfileAttributes>,
    ) -> Result<ProfileFilteringSettingsUpdateValidated, String> {
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

        if let Some(value) = self.max_distance_km {
            if value.value <= 0 {
                return Err("Max distance can't be less or equal to 0".to_string());
            }
        }

        Ok(ProfileFilteringSettingsUpdateValidated {
            filters: self.filters,
            last_seen_time_filter: self.last_seen_time_filter,
            unlimited_likes_filter: self.unlimited_likes_filter,
            max_distance_km: self.max_distance_km,
            random_profile_order: self.random_profile_order,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileFilteringSettingsUpdateValidated {
    pub filters: Vec<ProfileAttributeFilterValueUpdate>,
    pub last_seen_time_filter: Option<LastSeenTimeFilter>,
    pub unlimited_likes_filter: Option<bool>,
    pub max_distance_km: Option<MaxDistanceKm>,
    pub random_profile_order: bool,
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
pub struct GetProfileFilteringSettings {
    pub filters: Vec<ProfileAttributeFilterValue>,
    pub last_seen_time_filter: Option<LastSeenTimeFilter>,
    pub unlimited_likes_filter: Option<bool>,
    /// Show profiles until this far from current location. The value
    /// is in kilometers.
    ///
    /// The value must be `None`, 1 or greater number.
    pub max_distance_km: Option<MaxDistanceKm>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    /// Randomize iterator starting position within the profile index area which
    /// current position and [Self::max_distance_km] defines.
    pub random_profile_order: bool,
}
