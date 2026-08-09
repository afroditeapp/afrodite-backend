use std::{collections::HashMap, net::IpAddr, sync::Arc};

use config::Config;
use database::{DbWriteMode, DieselDatabaseError, current::write::GetDbWriteCommandsCommon};
use database_account::current::write::GetDbWriteCommandsAccount;
use database_chat::current::write::GetDbWriteCommandsChat;
use database_media::current::write::GetDbWriteCommandsMedia;
use database_profile::current::write::GetDbWriteCommandsProfile;
use model::{AccountIdDb, IpAddressStorage};
use model_account::{
    AccountIdInternal, EmailAddress, EmailAddressStateInternal, SharedStateRaw, SignInWithInfo,
};
use model_chat::{ProfileContentModificationMetadata, ProfileModificationMetadata};
use server_data::{
    DataError, IntoDataError,
    db_manager::InternalWriting,
    define_cmd_wrapper_write,
    index::{LocationIndexIteratorHandle, LocationIndexWriteHandle},
    result::Result,
    write::DbTransaction,
};
use tokio::sync::Mutex;

use crate::load::DbDataToCacheLoader;

define_cmd_wrapper_write!(RegisterAccount);

impl RegisterAccount<'_> {
    pub async fn register(
        &self,
        account_id: AccountIdInternal,
        sign_in_with_info: SignInWithInfo,
        email: Option<EmailAddress>,
        ip: IpAddr,
    ) -> Result<(), DataError> {
        let config = self.config_arc().clone();
        self.db_transaction(move |current| {
            Self::register_db_action(config, account_id, sign_in_with_info, email, ip, current)
        })
        .await?;

        // Mutex is unnecessary here because WriteRunnerCommandHandle
        // prevents concurrent writes.
        let location_index_write_handle =
            Mutex::new(LocationIndexWriteHandle::new(self.location()));

        DbDataToCacheLoader::load_account_from_db(
            self.cache(),
            account_id,
            self.current_read_handle(),
            self.location(),
            LocationIndexIteratorHandle::new(self.location()),
            &location_index_write_handle,
        )
        .await
        .into_data_error(account_id)?;

        Ok(())
    }

    pub fn register_db_action(
        config: Arc<Config>,
        id: AccountIdInternal,
        sign_in_with_info: SignInWithInfo,
        email: Option<EmailAddress>,
        ip: IpAddr,
        mut current: DbWriteMode,
    ) -> simple_backend_utils::Result<AccountIdInternal, DieselDatabaseError> {
        // Common
        current.common().insert_account_id(id)?;
        current
            .common()
            .state()
            .insert_default_account_permissions(id)?;
        current
            .common()
            .state()
            .insert_shared_state(id, SharedStateRaw::default())?;
        current.common().insert_common_state(id)?;
        current.common().insert_push_notification(id)?;

        // Account

        current
            .account()
            .data()
            .update_account_created_unix_time(id)?;
        current
            .account()
            .data()
            .insert_email_address_state(id, EmailAddressStateInternal::default())?;
        current.account().data().insert_account_state(id)?;
        current
            .account()
            .sign_in_with()
            .insert_sign_in_with_info(id, &sign_in_with_info)?;
        if let Some(email) = email {
            current.account().data().update_account_email(id, &email)?;
        }

        // Profile

        let modification = ProfileModificationMetadata::generate();
        current.profile().data().insert_profile(id, &modification)?;
        current
            .profile()
            .data()
            .insert_profile_state(id, &modification)?;

        // Media

        let modification = ProfileContentModificationMetadata::generate();
        current.media().insert_media_state(id, &modification)?;

        current
            .media()
            .media_content()
            .insert_current_account_media(id, &modification)?;

        // Chat

        current.chat().insert_chat_state(id)?;
        current.chat().limits().insert_daily_likes_left(id)?;
        if let Some(config) = &config.client_features_internal().likes.daily {
            current
                .chat()
                .limits()
                .reset_daily_likes_left(id, config.daily_likes.into())?;
        }

        // Save the IP address used at registration time.
        let mut ip_data = HashMap::new();
        ip_data.insert(AccountIdDb::from(id), IpAddressStorage::new(ip.into()));
        current
            .common_admin()
            .statistics()
            .save_ip_address_data(ip_data)?;

        Ok(id)
    }
}
