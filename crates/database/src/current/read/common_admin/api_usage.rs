use std::collections::HashMap;

use crate::{DieselDatabaseError, IntoDatabaseError};
use diesel::prelude::*;
use error_stack::{Result, ResultExt};
use model::{AccountIdInternal, ApiUsageCount, ApiUsageStatistics, GetApiUsageStatisticsResult, GetApiUsageStatisticsSettings, UnixTime};

use crate::define_current_read_commands;

// TODO(prod): Rename start_time and end_time to max_time and min_time.

define_current_read_commands!(CurrentReadAccountAdminApiUsage);

impl CurrentReadAccountAdminApiUsage<'_> {
    pub fn api_usage_statistics(
        &mut self,
        account: AccountIdInternal,
        settings: GetApiUsageStatisticsSettings,
    ) -> Result<GetApiUsageStatisticsResult, DieselDatabaseError> {
        let max_time = settings.start_time.unwrap_or(UnixTime::new(i64::MAX));
        let min_time = settings.end_time.unwrap_or(UnixTime::new(0));

        let values: Vec<(UnixTime, i64, i64)> = {
            use crate::schema::{
                api_usage_statistics_metric_value::dsl::*,
                api_usage_statistics_save_time,
            };

            api_usage_statistics_metric_value
                .inner_join(api_usage_statistics_save_time::table.on(time_id.eq(api_usage_statistics_save_time::id)))
                .filter(api_usage_statistics_save_time::unix_time.le(max_time))
                .filter(api_usage_statistics_save_time::unix_time.ge(min_time))
                .filter(account_id.eq(account.as_db_id()))
                .select((
                    api_usage_statistics_save_time::unix_time,
                    metric_id,
                    metric_value,
                ))
                .order((metric_id.asc(), api_usage_statistics_save_time::unix_time.desc()))
                .load(self.conn())
                .change_context(DieselDatabaseError::Execute)?
        };

        let mut data = HashMap::<i64, ApiUsageStatistics>::new();
        for (time, m_id, m_value) in values {
            let v = ApiUsageCount {
                t: time,
                c: m_value,
            };
            if let Some(statistics) = data.get_mut(&m_id) {
                statistics.values.push(v);
            } else {
                use crate::schema::api_usage_statistics_metric_name::dsl::*;
                let name: String = api_usage_statistics_metric_name
                    .find(m_id)
                    .select(metric_name)
                    .first(self.conn())
                    .into_db_error(())?;
                data.insert(m_id, ApiUsageStatistics { name, values: vec![v] });
            }
        }

        Ok(GetApiUsageStatisticsResult { values: data.into_values().collect::<Vec<_>>() })
    }
}
