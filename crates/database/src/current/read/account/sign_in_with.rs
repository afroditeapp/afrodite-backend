use diesel::prelude::*;
use error_stack::Result;
use model::{AccountIdInternal, GoogleAccountId, SignInWithInfoRaw};
use simple_backend_database::diesel_db::{ConnectionProvider, DieselDatabaseError};

use crate::IntoDatabaseError;

define_read_commands!(
    CurrentReadAccountSignInWith,
    CurrentSyncReadAccountSignInWith
);

impl<C: ConnectionProvider> CurrentSyncReadAccountSignInWith<C> {
    pub fn google_account_id_to_account_id(
        &mut self,
        google_id: GoogleAccountId,
    ) -> Result<Option<AccountIdInternal>, DieselDatabaseError> {
        use crate::schema::{account_id, sign_in_with_info};

        sign_in_with_info::table
            .inner_join(account_id::table)
            .filter(sign_in_with_info::google_account_id.eq(google_id.as_str()))
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
}
