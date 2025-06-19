use database::{DieselDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use error_stack::{Result, ResultExt};
use model::AccountIdInternal;

define_current_read_commands!(CurrentReadProfileFavorite);

impl CurrentReadProfileFavorite<'_> {
    pub fn favorites(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<Vec<AccountIdInternal>, DieselDatabaseError> {
        use crate::schema::{account_id, favorite_profile};

        let favorites = favorite_profile::table
            .inner_join(
                account_id::table.on(favorite_profile::favorite_account_id.eq(account_id::id)),
            )
            .filter(favorite_profile::account_id.eq(id.as_db_id()))
            .order((
                favorite_profile::unix_time.asc(),
                favorite_profile::favorite_account_id.asc(),
            ))
            .select(AccountIdInternal::as_select())
            .load(self.conn())
            .change_context(DieselDatabaseError::Execute)?;

        Ok(favorites)
    }
}
