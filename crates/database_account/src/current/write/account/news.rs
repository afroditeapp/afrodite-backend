use database::{define_current_write_commands, DieselDatabaseError};
use diesel::{prelude::*, update};
use error_stack::Result;
use model::{AccountIdInternal, NewsSyncVersion, SyncVersion, UnreadNewsCount};

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteAccountNews);

impl CurrentWriteAccountNews<'_> {
    pub fn reset_news_sync_version(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_state::dsl::*;

        update(account_state)
            .filter(account_id.eq(id.as_db_id()))
            .set(news_sync_version.eq(0))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn increment_news_sync_version_for_specific_account(
        &mut self,
        id_value: AccountIdInternal,
    ) -> Result<NewsSyncVersion, DieselDatabaseError> {
        use model::schema::account_state::dsl::*;

        update(account_state)
            .filter(account_id.eq(id_value.as_db_id()))
            .filter(news_sync_version.lt(SyncVersion::MAX_VALUE))
            .set(news_sync_version.eq(news_sync_version + 1))
            .returning(news_sync_version)
            .get_result(self.conn())
            .into_db_error(())
    }

    pub fn reset_news_unread_count(
        &mut self,
        id_value: AccountIdInternal,
    ) -> Result<UnreadNewsCount, DieselDatabaseError> {
        use model::schema::account_state::dsl::*;

        update(account_state)
            .filter(account_id.eq(id_value.as_db_id()))
            .set(unread_news_count.eq(0))
            .returning(unread_news_count)
            .get_result(self.conn())
            .into_db_error(())
    }
}
