use std::collections::HashMap;

use diesel::{ExpressionMethods, RunQueryDsl, insert_into};
use error_stack::Result;
use model::{StatisticsSaveTimeId, UnixTime};
use simple_backend_database::diesel_db::DieselDatabaseError;
use simple_backend_model::{IpCountry, IpCountryCounters, MetricKey, PerfMetricValueArea};

use crate::{IntoDatabaseError, define_history_write_commands};

define_history_write_commands!(HistoryWriteCommon);

impl HistoryWriteCommon<'_> {
    pub fn get_or_create_save_time_id(
        &mut self,
        time: UnixTime,
    ) -> Result<StatisticsSaveTimeId, DieselDatabaseError> {
        use model::schema::history_common_statistics_save_time::dsl::*;
        insert_into(history_common_statistics_save_time)
            .values(unix_time.eq(time))
            .on_conflict(unix_time)
            .do_update()
            .set(unix_time.eq(unix_time))
            .returning(id)
            .get_result(self.conn())
            .into_db_error(())
    }

    pub fn write_perf_data(
        mut self,
        data: HashMap<MetricKey, PerfMetricValueArea>,
    ) -> Result<(), DieselDatabaseError> {
        let current_time = UnixTime::current_time();

        let mut time_id_value = None;

        for (k, v) in data.iter() {
            let value = v.average() as i64;
            if value == 0 {
                continue;
            }

            let time_id_value = if let Some(id) = time_id_value {
                id
            } else {
                let id = self.get_or_create_save_time_id(current_time)?;
                time_id_value = Some(id);
                id
            };

            let name = k.to_name();
            let metric_name_id: i64 = {
                use model::schema::history_performance_statistics_metric_name::dsl::*;
                insert_into(history_performance_statistics_metric_name)
                    .values((metric_name.eq(&name),))
                    .on_conflict(metric_name)
                    .do_update()
                    .set(metric_name.eq(metric_name))
                    .returning(id)
                    .get_result(self.conn())
                    .into_db_error(())?
            };

            {
                use model::schema::history_performance_statistics_metric_value::dsl::*;
                insert_into(history_performance_statistics_metric_value)
                    .values((
                        time_id.eq(time_id_value),
                        metric_id.eq(metric_name_id),
                        metric_value.eq(value),
                    ))
                    .execute(self.conn())
                    .into_db_error(())?
            };
        }

        Ok(())
    }

    pub fn write_ip_country_data(
        mut self,
        data: HashMap<IpCountry, IpCountryCounters>,
    ) -> Result<(), DieselDatabaseError> {
        let current_time = UnixTime::current_time();

        let mut time_id_value = None;

        for (country, v) in data.iter() {
            let tcp_connections_value = v.tcp_connections();
            let http_requests_value = v.http_requests();
            if tcp_connections_value == 0 && http_requests_value == 0 {
                continue;
            }

            let time_id_value = if let Some(id) = time_id_value {
                id
            } else {
                let id = self.get_or_create_save_time_id(current_time)?;
                time_id_value = Some(id);
                id
            };

            let name = country.0.as_str();
            let country_name_id: i64 = {
                use model::schema::history_ip_country_statistics_country_name::dsl::*;
                insert_into(history_ip_country_statistics_country_name)
                    .values(country_name.eq(name))
                    .on_conflict(country_name)
                    .do_update()
                    .set(country_name.eq(country_name))
                    .returning(id)
                    .get_result(self.conn())
                    .into_db_error(())?
            };

            if tcp_connections_value != 0 {
                use model::schema::history_ip_country_statistics_new_tcp_connections::dsl::*;
                insert_into(history_ip_country_statistics_new_tcp_connections)
                    .values((
                        time_id.eq(time_id_value),
                        country_id.eq(country_name_id),
                        new_tcp_connections.eq(tcp_connections_value),
                    ))
                    .execute(self.conn())
                    .into_db_error(())?;
            };

            if http_requests_value != 0 {
                use model::schema::history_ip_country_statistics_new_http_requests::dsl::*;
                insert_into(history_ip_country_statistics_new_http_requests)
                    .values((
                        time_id.eq(time_id_value),
                        country_id.eq(country_name_id),
                        new_http_requests.eq(http_requests_value),
                    ))
                    .execute(self.conn())
                    .into_db_error(())?;
            };
        }

        Ok(())
    }
}
