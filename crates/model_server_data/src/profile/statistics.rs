use model::{ProfileAge, UnixTime};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq)]
pub enum StatisticsGender {
    Man,
    Woman,
    NonBinary,
}

/// Deprecated. Profile statistics are now always calculated
/// for public and private profiles and are returned separately
/// in [GetProfileStatisticsResult].
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, ToSchema, Default)]
#[schema(deprecated)]
pub enum StatisticsProfileVisibility {
    #[default]
    Public,
    Private,
    All,
}

#[derive(Debug, Clone, Default)]
pub struct PublicProfileCounts {
    pub men: i64,
    pub women: i64,
    pub nonbinaries: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct ProfileAgeCounts {
    /// Age for first count
    pub start_age: i16,
    pub men: Vec<i64>,
    pub women: Vec<i64>,
    pub nonbinaries: Vec<i64>,
}

impl ProfileAgeCounts {
    const AVAILABLE_AGE_VALUE_COUNT: u8 = ProfileAge::MAX_AGE - ProfileAge::MIN_AGE + 1;

    pub fn empty() -> Self {
        let empty = vec![0; Self::AVAILABLE_AGE_VALUE_COUNT.into()];
        Self {
            start_age: ProfileAge::MIN_AGE.into(),
            men: empty.clone(),
            women: empty.clone(),
            nonbinaries: empty,
        }
    }

    pub fn increment_age(&mut self, gender: StatisticsGender, age: u8) {
        let i = age - ProfileAge::MIN_AGE;
        let v = match gender {
            StatisticsGender::Man => &mut self.men,
            StatisticsGender::Woman => &mut self.women,
            StatisticsGender::NonBinary => &mut self.nonbinaries,
        };
        if let Some(c) = v.get_mut::<usize>(i.into()) {
            *c += 1;
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProfileStatisticsInternal {
    pub generation_time: UnixTime,
    /// Age counts for public profiles, excluding bots.
    pub age_counts: ProfileAgeCounts,
    /// Age counts for private profiles which have completed the initial
    /// setup, excluding bots.
    pub private_age_counts: ProfileAgeCounts,
    pub account_count: i64,
    pub account_count_bots_excluded: i64,
    pub online_account_count_bots_excluded: i64,
    pub public_profile_counts: PublicProfileCounts,
    pub connections_min: ConnectionStatistics,
    pub connections_max: ConnectionStatistics,
    pub connections_average: ConnectionStatistics,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct GetProfileStatisticsResult {
    pub generation_time: UnixTime,
    pub account_count_bots_excluded: i64,
    pub online_account_count_bots_excluded: i64,
    /// Age counts for public profiles, excluding bots.
    pub age_counts: ProfileAgeCounts,
    /// Age counts for private profiles which have completed the initial
    /// setup, excluding bots.
    ///
    /// This is always `Some` currently. The value is optional so that in future,
    /// if more privacy is needed, private statistics can be omitted
    /// and clients can default to public statistics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_age_counts: Option<ProfileAgeCounts>,
    /// Min WebSocket connections per hour
    pub connections_min: ConnectionStatistics,
    /// Max WebSocket connections per hour
    pub connections_max: ConnectionStatistics,
    /// Average WebSocket connections per hour
    pub connections_average: ConnectionStatistics,
}

impl From<ProfileStatisticsInternal> for GetProfileStatisticsResult {
    fn from(value: ProfileStatisticsInternal) -> Self {
        Self {
            generation_time: value.generation_time,
            account_count_bots_excluded: value.account_count_bots_excluded,
            online_account_count_bots_excluded: value.online_account_count_bots_excluded,
            age_counts: value.age_counts,
            private_age_counts: Some(value.private_age_counts),
            connections_min: value.connections_min,
            connections_max: value.connections_max,
            connections_average: value.connections_average,
        }
    }
}

/// WebSocket connection statistics for 24 hours.
///
/// All lists contain 24 values starting from UTC time 00:00.
///
/// Bots are not included in this data.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, ToSchema)]
pub struct ConnectionStatistics {
    pub all: Vec<u32>,
    pub men: Vec<u32>,
    pub women: Vec<u32>,
    pub nonbinaries: Vec<u32>,
}
