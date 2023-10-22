//! Synchronous write commands combining cache and database operations.

use std::{ops::DerefMut, sync::Arc};

use config::Config;
use database::{
    current::{
        read::CurrentSyncReadCommands,
        write::{CurrentSyncWriteCommands, TransactionConnection},
    },
    diesel::{
        DieselConnection, DieselCurrentWriteHandle, DieselDatabaseError, DieselHistoryWriteHandle,
    },
    history::write::{HistorySyncWriteCommands, HistoryWriteCommands},
    sqlite::{CurrentDataWriteHandle, HistoryWriteHandle},
    PoolObject, TransactionError,
};
use error_stack::{Result, ResultExt};
use model::{Account, AccountId, AccountIdInternal, AccountSetup, SignInWithInfo, SharedState, AccountInternal};
use utils::{ IntoReportFromString};

use self::{
    account::WriteCommandsAccount, account_admin::WriteCommandsAccountAdmin,
    chat::WriteCommandsChat, chat_admin::WriteCommandsChatAdmin, common::WriteCommandsCommon,
    media::WriteCommandsMedia, media_admin::WriteCommandsMediaAdmin, profile::WriteCommandsProfile,
    profile_admin::WriteCommandsProfileAdmin,
};
use super::{
    cache::DatabaseCache,
    file::utils::FileDir,
    index::{LocationIndexIteratorHandle, LocationIndexManager, LocationIndexWriteHandle},
    IntoDataError,
};
use crate::{data::DataError, media_backup::MediaBackupHandle};

macro_rules! define_write_commands {
    ($struct_name:ident) => {
        pub struct $struct_name<'a> {
            cmds: super::WriteCommands<'a>,
        }

        impl<'a> $struct_name<'a> {
            pub fn new(cmds: super::WriteCommands<'a>) -> Self {
                Self { cmds }
            }

            #[allow(dead_code)]
            fn current_write(&self) -> &database::sqlite::CurrentDataWriteHandle {
                &self.cmds.current_write
            }

            #[allow(dead_code)]
            fn history_write(&self) -> &database::sqlite::HistoryWriteHandle {
                &self.cmds.history_write
            }

            #[allow(dead_code)]
            fn cache(&self) -> &super::super::cache::DatabaseCache {
                &self.cmds.cache
            }

            #[allow(dead_code)]
            fn file_dir(&self) -> &super::super::FileDir {
                &self.cmds.file_dir
            }

            #[allow(dead_code)]
            fn location(&self) -> super::super::index::LocationIndexWriteHandle<'a> {
                super::super::index::LocationIndexWriteHandle::new(&self.cmds.location_index)
            }

            #[allow(dead_code)]
            fn location_iterator(&self) -> super::super::index::LocationIndexIteratorHandle<'a> {
                super::super::index::LocationIndexIteratorHandle::new(&self.cmds.location_index)
            }

            #[allow(dead_code)]
            fn media_backup(&self) -> &crate::media_backup::MediaBackupHandle {
                &self.cmds.media_backup
            }

            #[allow(dead_code)]
            fn current(&self) -> database::current::write::CurrentWriteCommands {
                database::current::write::CurrentWriteCommands::new(self.current_write())
            }

            #[allow(dead_code)]
            fn history(&self) -> super::super::write::HistoryWriteCommands {
                super::super::write::HistoryWriteCommands::new(&self.history_write())
            }

            #[track_caller]
            pub async fn db_write<
                T: FnOnce(
                        database::current::write::CurrentSyncWriteCommands<
                            &mut database::diesel::DieselConnection,
                        >,
                    )
                        -> error_stack::Result<R, database::diesel::DieselDatabaseError>
                    + Send
                    + 'static,
                R: Send + 'static,
            >(
                &self,
                cmd: T,
            ) -> error_stack::Result<R, crate::data::DataError> {
                self.cmds.db_write(cmd).await
            }

            #[track_caller]
            pub async fn db_transaction<
                T: FnOnce(
                        &mut database::diesel::DieselConnection,
                    ) -> std::result::Result<
                        R,
                        database::TransactionError<database::diesel::DieselDatabaseError>,
                    > + Send
                    + 'static,
                R: Send + 'static,
            >(
                &self,
                cmd: T,
            ) -> error_stack::Result<R, crate::data::DataError> {
                self.cmds.db_transaction(cmd).await
            }

            #[track_caller]
            pub async fn db_read<
                T: FnOnce(
                        database::current::read::CurrentSyncReadCommands<
                            &mut database::diesel::DieselConnection,
                        >,
                    )
                        -> error_stack::Result<R, database::diesel::DieselDatabaseError>
                    + Send
                    + 'static,
                R: Send + 'static,
            >(
                &self,
                cmd: T,
            ) -> error_stack::Result<R, crate::data::DataError> {
                self.cmds.db_read(cmd).await
            }

            #[track_caller]
            pub async fn write_cache<T, Id: Into<model::AccountId>>(
                &self,
                id: Id,
                cache_operation: impl FnOnce(
                    &mut crate::data::cache::CacheEntry,
                ) -> error_stack::Result<T, crate::data::CacheError>,
            ) -> error_stack::Result<T, crate::data::DataError> {
                use error_stack::ResultExt;
                self.cache()
                    .write_cache(id, cache_operation)
                    .await
                    .change_context(crate::data::DataError::Cache)
            }
        }
    };
}

