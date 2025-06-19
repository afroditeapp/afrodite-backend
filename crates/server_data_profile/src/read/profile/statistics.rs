use std::{collections::HashMap, sync::Arc};

use model_profile::{
    ConnectionStatistics, ProfileAgeCounts, ProfileStatisticsInternal, PublicProfileCounts,
    StatisticsGender, StatisticsProfileVisibility, UnixTime,
};
use server_data::{DataError, define_cmd_wrapper_read, result::Result};
use simple_backend::perf::{PerfMetricsManagerData, websocket};
use simple_backend_model::{MetricKey, PerfMetricValueArea, TimeGranularity};

use crate::cache::CacheReadProfile;

define_cmd_wrapper_read!(ReadCommandsProfileStatistics);

impl ReadCommandsProfileStatistics<'_> {
    pub async fn profile_statistics(
        &self,
        profile_visibility: StatisticsProfileVisibility,
        perf_data: Arc<PerfMetricsManagerData>,
    ) -> Result<ProfileStatisticsInternal, DataError> {
        let generation_time = UnixTime::current_time();
        let mut account_count = 0;
        let mut account_count_bots_excluded = 0;
        let mut public_profile_counts = PublicProfileCounts::default();

        let mut age_counts = ProfileAgeCounts::empty();

        self.read_cache_profile_and_common_for_all_accounts(|p, e| {
            account_count += 1;

            if !e.other_shared_state.is_bot_account {
                account_count_bots_excluded += 1;
            }

            let visibility = e.account_state_related_shared_state.profile_visibility();

            let groups = p.state.search_group_flags;
            if visibility.is_currently_public() {
                if groups.is_man() {
                    public_profile_counts.men += 1;
                } else if groups.is_woman() {
                    public_profile_counts.women += 1;
                } else if groups.is_non_binary() {
                    public_profile_counts.nonbinaries += 1;
                }
            }

            match profile_visibility {
                StatisticsProfileVisibility::All => (),
                StatisticsProfileVisibility::Public => {
                    if !visibility.is_currently_public() {
                        return;
                    }
                }
                StatisticsProfileVisibility::Private => {
                    if visibility.is_currently_public() {
                        return;
                    }
                }
            }

            if groups.is_man() {
                age_counts.increment_age(StatisticsGender::Man, p.profile_internal().age.value());
            } else if groups.is_woman() {
                age_counts.increment_age(StatisticsGender::Woman, p.profile_internal().age.value());
            } else if groups.is_non_binary() {
                age_counts.increment_age(
                    StatisticsGender::NonBinary,
                    p.profile_internal().age.value(),
                );
            }
        })
        .await?;

        let history = perf_data.get_history_raw(false).await;
        let statistics_creator = ConnectionStatisticsCreator::new(history);

        Ok(ProfileStatisticsInternal {
            generation_time,
            age_counts,
            account_count,
            account_count_bots_excluded,
            online_account_count_bots_excluded: websocket::Connections::connection_count().into(),
            public_profile_counts,
            connections_min: statistics_creator.to_connection_statistics(|v| v.min),
            connections_max: statistics_creator.to_connection_statistics(|v| v.max),
            connections_average: statistics_creator.to_connection_statistics(|v| v.average),
        })
    }
}

struct ConnectionStatisticsCreator {
    all: Vec<Values>,
    men: Vec<Values>,
    women: Vec<Values>,
    nonbinaries: Vec<Values>,
}

impl ConnectionStatisticsCreator {
    fn new(data: HashMap<MetricKey, PerfMetricValueArea>) -> Self {
        let min_time = UnixTime::current_time().ut - 60 * 60 * 24;
        Self {
            all: areas_to_values(data.get(&MetricKey::CONNECTIONS), min_time),
            men: areas_to_values(data.get(&MetricKey::CONNECTIONS_MEN), min_time),
            women: areas_to_values(data.get(&MetricKey::CONNECTIONS_WOMEN), min_time),
            nonbinaries: areas_to_values(data.get(&MetricKey::CONNECTIONS_NONBINARIES), min_time),
        }
    }

    fn to_connection_statistics(&self, getter: impl Fn(&Values) -> u32) -> ConnectionStatistics {
        ConnectionStatistics {
            all: self.all.iter().map(&getter).collect(),
            men: self.men.iter().map(&getter).collect(),
            women: self.women.iter().map(&getter).collect(),
            nonbinaries: self.nonbinaries.iter().map(&getter).collect(),
        }
    }
}

#[derive(Default)]
struct Values {
    min: u32,
    max: u32,
    average: u32,
}

impl Values {
    fn new(values: Vec<u32>) -> Self {
        if values.is_empty() {
            return Self::default();
        }

        let mut min = u32::MAX;
        let mut max = 0;
        let mut sum: u64 = 0;

        for &v in &values {
            min = min.min(v);
            max = max.max(v);
            sum += Into::<u64>::into(v);
        }

        Self {
            min,
            max,
            average: (sum / values.len() as u64) as u32,
        }
    }
}

fn areas_to_values(data: Option<&PerfMetricValueArea>, min_time: i64) -> Vec<Values> {
    let mut hour_and_values = HashMap::<u32, Vec<u32>>::new();

    for h in 0..=23 {
        hour_and_values.insert(h, vec![]);
    }

    for a in data.into_iter() {
        if a.time_granularity != TimeGranularity::Minutes {
            continue;
        }

        for (i, v) in a.values.iter().enumerate() {
            let time = a.first_time_value.ut + (i * 60) as i64;
            if time <= min_time {
                continue;
            }

            let Some(hour) = UnixTime::new(time).hour() else {
                continue;
            };

            if let Some(values) = hour_and_values.get_mut(&hour) {
                values.push(*v);
            }
        }
    }

    let mut vec: Vec<(u32, Values)> = hour_and_values
        .into_iter()
        .map(|(k, v)| (k, Values::new(v)))
        .collect();
    vec.sort_by_key(|(k, _)| *k);
    vec.into_iter().map(|(_, v)| v).collect()
}
