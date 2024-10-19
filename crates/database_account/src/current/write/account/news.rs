use database::{define_current_write_commands, ConnectionProvider, DieselDatabaseError};
use diesel::{prelude::*, update};
use error_stack::Result;
use model::AccountIdInternal;

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteAccountNews, CurrentSyncWriteAccountNews);

impl<C: ConnectionProvider> CurrentSyncWriteAccountNews<C> {
    pub fn reset_news_count_sync_version(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_state::dsl::*;

        update(account_state)
            .filter(account_id.eq(id.as_db_id()))
            .set(news_count_sync_version.eq(0))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
