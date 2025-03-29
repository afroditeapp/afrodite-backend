use std::{collections::HashMap, sync::Arc};

use crate::{DieselDatabaseError, IntoDatabaseError};
use config::Config;
use diesel::prelude::*;
use error_stack::{Result, ResultExt};
use model::{AccountIdInternal, ApiUsageCount, ApiUsageStatistics, GetApiUsageStatisticsResult, GetApiUsageStatisticsSettings, GetIpAddressStatisticsResult, IpAddressInfo, IpAddressInfoInternal, UnixTime};

use crate::define_current_read_commands;

define_current_read_commands!(CurrentReadAccountAdminStatistics);

impl CurrentReadAccountAdminStatistics<'_> {
    pub fn api_usage_statistics(
        &mut self,
        account: AccountIdInternal,
        settings: GetApiUsageStatisticsSettings,
    ) -> Result<GetApiUsageStatisticsResult, DieselDatabaseError> {
        let max_time = settings.max_time.unwrap_or(UnixTime::new(i64::MAX));
        let min_time = settings.min_time.unwrap_or(UnixTime::new(0));

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

    pub fn ip_address_statistics(
        &mut self,
        account: AccountIdInternal,
        config: Arc<Config>,
    ) -> Result<GetIpAddressStatisticsResult, DieselDatabaseError> {
        let values: Vec<IpAddressInfoInternal> = {
            use crate::schema::ip_address_usage_statistics::dsl::*;

            ip_address_usage_statistics
                .filter(account_id.eq(account.as_db_id()))
                .select(IpAddressInfoInternal::as_select())
                .order(latest_usage_unix_time.desc())
                .load(self.conn())
                .change_context(DieselDatabaseError::Execute)?
        };

        Ok(GetIpAddressStatisticsResult {
            values: values
                .into_iter()
                .map(|v| {
                    let ip_address = v.ip_address.to_ip_addr();
                    let mut lists = vec![];
                    for l in config.simple_backend().ip_lists() {
                        if l.contains(ip_address) {
                            lists.push(l.name().to_string());
                        }
                    }
                    IpAddressInfo {
                        a: ip_address.to_string(),
                        c: v.usage_count,
                        f: v.first_usage_unix_time,
                        l: v.latest_usage_unix_time,
                        lists,
                    }
                })
                .collect::<Vec<_>>()
        })
    }
}
