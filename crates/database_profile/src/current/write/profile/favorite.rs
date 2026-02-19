use database::{DieselDatabaseError, define_current_write_commands};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, delete, insert_into};
use error_stack::Result;
use model::AccountIdInternal;
use model_profile::AddFavoriteProfileResult;
use simple_backend_utils::current_unix_time;

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteProfileFavorite);

const REMAINING_FAVORITES_WARNING_THRESHOLD: u16 = 5;

impl CurrentWriteProfileFavorite<'_> {
    pub fn insert_favorite_profile(
        &mut self,
        id: AccountIdInternal,
        favorite: AccountIdInternal,
        max_count: u16,
    ) -> Result<AddFavoriteProfileResult, DieselDatabaseError> {
        use model::schema::favorite_profile::dsl::*;

        let current_count: i64 = favorite_profile
            .filter(account_id.eq(id.as_db_id()))
            .count()
            .get_result(self.conn())
            .into_db_error(id)?;

        if current_count >= max_count as i64 {
            return Ok(AddFavoriteProfileResult::too_many_favorites());
        }

        let time = current_unix_time();

        insert_into(favorite_profile)
            .values((
                account_id.eq(id.as_db_id()),
                favorite_account_id.eq(favorite.as_db_id()),
                unix_time.eq(time),
            ))
            .execute(self.conn())
            .into_db_error(id)?;

        let remaining = (max_count as i64).saturating_sub(current_count + 1) as u16;
        if remaining <= REMAINING_FAVORITES_WARNING_THRESHOLD {
            Ok(AddFavoriteProfileResult::ok_with_remaining_count(remaining))
        } else {
            Ok(AddFavoriteProfileResult::ok())
        }
    }

    pub fn remove_favorite_profile(
        &mut self,
        id: AccountIdInternal,
        favorite: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::favorite_profile::dsl::*;

        delete(favorite_profile)
            .filter(account_id.eq(id.as_db_id()))
            .filter(favorite_account_id.eq(favorite.as_db_id()))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }
}
