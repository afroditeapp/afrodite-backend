use std::sync::Arc;

use config::Config;
use database::{
    current::write::GetDbWriteCommandsCommon, DbWriteMode, DieselDatabaseError
};
use database_account::current::write::GetDbWriteCommandsAccount;
use database_chat::current::write::GetDbWriteCommandsChat;
use database_media::current::write::GetDbWriteCommandsMedia;
use database_profile::current::write::GetDbWriteCommandsProfile;
use model_account::{
    AccountId, AccountIdInternal, AccountInternal, EmailAddress, SharedStateRaw,
    SignInWithInfo,
};
use server_data::{
    db_manager::InternalWriting, define_cmd_wrapper_write, index::LocationIndexIteratorHandle, result::Result, write::DbTransaction, DataError, IntoDataError
};

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
            .db_transaction(move |current| {
                Self::register_db_action(
                    config,
                    account_id,
                    sign_in_with_info,
                    email,
                    current,
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
        mut current: DbWriteMode,
    ) -> error_stack::Result<AccountIdInternal, DieselDatabaseError> {
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

        if config.components().account {
            current
                .common()
                .state()
                .update_account_created_unix_time(id)?;
            current
                .account()
                .data()
                .insert_account(id, AccountInternal::default())?;
            current.account().data().insert_default_account_setup(id)?;
            current.account().data().insert_account_state(id)?;
            current
                .account()
                .sign_in_with()
                .insert_sign_in_with_info(id, &sign_in_with_info)?;
            if let Some(email) = email {
                current.account().data().update_account_email(id, &email)?;
            }
        }

        if config.components().profile {
            current.profile().data().insert_profile(id)?;
            current.profile().data().insert_profile_state(id)?;
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
