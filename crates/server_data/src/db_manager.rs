use std::{fmt::Debug, fs, path::Path, sync::Arc};

use config::Config;
use database::{
    current::write::TransactionConnection, CurrentReadHandle, CurrentWriteHandle, DatabaseHandleCreator, DbReadCloseHandle, DbReaderHistoryRaw, DbReaderRaw, DbReaderRawUsingWriteHandle, DbWriteCloseHandle, DbWriter, DbWriterHistory, DbWriterWithHistory, DieselConnection, DieselDatabaseError, HistoryReadHandle, HistoryWriteHandle, PoolObject, TransactionError
};
pub use server_common::{
    data::{DataError, IntoDataError},
    result,
};
use server_common::{app::EmailSenderImpl, push_notifications::PushNotificationSender, result::Result};
use simple_backend::media_backup::MediaBackupHandle;
use tracing::info;

use crate::{
    cache::DatabaseCache, event::EventManagerWithCacheReference, file::utils::FileDir, index::{LocationIndexIteratorHandle, LocationIndexManager, LocationIndexWriteHandle}, utils::{AccessTokenManager, AccountIdManager}, write_concurrent::WriteCommandsConcurrent
};

pub const DB_FILE_DIR_NAME: &str = "files";

/// Absolsute path to database root directory.
#[derive(Clone, Debug)]
pub struct DatabaseRoot {
    file_dir: FileDir,
}

impl DatabaseRoot {
    pub fn new<T: AsRef<Path>>(path: T) -> Result<Self, DataError> {
        let root = path.as_ref().to_path_buf();
        if !root.exists() {
            fs::create_dir(&root)?;
        }

        let file_dir = root.join(DB_FILE_DIR_NAME);
        if !file_dir.exists() {
            fs::create_dir(&file_dir)?;
        }
        let file_dir = FileDir::new(file_dir);

        Ok(Self { file_dir })
    }

    pub fn file_dir(&self) -> &FileDir {
        &self.file_dir
    }
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
    pub async fn new<T: AsRef<Path>>(
        database_dir: T,
        config: Arc<Config>,
        media_backup: MediaBackupHandle,
        push_notification_sender: PushNotificationSender,
        email_sender: EmailSenderImpl,
    ) -> Result<(Self, RouterDatabaseReadHandle, RouterDatabaseWriteHandle), DataError> {
        info!("Creating DatabaseManager");

        let root = DatabaseRoot::new(database_dir)?;

        // Write handles

        let (current_write, current_write_close) =
            DatabaseHandleCreator::create_write_handle_from_config(
                config.simple_backend(),
                "current",
                database::DIESEL_MIGRATIONS,
            )
            .await?;

        let diesel_sqlite = current_write.diesel().sqlite_version().await?;
        info!("Diesel SQLite version: {}", diesel_sqlite);

        let (history_write, history_write_close) =
            DatabaseHandleCreator::create_write_handle_from_config(
                config.simple_backend(),
                "history",
                database::DIESEL_MIGRATIONS,
            )
            .await?;

        // Read handles

        let (current_read, current_read_close) =
            DatabaseHandleCreator::create_read_handle_from_config(
                config.simple_backend(),
                "current",
            )
            .await?;

        let (history_read, history_read_close) =
            DatabaseHandleCreator::create_read_handle_from_config(
                config.simple_backend(),
                "history",
            )
            .await?;

        let index = LocationIndexManager::new(config.clone());
        let current_read_handle = CurrentReadHandle(current_read);
        let current_write_handle = CurrentWriteHandle(current_write);
        let history_read_handle = HistoryReadHandle(history_read);
        let history_write_handle = HistoryWriteHandle(history_write);

        // let cache = DatabaseCache::new(&current_read_handle, &index,
        // &config).await?;
        let cache = DatabaseCache::new();

        let router_write_handle = RouterDatabaseWriteHandle {
            config: config.clone(),
            current_write_handle: current_write_handle.clone(),
            history_write_handle: history_write_handle.clone(),
            current_read_handle: current_write_handle.to_read_handle(),
            history_read_handle: history_write_handle.to_read_handle(),
            root: root.into(),
            cache: cache.into(),
            location: index.into(),
            media_backup,
            push_notification_sender,
            email_sender,
        };

        let root = router_write_handle.root.clone();
        let cache = router_write_handle.cache.clone();
        let router_read_handle = RouterDatabaseReadHandle {
            current_read_handle: current_read_handle.clone(),
            history_read_handle: history_read_handle.clone(),
            root,
            cache,
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

#[derive(Clone, Debug)]
pub struct RouterDatabaseWriteHandle {
    config: Arc<Config>,
    root: Arc<DatabaseRoot>,
    current_write_handle: CurrentWriteHandle,
    history_write_handle: HistoryWriteHandle,
    /// This is actually the write handle
    current_read_handle: CurrentReadHandle,
    /// This is actually the write handle
    history_read_handle: HistoryReadHandle,
    cache: Arc<DatabaseCache>,
    location: Arc<LocationIndexManager>,
    media_backup: MediaBackupHandle,
    push_notification_sender: PushNotificationSender,
    email_sender: EmailSenderImpl,
}

impl RouterDatabaseWriteHandle {
    pub fn user_write_commands_account(&self) -> WriteCommandsConcurrent {
        WriteCommandsConcurrent::new(
            &self.cache,
            &self.root.file_dir,
            LocationIndexIteratorHandle::new(&self.location),
        )
    }

    pub fn read(&self) -> ReadAdapter<'_> {
        ReadAdapter::new(
            self
        )
    }

    pub fn events(&self) -> EventManagerWithCacheReference<'_> {
        EventManagerWithCacheReference::new(&self.cache, &self.push_notification_sender)
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn location_raw(&self) -> &LocationIndexManager {
        &self.location
    }
}

pub struct RouterDatabaseWriteHandleRef<'a> {
    pub handle: &'a RouterDatabaseWriteHandle,
}

pub trait InternalWriting {
    fn config(&self) -> &Config;
    fn config_arc(&self) -> Arc<Config>;
    fn root(&self) -> &DatabaseRoot;
    fn current_write_handle(&self) -> &CurrentWriteHandle;
    fn history_write_handle(&self) -> &HistoryWriteHandle;
    fn current_read_handle(&self) -> &CurrentReadHandle;
    fn history_read_handle(&self) -> &HistoryReadHandle;
    fn cache(&self) -> &DatabaseCache;
    fn location(&self) -> &LocationIndexManager;
    fn media_backup(&self) -> &MediaBackupHandle;
    fn push_notification_sender(&self) -> &PushNotificationSender;
    fn email_sender(&self) -> &EmailSenderImpl;

