use database::{DieselDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use model::AccountIdInternal;
use model_account::{AppleAccountId, GoogleAccountId, SignInWithHistoryEntry, SignInWithInfoRaw};
use simple_backend_utils::Result;

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadAccountSignInWith);

impl CurrentReadAccountSignInWith<'_> {
    pub fn apple_account_id_to_account_id(
        &mut self,
        apple_id: AppleAccountId,
    ) -> Result<Option<AccountIdInternal>, DieselDatabaseError> {
        use crate::schema::{account_id, sign_in_with_info};

        sign_in_with_info::table
            .inner_join(account_id::table)
            .filter(sign_in_with_info::apple_account_id.eq(&apple_id))
            .select(AccountIdInternal::as_select())
            .first(self.conn())
            .optional()
            .into_db_error(apple_id)
    }

    pub fn google_account_id_to_account_id(
        &mut self,
        google_id: GoogleAccountId,
    ) -> Result<Option<AccountIdInternal>, DieselDatabaseError> {
        use crate::schema::{account_id, sign_in_with_info};

        sign_in_with_info::table
            .inner_join(account_id::table)
            .filter(sign_in_with_info::google_account_id.eq(&google_id))
            .select(AccountIdInternal::as_select())
            .first(self.conn())
            .optional()
            .into_db_error(google_id)
    }

    pub fn sign_in_with_info_raw(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<SignInWithInfoRaw, DieselDatabaseError> {
        use crate::schema::sign_in_with_info::dsl::*;

        sign_in_with_info
            .filter(account_id.eq(id.as_db_id()))
            .select(SignInWithInfoRaw::as_select())
            .first(self.conn())
            .into_db_error(id)
    }

    pub fn sign_in_with_history_entries(
        &mut self,
        account: AccountIdInternal,
    ) -> Result<Vec<SignInWithHistoryEntry>, DieselDatabaseError> {
        use crate::schema::account_sign_in_with_history::dsl::*;

        let entries: Vec<SignInWithHistoryEntry> = account_sign_in_with_history
            .filter(account_id.eq(account.as_db_id()))
            .order(change_unix_time.asc())
            .select(SignInWithHistoryEntry::as_select())
            .load(self.conn())
            .into_db_error(account)?;

        Ok(entries)
    }

    pub fn sign_in_with_history_count(
        &mut self,
        account: AccountIdInternal,
    ) -> Result<i64, DieselDatabaseError> {
        use crate::schema::account_sign_in_with_history::dsl::*;

        let count = account_sign_in_with_history
            .filter(account_id.eq(account.as_db_id()))
            .count()
            .get_result(self.conn())
            .into_db_error(account)?;

        Ok(count)
    }
}
