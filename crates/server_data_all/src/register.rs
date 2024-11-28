use std::sync::Arc;

use config::Config;
use database::current::write::GetDbWriteCommandsCommon;
use database::{
    current::write::TransactionConnection, TransactionError, DbWriteModeHistory
};
use database_account::current::write::GetDbWriteCommandsAccount;
use database_account::history::write::GetDbHistoryWriteCommandsAccount;
use database_chat::current::write::GetDbWriteCommandsChat;
use database_media::current::write::GetDbWriteCommandsMedia;
use database_profile::current::write::GetDbWriteCommandsProfile;
use model_account::{
    Account, AccountId, AccountIdInternal, AccountInternal, EmailAddress, SharedStateRaw, SignInWithInfo
};
use server_data::db_manager::InternalWriting;
use server_data::define_cmd_wrapper_write;
use server_data::{
    result::Result,
    DataError, IntoDataError,
};
use server_data::index::LocationIndexIteratorHandle;

use crate::load::DbDataToCacheLoader;

define_cmd_wrapper_write!(RegisterAccount);

impl RegisterAccount<'_> {
    pub async fn register(
        &self,
        account_id: AccountId,
        sign_in_with_info: SignInWithInfo,
        email: Option<EmailAddress>,
    ) -> Result<AccountIdInternal, DataError> {
        let config = self.config_arc().clone();
        let id: AccountIdInternal = self
            .db_transaction_with_history(move |transaction, history_conn| {
                Self::register_db_action(
                    config,
                    account_id,
                    sign_in_with_info,
                    email,
                    transaction,
                    history_conn,
                )
            })
            .await?;

        DbDataToCacheLoader::load_account_from_db(
            self.cache(),
            id,
            self.config(),
            self.current_read_handle(),
            LocationIndexIteratorHandle::new(self.location()),
            self.location_index_write_handle(),
        )
        .await
        .into_data_error(id)?;

        Ok(id)
    }

    pub fn register_db_action(
        config: Arc<Config>,
        account_id: AccountId,
        sign_in_with_info: SignInWithInfo,
        email: Option<EmailAddress>,
        transaction: TransactionConnection,
        history_conn: DbWriteModeHistory,
    ) -> std::result::Result<AccountIdInternal, TransactionError> {
        let account = Account::default();

        let mut current = transaction.into_conn();

        // No transaction for history as it does not matter if some default
        // data will be left there if there is some error.
        let mut history = history_conn;

        // Common
        let id = current.common().insert_account_id(account_id)?;
        current.common().token().insert_access_token(id, None)?;
        current.common().token().insert_refresh_token(id, None)?;
        current
            .common()
            .state()
            .insert_default_account_permissions(id)?;
        current
            .common()
            .state()
            .insert_shared_state(id, SharedStateRaw::default())?;

        // Common history
        history.account_history().insert_account_id(id)?;

        if config.components().account {
            current
                .account()
                .data()
                .insert_account(id, AccountInternal::default())?;
            current
                .account()
                .data()
                .insert_default_account_setup(id)?;
            current
                .account()
                .data()
                .insert_account_state(id)?;
            current
                .account()
                .sign_in_with()
                .insert_sign_in_with_info(id, &sign_in_with_info)?;
            if let Some(email) = email {
                current.account().data().update_account_email(id, &email)?;
            }

            // Account history
            history.account_history().insert_account(id, &account)?;
        }

        if config.components().profile {
            let _profile = current.profile().data().insert_profile(id)?;
            current.profile().data().insert_profile_state(id)?;

            // // Profile history
            // let attributes = current
            //     .read()
            //     .profile()
            //     .data()
            //     .profile_attribute_values(id)?;
            // let profile = Profile::new(profile, attributes);
            // history.profile().insert_profile(id, &profile)?;
        }

        if config.components().media {
            current.media().insert_media_state(id)?;

            current
                .media()
                .media_content()
                .insert_current_account_media(id)?;
        }

        if config.components().chat {
            current.chat().insert_chat_state(id)?;
        }

        Ok(id)
    }
}
