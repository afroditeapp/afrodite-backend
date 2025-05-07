use database::{define_current_write_commands, DieselDatabaseError};
use diesel::{insert_into, prelude::*};
use error_stack::Result;
use model::AccountIdInternal;
use model_account::SignInWithInfo;

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteAccountSignInWith);

impl CurrentWriteAccountSignInWith<'_> {
    pub fn insert_sign_in_with_info(
        &mut self,
        id: AccountIdInternal,
        data: &SignInWithInfo,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::sign_in_with_info::dsl::*;

        insert_into(sign_in_with_info)
            .values((
                account_id.eq(id.as_db_id()),
                data,
            ))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }
}
