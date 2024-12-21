use database::{define_current_read_commands, DieselDatabaseError};
use diesel::prelude::*;
use error_stack::Result;
use model::AccountIdInternal;
use model_account::GetAccountBanTimeResult;

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadAccountBan);

impl CurrentReadAccountBan<'_> {
    pub fn account_ban_time(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<GetAccountBanTimeResult, DieselDatabaseError> {
        use crate::schema::account_state::dsl::*;

        account_state
            .filter(account_id.eq(id.as_db_id()))
            .select(account_banned_until_unix_time)
            .first(self.conn())
            .into_db_error(id)
            .map(|banned_until| GetAccountBanTimeResult { banned_until })
    }
}