    fn location_index_write_handle(&self) -> LocationIndexWriteHandle {
        LocationIndexWriteHandle::new(self.location())
    }

    async fn db_transaction_raw<
        T: FnOnce(&mut database::DieselConnection) -> error_stack::Result<R, DieselDatabaseError>
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
        T: FnOnce(&mut database::DieselConnection) -> error_stack::Result<R, DieselDatabaseError>
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
                PoolObject,
            ) -> std::result::Result<R, TransactionError>
            + Send
            + 'static,
    {
        DbWriterWithHistory::new(self.current_write_handle(), self.history_write_handle())
            .db_transaction_with_history(cmd)
            .await
    }

    async fn db_read_raw<
        T: FnOnce(&mut DieselConnection) -> error_stack::Result<R, DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, DieselDatabaseError> {
        DbReaderRawUsingWriteHandle::new(self.current_write_handle())
            .db_read(cmd)
            .await
    }
}

impl InternalWriting for &RouterDatabaseWriteHandle {
    fn config(&self) -> &Config {
        &self.config
    }

    fn config_arc(&self) -> Arc<Config> {
        self.config.clone()
    }

    fn root(&self) -> &DatabaseRoot {
        &self.root
    }

    fn current_write_handle(&self) -> &CurrentWriteHandle {
        &self.current_write_handle
    }

    fn history_write_handle(&self) -> &HistoryWriteHandle {
        &self.history_write_handle
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

    fn location(&self) -> &LocationIndexManager {
        &self.location
    }

    fn media_backup(&self) -> &MediaBackupHandle {
        &self.media_backup
    }

    fn push_notification_sender(&self) -> &PushNotificationSender {
        &self.push_notification_sender
    }

    fn email_sender(&self) -> &EmailSenderImpl {
        &self.email_sender
    }
}

pub trait WriteAccessProvider {}
impl WriteAccessProvider for &RouterDatabaseWriteHandle {}

pub struct RouterDatabaseReadHandle {
    root: Arc<DatabaseRoot>,
    current_read_handle: CurrentReadHandle,
    history_read_handle: HistoryReadHandle,
    cache: Arc<DatabaseCache>,
}

impl RouterDatabaseReadHandle {
    pub fn access_token_manager(&self) -> AccessTokenManager<'_> {
        AccessTokenManager::new(&self.cache)
    }

    pub fn account_id_manager(&self) -> AccountIdManager<'_> {
        AccountIdManager::new(&self.cache)
    }

    pub fn cache(&self) -> &DatabaseCache {
        &self.cache
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

pub trait ReadAccessProvider {}
impl ReadAccessProvider for &RouterDatabaseReadHandle {}
impl ReadAccessProvider for ReadAdapter<'_> {}

pub trait InternalReading {
    fn root(&self) -> &DatabaseRoot;
    fn current_read_handle(&self) -> &CurrentReadHandle;
    fn history_read_handle(&self) -> &HistoryReadHandle;
    fn cache(&self) -> &DatabaseCache;

    async fn db_read_raw<
        T: FnOnce(&mut DieselConnection) -> error_stack::Result<R, DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, DieselDatabaseError> {
        DbReaderRaw::new(self.current_read_handle()).db_read(cmd).await
    }

    async fn db_read_history_raw<
        T: FnOnce(&mut DieselConnection) -> error_stack::Result<R, DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, DieselDatabaseError> {
        DbReaderHistoryRaw::new(self.history_read_handle()).db_read_history(cmd).await
    }
}


impl InternalReading for &RouterDatabaseReadHandle {
    fn root(&self) -> &DatabaseRoot {
        &self.root
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
}

impl <I: InternalWriting> InternalReading for I {
    fn root(&self) -> &DatabaseRoot {
        self.root()
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
}


impl InternalReading for ReadAdapter<'_> {
    fn root(&self) -> &DatabaseRoot {
        &self.cmds.root
    }

    fn current_read_handle(&self) -> &CurrentReadHandle {
        &self.cmds.current_read_handle
    }

    fn history_read_handle(&self) -> &HistoryReadHandle {
        &self.cmds.history_read_handle
    }

    fn cache(&self) -> &DatabaseCache {
        &self.cmds.cache
    }
}
