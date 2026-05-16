use database::{DieselDatabaseError, define_current_write_commands};
use diesel::{prelude::*, update};
use error_stack::Result;
use model::{AccountIdInternal, UnixTime};
use model_account::{AccountVerificationErrorFlagsValue, VerificationMethod};

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteAccountVerification);

impl CurrentWriteAccountVerification<'_> {
    pub fn set_account_verification_data(
        &mut self,
        id: AccountIdInternal,
        method: VerificationMethod,
        time: UnixTime,
        error_flags: AccountVerificationErrorFlagsValue,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_state::dsl::*;

        update(account_state.find(id.as_db_id()))
            .set((
                account_verification_method.eq(Some(method)),
                account_verification_unix_time.eq(Some(time)),
                account_verification_error_flags.eq(error_flags),
            ))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }
}
