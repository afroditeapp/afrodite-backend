use std::{fmt::Debug, sync::Arc};

use config::Config;
use database::{
    CurrentReadHandle, CurrentWriteHandle, DatabaseHandleCreator, DbReadCloseHandle,
    DbReaderHistoryRaw, DbReaderRaw, DbWriteCloseHandle, DbWriter, DbWriterHistory,
    DbWriterWithHistory, DieselDatabaseError, HistoryReadHandle, HistoryWriteHandle,
    TransactionError, current::write::TransactionConnection,
};
use server_common::{
    app::EmailSenderImpl, data::WithInfo, push_notifications::PushNotificationSender,
    result::Result,
};
pub use server_common::{
    data::{DataError, IntoDataError},
    result,
};
use tracing::info;

use crate::{
    cache::DatabaseCache,
    event::EventManagerWithCacheReference,
    file::utils::FileDir,
    index::{LocationIndexIteratorHandle, LocationIndexManager, LocationIndexWriteHandle},
    profile_attributes::load_profile_attributes_from_db,
    utils::{AccessTokenManager, AccountIdManager},
    write_commands::Cmds,
    write_concurrent::WriteCommandsConcurrent,
};

pub mod handle_types {
    pub use config::Config;
    pub use database::{
        CurrentReadHandle, CurrentWriteHandle, HistoryReadHandle, HistoryWriteHandle,
    };
    pub use server_common::{app::EmailSenderImpl, push_notifications::PushNotificationSender};

    pub type ReadHandleType = super::RouterDatabaseReadHandle;
    pub type WriteHandleType = super::RouterDatabaseWriteHandle;
}

/// Handle SQLite databases and write command runner.
pub struct DatabaseManager {
    current_read_close: DbReadCloseHandle,
    current_write_close: DbWriteCloseHandle,
    history_read_close: DbReadCloseHandle,
    history_write_close: DbWriteCloseHandle,
}

impl DatabaseManager {
    /// Runs also some blocking file system code.
    pub async fn new(
        config: Arc<Config>,
        push_notification_sender: PushNotificationSender,
        email_sender: EmailSenderImpl,
    ) -> Result<(Self, RouterDatabaseReadHandle, RouterDatabaseWriteHandle), DataError> {
        info!("Creating DatabaseManager");

        let databases = config.simple_backend().database_info();
        let get_migrations = || {
            if config.simple_backend().database_config().postgres.is_some() {
                database::DIESEL_POSTGRES_MIGRATIONS
            } else {
                database::DIESEL_SQLITE_MIGRATIONS
            }
        };

        // Write handles

        let (current_write, current_write_close) =
            DatabaseHandleCreator::create_write_handle_from_config(
                config.simple_backend(),
                &databases.current,
                get_migrations(),
            )
            .await?;

        if config.simple_backend().database_config().is_sqlite() {
            let diesel_sqlite = current_write.diesel().sqlite_version().await?;
            info!("Diesel SQLite version: {}", diesel_sqlite);
        }

        let (history_write, history_write_close) =
            DatabaseHandleCreator::create_write_handle_from_config(
                config.simple_backend(),
                &databases.history,
                get_migrations(),
            )
            .await?;

        // Read handles

        let (current_read, current_read_close) =
            DatabaseHandleCreator::create_read_handle_from_config(
                config.simple_backend(),
                &databases.current,
            )
            .await?;

        let (history_read, history_read_close) =
            DatabaseHandleCreator::create_read_handle_from_config(
                config.simple_backend(),
                &databases.history,
            )
            .await?;

        let index = LocationIndexManager::new(config.clone());
        let current_read_handle = CurrentReadHandle(current_read);
        let current_write_handle = CurrentWriteHandle(current_write);
        let history_read_handle = HistoryReadHandle(history_read);
        let history_write_handle = HistoryWriteHandle(history_write);

        let cache = DatabaseCache::new();

        let profile_attributes =
            load_profile_attributes_from_db(&DbReaderRaw::new(&current_read_handle))
                .await
                .into_error_without_context()?;

        let router_write_handle = RouterDatabaseWriteHandle {
            read: RouterDatabaseReadHandle {
                current_read_handle: current_write_handle.to_read_handle(),
                history_read_handle: history_write_handle.to_read_handle(),
                file_dir: FileDir::new(&config)?.into(),
                cache: cache.into(),
                config: config.clone(),
                profile_attributes: profile_attributes.clone(),
            },
            config: config.clone(),
            current_write_handle: current_write_handle.clone(),
            history_write_handle: history_write_handle.clone(),
            location: index.into(),
            profile_attributes: profile_attributes.clone(),
            push_notification_sender,
            email_sender,
        };

        let router_read_handle = RouterDatabaseReadHandle {
            current_read_handle: current_read_handle.clone(),
            history_read_handle: history_read_handle.clone(),
            file_dir: router_write_handle.read.file_dir.clone(),
            cache: router_write_handle.read.cache.clone(),
            config,
            profile_attributes,
        };

        let database_manager = DatabaseManager {
            current_write_close,
            current_read_close,
            history_write_close,
            history_read_close,
        };

        info!("DatabaseManager created");

        Ok((database_manager, router_read_handle, router_write_handle))
    }

