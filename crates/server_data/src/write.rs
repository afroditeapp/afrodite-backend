//! Synchronous write commands combining cache and database operations.

use std::sync::Arc;

use config::Config;
use database::{
    current::{read::CurrentSyncReadCommands, write::TransactionConnection}, CurrentWriteHandle, DbReaderRawUsingWriteHandle, DbReaderUsingWriteHandle, DbWriter, DbWriterHistory, DbWriterWithHistory, DieselConnection, DieselDatabaseError, HistoryWriteHandle, PoolObject, TransactionError
};
use server_common::{app::EmailSenderImpl, push_notifications::PushNotificationSender};
use simple_backend::media_backup::MediaBackupHandle;

use self::common::WriteCommandsCommon;
use super::{cache::DatabaseCache, file::utils::FileDir, index::LocationIndexManager};
use crate::db_manager::{SyncWriteHandleRef, SyncWriteHandleRefRef};

macro_rules! define_write_commands {
    ($struct_name:ident) => {
        pub struct $struct_name<C: $crate::write::WriteCommandsProvider> {
            cmds: C,
        }

        impl<C: $crate::write::WriteCommandsProvider> $struct_name<C> {
            pub fn new(cmds: C) -> Self {
                Self { cmds }
            }

            #[allow(dead_code)]
            fn cache(&self) -> &$crate::cache::DatabaseCache {
                &self.cmds.write_cmds().cache
            }

            #[allow(dead_code)]
            fn events(&self) -> $crate::event::EventManagerWithCacheReference<'_> {
                $crate::event::EventManagerWithCacheReference::new(
                    &self.cmds.write_cmds().cache,
                    &self.cmds.write_cmds().push_notification_sender,
                )
            }

            #[allow(dead_code)]
            fn email(&self) -> &server_common::app::EmailSenderImpl {
                &self.cmds.write_cmds().email_sender
            }

            #[allow(dead_code)]
            fn config(&self) -> &config::Config {
                &self.cmds.write_cmds().config
            }

            #[allow(dead_code)]
            fn files(&self) -> &$crate::FileDir {
                &self.cmds.write_cmds().file_dir
            }

            #[allow(dead_code)]
            fn location(&self) -> $crate::index::LocationIndexWriteHandle<'_> {
                $crate::index::LocationIndexWriteHandle::new(self.cmds.write_cmds().location_index)
            }

            #[allow(dead_code)]
            fn location_iterator(&self) -> $crate::index::LocationIndexIteratorHandle<'_> {
                $crate::index::LocationIndexIteratorHandle::new(
                    &self.cmds.write_cmds().location_index,
                )
            }

            #[allow(dead_code)]
            fn media_backup(&self) -> &simple_backend::media_backup::MediaBackupHandle {
                &self.cmds.write_cmds().media_backup
            }

            pub async fn db_transaction<
                T: FnOnce(
                        database::current::write::CurrentSyncWriteCommands<
                            &mut database::DieselConnection,
                        >,
                    ) -> error_stack::Result<R, database::DieselDatabaseError>
                    + Send
                    + 'static,
                R: Send + 'static,
            >(
                &self,
                cmd: T,
            ) -> error_stack::Result<R, database::DieselDatabaseError> {
                self.cmds.write_cmds().db_transaction_common(cmd).await
            }

            pub async fn db_read<
                T: FnOnce(
                        database::current::read::CurrentSyncReadCommands<
                            &mut database::DieselConnection,
                        >,
                    ) -> error_stack::Result<R, database::DieselDatabaseError>
                    + Send
                    + 'static,
                R: Send + 'static,
            >(
                &self,
                cmd: T,
            ) -> error_stack::Result<R, database::DieselDatabaseError> {
                self.cmds.write_cmds().db_read(cmd).await
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
pub struct WriteCommands<'a> {
    pub config: &'a Arc<Config>,
    pub current_write_handle: &'a CurrentWriteHandle,
    pub history_write_handle: &'a HistoryWriteHandle,
    pub cache: &'a DatabaseCache,
    pub file_dir: &'a FileDir,
    pub location_index: &'a LocationIndexManager,
    pub media_backup: &'a MediaBackupHandle,
    pub push_notification_sender: &'a PushNotificationSender,
    pub email_sender: &'a EmailSenderImpl,
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
        email_sender: &'a EmailSenderImpl,
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
            email_sender,
        }
    }

    pub fn common(&self) -> WriteCommandsCommon<&WriteCommands<'_>> {
        WriteCommandsCommon::new(self)
    }

    pub fn into_common(self) -> WriteCommandsCommon<WriteCommandsContainer<'a>> {
        WriteCommandsCommon::new(WriteCommandsContainer::new(self))
    }

    pub async fn db_transaction_common<
        T: FnOnce(
                database::current::write::CurrentSyncWriteCommands<&mut database::DieselConnection>,
            ) -> error_stack::Result<R, DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, DieselDatabaseError> {
        DbWriter::new(self.current_write_handle)
            .db_transaction(cmd)
            .await
    }

    pub async fn db_transaction_raw<
        T: FnOnce(&mut database::DieselConnection) -> error_stack::Result<R, DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, DieselDatabaseError> {
        DbWriter::new(self.current_write_handle)
            .db_transaction_raw(cmd)
            .await
    }

    pub async fn db_transaction_history_raw<
        T: FnOnce(&mut database::DieselConnection) -> error_stack::Result<R, DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, DieselDatabaseError> {
        DbWriterHistory::new(self.history_write_handle)
            .db_transaction_raw(cmd)
            .await
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
        DbWriterWithHistory::new(self.current_write_handle, self.history_write_handle)
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
        DbReaderUsingWriteHandle::new(self.current_write_handle)
            .db_read(cmd)
            .await
    }

    pub async fn db_read_raw<
        T: FnOnce(&mut DieselConnection) -> error_stack::Result<R, DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, DieselDatabaseError> {
        DbReaderRawUsingWriteHandle::new(self.current_write_handle)
            .db_read(cmd)
            .await
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
pub struct WriteCommandsContainer<'a> {
    pub cmds: WriteCommands<'a>,
}

impl<'a> WriteCommandsContainer<'a> {
    pub fn new(cmds: WriteCommands<'a>) -> Self {
        Self { cmds }
    }
}

pub trait WriteCommandsProvider {
    fn write_cmds(&self) -> &WriteCommands;
}

impl<'a> WriteCommandsProvider for WriteCommandsContainer<'a> {
    fn write_cmds(&self) -> &WriteCommands {
        &self.cmds
    }
}

impl<'a> WriteCommandsProvider for &WriteCommands<'a> {
    fn write_cmds(&self) -> &WriteCommands {
        self
    }
}

pub trait GetWriteCommandsCommon<C: WriteCommandsProvider> {
    fn common(self) -> WriteCommandsCommon<C>;
}

impl<C: WriteCommandsProvider> GetWriteCommandsCommon<C> for C {
    fn common(self) -> WriteCommandsCommon<C> {
        WriteCommandsCommon::new(self)
    }
}

impl WriteCommandsProvider for SyncWriteHandleRef<'_> {
    fn write_cmds(&self) -> &WriteCommands {
        &self.write_cmds
    }
}

impl WriteCommandsProvider for SyncWriteHandleRefRef<'_> {
    fn write_cmds(&self) -> &WriteCommands {
        self.handle.write_cmds()
    }
}
