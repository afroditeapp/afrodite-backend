use diesel::{insert_into, prelude::*, update};
use error_stack::Result;
use model::{
    Account, AccountIdDb, AccountIdInternal, AccountId, AccountSetup, AccessToken, RefreshToken,
    SignInWithInfo,
};
use utils::{IntoReportExt, current_unix_time};

use crate::{diesel::{DieselDatabaseError, HistoryConnectionProvider}, IntoDatabaseError};

define_write_commands!(HistoryWriteAccount, HistorySyncWriteAccount);

impl<C: HistoryConnectionProvider> HistorySyncWriteAccount<C> {
    pub fn insert_account_id(
        &mut self,
        account_id_internal: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_id::dsl::*;

        insert_into(account_id)
            .values((
                uuid.eq(account_id_internal.uuid),
                id.eq(account_id_internal.as_db_id()),
            ))
            .execute(self.conn())
            .into_db_error(DieselDatabaseError::Execute, account_id_internal)?;
        Ok(())
    }

    pub fn insert_account(
        &mut self,
        account_id_internal: AccountIdInternal,
        account: &Account,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::history_account::dsl::*;

        let text = serde_json::to_string(account)
            .into_error(DieselDatabaseError::SerdeSerialize)?;
        let time = current_unix_time();

        insert_into(history_account)
            .values((
                account_id.eq(account_id_internal.as_db_id()),
                unix_time.eq(time),
                json_text.eq(text),
            ))
            .execute(self.conn())
            .into_db_error(DieselDatabaseError::Execute, (account_id_internal, account_id_internal))?;
        Ok(())
    }

    pub fn insert_account_setup(
        &mut self,
        account_id_internal: AccountIdInternal,
        account: &AccountSetup,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::history_account_setup::dsl::*;

        let text = serde_json::to_string(account)
            .into_error(DieselDatabaseError::SerdeSerialize)?;
        let time = current_unix_time();

        insert_into(history_account_setup)
            .values((
                account_id.eq(account_id_internal.as_db_id()),
                unix_time.eq(time),
                json_text.eq(text),
            ))
            .execute(self.conn())
            .into_db_error(DieselDatabaseError::Execute, (account_id_internal, account_id_internal))?;
        Ok(())
    }

    // pub fn refresh_token(
    //     &mut self,
    //     id: AccountIdInternal,
    //     token_value: Option<RefreshToken>,
    // ) -> Result<(), DieselDatabaseError> {
    //     use model::schema::refresh_token::dsl::*;

    //     let token_value = if let Some(t) = token_value {
    //         Some(
    //             t.bytes()
    //                 .into_error(DieselDatabaseError::DataFormatConversion)?,
    //         )
    //     } else {
    //         None
    //     };

    //     update(refresh_token.find(id.as_db_id()))
    //         .set(token.eq(token_value))
    //         .execute(self.conn())
    //         .into_db_error(DieselDatabaseError::Execute, id)?;

    //     Ok(())
    // }
}
