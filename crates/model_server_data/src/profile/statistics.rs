use model::{ProfileAge, UnixTime};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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
    pub fn is_default_statistics(&self) -> bool {
        *self == Self::default()
    }
}

impl Default for StatisticsProfileVisibility {
    fn default() -> Self {
        Self::Public
    }
}

#[derive(Debug, Clone, Default)]
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
        if let Some(c) = v.get_mut::<usize>(i.into()) {
            *c += 1;
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProfileStatisticsInternal {
    pub generation_time: UnixTime,
    pub age_counts: ProfileAgeCounts,
    pub account_count: i64,
    pub account_count_bots_excluded: i64,
    pub public_profile_counts: PublicProfileCounts,
    pub connection_statistics: ConnectionStatistics,
}

impl ProfileStatisticsInternal {
    pub fn new(
        generation_time: UnixTime,
        age_counts: ProfileAgeCounts,
        account_count: i64,
        account_count_bots_excluded: i64,
        public_profile_counts: PublicProfileCounts,
        connection_statistics: ConnectionStatistics,
    ) -> Self {
        Self {
            generation_time,
            age_counts,
            account_count,
            account_count_bots_excluded,
            public_profile_counts,
            connection_statistics,
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct GetProfileStatisticsResult {
    pub generation_time: UnixTime,
    pub account_count_bots_excluded: i64,
    pub age_counts: ProfileAgeCounts,
    pub connection_statistics: ConnectionStatistics,
}

impl GetProfileStatisticsResult {
    fn new(
        generation_time: UnixTime,
        account_count_bots_excluded: i64,
        age_counts: ProfileAgeCounts,
        connection_statistics: ConnectionStatistics,
    ) -> Self {
        Self {
            generation_time,
            account_count_bots_excluded,
            age_counts,
            connection_statistics,
        }
    }
}

impl From<ProfileStatisticsInternal> for GetProfileStatisticsResult {
    fn from(value: ProfileStatisticsInternal) -> Self {
        Self::new(
            value.generation_time,
            value.account_count_bots_excluded,
            value.age_counts,
            value.connection_statistics,
        )
    }
}

/// WebSocket connection statistics for 24 hours.
///
/// All lists contain 24 values starting from UTC time 00:00.
///
/// The data points are averages from available measurements.
///
/// Bots are not included in this data.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, ToSchema)]
pub struct ConnectionStatistics {
    pub all: Vec<u32>,
    pub men: Vec<u32>,
    pub women: Vec<u32>,
    pub nonbinaries: Vec<u32>,
}
