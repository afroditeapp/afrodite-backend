use database::{DieselDatabaseError, define_current_write_commands};
use diesel::{ExpressionMethods, insert_into, prelude::*, update};
use error_stack::Result;
use model::{AccountIdInternal, SyncVersion, UnixTime};

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteChatLimits);

impl CurrentWriteChatLimits<'_> {
    pub fn insert_daily_likes_left(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::daily_likes_left::dsl::*;

        insert_into(daily_likes_left)
            .values((account_id.eq(id.as_db_id()),))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn reset_daily_likes_left(
        &mut self,
        id: AccountIdInternal,
        likes_left_value: i16,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::daily_likes_left::dsl::*;

        update(daily_likes_left)
            .filter(account_id.eq(id.as_db_id()))
            .set((
                likes_left.eq(likes_left_value),
                latest_limit_reset_unix_time.eq(UnixTime::current_time()),
            ))
            .execute(self.conn())
            .into_db_error(id)?;

        self.increment_daily_likes_sync_version(id)?;

        Ok(())
    }

    pub fn decrement_daily_likes_left(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::daily_likes_left::dsl::*;

        update(daily_likes_left)
            .filter(account_id.eq(id.as_db_id()))
            .filter(likes_left.gt(0))
            .set(likes_left.eq(likes_left - 1))
            .execute(self.conn())
            .into_db_error(id)?;

        self.increment_daily_likes_sync_version(id)?;

        Ok(())
    }

    pub fn reset_daily_likes_left_sync_version(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::daily_likes_left::dsl::*;

        update(daily_likes_left)
            .filter(account_id.eq(id.as_db_id()))
            .set(sync_version.eq(0))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn increment_daily_likes_sync_version(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::daily_likes_left::dsl::*;

        update(daily_likes_left)
            .filter(account_id.eq(id.as_db_id()))
            .filter(sync_version.lt(SyncVersion::MAX_VALUE))
            .set(sync_version.eq(sync_version + 1))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
