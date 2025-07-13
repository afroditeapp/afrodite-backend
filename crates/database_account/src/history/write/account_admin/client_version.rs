use std::collections::HashMap;

use database::{
    DieselDatabaseError, define_history_write_commands,
    history::write::GetDbHistoryWriteCommandsCommon,
};
use diesel::{insert_into, prelude::*};
use error_stack::Result;
use model::{ClientVersion, UnixTime};

use crate::IntoDatabaseError;

define_history_write_commands!(HistoryWriteAccountClientVersion);

impl HistoryWriteAccountClientVersion<'_> {
    pub fn save_client_version_statistics(
        &mut self,
        statistics: HashMap<ClientVersion, i64>,
    ) -> Result<(), DieselDatabaseError> {
        let current_time = UnixTime::current_time();
        let mut time_id_value = None;

        for (k, v) in statistics {
            if v <= 0 {
                continue;
            }

            let time_id_value = if let Some(id) = time_id_value {
                id
            } else {
                let id = self
                    .write()
                    .common_history()
                    .get_or_create_save_time_id(current_time)?;
                time_id_value = Some(id);
                id
            };

            let version_id_value: i64 = {
                use crate::schema::history_client_version_statistics_version_number::dsl::*;

                insert_into(history_client_version_statistics_version_number)
                    .values((
                        major.eq(Into::<i64>::into(k.major)),
                        minor.eq(Into::<i64>::into(k.minor)),
                        patch.eq(Into::<i64>::into(k.patch)),
                    ))
                    .on_conflict((major, minor, patch))
                    .do_update()
                    .set(major.eq(major))
                    .returning(id)
                    .get_result(self.conn())
                    .into_db_error(())?
            };

            {
                use crate::schema::history_client_version_statistics::dsl::*;

                insert_into(history_client_version_statistics)
                    .values((
                        time_id.eq(time_id_value),
                        version_id.eq(version_id_value),
                        count.eq(v),
                    ))
                    .execute(self.conn())
                    .into_db_error(())?;
            }
        }

        Ok(())
    }
}
