
use std::collections::HashMap;
use diesel::{insert_into, ExpressionMethods, RunQueryDsl};
use model::UnixTime;
use simple_backend_database::diesel_db::DieselDatabaseError;
use simple_backend_model::{MetricKey, PerfMetricValueArea};
use error_stack::Result;

use crate::{define_current_write_commands, IntoDatabaseError};

define_current_write_commands!(HistoryWriteCommon);

impl HistoryWriteCommon<'_> {
    pub fn write_perf_data(
        mut self,
        data: HashMap<MetricKey, PerfMetricValueArea>,
    ) -> Result<(), DieselDatabaseError> {
        let current_time = UnixTime::current_time();

        let time_id_value: i64 = {
            use model::schema::history_performance_statistics_save_time::dsl::*;
            insert_into(history_performance_statistics_save_time)
                .values((
                    unix_time.eq(current_time),
                ))
                .on_conflict(unix_time)
                .do_nothing()
                .returning(id)
                .get_result(self.conn())
                .into_db_error(())?
        };

        for (k, v) in data.iter() {
            let value = v.average() as i64;
            if value == 0 {
                continue;
            }

            let name = k.to_name();
            let metric_name_id: i64 = {
                use model::schema::history_performance_statistics_metric_name::dsl::*;
                insert_into(history_performance_statistics_metric_name)
                    .values((
                        metric_name.eq(name),
                    ))
                    .on_conflict(metric_name)
                    .do_nothing()
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
}
