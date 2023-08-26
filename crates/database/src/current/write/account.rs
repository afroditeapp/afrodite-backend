use diesel::{insert_into, prelude::*, update};
use error_stack::{Result, ResultExt};
use model::{
    AccessToken, Account, AccountId, AccountIdDb, AccountIdInternal, AccountSetup, RefreshToken,
    SignInWithInfo,
};


use super::ConnectionProvider;
use crate::{diesel::DieselDatabaseError, IntoDatabaseError};

define_write_commands!(CurrentWriteAccount, CurrentSyncWriteAccount);

impl<C: ConnectionProvider> CurrentSyncWriteAccount<C> {
    pub fn insert_account_id(
        mut self,
        account_uuid: AccountId,
    ) -> Result<AccountIdInternal, DieselDatabaseError> {
        use model::schema::account_id::dsl::*;

        let db_id: AccountIdDb = insert_into(account_id)
            .values(uuid.eq(account_uuid))
            .returning(id)
            .get_result(self.conn())
            .into_db_error(DieselDatabaseError::Execute, account_uuid)?;

        Ok(AccountIdInternal {
            uuid: account_uuid,
            id: db_id,
        })
    }

    pub fn insert_access_token(
        mut self,
        id: AccountIdInternal,
        token_value: Option<AccessToken>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::access_token::dsl::*;

        let token_value = token_value.as_ref().map(|k| k.as_str());

        insert_into(access_token)
            .values((account_id.eq(id.as_db_id()), token.eq(token_value)))
            .execute(self.conn())
            .into_db_error(DieselDatabaseError::Execute, id)?;

        Ok(())
    }

    pub fn access_token(
        mut self,
        id: AccountIdInternal,
        token_value: Option<AccessToken>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::access_token::dsl::*;

        let token_value = token_value.as_ref().map(|k| k.as_str());

        update(access_token.find(id.as_db_id()))
            .set(token.eq(token_value))
            .execute(self.conn())
            .into_db_error(DieselDatabaseError::Execute, id)?;

        Ok(())
    }

    pub fn insert_refresh_token(
        mut self,
        id: AccountIdInternal,
        token_value: Option<RefreshToken>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::refresh_token::dsl::*;

        let token_value = if let Some(t) = token_value {
            Some(
                t.bytes()
                    .change_context(DieselDatabaseError::DataFormatConversion)?,
            )
        } else {
            None
        };

        insert_into(refresh_token)
            .values((account_id.eq(id.as_db_id()), token.eq(token_value)))
            .execute(self.conn())
            .into_db_error(DieselDatabaseError::Execute, id)?;

        Ok(())
    }

    pub fn refresh_token(
        &mut self,
        id: AccountIdInternal,
        token_value: Option<RefreshToken>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::refresh_token::dsl::*;

        let token_value = if let Some(t) = token_value {
            Some(
                t.bytes()
                    .change_context(DieselDatabaseError::DataFormatConversion)?,
            )
        } else {
            None
        };

        update(refresh_token.find(id.as_db_id()))
            .set(token.eq(token_value))
            .execute(self.conn())
            .into_db_error(DieselDatabaseError::Execute, id)?;

        Ok(())
    }

    pub fn insert_account(
        &mut self,
        id: AccountIdInternal,
        account_data: &Account,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account::dsl::*;

        let data =
            serde_json::to_string(account_data).change_context(DieselDatabaseError::SerdeSerialize)?;

        insert_into(account)
            .values((account_id.eq(id.as_db_id()), json_text.eq(data)))
            .execute(self.conn())
            .into_db_error(DieselDatabaseError::Execute, id)?;

        Ok(())
    }

    pub fn account(
        mut self,
        id: AccountIdInternal,
        account_data: &Account,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account::dsl::*;

        let data =
            serde_json::to_string(account_data).change_context(DieselDatabaseError::SerdeSerialize)?;

        update(account.find(id.as_db_id()))
            .set(json_text.eq(data))
            .execute(self.conn())
            .into_db_error(DieselDatabaseError::Execute, id)?;

        Ok(())
    }

    pub fn insert_account_setup(
        &mut self,
        id: AccountIdInternal,
        account_data: &AccountSetup,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_setup::dsl::*;

        insert_into(account_setup)
            .values((account_id.eq(id.as_db_id()), account_data))
            .execute(self.conn())
            .into_db_error(DieselDatabaseError::Execute, id)?;

        Ok(())
    }

    pub fn account_setup(
        &mut self,
        id: AccountIdInternal,
        account_data: &AccountSetup,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_setup::dsl::*;

        update(account_setup.find(id.as_db_id()))
            .set(account_data)
            .execute(self.conn())
            .into_db_error(DieselDatabaseError::Execute, id)?;

        Ok(())
    }

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
            .into_db_error(DieselDatabaseError::Execute, id)?;

        Ok(())
    }

    pub fn sign_in_with_info(
        &mut self,
        id: AccountIdInternal,
        data: &SignInWithInfo,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::sign_in_with_info::dsl::*;

        update(sign_in_with_info.find(id.as_db_id()))
            .set((
                account_id.eq(id.as_db_id()),
                google_account_id.eq(&data.google_account_id),
            ))
            .execute(self.conn())
            .into_db_error(DieselDatabaseError::Execute, id)?;

        Ok(())
    }
}