    pub async fn close(self) {
        self.current_read_close.close().await;
        self.current_write_close.close().await;
        self.history_read_close.close().await;
        self.history_write_close.close().await;
    }
}

#[derive(Debug)]
pub struct RouterDatabaseWriteHandle {
    /// This is actually the write handle
    read: RouterDatabaseReadHandle,
    config: Arc<Config>,
    current_write_handle: CurrentWriteHandle,
    history_write_handle: HistoryWriteHandle,
    location: Arc<LocationIndexManager>,
    profile_attributes: crate::profile_attributes::ProfileAttributesSchemaManager,
    push_notification_sender: PushNotificationSender,
    email_sender: EmailSenderImpl,
}

impl RouterDatabaseWriteHandle {
    pub fn user_write_commands_account(&self) -> WriteCommandsConcurrent<'_> {
        WriteCommandsConcurrent::new(
            &self.read.cache,
            &self.read.file_dir,
            LocationIndexIteratorHandle::new(&self.location),
            &self.profile_attributes,
        )
    }

    pub fn read(&self) -> ReadAdapter<'_> {
        ReadAdapter::new(self)
    }

    pub fn events(&self) -> EventManagerWithCacheReference<'_> {
        EventManagerWithCacheReference::new(&self.read.cache, &self.push_notification_sender)
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn location_raw(&self) -> &LocationIndexManager {
        &self.location
    }
}

pub trait InternalWriting {
    fn config(&self) -> &Config;
    fn config_arc(&self) -> Arc<Config>;
    fn file_dir(&self) -> &FileDir;
    fn current_write_handle(&self) -> &CurrentWriteHandle;
    fn history_write_handle(&self) -> &HistoryWriteHandle;
    fn current_read_handle(&self) -> &CurrentReadHandle;
    fn history_read_handle(&self) -> &HistoryReadHandle;
    fn cache(&self) -> &DatabaseCache;
    fn location(&self) -> &LocationIndexManager;
    fn push_notification_sender(&self) -> &PushNotificationSender;
    fn email_sender(&self) -> &EmailSenderImpl;
    fn events(&self) -> EventManagerWithCacheReference<'_>;
    fn profile_attributes(&self) -> &crate::profile_attributes::ProfileAttributesSchemaManager;

    fn location_index_write_handle(&self) -> LocationIndexWriteHandle<'_> {
        LocationIndexWriteHandle::new(self.location())
    }

    async fn db_transaction_raw<
        T: FnOnce(database::DbWriteMode<'_>) -> error_stack::Result<R, DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, DieselDatabaseError> {
        DbWriter::new(self.current_write_handle())
            .db_transaction_raw(cmd)
            .await
    }

    async fn db_transaction_history_raw<
        T: FnOnce(database::DbWriteModeHistory<'_>) -> error_stack::Result<R, DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, DieselDatabaseError> {
        DbWriterHistory::new(self.history_write_handle())
            .db_transaction_raw(cmd)
            .await
    }

    async fn db_transaction_with_history<T, R: Send + 'static>(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, DieselDatabaseError>
    where
        T: FnOnce(
                TransactionConnection<'_>,
                database::DbWriteModeHistory<'_>,
            ) -> std::result::Result<R, TransactionError>
            + Send
            + 'static,
    {
        DbWriterWithHistory::new(self.current_write_handle(), self.history_write_handle())
            .db_transaction_with_history(cmd)
            .await
    }

    async fn db_read_raw<
        T: FnOnce(database::DbReadMode<'_>) -> error_stack::Result<R, DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, DieselDatabaseError> {
        DbReaderRaw::new(self.current_read_handle())
            .db_read(cmd)
            .await
    }
}

impl InternalWriting for RouterDatabaseWriteHandle {
    fn config(&self) -> &Config {
        &self.config
    }

    fn config_arc(&self) -> Arc<Config> {
        self.config.clone()
    }

    fn file_dir(&self) -> &FileDir {
        &self.read.file_dir
    }