pub mod account;
pub mod account_admin;
pub mod chat;
pub mod chat_admin;
pub mod common;
pub mod media;
pub mod media_admin;
pub mod profile;
pub mod profile_admin;

/// One Account can do only one write command at a time.
pub struct AccountWriteLock;

/// Globally synchronous write commands.
pub struct WriteCommands<'a> {
    config: &'a Arc<Config>,
    current_write: &'a CurrentDataWriteHandle,
    history_write: &'a HistoryWriteHandle,
    diesel_current_write: &'a DieselCurrentWriteHandle,
    diesel_history_write: &'a DieselHistoryWriteHandle,
    cache: &'a DatabaseCache,
    file_dir: &'a FileDir,
    location_index: &'a LocationIndexManager,
    media_backup: &'a MediaBackupHandle,
}

impl<'a> WriteCommands<'a> {
    pub fn new(
        config: &'a Arc<Config>,
        current_write: &'a CurrentDataWriteHandle,
        history_write: &'a HistoryWriteHandle,
        diesel_current_write: &'a DieselCurrentWriteHandle,
        diesel_history_write: &'a DieselHistoryWriteHandle,
        cache: &'a DatabaseCache,
        file_dir: &'a FileDir,
        location_index: &'a LocationIndexManager,
        media_backup: &'a MediaBackupHandle,
    ) -> Self {
        Self {
            config,
            current_write,
            history_write,
            diesel_current_write,
            diesel_history_write,
            cache,
            file_dir,
            location_index,
            media_backup,
        }
    }

