use serde::{Deserialize, Serialize};
use simple_backend_model::UnixTime;
use utoipa::{IntoParams, ToSchema};

use super::ProfileAge;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, IntoParams)]
pub struct GetProfileStatisticsParams {
    /// Control which profiles are included in
    /// [GetProfileStatisticsResult::profile_ages]
    /// by profile visibility.
    ///
    /// Non default value is only for admins.
    #[serde(default, skip_serializing_if = "StatisticsProfileVisibility::is_default_statistics")]
    #[param(default = StatisticsProfileVisibility::default)]
    pub profile_visibility: StatisticsProfileVisibility,
    /// Non default value is only for admins.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[param(default = false)]
    pub generate_new_statistics: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetProfileStatisticsResult {
    pub generation_time: UnixTime,
    pub profile_ages: Vec<ProfileAgesPage>,
    pub account_count: i64,
    pub public_profile_counts: PublicProfileCounts,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum StatisticsGender {
    Man,
    Woman,
    NonBinary,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProfileAgesPage {
    pub gender: StatisticsGender,
    pub start_age: i64,
    /// First item is count of profiles with age [Self::start_age] and
    /// the next is the age incremented by one and so on.
    pub profile_counts: Vec<i64>,
}

impl ProfileAgesPage {
    pub fn empty(gender: StatisticsGender) -> Self {
        let available_age_value_count = ProfileAge::MAX_AGE - ProfileAge::MIN_AGE + 1;

        Self {
            gender,
            start_age: ProfileAge::MIN_AGE.into(),
            profile_counts: vec![0; available_age_value_count.into()],
        }
    }

    pub fn increment_age(&mut self, age: u8) {
        let i = age - ProfileAge::MIN_AGE;
        if let Some(c) = self.profile_counts.get_mut::<usize>(i.into())  {
            *c += 1;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub enum StatisticsProfileVisibility {
    Public,
    /// Includes [crate::ProfileVisibility::PendingPublic]
    Private,
    All,
}

impl StatisticsProfileVisibility {
    fn is_default_statistics(&self) -> bool {
        *self == Self::default()
    }
}

impl Default for StatisticsProfileVisibility {
    fn default() -> Self {
        Self::Public
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct PublicProfileCounts {
    pub man: i64,
    pub woman: i64,
    pub non_binary: i64,
}
