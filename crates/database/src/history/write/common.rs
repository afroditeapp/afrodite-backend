use diesel::{insert_into, ExpressionMethods, RunQueryDsl};
use error_stack::Result;
use model::AccountIdInternal;
use simple_backend_database::diesel_db::DieselDatabaseError;

use crate::{define_current_write_commands, IntoDatabaseError};

define_current_write_commands!(HistoryWriteCommon);

impl HistoryWriteCommon<'_> {
    pub fn insert_account_id(
        mut self,
        account_id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::history_account_id::dsl::*;

        insert_into(history_account_id)
            .values((
                id.eq(account_id.as_db_id()),
                uuid.eq(account_id.as_id()),
            ))
            .execute(self.conn())
            .into_db_error(account_id)?;

        Ok(())
    }
}
