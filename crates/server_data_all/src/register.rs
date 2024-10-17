use std::{ops::DerefMut, sync::Arc};

use config::Config;
use database::{
    current::write::TransactionConnection, ConnectionProvider, PoolObject, TransactionError,
};
use model::{
    Account, AccountId, AccountIdInternal, AccountInternal, EmailAddress, SharedStateRaw, SignInWithInfo
};
use server_data::{
    index::{LocationIndexIteratorHandle, LocationIndexWriteHandle},
    result::Result,
    write::WriteCommandsProvider,
    DataError, IntoDataError,
};

use crate::load::DbDataToCacheLoader;

pub struct RegisterAccount<C: WriteCommandsProvider> {
    cmds: C,
}

impl<C: WriteCommandsProvider> RegisterAccount<C> {
    pub fn new(cmds: C) -> Self {
        Self { cmds }
    }

    pub async fn register(
        &self,
        account_id: AccountId,
        sign_in_with_info: SignInWithInfo,
        email: Option<EmailAddress>,
    ) -> Result<AccountIdInternal, DataError> {
        let config = self.cmds.write_cmds().config.clone();
        let id: AccountIdInternal = self
            .cmds
            .write_cmds()
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
            self.cmds.write_cmds().cache,
            id,
            self.cmds.write_cmds().config,
            &self.cmds.write_cmds().current_write_handle.to_read_handle(),
            LocationIndexIteratorHandle::new(self.cmds.write_cmds().location_index),
            LocationIndexWriteHandle::new(self.cmds.write_cmds().location_index),
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
        mut transaction: TransactionConnection<'_>,
        mut history_conn: PoolObject,
    ) -> std::result::Result<AccountIdInternal, TransactionError> {
        let account = Account::default();

        let mut conn = &mut transaction;

        // No transaction for history as it does not matter if some default
        // data will be left there if there is some error.
        let mut history =
            database_account::history::write::HistorySyncWriteCommands::new(history_conn.as_mut());

        // Common
        let mut current = database::current::write::CurrentSyncWriteCommands::new(conn.conn());
        let id = current.common().insert_account_id(account_id)?;
        current.common().token().insert_access_token(id, None)?;
        current.common().token().insert_refresh_token(id, None)?;
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
            let mut current =
                database_account::current::write::CurrentSyncWriteCommands::new(conn.conn());
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
            history.account().insert_account(id, &account)?;
        }

        if config.components().profile {
            let mut current =
                database_profile::current::write::CurrentSyncWriteCommands::new(conn.conn());
            let _history = database_profile::history::write::HistorySyncWriteCommands::new(
                history_conn.deref_mut(),
            );
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
            let mut current =
                database_media::current::write::CurrentSyncWriteCommands::new(conn.conn());
            current.media().insert_media_state(id)?;

            current
                .media()
                .media_content()
                .insert_current_account_media(id)?;
        }

        if config.components().chat {
            let mut current =
                database_chat::current::write::CurrentSyncWriteCommands::new(conn.conn());
            current.chat().insert_chat_state(id)?;
        }

        Ok(id)
    }
}
