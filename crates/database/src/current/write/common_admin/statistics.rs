use std::collections::HashMap;

use diesel::{
    ExpressionMethods, OptionalExtension, RunQueryDsl, insert_into,
    query_dsl::methods::{FindDsl, SelectDsl},
};
use error_stack::Result;
use model::{AccountIdDb, ApiUsage, IpAddressStorage, UnixTime};
use simple_backend_database::diesel_db::DieselDatabaseError;

use crate::{IntoDatabaseError, define_current_write_commands};

define_current_write_commands!(CurrentWriteCommonStatistics);

impl CurrentWriteCommonStatistics<'_> {
    pub fn save_api_usage_data(
        mut self,
        data: HashMap<AccountIdDb, ApiUsage>,
    ) -> Result<(), DieselDatabaseError> {
        let current_time = UnixTime::current_time();

        let mut time_id_value: Option<i64> = None;

        for (k, v) in data.iter() {
            {
                use model::schema::account_id::dsl::*;
                let account_exists: Option<i64> = account_id
                    .find(k)
                    .select(id)
                    .first(self.conn())
                    .optional()
                    .into_db_error(())?;
                if account_exists.is_none() {
                    // Account is removed
                    continue;
                }
            }

            for v in v.values() {
                let value = v.value;
                if value == 0 {
                    continue;
                }

                let time_id_value = if let Some(time_id_value) = time_id_value {
                    time_id_value
                } else {
                    use model::schema::api_usage_statistics_save_time::dsl::*;
                    let value: i64 = insert_into(api_usage_statistics_save_time)
                        .values((unix_time.eq(current_time),))
                        .on_conflict(unix_time)
                        .do_update()
                        .set(unix_time.eq(unix_time))
                        .returning(id)
                        .get_result(self.conn())
                        .into_db_error(())?;
                    time_id_value = Some(value);
                    value
                };

                let name = v.name;
                let metric_name_id: i64 = {
                    use model::schema::api_usage_statistics_metric_name::dsl::*;
                    insert_into(api_usage_statistics_metric_name)
                        .values((metric_name.eq(&name),))
                        .on_conflict(metric_name)
                        .do_update()
                        .set(metric_name.eq(metric_name))
                        .returning(id)
                        .get_result(self.conn())
                        .into_db_error(())?
                };

                {
                    use model::schema::api_usage_statistics_metric_value::dsl::*;
                    insert_into(api_usage_statistics_metric_value)
                        .values((
                            account_id.eq(k),
                            time_id.eq(time_id_value),
                            metric_id.eq(metric_name_id),
                            metric_value.eq(Into::<i64>::into(value)),
                        ))
                        .execute(self.conn())
                        .into_db_error(())?
                };
            }
        }

        Ok(())
    }

    pub fn save_ip_address_data(
        mut self,
        data: HashMap<AccountIdDb, IpAddressStorage>,
    ) -> Result<(), DieselDatabaseError> {
        for (k, v) in data.into_iter() {
            {
                use model::schema::account_id::dsl::*;
                let account_exists: Option<i64> = account_id
                    .find(k)
                    .select(id)
                    .first(self.conn())
                    .optional()
                    .into_db_error(())?;
                if account_exists.is_none() {
                    // Account is removed
                    continue;
                }
            }

            for (ip, info) in v.ips.into_iter() {
                use model::schema::ip_address_usage_statistics::dsl::*;
                insert_into(ip_address_usage_statistics)
                    .values((
                        account_id.eq(k),
                        ip_address.eq(ip),
                        usage_count.eq(info.usage_count()),
                        first_usage_unix_time.eq(info.first_usage()),
                        latest_usage_unix_time.eq(info.latest_usage()),
                    ))
                    .on_conflict((account_id, ip_address))
                    .do_update()
                    .set((
                        usage_count.eq(usage_count + info.usage_count()),
                        latest_usage_unix_time.eq(info.latest_usage()),
                    ))
                    .execute(self.conn())
                    .into_db_error(())?;
            }
        }

        Ok(())
    }
}
