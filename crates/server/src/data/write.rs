//! Synchronous write commands combining cache and database operations.

use std::{ops::DerefMut, sync::Arc};

use config::Config;
use database::{
    current::{
        read::CurrentSyncReadCommands,
        write::{CurrentSyncWriteCommands, TransactionConnection},
    },
    history::write::HistorySyncWriteCommands,
    CurrentWriteHandle, HistoryWriteHandle, TransactionError,
};
use model::{
    Account, AccountId, AccountIdInternal, AccountInternal, AccountSetup, EmailAddress, Profile,
    SharedStateRaw, SignInWithInfo,
};
use simple_backend::media_backup::MediaBackupHandle;
use simple_backend_database::{
    diesel_db::{DieselConnection, DieselDatabaseError},
    PoolObject,
};
use simple_backend_utils::IntoReportFromString;

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
use crate::{data::DataError, push_notifications::PushNotificationSender, result::Result};

macro_rules! define_write_commands {
    ($struct_name:ident) => {
        pub struct $struct_name<'a> {
            cmds: $crate::data::write::WriteCommands<'a>,
        }

        impl<'a> $struct_name<'a> {
            pub fn new(cmds: $crate::data::write::WriteCommands<'a>) -> Self {
                Self { cmds }
            }

            #[allow(dead_code)]
            fn current_write(&self) -> &database::CurrentWriteHandle {
                &self.cmds.current_write_handle
            }

            #[allow(dead_code)]
            fn history_write(&self) -> &database::HistoryWriteHandle {
                &self.cmds.history_write_handle
            }

            #[allow(dead_code)]
            fn cache(&self) -> &$crate::data::cache::DatabaseCache {
                &self.cmds.cache
            }

            #[allow(dead_code)]
            fn events(&self) -> $crate::event::EventManagerWithCacheReference {
                $crate::event::EventManagerWithCacheReference::new(
                    &self.cmds.cache,
                    &self.cmds.push_notification_sender,
                )
            }

            #[allow(dead_code)]
            fn config(&self) -> &config::Config {
                &self.cmds.config
            }

            #[allow(dead_code)]
            fn file_dir(&self) -> &$crate::data::FileDir {
                &self.cmds.file_dir
            }

            #[allow(dead_code)]
            fn location(&self) -> $crate::data::index::LocationIndexWriteHandle<'a> {
                $crate::data::index::LocationIndexWriteHandle::new(&self.cmds.location_index)
            }

            #[allow(dead_code)]
            fn location_iterator(&self) -> $crate::data::index::LocationIndexIteratorHandle<'a> {
                $crate::data::index::LocationIndexIteratorHandle::new(&self.cmds.location_index)
            }

            #[allow(dead_code)]
            fn media_backup(&self) -> &simple_backend::media_backup::MediaBackupHandle {
                &self.cmds.media_backup
            }

            pub async fn db_write<
                T: FnOnce(
                        database::current::write::CurrentSyncWriteCommands<
                            &mut simple_backend_database::diesel_db::DieselConnection,
                        >,
                    ) -> error_stack::Result<
                        R,
                        simple_backend_database::diesel_db::DieselDatabaseError,
                    > + Send
                    + 'static,
                R: Send + 'static,
            >(
                &self,
                cmd: T,
            ) -> error_stack::Result<R, simple_backend_database::diesel_db::DieselDatabaseError>
            {
                self.cmds.db_write(cmd).await
            }

            pub async fn db_transaction<
                T: FnOnce(
                        database::current::write::CurrentSyncWriteCommands<
                            &mut simple_backend_database::diesel_db::DieselConnection,
                        >,
                    ) -> error_stack::Result<
                        R,
                        simple_backend_database::diesel_db::DieselDatabaseError,
                    > + Send
                    + 'static,
                R: Send + 'static,
            >(
                &self,
                cmd: T,
            ) -> error_stack::Result<R, simple_backend_database::diesel_db::DieselDatabaseError>
            {
                self.cmds.db_transaction(cmd).await
            }

            pub async fn db_read<
                T: FnOnce(
                        database::current::read::CurrentSyncReadCommands<
                            &mut simple_backend_database::diesel_db::DieselConnection,
                        >,
                    ) -> error_stack::Result<
                        R,
                        simple_backend_database::diesel_db::DieselDatabaseError,
                    > + Send
                    + 'static,
                R: Send + 'static,
            >(
                &self,
                cmd: T,
            ) -> error_stack::Result<R, simple_backend_database::diesel_db::DieselDatabaseError>
            {
                self.cmds.db_read(cmd).await
            }

            pub async fn write_cache<T, Id: Into<model::AccountId>>(
                &self,
                id: Id,
                cache_operation: impl FnOnce(
                    &mut $crate::data::cache::CacheEntry,
                )
                    -> error_stack::Result<T, $crate::data::CacheError>,
            ) -> error_stack::Result<T, $crate::data::CacheError> {
                self.cache().write_cache(id, cache_operation).await
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
    current_write_handle: &'a CurrentWriteHandle,
    history_write_handle: &'a HistoryWriteHandle,
    cache: &'a DatabaseCache,
    file_dir: &'a FileDir,
    location_index: &'a LocationIndexManager,
    media_backup: &'a MediaBackupHandle,
    push_notification_sender: &'a PushNotificationSender,
}