    fn current_write_handle(&self) -> &CurrentWriteHandle {
        &self.current_write_handle
    }

    fn history_write_handle(&self) -> &HistoryWriteHandle {
        &self.history_write_handle
    }

    fn current_read_handle(&self) -> &CurrentReadHandle {
        &self.read.current_read_handle
    }

    fn history_read_handle(&self) -> &HistoryReadHandle {
        &self.read.history_read_handle
    }

    fn profile_attributes(&self) -> &crate::profile_attributes::ProfileAttributesSchemaManager {
        &self.profile_attributes
    }

    fn cache(&self) -> &DatabaseCache {
        &self.read.cache
    }

    fn location(&self) -> &LocationIndexManager {
        &self.location
    }

    fn push_notification_sender(&self) -> &PushNotificationSender {
        &self.push_notification_sender
    }

    fn email_sender(&self) -> &EmailSenderImpl {
        &self.email_sender
    }

    fn events(&self) -> EventManagerWithCacheReference<'_> {
        EventManagerWithCacheReference::new(&self.read.cache, &self.push_notification_sender)
    }
}

impl InternalWriting for Cmds {
    fn config(&self) -> &Config {
        InternalWriting::config(self.write_handle())
    }

    fn config_arc(&self) -> Arc<Config> {
        InternalWriting::config_arc(self.write_handle())
    }

    fn file_dir(&self) -> &FileDir {
        InternalWriting::file_dir(self.write_handle())
    }

    fn current_write_handle(&self) -> &CurrentWriteHandle {
        InternalWriting::current_write_handle(self.write_handle())
    }

    fn history_write_handle(&self) -> &HistoryWriteHandle {
        InternalWriting::history_write_handle(self.write_handle())
    }

    fn current_read_handle(&self) -> &CurrentReadHandle {
        InternalWriting::current_read_handle(self.write_handle())
    }

    fn history_read_handle(&self) -> &HistoryReadHandle {
        InternalWriting::history_read_handle(self.write_handle())
    }

    fn cache(&self) -> &DatabaseCache {
        InternalWriting::cache(self.write_handle())
    }

    fn location(&self) -> &LocationIndexManager {
        InternalWriting::location(self.write_handle())
    }

    fn push_notification_sender(&self) -> &PushNotificationSender {
        InternalWriting::push_notification_sender(self.write_handle())
    }

    fn email_sender(&self) -> &EmailSenderImpl {
        InternalWriting::email_sender(self.write_handle())
    }

    fn events(&self) -> EventManagerWithCacheReference<'_> {
        InternalWriting::events(self.write_handle())
    }

    fn profile_attributes(&self) -> &crate::profile_attributes::ProfileAttributesSchemaManager {
        InternalWriting::profile_attributes(self.write_handle())
    }
}

pub trait WriteAccessProvider {
    fn handle(&self) -> &RouterDatabaseWriteHandle;
}
impl WriteAccessProvider for &RouterDatabaseWriteHandle {
    fn handle(&self) -> &RouterDatabaseWriteHandle {
        self
    }
}
impl WriteAccessProvider for Cmds {
    fn handle(&self) -> &RouterDatabaseWriteHandle {
        self.write_handle()
    }
}

/// Read only access to SQLite databases and
/// read and write access to cache.
#[derive(Debug)]
pub struct RouterDatabaseReadHandle {
    file_dir: Arc<FileDir>,
    current_read_handle: CurrentReadHandle,
    history_read_handle: HistoryReadHandle,
    cache: Arc<DatabaseCache>,
    config: Arc<Config>,
    profile_attributes: crate::profile_attributes::ProfileAttributesSchemaManager,
}

impl RouterDatabaseReadHandle {
    pub fn access_token_manager(&self) -> AccessTokenManager<'_> {
        AccessTokenManager::new(&self.cache)
    }

    pub fn account_id_manager(&self) -> AccountIdManager<'_> {
        AccountIdManager::new(&self.cache)
    }

    pub fn profile_attributes(&self) -> &crate::profile_attributes::ProfileAttributesSchemaManager {
        &self.profile_attributes
    }

    pub fn cache_read_write_access(&self) -> &DatabaseCache {
        &self.cache
    }

    pub fn file_dir_write_access(&self) -> &FileDir {
        &self.file_dir
    }

    pub fn read_handle_raw(&self) -> &CurrentReadHandle {
        &self.current_read_handle
    }
}

pub struct ReadAdapter<'a> {
    pub cmds: &'a RouterDatabaseWriteHandle,
}

