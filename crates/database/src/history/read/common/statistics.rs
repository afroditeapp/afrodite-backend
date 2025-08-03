use std::collections::HashMap;

use diesel::prelude::*;
use error_stack::{Result, ResultExt};
use model::{
    GetIpCountryStatisticsResult, GetIpCountryStatisticsSettings, IpCountryStatistics,
    IpCountryStatisticsType, IpCountryStatisticsValue, UnixTime,
};
use simple_backend_database::diesel_db::DieselDatabaseError;

use crate::define_history_read_commands;

define_history_read_commands!(HistoryReadCommonStatistics);

impl HistoryReadCommonStatistics<'_> {
    pub fn ip_country_statistics(
        &mut self,
        settings: GetIpCountryStatisticsSettings,
    ) -> Result<GetIpCountryStatisticsResult, DieselDatabaseError> {
        let max_time = settings.max_time.unwrap_or(UnixTime::new(i64::MAX));
        let min_time = settings.min_time.unwrap_or(UnixTime::new(0));

        let values: Vec<(UnixTime, String, i64)> = match settings.statistics_type {
            IpCountryStatisticsType::NewTcpConnections => {
                use crate::schema::{
                    history_common_statistics_save_time,
                    history_ip_country_statistics_country_name,
                    history_ip_country_statistics_new_tcp_connections::dsl::*,
                };
                history_ip_country_statistics_new_tcp_connections
                    .inner_join(
                        history_common_statistics_save_time::table
                            .on(time_id.eq(history_common_statistics_save_time::id)),
                    )
                    .inner_join(
                        history_ip_country_statistics_country_name::table
                            .on(country_id.eq(history_ip_country_statistics_country_name::id)),
                    )
                    .filter(history_common_statistics_save_time::unix_time.le(max_time))
                    .filter(history_common_statistics_save_time::unix_time.ge(min_time))
                    .select((
                        history_common_statistics_save_time::unix_time,
                        history_ip_country_statistics_country_name::country_name,
                        new_tcp_connections,
                    ))
                    .order((history_common_statistics_save_time::unix_time.desc(),))
                    .load(self.conn())
                    .change_context(DieselDatabaseError::Execute)?
            }
            IpCountryStatisticsType::NewHttpRequests => {
                use crate::schema::{
                    history_common_statistics_save_time,
                    history_ip_country_statistics_country_name,
                    history_ip_country_statistics_new_http_requests::dsl::*,
                };
                history_ip_country_statistics_new_http_requests
                    .inner_join(
                        history_common_statistics_save_time::table
                            .on(time_id.eq(history_common_statistics_save_time::id)),
                    )
                    .inner_join(
                        history_ip_country_statistics_country_name::table
                            .on(country_id.eq(history_ip_country_statistics_country_name::id)),
                    )
                    .filter(history_common_statistics_save_time::unix_time.le(max_time))
                    .filter(history_common_statistics_save_time::unix_time.ge(min_time))
                    .select((
                        history_common_statistics_save_time::unix_time,
                        history_ip_country_statistics_country_name::country_name,
                        new_http_requests,
                    ))
                    .order((history_common_statistics_save_time::unix_time.desc(),))
                    .load(self.conn())
                    .change_context(DieselDatabaseError::Execute)?
            }
        };

        let mut data = HashMap::<String, IpCountryStatistics>::new();
        for (time, country, count) in values {
            let v = IpCountryStatisticsValue { t: time, c: count };
            if let Some(statistics) = data.get_mut(&country) {
                statistics.values.push(v);
            } else {
                data.insert(
                    country.clone(),
                    IpCountryStatistics {
                        country,
                        values: vec![v],
                    },
                );
            }
        }

        Ok(GetIpCountryStatisticsResult {
            values: data.into_values().collect::<Vec<_>>(),
        })
    }
}
