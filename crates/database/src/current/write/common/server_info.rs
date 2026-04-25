use diesel::{insert_into, prelude::*, update};
use error_stack::Result;
use model::UnixTime;
use simple_backend_utils::db::MyRunQueryDsl;

use crate::{DieselDatabaseError, IntoDatabaseError, define_current_write_commands};

define_current_write_commands!(CurrentWriteCommonServerInfo);

impl CurrentWriteCommonServerInfo<'_> {
    pub fn upsert_server_info(
        &mut self,
        version: &str,
        server_start_time: UnixTime,
    ) -> Result<(), DieselDatabaseError> {
        self.upsert_server_version_if_changed(version)?;
        self.upsert_server_start_time(server_start_time)?;

        Ok(())
    }

    fn upsert_server_version_if_changed(
        &mut self,
        version_value: &str,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::info_server_version::dsl::*;

        insert_into(info_server_version)
            .values((row_type.eq(0), version.eq(version_value)))
            .on_conflict(row_type)
            .do_nothing()
            .execute_my_conn(self.conn())
            .into_db_error(())?;

        update(
            info_server_version
                .filter(row_type.eq(0))
                .filter(version.ne(version_value)),
        )
        .set(version.eq(version_value))
        .execute(self.conn())
        .into_db_error(())?;

        Ok(())
    }

    fn upsert_server_start_time(
        &mut self,
        start_time: UnixTime,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::info_server_start_time::dsl::*;

        insert_into(info_server_start_time)
            .values((row_type.eq(0), unix_time.eq(start_time)))
            .on_conflict(row_type)
            .do_update()
            .set(unix_time.eq(start_time))
            .execute_my_conn(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn update_scheduled_tasks_start_time(
        &mut self,
        start_time: UnixTime,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::info_scheduled_tasks_start_time::dsl::*;

        insert_into(info_scheduled_tasks_start_time)
            .values((row_type.eq(0), unix_time.eq(start_time)))
            .on_conflict(row_type)
            .do_update()
            .set(unix_time.eq(start_time))
            .execute_my_conn(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
