use database::{DieselDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use error_stack::Result;
use model::AccountIdInternal;
use model_account::AccountVerificationDataInternal;

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadAccountVerification);

impl CurrentReadAccountVerification<'_> {
    pub fn account_verification_data(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<AccountVerificationDataInternal, DieselDatabaseError> {
        use crate::schema::account_state::dsl::*;

        account_state
            .filter(account_id.eq(id.as_db_id()))
            .select((
                account_verification_method,
                account_verification_unix_time,
                account_verification_error_flags,
            ))
            .first::<AccountVerificationDataInternal>(self.conn())
            .into_db_error(id)
    }
}
