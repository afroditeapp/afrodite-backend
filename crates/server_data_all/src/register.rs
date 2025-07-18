use std::sync::Arc;

use config::Config;
use database::{DbWriteMode, DieselDatabaseError, current::write::GetDbWriteCommandsCommon};
use database_account::current::write::GetDbWriteCommandsAccount;
use database_chat::current::write::GetDbWriteCommandsChat;
use database_media::current::write::GetDbWriteCommandsMedia;
use database_profile::current::write::GetDbWriteCommandsProfile;
use model_account::{
    AccountId, AccountIdInternal, AccountInternal, EmailAddress, SharedStateRaw, SignInWithInfo,
};
use model_chat::{ProfileContentModificationMetadata, ProfileModificationMetadata};
use server_data::{
    DataError, IntoDataError, db_manager::InternalWriting, define_cmd_wrapper_write,
    index::LocationIndexIteratorHandle, result::Result, write::DbTransaction,
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
                Self::register_db_action(config, account_id, sign_in_with_info, email, current)
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
        current
            .common()
            .state()
            .insert_default_account_permissions(id)?;
        current
            .common()
            .state()
            .insert_shared_state(id, SharedStateRaw::default())?;
        current.common().insert_common_state(id)?;

        if config.components().account {
            current
                .account()
                .data()
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
            let modification = ProfileModificationMetadata::generate();
            current.profile().data().insert_profile(id, &modification)?;
            current
                .profile()
                .data()
                .insert_profile_state(id, &modification)?;
        }

        if config.components().media {
            let modification = ProfileContentModificationMetadata::generate();
            current.media().insert_media_state(id, &modification)?;

            current
                .media()
                .media_content()
                .insert_current_account_media(id, &modification)?;
        }

        if config.components().chat {
            current.chat().insert_chat_state(id)?;
            current.chat().limits().insert_daily_likes_left(id)?;
            if let Some(daily_likes) = config.client_features().and_then(|v| v.daily_likes()) {
                current
                    .chat()
                    .limits()
                    .reset_daily_likes_left(id, daily_likes)?;
            }
        }

        Ok(id)
    }
}
