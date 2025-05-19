use std::collections::HashSet;

use model::AttributeId;
use model_server_data::{ProfileCreatedTimeFilter, MaxDistanceKm, ProfileAttributeFilterValue, ProfileEditedTimeFilter};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::ATTRIBUTE_MAX_VALUES;
use crate::{LastSeenTimeFilter, ProfileAttributesInternal};

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct ProfileFilteringSettingsUpdate {
    filters: Vec<ProfileAttributeFilterValueUpdate>,
    last_seen_time_filter: Option<LastSeenTimeFilter>,
    unlimited_likes_filter: Option<bool>,
    max_distance_km_filter: Option<MaxDistanceKm>,
    profile_created_filter: Option<ProfileCreatedTimeFilter>,
    profile_edited_filter: Option<ProfileEditedTimeFilter>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    random_profile_order: bool,
}

impl ProfileFilteringSettingsUpdate {
    pub fn validate(
        self,
        attribute_info: Option<&ProfileAttributesInternal>,
    ) -> Result<ProfileFilteringSettingsUpdateValidated, String> {
        let mut hash_set = HashSet::new();
        for a in &self.filters {
            if !hash_set.insert(a.id) {
                return Err("Duplicate attribute ID".to_string());
            }

            if let Some(info) = attribute_info {
                let attribute_info = info.get_attribute(a.id);
                match attribute_info {
                    None => return Err("Unknown attribute ID".to_string()),
                    Some(info) => {
                        if !info.mode.data_type().is_bitflag()
                            && a.filter_values.len() > ATTRIBUTE_MAX_VALUES
                        {
                            return Err(format!(
                                "Non bitflag attributes supports max {} filters",
                                ATTRIBUTE_MAX_VALUES
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

        if let Some(value) = self.max_distance_km_filter {
            if value.value <= 0 {
                return Err("Max distance can't be less or equal to 0".to_string());
            }
        }

        if let Some(value) = self.profile_created_filter {
            if value.value < 0 {
                return Err("Profile created time filter can't be less than zero".to_string());
            }
        }

        if let Some(value) = self.profile_edited_filter {
            if value.value < 0 {
                return Err("Profile edited time filter can't be less than zero".to_string());
            }
        }

        Ok(ProfileFilteringSettingsUpdateValidated {
            filters: self.filters,
            last_seen_time_filter: self.last_seen_time_filter,
            unlimited_likes_filter: self.unlimited_likes_filter,
            max_distance_km_filter: self.max_distance_km_filter,
            profile_created_filter: self.profile_created_filter,
            profile_edited_filter: self.profile_edited_filter,
            random_profile_order: self.random_profile_order,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileFilteringSettingsUpdateValidated {
    pub filters: Vec<ProfileAttributeFilterValueUpdate>,
    pub last_seen_time_filter: Option<LastSeenTimeFilter>,
    pub unlimited_likes_filter: Option<bool>,
    pub max_distance_km_filter: Option<MaxDistanceKm>,
    pub profile_created_filter: Option<ProfileCreatedTimeFilter>,
    pub profile_edited_filter: Option<ProfileEditedTimeFilter>,
    pub random_profile_order: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct ProfileAttributeFilterValueUpdate {
    /// Attribute ID
    pub id: AttributeId,
    /// For bitflag filters the list only has one u16 value.
    ///
    /// For one level attributes the values are u16 attribute value
    /// IDs.
    ///
    /// For two level attributes the values are u32 values
    /// with most significant u16 containing attribute value ID and
    /// least significant u16 containing group value ID.
    pub filter_values: Vec<u32>,
    /// Defines should missing attribute be accepted.
    ///
    /// Setting this to `None` disables the filter.
    pub accept_missing_attribute: Option<bool>,
    /// Defines should attribute values be checked with logical operator AND.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub use_logical_operator_and: bool,
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
    pub max_distance_km_filter: Option<MaxDistanceKm>,
    pub profile_created_filter: Option<ProfileCreatedTimeFilter>,
    pub profile_edited_filter: Option<ProfileEditedTimeFilter>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    /// Randomize iterator starting position within the profile index area which
    /// current position and [Self::max_distance_km] defines.
    pub random_profile_order: bool,
}
