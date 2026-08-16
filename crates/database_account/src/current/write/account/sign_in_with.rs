use database::{DieselDatabaseError, define_current_write_commands};
use diesel::{delete, insert_into, prelude::*, update};
use model::{AccountIdInternal, UnixTime};
use model_account::{
    AppleAccountId, GoogleAccountId, SignInWithInfo, SignInWithProviderTypeNumber,
};
use simple_backend_utils::{Result, db::MyRunQueryDsl};

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
            .values((account_id.eq(id.as_db_id()), data))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn update_apple_account_id(
        &mut self,
        id: AccountIdInternal,
        new_apple_id: Option<AppleAccountId>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::sign_in_with_info::dsl::*;

        let old_apple_id: Option<AppleAccountId> = sign_in_with_info
            .filter(account_id.eq(id.as_db_id()))
            .select(apple_account_id)
            .first(self.conn())
            .optional()
            .into_db_error(id)?
            .flatten();

        if old_apple_id == new_apple_id {
            return Ok(());
        }

        update(sign_in_with_info.find(id.as_db_id()))
            .set(apple_account_id.eq(new_apple_id.clone()))
            .execute(self.conn())
            .into_db_error(id)?;

        self.insert_sign_in_with_history_entry(
            id,
            SignInWithProviderTypeNumber::Apple,
            old_apple_id.map(|v| v.0),
            new_apple_id.map(|v| v.0),
            UnixTime::current_time(),
        )?;

        Ok(())
    }

    pub fn update_google_account_id(
        &mut self,
        id: AccountIdInternal,
        new_google_id: Option<GoogleAccountId>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::sign_in_with_info::dsl::*;

        let old_google_id: Option<GoogleAccountId> = sign_in_with_info
            .filter(account_id.eq(id.as_db_id()))
            .select(google_account_id)
            .first(self.conn())
            .optional()
            .into_db_error(id)?
            .flatten();

        if old_google_id == new_google_id {
            return Ok(());
        }

        update(sign_in_with_info.find(id.as_db_id()))
            .set(google_account_id.eq(new_google_id.clone()))
            .execute(self.conn())
            .into_db_error(id)?;

        self.insert_sign_in_with_history_entry(
            id,
            SignInWithProviderTypeNumber::Google,
            old_google_id.map(|v| v.0),
            new_google_id.map(|v| v.0),
            UnixTime::current_time(),
        )?;

        Ok(())
    }

    fn insert_sign_in_with_history_entry(
        &mut self,
        account: AccountIdInternal,
        provider_type_number_value: SignInWithProviderTypeNumber,
        old_id_value: Option<String>,
        new_id_value: Option<String>,
        change_unix_time_value: UnixTime,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_sign_in_with_history::dsl::*;

        insert_into(account_sign_in_with_history)
            .values((
                account_id.eq(account.as_db_id()),
                provider_type_number.eq(provider_type_number_value),
                old_id.eq(old_id_value),
                new_id.eq(new_id_value),
                change_unix_time.eq(change_unix_time_value),
            ))
            .execute_my_conn(self.conn())
            .into_db_error(account)?;

        Ok(())
    }

    /// Prune sign in with history for all accounts.
    ///
    /// Deletes entries older than `retention_unix_time`.
    pub fn prune_sign_in_with_history(
        &mut self,
        retention_unix_time: UnixTime,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_sign_in_with_history::dsl::*;

        delete(account_sign_in_with_history)
            .filter(change_unix_time.lt(retention_unix_time))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