impl<'a> ReadAdapter<'a> {
    pub fn new(cmds: &'a RouterDatabaseWriteHandle) -> Self {
        Self { cmds }
    }
}

pub trait ReadAccessProvider<'a> {
    fn handle(self) -> &'a RouterDatabaseReadHandle;
}
impl<'a> ReadAccessProvider<'a> for &'a RouterDatabaseReadHandle {
    fn handle(self) -> &'a RouterDatabaseReadHandle {
        self
    }
}
impl<'a> ReadAccessProvider<'a> for ReadAdapter<'a> {
    fn handle(self) -> &'a RouterDatabaseReadHandle {
        &self.cmds.read
    }
}

pub trait InternalReading {
    fn file_dir(&self) -> &FileDir;
    fn current_read_handle(&self) -> &CurrentReadHandle;
    fn history_read_handle(&self) -> &HistoryReadHandle;
    fn cache(&self) -> &DatabaseCache;
    fn config(&self) -> &Config;
    fn config_arc(&self) -> Arc<Config>;
    fn profile_attributes(&self) -> &crate::profile_attributes::ProfileAttributesSchemaManager;

    async fn db_read_raw<
        T: FnOnce(database::DbReadMode<'_>) -> error_stack::Result<R, DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, DieselDatabaseError> {
        DbReaderRaw::new(self.current_read_handle())
            .db_read(cmd)
            .await
    }

    /// Use only for database backups
    async fn db_read_raw_no_transaction<
        T: FnOnce(database::DbReadMode<'_>) -> error_stack::Result<R, DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, DieselDatabaseError> {
        DbReaderRaw::new(self.current_read_handle())
            .db_read_no_transaction(cmd)
            .await
    }

    async fn db_read_history_raw<
        T: FnOnce(database::DbReadModeHistory<'_>) -> error_stack::Result<R, DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, DieselDatabaseError> {
        DbReaderHistoryRaw::new(self.history_read_handle())
            .db_read_history(cmd)
            .await
    }

    /// Use only for database backups
    async fn db_read_history_raw_no_transaction<
        T: FnOnce(database::DbReadModeHistory<'_>) -> error_stack::Result<R, DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, DieselDatabaseError> {
        DbReaderHistoryRaw::new(self.history_read_handle())
            .db_read_history_no_transaction(cmd)
            .await
    }
}

impl InternalReading for &RouterDatabaseReadHandle {
    fn file_dir(&self) -> &FileDir {
        &self.file_dir
    }

    fn current_read_handle(&self) -> &CurrentReadHandle {
        &self.current_read_handle
    }

    fn history_read_handle(&self) -> &HistoryReadHandle {
        &self.history_read_handle
    }

    fn cache(&self) -> &DatabaseCache {
        &self.cache
    }

    fn config(&self) -> &Config {
        &self.config
    }

    fn config_arc(&self) -> Arc<Config> {
        self.config.clone()
    }

    fn profile_attributes(&self) -> &crate::profile_attributes::ProfileAttributesSchemaManager {
        &self.profile_attributes
    }
}

impl<I: InternalWriting> InternalReading for I {
    fn file_dir(&self) -> &FileDir {
        self.file_dir()
    }

    fn current_read_handle(&self) -> &CurrentReadHandle {
        self.current_read_handle()
    }

    fn history_read_handle(&self) -> &HistoryReadHandle {
        self.history_read_handle()
    }

    fn cache(&self) -> &DatabaseCache {
        self.cache()
    }

    fn config(&self) -> &Config {
        InternalWriting::config(self)
    }

    fn config_arc(&self) -> Arc<Config> {
        InternalWriting::config_arc(self)
    }

    fn profile_attributes(&self) -> &crate::profile_attributes::ProfileAttributesSchemaManager {
        InternalWriting::profile_attributes(self)
    }
}

impl InternalReading for ReadAdapter<'_> {
    fn file_dir(&self) -> &FileDir {
        &self.cmds.read.file_dir
    }

    fn current_read_handle(&self) -> &CurrentReadHandle {
        &self.cmds.read.current_read_handle
    }

    fn history_read_handle(&self) -> &HistoryReadHandle {
        &self.cmds.read.history_read_handle
    }

    fn cache(&self) -> &DatabaseCache {
        &self.cmds.read.cache
    }

    fn config(&self) -> &Config {
        &self.cmds.read.config
    }

    fn config_arc(&self) -> Arc<Config> {
        self.cmds.read.config.clone()
    }

    fn profile_attributes(&self) -> &crate::profile_attributes::ProfileAttributesSchemaManager {
        &self.cmds.read.profile_attributes
    }
}
