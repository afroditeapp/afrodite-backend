
use std::collections::HashMap;
use diesel::{insert_into, query_dsl::methods::{FindDsl, SelectDsl}, ExpressionMethods, OptionalExtension, RunQueryDsl};
use model::{AccountIdDb, ApiUsage, UnixTime};
use simple_backend_database::diesel_db::DieselDatabaseError;
use error_stack::Result;

use crate::{define_current_write_commands, IntoDatabaseError};

define_current_write_commands!(CurrentWriteCommonApiUsage);

// TODO(prod): Change other statistics to save time value lazily

impl CurrentWriteCommonApiUsage<'_> {
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

                let time_id_value = if let Some(time_id_value) = time_id_value{
                    time_id_value
                } else {
                    use model::schema::api_usage_statistics_save_time::dsl::*;
                    let value: i64 = insert_into(api_usage_statistics_save_time)
                        .values((
                            unix_time.eq(current_time),
                        ))
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
                        .values((
                            metric_name.eq(&name),
                        ))
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
}
