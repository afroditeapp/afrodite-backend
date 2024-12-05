use database::{define_history_write_commands, DieselDatabaseError};
use diesel::{insert_into, prelude::*};
use error_stack::Result;
use model::AccountId;

use crate::IntoDatabaseError;

define_history_write_commands!(HistoryWriteAccount);

impl HistoryWriteAccount<'_> {
    pub fn new_unique_account_id(
        &mut self,
    ) -> Result<AccountId, DieselDatabaseError> {
        use model::schema::history_used_account_ids::dsl::*;

        let random_aid = AccountId::new_random();

        insert_into(history_used_account_ids)
            .values((
                uuid.eq(random_aid),
            ))
            .execute(self.conn())
            .into_db_error(random_aid)?;

        Ok(random_aid)
    }
}