impl<'a> WriteCommands<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        config: &'a Arc<Config>,
        current_write_handle: &'a CurrentWriteHandle,
        history_write_handle: &'a HistoryWriteHandle,
        cache: &'a DatabaseCache,
        file_dir: &'a FileDir,
        location_index: &'a LocationIndexManager,
        media_backup: &'a MediaBackupHandle,
        push_notification_sender: &'a PushNotificationSender,
    ) -> Self {
        Self {
            config,
            current_write_handle,
            history_write_handle,
            cache,
            file_dir,
            location_index,
            media_backup,
            push_notification_sender,
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

    pub async fn db_write<
        T: FnOnce(
                CurrentSyncWriteCommands<&mut DieselConnection>,
            ) -> error_stack::Result<R, DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, DieselDatabaseError> {
        use error_stack::ResultExt;

        let conn = self
            .current_write_handle
            .0
            .diesel()
            .pool()
            .get()
            .await
            .change_context(DieselDatabaseError::GetConnection)?;

        conn.interact(move |conn| cmd(CurrentSyncWriteCommands::new(conn)))
            .await
            .into_error_string(DieselDatabaseError::Execute)?
    }

    pub async fn db_transaction<
        T: FnOnce(
                database::current::write::CurrentSyncWriteCommands<
                    &mut simple_backend_database::diesel_db::DieselConnection,
                >,
            ) -> error_stack::Result<R, DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, DieselDatabaseError> {
        use error_stack::ResultExt;

        let conn = self
            .current_write_handle
            .0
            .diesel()
            .pool()
            .get()
            .await
            .change_context(DieselDatabaseError::GetConnection)?;

        let result = conn
            .interact(move |conn| {
                CurrentSyncWriteCommands::new(conn).transaction(move |conn| {
                    cmd(CurrentSyncWriteCommands::new(conn)).map_err(|err| err.into())
                })
            })
            .await
            .into_error_string(DieselDatabaseError::Execute);

        match result {
            Ok(result) => match result {
                Ok(result) => Ok(result),
                Err(err) => Err(err),
            },
            Err(err) => Err(err),
        }
    }

    pub async fn db_transaction_with_history<T, R: Send + 'static>(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, DieselDatabaseError>
    where
        T: FnOnce(
                TransactionConnection<'_>,
                PoolObject,
            ) -> std::result::Result<R, TransactionError<DieselDatabaseError>>
            + Send
            + 'static,
    {
        use error_stack::ResultExt;

        let conn = self
            .current_write_handle
            .0
            .diesel()
            .pool()
            .get()
            .await
            .change_context(DieselDatabaseError::GetConnection)?;

        let conn_history = self
            .history_write_handle
            .0
            .diesel()
            .pool()
            .get()
            .await
            .change_context(DieselDatabaseError::GetConnection)?;

        conn.interact(move |conn| {
            CurrentSyncWriteCommands::new(conn).transaction(move |conn| {
                let transaction_connection = TransactionConnection::new(conn);
                cmd(transaction_connection, conn_history)
            })
        })
        .await
        .into_error_string(DieselDatabaseError::Execute)?
    }

    pub async fn db_read<
        T: FnOnce(
                CurrentSyncReadCommands<&mut DieselConnection>,
            ) -> error_stack::Result<R, DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, DieselDatabaseError> {
        use error_stack::ResultExt;

        let conn = self
            .current_write_handle
            .0
            .diesel()
            .pool()
            .get()
            .await
            .change_context(DieselDatabaseError::GetConnection)?;

        conn.interact(move |conn| cmd(CurrentSyncReadCommands::new(conn)))
            .await
            .into_error_string(DieselDatabaseError::Execute)?
    }
}

/// Macro for writing to current database with transaction.
/// Calls await automatically.
///
/// ```ignore
/// use server::data::DataError;
/// use server::data::write::{define_write_commands, db_transaction};
///
/// define_write_commands!(WriteCommandsTest);
///
/// impl WriteCommandsTest<'_> {
///     pub async fn test(
///         &self,
///     ) -> server::result::Result<(), DataError> {
///         db_transaction!(self, move |mut cmds| {
///             Ok(())
///         })?;
///         Ok(())
///     }
/// }
/// ```
macro_rules! db_transaction {
    ($state:expr, move |mut $cmds:ident| $commands:expr) => {{
        $crate::data::IntoDataError::into_error(
            $state.db_transaction(move |mut $cmds| ($commands)).await,
        )
    }};
    ($state:expr, move |$cmds:ident| $commands:expr) => {{
        $crate::data::IntoDataError::into_error(
            $state.db_transaction(move |$cmds| ($commands)).await,
        )
    }};
}

// Make db_transaction available in all modules
pub(crate) use db_transaction;
