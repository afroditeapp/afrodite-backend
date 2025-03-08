
use diesel::{sql_query, sql_types::Text, RunQueryDsl};
use simple_backend_database::diesel_db::DieselDatabaseError;
use error_stack::Result;

use crate::{define_current_read_commands, IntoDatabaseError};

define_current_read_commands!(HistoryReadCommon);

impl HistoryReadCommon<'_> {
    pub fn backup_history_database(&mut self, file_name: String) -> Result<(), DieselDatabaseError> {
        sql_query("VACUUM INTO ?")
            .bind::<Text, _>(file_name)
            .execute(self.conn())
            .into_db_error(())?;
        Ok(())
    }
}
