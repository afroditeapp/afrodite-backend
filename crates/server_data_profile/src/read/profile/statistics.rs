use std::{collections::HashMap, sync::Arc};

use model_profile::{
    ConnectionStatistics, ProfileAgeCounts, ProfileStatisticsInternal, PublicProfileCounts, StatisticsGender, StatisticsProfileVisibility, UnixTime
};
use server_data::{define_cmd_wrapper_read, result::Result, DataError};
use simple_backend::perf::{websocket, PerfMetricsManagerData};
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
                    public_profile_counts.man += 1;
                } else if groups.is_woman() {
                    public_profile_counts.woman += 1;
                } else if groups.is_non_binary() {
                    public_profile_counts.non_binary += 1;
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
                age_counts.increment_age(StatisticsGender::NonBinary, p.profile_internal().age.value());
            }
        })
        .await?;

        let history = perf_data.get_history_raw(false).await;

        Ok(ProfileStatisticsInternal {
            generation_time,
            age_counts,
            account_count,
            account_count_bots_excluded,
            online_account_count_bots_excluded: websocket::Connections::connection_count().into(),
            public_profile_counts,
            connection_statistics: convert_history_to_connection_statistics(history),
        })
    }
}

fn convert_history_to_connection_statistics(data: HashMap<MetricKey, PerfMetricValueArea>) -> ConnectionStatistics {
    let min_time = UnixTime::current_time().ut - 60 * 60 * 24;

    ConnectionStatistics {
        all: areas_to_max_values(data.get(&MetricKey::CONNECTIONS), min_time),
        men: areas_to_max_values(data.get(&MetricKey::CONNECTIONS_MEN), min_time),
        women: areas_to_max_values(data.get(&MetricKey::CONNECTIONS_WOMEN), min_time),
        nonbinaries: areas_to_max_values(data.get(&MetricKey::CONNECTIONS_NONBINARIES), min_time),
    }
}

fn areas_to_max_values(data: Option<&PerfMetricValueArea>, min_time: i64) -> Vec<u32> {
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

            if let Some(values)  = hour_and_values.get_mut(&hour) {
                values.push(*v);
            }
        }
    }

    let mut vec: Vec<(u32, u32)> = hour_and_values
        .into_iter()
        .map(|(k, v)| (k, v.into_iter().max().unwrap_or_default()))
        .collect();
    vec.sort_by_key(|(k, _)| *k);
    vec.into_iter().map(|(_, v)| v).collect()
}
