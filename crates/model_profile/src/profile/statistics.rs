use model::ProfileAge;
use serde::{Deserialize, Serialize};
use simple_backend_model::UnixTime;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, IntoParams)]
pub struct GetProfileStatisticsParams {
    /// Control which profiles are included in
    /// [GetProfileStatisticsResult::age_counts]
    /// by profile visibility.
    ///
    /// Non default value is only for admins.
    pub profile_visibility: Option<StatisticsProfileVisibility>,
    /// Non default value is only for admins.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[param(default = false)]
    pub generate_new_statistics: bool,
}

impl GetProfileStatisticsParams {
    pub fn contains_admin_settings(&self) -> bool {
        !self.profile_visibility.unwrap_or_default().is_default_statistics() ||
        self.generate_new_statistics
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetProfileStatisticsResult {
    pub generation_time: UnixTime,
    pub age_counts: ProfileAgeCounts,
    pub account_count: i64,
    pub public_profile_counts: PublicProfileCounts,
}

impl GetProfileStatisticsResult {
    pub fn new(
        generation_time: UnixTime,
        age_counts: ProfileAgeCounts,
        account_count: i64,
        public_profile_counts: PublicProfileCounts,
    ) -> Self {
        Self {
            generation_time,
            age_counts,
            account_count,
            public_profile_counts,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatisticsGender {
    Man,
    Woman,
    NonBinary,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, ToSchema)]
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

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct ProfileAgeCounts {
    /// Age for first count
    pub start_age: i64,
    pub man: Vec<i64>,
    pub woman: Vec<i64>,
    pub non_binary: Vec<i64>,
}

impl ProfileAgeCounts {
    const AVAILABLE_AGE_VALUE_COUNT: u8 = ProfileAge::MAX_AGE - ProfileAge::MIN_AGE + 1;

    pub fn empty() -> Self {
        let empty = vec![0; Self::AVAILABLE_AGE_VALUE_COUNT.into()];
        Self {
            start_age: ProfileAge::MIN_AGE.into(),
            man: empty.clone(),
            woman: empty.clone(),
            non_binary: empty,
        }
    }

    pub fn increment_age(&mut self, gender: StatisticsGender, age: u8) {
        let i = age - ProfileAge::MIN_AGE;
        let v = match gender {
            StatisticsGender::Man => &mut self.man,
            StatisticsGender::Woman => &mut self.woman,
            StatisticsGender::NonBinary => &mut self.non_binary,
        };
        if let Some(c) = v.get_mut::<usize>(i.into())  {
            *c += 1;
        }
    }
}
