use diesel::{insert_into, prelude::*, update};
use error_stack::Result;
use model::{AccountIdInternal, SignInWithInfo};
use simple_backend_database::diesel_db::{ConnectionProvider, DieselDatabaseError};

use crate::IntoDatabaseError;

define_write_commands!(
    CurrentWriteAccountSignInWith,
    CurrentSyncWriteAccountSignInWith
);

impl<C: ConnectionProvider> CurrentSyncWriteAccountSignInWith<C> {
    pub fn insert_sign_in_with_info(
        &mut self,
        id: AccountIdInternal,
        data: &SignInWithInfo,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::sign_in_with_info::dsl::*;

        insert_into(sign_in_with_info)
            .values((
                account_id.eq(id.as_db_id()),
                google_account_id.eq(&data.google_account_id),
            ))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn sign_in_with_info(
        &mut self,
        id: AccountIdInternal,
        data: &SignInWithInfo,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::sign_in_with_info::dsl::*;

        update(sign_in_with_info.find(id.as_db_id()))
            .set((google_account_id.eq(&data.google_account_id),))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn set_is_bot_account(
        &mut self,
        id: AccountIdInternal,
        value_for_is_bot_account: bool,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::sign_in_with_info::dsl::*;

        update(sign_in_with_info.find(id.as_db_id()))
            .set((is_bot_account.eq(value_for_is_bot_account),))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }
}
