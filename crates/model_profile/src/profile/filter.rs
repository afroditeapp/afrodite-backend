use std::collections::HashSet;

use model::AttributeId;
use model_server_data::{
    MaxDistanceKm, MinDistanceKm, ProfileAttributeFilterValue, ProfileCreatedTimeFilter,
    ProfileEditedTimeFilter, ProfileTextMaxCharactersFilter, ProfileTextMinCharactersFilter,
    ProfileVerificationStatusFilter,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{LastSeenTimeFilter, ProfileAttributesInternal};

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct ProfileFiltersUpdate {
    attribute_filters: Vec<ProfileAttributeFilterValueUpdate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    last_seen_time_filter: Option<LastSeenTimeFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    unlimited_likes_filter: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    min_distance_km_filter: Option<MinDistanceKm>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    max_distance_km_filter: Option<MaxDistanceKm>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    profile_created_filter: Option<ProfileCreatedTimeFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    profile_edited_filter: Option<ProfileEditedTimeFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    profile_text_min_characters_filter: Option<ProfileTextMinCharactersFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    profile_text_max_characters_filter: Option<ProfileTextMaxCharactersFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    profile_verification_status_filter: Option<ProfileVerificationStatusFilter>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    random_profile_order: bool,
}

impl ProfileFiltersUpdate {
    pub fn validate(
        self,
        attribute_info: &ProfileAttributesInternal,
    ) -> Result<ProfileFiltersUpdateValidated, String> {
        let mut hash_set = HashSet::new();
        for a in &self.attribute_filters {
            if !hash_set.insert(a.id) {
                return Err("Duplicate attribute ID".to_string());
            }

            let attribute_info = attribute_info.get_attribute(a.id);
            match attribute_info {
                None => return Err("Unknown attribute ID".to_string()),
                Some(info) => {
                    if info.mode.is_unsigned_integer() {
                        if !a.unwanted.is_empty() {
                            return Err(
                                "unsigned integer attribute filter cannot have unwanted values"
                                    .to_string(),
                            );
                        }
                        if a.wanted.len() == 2 {
                            let min = a.wanted[0];
                            let max = a.wanted[1];
                            let unsigned_integer_config = info
                                .unsigned_integer_config
                                .as_ref()
                                .ok_or_else(|| {
                                    format!(
                                        "Attribute {} in unsigned integer mode must have unsigned_integer_config",
                                        info.key
                                    )
                                })?;
                            if min > max {
                                return Err(
                                    "unsigned integer attribute filter min must be less or equal to max"
                                        .to_string(),
                                );
                            }
                            if min < unsigned_integer_config.min.into()
                                || max > unsigned_integer_config.max.into()
                            {
                                return Err(format!(
                                    "unsigned integer attribute filter min..=max must be within {}..={}",
                                    unsigned_integer_config.min, unsigned_integer_config.max
                                ));
                            }
                        } else if a.wanted.is_empty() {
                            // Valid value
                        } else {
                            return Err(
                                "unsigned integer attribute filter must have exactly zero or two wanted values"
                                    .to_string(),
                            );
                        }
                    } else {
                        let check = |values: &[u32]| {
                            let error = || {
                                Err(format!(
                                    "Attribute supports max {} filter values",
                                    info.max_filters,
                                ))
                            };
                            if info.mode.is_bitflag() {
                                let selected =
                                    values.first().copied().unwrap_or_default().count_ones();
                                if selected > info.max_filters.get().into() {
                                    return error();
                                }
                            } else if values.len() > info.max_filters.get().into() {
                                return error();
                            }

                            Ok(())
                        };

                        check(&a.wanted)?;
                        check(&a.unwanted)?;
                    }
                }
            }
        }

        if let Some(value) = self.last_seen_time_filter
            && value.value < LastSeenTimeFilter::MIN_VALUE
        {
            return Err("Invalid LastSeenTimeFilter value".to_string());
        }

        if let Some(value) = self.min_distance_km_filter
            && value.value <= 0
        {
            return Err("Min distance can't be less or equal to 0".to_string());
        }

        if let Some(value) = self.max_distance_km_filter
            && value.value <= 0
        {
            return Err("Max distance can't be less or equal to 0".to_string());
        }

        if let Some(value) = self.profile_created_filter
            && value.value < 0
        {
            return Err("Profile created time filter can't be less than zero".to_string());
        }

        if let Some(value) = self.profile_edited_filter
            && value.value < 0
        {
            return Err("Profile edited time filter can't be less than zero".to_string());
        }

        Ok(ProfileFiltersUpdateValidated {
            attribute_filters: self.attribute_filters,
            last_seen_time_filter: self.last_seen_time_filter,
            unlimited_likes_filter: self.unlimited_likes_filter,
            min_distance_km_filter: self.min_distance_km_filter,
            max_distance_km_filter: self.max_distance_km_filter,
            profile_created_filter: self.profile_created_filter,
            profile_edited_filter: self.profile_edited_filter,
            profile_text_min_characters_filter: self.profile_text_min_characters_filter,
            profile_text_max_characters_filter: self.profile_text_max_characters_filter,
            profile_verification_status_filter: self.profile_verification_status_filter,
            random_profile_order: self.random_profile_order,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileFiltersUpdateValidated {
    pub attribute_filters: Vec<ProfileAttributeFilterValueUpdate>,
    pub last_seen_time_filter: Option<LastSeenTimeFilter>,
    pub unlimited_likes_filter: Option<bool>,
    pub min_distance_km_filter: Option<MinDistanceKm>,
    pub max_distance_km_filter: Option<MaxDistanceKm>,
    pub profile_created_filter: Option<ProfileCreatedTimeFilter>,
    pub profile_edited_filter: Option<ProfileEditedTimeFilter>,
    pub profile_text_min_characters_filter: Option<ProfileTextMinCharactersFilter>,
    pub profile_text_max_characters_filter: Option<ProfileTextMaxCharactersFilter>,
    pub profile_verification_status_filter: Option<ProfileVerificationStatusFilter>,
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[schema(default = json!([]))]
    pub wanted: Vec<u32>,
    /// Same as [Self::wanted] but for unwanted values.
    ///
    /// The unwanted values are checked always with AND operator.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[schema(default = json!([]))]
    pub unwanted: Vec<u32>,
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
pub struct GetProfileFilters {
    pub attribute_filters: Vec<ProfileAttributeFilterValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub last_seen_time_filter: Option<LastSeenTimeFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub unlimited_likes_filter: Option<bool>,
    /// Show profiles starting this far from current location. The value
    /// is in kilometers.
    ///
    /// The value must be `None`, 1 or greater number.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub min_distance_km_filter: Option<MinDistanceKm>,
    /// Show profiles until this far from current location. The value
    /// is in kilometers.
    ///
    /// The value must be `None`, 1 or greater number.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub max_distance_km_filter: Option<MaxDistanceKm>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub profile_created_filter: Option<ProfileCreatedTimeFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub profile_edited_filter: Option<ProfileEditedTimeFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub profile_text_min_characters_filter: Option<ProfileTextMinCharactersFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub profile_text_max_characters_filter: Option<ProfileTextMaxCharactersFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub profile_verification_status_filter: Option<ProfileVerificationStatusFilter>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    /// Randomize iterator starting position within the profile index area which
    /// current position and [Self::max_distance_km] defines.
    pub random_profile_order: bool,
}
