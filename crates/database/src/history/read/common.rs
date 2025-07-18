use diesel::{RunQueryDsl, sql_query, sql_types::Text};
use error_stack::Result;
use simple_backend_database::diesel_db::DieselDatabaseError;

use crate::{IntoDatabaseError, define_history_read_commands};

define_history_read_commands!(HistoryReadCommon);

impl HistoryReadCommon<'_> {
    pub fn backup_history_database(
        &mut self,
        file_name: String,
    ) -> Result<(), DieselDatabaseError> {
        sql_query("VACUUM INTO ?")
            .bind::<Text, _>(file_name)
            .execute(self.conn())
            .into_db_error(())?;
        Ok(())
    }
}
