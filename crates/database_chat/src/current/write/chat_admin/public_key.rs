use database::{DieselDatabaseError, IntoDatabaseError, define_current_write_commands};
use diesel::{prelude::*, update};
use error_stack::Result;
use model::AccountIdInternal;

define_current_write_commands!(CurrentWriteChatAdminPublicKey);

impl CurrentWriteChatAdminPublicKey<'_> {
    pub fn set_max_public_key_count(
        &mut self,
        id: AccountIdInternal,
        count: i64,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::chat_state::dsl::*;

        update(chat_state.find(id.as_db_id()))
            .set((max_public_key_count.eq(count),))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }
}
