//! Synchronous write commands combining cache and database operations.

use std::{ops::DerefMut, sync::Arc};

use config::Config;
use database::{
    current::{
        read::CurrentSyncReadCommands,
        write::TransactionConnection,
    }, history::write::HistorySyncWriteCommands, CurrentWriteHandle, DbReaderRaw, DbReaderRawUsingWriteHandle, DbReaderUsingWriteHandle, DbWriter, DbWriterWithHistory, DieselConnection, DieselDatabaseError, HistoryWriteHandle, PoolObject, TransactionError
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

macro_rules! define_write_commands {
    ($struct_name:ident) => {
        pub struct $struct_name<'a> {
            cmds: $crate::write::WriteCommands<'a>,
        }

        impl<'a> $struct_name<'a> {
            pub fn new(cmds: $crate::write::WriteCommands<'a>) -> Self {
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
            fn cache(&self) -> &$crate::cache::DatabaseCache {
                &self.cmds.cache
            }

            #[allow(dead_code)]
            fn events(&self) -> $crate::event::EventManagerWithCacheReference<'_> {
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
            fn file_dir(&self) -> &$crate::FileDir {
                &self.cmds.file_dir
            }

            #[allow(dead_code)]
            fn location(&self) -> $crate::index::LocationIndexWriteHandle<'a> {
                $crate::index::LocationIndexWriteHandle::new(&self.cmds.location_index)
            }

            #[allow(dead_code)]
            fn location_iterator(&self) -> $crate::index::LocationIndexIteratorHandle<'a> {
                $crate::index::LocationIndexIteratorHandle::new(&self.cmds.location_index)
            }

            #[allow(dead_code)]
            fn media_backup(&self) -> &simple_backend::media_backup::MediaBackupHandle {
                &self.cmds.media_backup
            }

            pub async fn db_transaction<
                T: FnOnce(
                        database::current::write::CurrentSyncWriteCommands<
                            &mut database::DieselConnection,
                        >,
                    ) -> error_stack::Result<
                        R,
                        database::DieselDatabaseError,
                    > + Send
                    + 'static,
                R: Send + 'static,
            >(
                &self,
                cmd: T,
            ) -> error_stack::Result<R, database::DieselDatabaseError>
            {
                self.cmds.db_transaction_common(cmd).await
            }

            pub async fn db_read<
                T: FnOnce(
                        database::current::read::CurrentSyncReadCommands<
                            &mut database::DieselConnection,
                        >,
                    ) -> error_stack::Result<
                        R,
                        database::DieselDatabaseError,
                    > + Send
                    + 'static,
                R: Send + 'static,
            >(
                &self,
                cmd: T,
            ) -> error_stack::Result<R, database::DieselDatabaseError>
            {
                self.cmds.db_read(cmd).await
            }

            pub async fn write_cache<T, Id: Into<model::AccountId>>(
                &self,
                id: Id,
                cache_operation: impl FnOnce(
                    &mut $crate::cache::CacheEntry,
                ) -> error_stack::Result<T, $crate::CacheError>,
            ) -> error_stack::Result<T, $crate::CacheError> {
                self.cache().write_cache(id, cache_operation).await
            }
        }
    };
}

pub mod common;

/// One Account can do only one write command at a time.
pub struct AccountWriteLock;

// TODO: Perhaps in the future server_data crates could use
// database style accessor structs so #[derive(Clone)] would not
// be needed.

/// Globally synchronous write commands.
#[derive(Clone)]
pub struct WriteCommands<'a> {
    pub config: &'a Arc<Config>,
    pub current_write_handle: &'a CurrentWriteHandle,
    pub history_write_handle: &'a HistoryWriteHandle,
    pub cache: &'a DatabaseCache,
    pub file_dir: &'a FileDir,
    pub location_index: &'a LocationIndexManager,
    pub media_backup: &'a MediaBackupHandle,
    pub push_notification_sender: &'a PushNotificationSender,
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

    pub async fn db_transaction_common<
        T: FnOnce(
                database::current::write::CurrentSyncWriteCommands<
                    &mut database::DieselConnection,
                >,
            ) -> error_stack::Result<R, DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, DieselDatabaseError> {
        DbWriter::new(self.current_write_handle).db_transaction(cmd).await
    }

    pub async fn db_transaction_raw<
        T: FnOnce(
                &mut database::DieselConnection,
            ) -> error_stack::Result<R, DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, DieselDatabaseError> {
        DbWriter::new(self.current_write_handle).db_transaction_raw(cmd).await
    }

    pub async fn db_transaction_with_history<T, R: Send + 'static>(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, DieselDatabaseError>
    where
        T: FnOnce(
                TransactionConnection<'_>,
                PoolObject,
            ) -> std::result::Result<R, TransactionError>
            + Send
            + 'static,
    {
        DbWriterWithHistory::new(
            self.current_write_handle,
            self.history_write_handle
        )
            .db_transaction_with_history(cmd)
            .await
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
        DbReaderUsingWriteHandle::new(self.current_write_handle).db_read(cmd).await
    }

    pub async fn db_read_raw<
        T: FnOnce(
                &mut DieselConnection,
            ) -> error_stack::Result<R, DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, DieselDatabaseError> {
        DbReaderRawUsingWriteHandle::new(self.current_write_handle).db_read(cmd).await
    }
}

/// Macro for writing to current database with transaction.
/// Calls await automatically.
///
/// ```ignore
/// use server::DataError;
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
        $crate::IntoDataError::into_error($state.db_transaction(move |mut $cmds| ($commands)).await)
    }};
    ($state:expr, move |$cmds:ident| $commands:expr) => {{
        $crate::data::IntoDataError::into_error(
            $state.db_transaction(move |$cmds| ($commands)).await,
        )
    }};
}

// Make db_transaction available in all modules
pub(crate) use db_transaction;



// pub fn account(self) -> WriteCommandsAccount<'a> {
//     WriteCommandsAccount::new(self)
// }

// pub fn account_admin(self) -> WriteCommandsAccountAdmin<'a> {
//     WriteCommandsAccountAdmin::new(self)
// }

// pub fn media(self) -> WriteCommandsMedia<'a> {
//     WriteCommandsMedia::new(self)
// }

// pub fn media_admin(self) -> WriteCommandsMediaAdmin<'a> {
//     WriteCommandsMediaAdmin::new(self)
// }

// pub fn profile(self) -> WriteCommandsProfile<'a> {
//     WriteCommandsProfile::new(self)
// }

// pub fn profile_admin(self) -> WriteCommandsProfileAdmin<'a> {
//     WriteCommandsProfileAdmin::new(self)
// }

// pub fn chat(self) -> WriteCommandsChat<'a> {
//     WriteCommandsChat::new(self)
// }

// pub fn chat_admin(self) -> WriteCommandsChatAdmin<'a> {
//     WriteCommandsChatAdmin::new(self)
// }
