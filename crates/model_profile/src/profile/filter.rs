use std::collections::HashSet;

use model::AttributeId;
use model_server_data::{
    MaxDistanceKm, ProfileAttributeFilterValue, ProfileCreatedTimeFilter, ProfileEditedTimeFilter, ProfileTextMaxCharactersFilter, ProfileTextMinCharactersFilter
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{LastSeenTimeFilter, ProfileAttributesInternal};

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct ProfileFilteringSettingsUpdate {
    filters: Vec<ProfileAttributeFilterValueUpdate>,
    last_seen_time_filter: Option<LastSeenTimeFilter>,
    unlimited_likes_filter: Option<bool>,
    max_distance_km_filter: Option<MaxDistanceKm>,
    profile_created_filter: Option<ProfileCreatedTimeFilter>,
    profile_edited_filter: Option<ProfileEditedTimeFilter>,
    profile_text_min_characters_filter: Option<ProfileTextMinCharactersFilter>,
    profile_text_max_characters_filter: Option<ProfileTextMaxCharactersFilter>,
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
                        let check = |values: &[u32]| {
                            let error = || Err(format!(
                                "Attribute supports max {} filter values",
                                info.max_filters,
                            ));
                            if info.mode.is_bitflag() {
                                let selected = values.first().copied().unwrap_or_default().count_ones();
                                if selected > info.max_filters.into() {
                                    return error();
                                }
                            } else if values.len() > info.max_filters.into() {
                                return error();
                            }

                            Ok(())
                        };

                        check(&a.filter_values)?;
                        check(&a.filter_values_nonselected)?;
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
            profile_text_min_characters_filter: self.profile_text_min_characters_filter,
            profile_text_max_characters_filter: self.profile_text_max_characters_filter,
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
    pub profile_text_min_characters_filter: Option<ProfileTextMinCharactersFilter>,
    pub profile_text_max_characters_filter: Option<ProfileTextMaxCharactersFilter>,
    pub random_profile_order: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct ProfileAttributeFilterValueUpdate {
    /// Attribute ID
    pub id: AttributeId,
    /// Value `false` ignores the settings in this object and
    /// removes current filter settings for this attribute.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub enabled: bool,
    /// For bitflag filters the list only has one u16 value.
    ///
    /// For one level attributes the values are u16 attribute value
    /// IDs.
    ///
    /// For two level attributes the values are u32 values
    /// with most significant u16 containing attribute value ID and
    /// least significant u16 containing group value ID.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[schema(default = json!([]))]
    pub filter_values: Vec<u32>,
    /// Same as [Self::filter_values] but for nonselected values.
    ///
    /// The nonselected values are checked always with AND operator.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[schema(default = json!([]))]
    pub filter_values_nonselected: Vec<u32>,
    /// Defines should missing attribute be accepted.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub accept_missing_attribute: bool,
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
    pub profile_text_min_characters_filter: Option<ProfileTextMinCharactersFilter>,
    pub profile_text_max_characters_filter: Option<ProfileTextMaxCharactersFilter>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    /// Randomize iterator starting position within the profile index area which
    /// current position and [Self::max_distance_km] defines.
    pub random_profile_order: bool,
}
