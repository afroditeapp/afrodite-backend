use database::{DieselDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use error_stack::Result;
use model::AccountId;
use model_account::{EmailAddress, GetAccountIdFromEmailResult};

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadAccountSearchAdmin);

impl CurrentReadAccountSearchAdmin<'_> {
    pub fn account_id_from_email(
        &mut self,
        email: EmailAddress,
    ) -> Result<GetAccountIdFromEmailResult, DieselDatabaseError> {
        use crate::schema::{account, account_id};

        let found_account: Option<AccountId> = account::table
            .inner_join(account_id::table)
            .filter(account::email.eq(email))
            .select(account_id::uuid)
            .first(self.conn())
            .optional()
            .into_db_error(())?;

        Ok(GetAccountIdFromEmailResult { aid: found_account })
    }
}
