
use std::{ops::DerefMut, sync::Arc};

use config::Config;
use database::{
    current::{
        read::CurrentSyncReadCommands,
        write::TransactionConnection,
    }, history::write::HistorySyncWriteCommands, CurrentWriteHandle, DbReaderUsingWriteHandle, DbWriter, DbWriterWithHistory, DieselConnection, DieselDatabaseError, HistoryWriteHandle, PoolObject, TransactionError
};
use model::{
    Account, AccountId, AccountIdInternal, AccountInternal, AccountSetup, EmailAddress, Profile,
    SharedStateRaw, SignInWithInfo,
};
use server_common::push_notifications::PushNotificationSender;
use simple_backend::media_backup::MediaBackupHandle;
use simple_backend_utils::IntoReportFromString;

use self::{
    common::WriteCommandsCommon,
};
use super::{
    cache::DatabaseCache,
    file::utils::FileDir,
    index::{LocationIndexIteratorHandle, LocationIndexManager, LocationIndexWriteHandle},
    IntoDataError,
};
use crate::{result::Result, DataError};


pub struct RegisterAccount;

impl RegisterAccount {
    pub async fn register(
        &self,
        id_light: AccountId,
        sign_in_with_info: SignInWithInfo,
        email: Option<EmailAddress>,
    ) -> Result<AccountIdInternal, DataError> {
        let config = self.config.clone();
        let id: AccountIdInternal = self
            .db_transaction_with_history(move |transaction, history_conn| {
                Self::register_db_action(
                    config,
                    id_light,
                    sign_in_with_info,
                    email,
                    transaction,
                    history_conn,
                )
            })
            .await?;

        self.cache
            .load_account_from_db(
                id,
                self.config,
                &self.current_write_handle.to_read_handle(),
                LocationIndexIteratorHandle::new(self.location_index),
                LocationIndexWriteHandle::new(self.location_index),
            )
            .await
            .into_data_error(id)?;

        Ok(id)
    }

    pub fn register_db_action(
        config: Arc<Config>,
        id_light: AccountId,
        sign_in_with_info: SignInWithInfo,
        email: Option<EmailAddress>,
        transaction: TransactionConnection<'_>,
        history_conn: PoolObject,
    ) -> std::result::Result<AccountIdInternal, TransactionError> {
        let account = Account::default();
        let account_setup = AccountSetup::default();

        let mut current = transaction.into_cmds();

        // No transaction for history as it does not matter if some default
        // data will be left there if there is some error.
        let mut history_conn = history_conn
            .lock()
            .into_error_string(DieselDatabaseError::LockConnectionFailed)?;
        let mut history = HistorySyncWriteCommands::new(history_conn.deref_mut());

        // Common
        let id = current.account().data().insert_account_id(id_light)?;
        current.account().token().insert_access_token(id, None)?;
        current.account().token().insert_refresh_token(id, None)?;
        current
            .common()
            .state()
            .insert_default_account_capabilities(id)?;
        current
            .common()
            .state()
            .insert_shared_state(id, SharedStateRaw::default())?;

        // Common history
        history.account().insert_account_id(id)?;

        if config.components().account {
            current
                .account()
                .data()
                .insert_account(id, AccountInternal::default())?;
            current
                .account()
                .data()
                .insert_account_setup(id, &account_setup)?;
            current
                .account()
                .sign_in_with()
                .insert_sign_in_with_info(id, &sign_in_with_info)?;
            if let Some(email) = email {
                current.account().data().update_account_email(id, &email)?;
            }

            // Account history
            history.account().insert_account(id, &account)?;
            history.account().insert_account_setup(id, &account_setup)?;
        }

        if config.components().profile {
            let profile = current.profile().data().insert_profile(id)?;
            current.profile().data().insert_profile_state(id)?;

            // Profile history
            let attributes = current
                .read()
                .profile()
                .data()
                .profile_attribute_values(id)?;
            let profile = Profile::new(profile, attributes);
            history.profile().insert_profile(id, &profile)?;
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
