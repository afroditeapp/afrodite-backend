use database::{DieselDatabaseError, define_current_read_commands};
use diesel::{prelude::*, update};
use error_stack::Result;
use model::{AccountIdInternal, UnixTime};
use model_account::{AccountBanReasonCategory, AccountBanReasonDetails};

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentWriteAccountBanAdmin);

impl CurrentWriteAccountBanAdmin<'_> {
    pub fn set_banned_state(
        &mut self,
        id: AccountIdInternal,
        admin_id: Option<AccountIdInternal>,
        banned_until: Option<UnixTime>,
        reason_category: Option<AccountBanReasonCategory>,
        reason_details: Option<AccountBanReasonDetails>,
    ) -> Result<(), DieselDatabaseError> {
        use crate::schema::account_state::dsl::*;

        let current_time = UnixTime::current_time();

        update(account_state)
            .filter(account_id.eq(id.as_db_id()))
            .set((
                account_banned_until_unix_time.eq(banned_until),
                account_banned_state_change_unix_time.eq(current_time),
                account_banned_admin_account_id.eq(admin_id.map(|v| v.into_db_id())),
                account_banned_reason_category.eq(reason_category),
                account_banned_reason_details.eq(reason_details),
            ))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