    pub fn common(self) -> WriteCommandsCommon<'a> {
        WriteCommandsCommon::new(self)
    }

    pub fn account(self) -> WriteCommandsAccount<'a> {
        WriteCommandsAccount::new(self)
    }

    pub fn account_admin(self) -> WriteCommandsAccountAdmin<'a> {
        WriteCommandsAccountAdmin::new(self)
    }

    pub fn media(self) -> WriteCommandsMedia<'a> {
        WriteCommandsMedia::new(self)
    }

    pub fn media_admin(self) -> WriteCommandsMediaAdmin<'a> {
        WriteCommandsMediaAdmin::new(self)
    }

    pub fn profile(self) -> WriteCommandsProfile<'a> {
        WriteCommandsProfile::new(self)
    }

    pub fn profile_admin(self) -> WriteCommandsProfileAdmin<'a> {
        WriteCommandsProfileAdmin::new(self)
    }

    pub fn chat(self) -> WriteCommandsChat<'a> {
        WriteCommandsChat::new(self)
    }

    pub fn chat_admin(self) -> WriteCommandsChatAdmin<'a> {
        WriteCommandsChatAdmin::new(self)
    }

    pub async fn register(
        &self,
        id_light: AccountId,
        sign_in_with_info: SignInWithInfo,
    ) -> Result<AccountIdInternal, DataError> {
        let config = self.config.clone();
        let id: AccountIdInternal = self
            .db_transaction_with_history(move |transaction, history_conn| {
                Self::register_db_action(
                    config,
                    id_light,
                    sign_in_with_info,
                    transaction,
                    history_conn,
                )
            })
            .await?;

        self.cache
            .load_account_from_db(
                id,
                &self.config,
                &self.diesel_current_write.to_read_handle(),
                LocationIndexIteratorHandle::new(&self.location_index),
                LocationIndexWriteHandle::new(&self.location_index),
            )
            .await
            .into_data_error(id)?;

        Ok(id)
    }

    pub fn register_db_action(
        config: Arc<Config>,
        id_light: AccountId,
        sign_in_with_info: SignInWithInfo,
        transaction: TransactionConnection<'_>,
        history_conn: PoolObject,
    ) -> std::result::Result<AccountIdInternal, TransactionError<DieselDatabaseError>> {
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
        let id = current.account().insert_account_id(id_light)?;
        current.account().insert_access_token(id, None)?;
        current.account().insert_refresh_token(id, None)?;
        current.common().insert_default_account_capabilities(id)?;
        current.common().insert_shared_state(id, SharedState::default())?;

        // Common history
        history.account().insert_account_id(id)?;

        if config.components().account {
            current.account().insert_account(id, AccountInternal::default())?;
            current.account().insert_account_setup(id, &account_setup)?;
            current
                .account()
                .insert_sign_in_with_info(id, &sign_in_with_info)?;

            // Account history
            history.account().insert_account(id, &account)?;
            history.account().insert_account_setup(id, &account_setup)?;
        }

        if config.components().profile {
            let profile = current.profile().insert_profile(id)?;
            current.profile().insert_profile_location(id)?;

            // Profile history
            history.profile().insert_profile(id, &profile.into())?;
        }

        if config.components().media {
            current.media().insert_current_account_media(id)?;
        }

        Ok(id.clone())
    }

    #[track_caller]
    pub async fn db_write<
        T: FnOnce(
                CurrentSyncWriteCommands<&mut DieselConnection>,
            ) -> Result<R, DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> Result<R, DataError> {
        let conn = self
            .diesel_current_write
            .pool()
            .get()
            .await
            .change_context(DieselDatabaseError::GetConnection)
            .change_context(DataError::Diesel)?;

        conn.interact(move |conn| cmd(CurrentSyncWriteCommands::new(conn)))
            .await
            .into_error_string(DieselDatabaseError::Execute)
            .change_context(DataError::Diesel)?
            .change_context(DataError::Diesel)
    }

    #[track_caller]
    pub async fn db_transaction<
        T: FnOnce(
                &mut database::diesel::DieselConnection,
            ) -> std::result::Result<R, TransactionError<DieselDatabaseError>>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> Result<R, DataError> {
        let conn = self
            .diesel_current_write
            .pool()
            .get()
            .await
            .change_context(DieselDatabaseError::GetConnection)
            .change_context(DataError::Diesel)?;

        conn.interact(move |conn| CurrentSyncWriteCommands::new(conn).transaction(cmd))
            .await
            .into_error_string(DieselDatabaseError::Execute)
            .change_context(DataError::Diesel)?
            .change_context(DataError::Diesel)
    }

    #[track_caller]
    pub async fn db_transaction_with_history<T, R: Send + 'static>(
        &self,
        cmd: T,
    ) -> Result<R, DataError>
    where
        T: FnOnce(
                TransactionConnection<'_>,
                PoolObject,
            ) -> std::result::Result<R, TransactionError<DieselDatabaseError>>
            + Send
            + 'static,
    {
        let conn = self
            .diesel_current_write
            .pool()
            .get()
            .await
            .change_context(DieselDatabaseError::GetConnection)
            .change_context(DataError::Diesel)?;

        let conn_history = self
            .diesel_history_write
            .pool()
            .get()
            .await
            .change_context(DieselDatabaseError::GetConnection)
            .change_context(DataError::Diesel)?;

        conn.interact(move |conn| {
            CurrentSyncWriteCommands::new(conn).transaction(move |conn| {
                let transaction_connection = TransactionConnection::new(conn);
                cmd(transaction_connection, conn_history)
            })
        })
        .await
        .into_error_string(DieselDatabaseError::Execute)
        .change_context(DataError::Diesel)?
        .change_context(DataError::Diesel)
    }

    #[track_caller]
    pub async fn db_read<
        T: FnOnce(CurrentSyncReadCommands<&mut DieselConnection>) -> Result<R, DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> Result<R, DataError> {
        let conn = self
            .diesel_current_write
            .pool()
            .get()
            .await
            .change_context(DieselDatabaseError::GetConnection)
            .change_context(DataError::Diesel)?;

        conn.interact(move |conn| cmd(CurrentSyncReadCommands::new(conn)))
            .await
            .into_error_string(DieselDatabaseError::Execute)
            .change_context(DataError::Diesel)?
            .change_context(DataError::Diesel)
    }
}
