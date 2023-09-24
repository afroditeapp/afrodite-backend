use std::{
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use ::utils::ContextExt;
use config::Config;
use database::{
    current::read::SqliteReadCommands,
    diesel::{
        DieselCurrentReadHandle, DieselCurrentWriteHandle, DieselHistoryReadHandle,
        DieselHistoryWriteHandle, DieselReadCloseHandle, DieselReadHandle, DieselWriteCloseHandle,
        DieselWriteHandle,
    },
    history::read::HistoryReadCommands,
    sqlite::{
        CurrentDataWriteHandle, DatabaseType, HistoryWriteHandle, SqliteDatabasePath,
        SqliteWriteCloseHandle, SqliteWriteHandle, SqlxReadCloseHandle, SqlxReadHandle,
    },
    ErrorContext,
};
use error_stack::{Context, ResultExt, Result};
use model::{AccountId, AccountIdInternal, IsLoggingAllowed, SignInWithInfo};
use tracing::info;

use self::{
    cache::{CacheError, DatabaseCache},
    file::{read::FileReadCommands, utils::FileDir, FileError},
    index::{LocationIndexManager, LocationIndexIteratorHandle},
    read::ReadCommands,
    utils::{AccessTokenManager, AccountIdManager},
    write::{
        account::WriteCommandsAccount, account_admin::WriteCommandsAccountAdmin,
        chat::WriteCommandsChat, chat_admin::WriteCommandsChatAdmin, common::WriteCommandsCommon,
        media::WriteCommandsMedia, media_admin::WriteCommandsMediaAdmin,
        profile::WriteCommandsProfile, profile_admin::WriteCommandsProfileAdmin, WriteCommands,
    },
    write_concurrent::WriteCommandsConcurrent,
};
use crate::{internal::InternalApiError, media_backup::MediaBackupHandle};

pub mod cache;
pub mod file;
pub mod index;
pub mod read;
pub mod utils;
pub mod write;
pub mod write_commands;
pub mod write_concurrent;

pub const DB_HISTORY_DIR_NAME: &str = "history";
pub const DB_CURRENT_DATA_DIR_NAME: &str = "current";
pub const DB_FILE_DIR_NAME: &str = "files";

pub type DatabeseEntryId = String;

#[derive(thiserror::Error, Debug)]
pub enum DataError {
    #[error("Git error")]
    Git,
    #[error("SQLite error")]
    Sqlite,
    #[error("Cache error")]
    Cache,
    #[error("File error")]
    File,
    #[error("Profile index error")]
    ProfileIndex,
    #[error("Media backup error")]
    MediaBackup,
    #[error("Image process error")]
    ImageProcess,

    #[error("Diesel error")]
    Diesel,

    #[error("Database command sending failed")]
    CommandSendingFailed,
    #[error("Database command result receiving failed")]
    CommandResultReceivingFailed,

    // Other errors
    #[error("Database initialization error")]
    Init,
    #[error("Database SQLite and Git integrity check")]
    Integrity,
    #[error("Feature disabled from config file")]
    FeatureDisabled,
    #[error("Not found")]
    NotFound,
    #[error("Content slot not empty")]
    ContentSlotNotEmpty,

    #[error("Command runner quit too early")]
    CommandRunnerQuit,

    #[error("Different SQLite versions detected between diesel and sqlx")]
    SqliteVersionMismatch,
}

impl DataError {
    #[track_caller]
    pub fn report(self) -> error_stack::Report<Self> {
        error_stack::report!(self)
    }
}
pub trait WithInfo<Ok, Err: Context>: Sized {
    fn into_error_without_context(self) -> Result<Ok, Err>;

    #[track_caller]
    fn with_info<T: Debug + IsLoggingAllowed>(self, request_context: T) -> Result<Ok, Err> {
        self.into_error_without_context()
            .attach_printable_lazy(move || {
                let context = ErrorContext::<T, Ok>::new(request_context);

                format!("{:#?}", context)
            })
    }
}

impl<Ok> WithInfo<Ok, DataError> for Result<Ok, DataError> {
    #[track_caller]
    fn into_error_without_context(self) -> Result<Ok, DataError> {
        self
    }
}
impl<Ok> WithInfo<Ok, CacheError> for Result<Ok, CacheError> {
    #[track_caller]
    fn into_error_without_context(self) -> Result<Ok, CacheError> {
        self
    }
}
impl<Ok> WithInfo<Ok, InternalApiError> for Result<Ok, InternalApiError> {
    #[track_caller]
    fn into_error_without_context(self) -> Result<Ok, InternalApiError> {
        self
    }
}
impl<Ok> WithInfo<Ok, InternalApiError> for std::result::Result<Ok, InternalApiError> {
    #[track_caller]
    fn into_error_without_context(self) -> Result<Ok, InternalApiError> {
        self.map_err(|e| e.report())
    }
}

pub trait IntoDataError<Ok, Err: Context>: Sized {
    fn into_data_error_without_context(self) -> Result<Ok, Err>;

    #[track_caller]
    fn into_data_error<T: Debug + IsLoggingAllowed>(self, request_context: T) -> Result<Ok, Err> {
        self.into_data_error_without_context()
            .attach_printable_lazy(move || {
                let context = ErrorContext::<T, Ok>::new(request_context);

                format!("{:#?}", context)
            })
    }
}

impl<Ok> IntoDataError<Ok, DataError> for Result<Ok, crate::data::file::FileError> {
    #[track_caller]
    fn into_data_error_without_context(self) -> Result<Ok, DataError> {
        self.change_context(DataError::File)
    }
}

impl<Ok> IntoDataError<Ok, DataError> for Result<Ok, crate::data::cache::CacheError> {
    #[track_caller]
    fn into_data_error_without_context(self) -> Result<Ok, DataError> {
        self.change_context(DataError::Cache)
    }
}

/// Absolsute path to database root directory.
#[derive(Clone, Debug)]
pub struct DatabaseRoot {
    root: PathBuf,
    history: SqliteDatabasePath,
    current: SqliteDatabasePath,
    file_dir: FileDir,
}

impl DatabaseRoot {
    pub fn new<T: AsRef<Path>>(path: T) -> Result<Self, DataError> {
        let root = path.as_ref().to_path_buf();
        if !root.exists() {
            fs::create_dir(&root).change_context(DataError::Init)?;
        }

        let history = root.join(DB_HISTORY_DIR_NAME);
        if !history.exists() {
            fs::create_dir(&history).change_context(DataError::Init)?;
        }
        let history = SqliteDatabasePath::new(history);

        let current = root.join(DB_CURRENT_DATA_DIR_NAME);
        if !current.exists() {
            fs::create_dir(&current).change_context(DataError::Init)?;
        }
        let current = SqliteDatabasePath::new(current);

        let file_dir = root.join(DB_FILE_DIR_NAME);
        if !file_dir.exists() {
            fs::create_dir(&file_dir).change_context(DataError::Init)?;
        }
        let file_dir = FileDir::new(file_dir);

        Ok(Self {
            root,
            history,
            current,
            file_dir,
        })
    }

    /// History Sqlite database path
    pub fn history(&self) -> SqliteDatabasePath {
        self.history.clone()
    }

    pub fn history_ref(&self) -> &SqliteDatabasePath {
        &self.history
    }

    /// Sqlite database path
    pub fn current(&self) -> SqliteDatabasePath {
        self.current.clone()
    }

    pub fn current_ref(&self) -> &SqliteDatabasePath {
        &self.current
    }

    pub fn file_dir(&self) -> &FileDir {
        &self.file_dir
    }

    pub fn current_db_file(&self) -> PathBuf {
        self.current
            .clone()
            .path()
            .join(DatabaseType::Current.to_file_name())
    }

    pub fn history_db_file(&self) -> PathBuf {
        self.history
            .clone()
            .path()
            .join(DatabaseType::History.to_file_name())
    }
}

/// Handle SQLite databases and write command runner.
pub struct DatabaseManager {
    sqlite_write_close: SqliteWriteCloseHandle,
    sqlite_read_close: SqlxReadCloseHandle,
    history_write_close: SqliteWriteCloseHandle,
    history_read_close: SqlxReadCloseHandle,
    diesel_current_write_close: DieselWriteCloseHandle,
    diesel_current_read_close: DieselReadCloseHandle,
    diesel_history_write_close: DieselWriteCloseHandle,
    diesel_history_read_close: DieselReadCloseHandle,
}

impl DatabaseManager {
    /// Runs also some blocking file system code.
    pub async fn new<T: AsRef<Path>>(
        database_dir: T,
        config: Arc<Config>,
        media_backup: MediaBackupHandle,
    ) -> Result<(Self, RouterDatabaseReadHandle, RouterDatabaseWriteHandle), DataError> {
        info!("Creating DatabaseManager");

        let root = DatabaseRoot::new(database_dir)?;

        // Diesel

        // Run migrations and print SQLite version used by Diesel
        let (diesel_current_write, diesel_current_write_close) =
            DieselWriteHandle::new(&config, root.current_db_file())
                .await
                .change_context(DataError::Init)?;

        let diesel_sqlite = diesel_current_write
            .sqlite_version()
            .await
            .change_context(DataError::Sqlite)?;
        info!("Diesel SQLite version: {}", diesel_sqlite);

        let (diesel_current_read, diesel_current_read_close) =
            DieselReadHandle::new(&config, root.current_db_file())
                .await
                .change_context(DataError::Init)?;

        let (diesel_history_write, diesel_history_write_close) =
            DieselWriteHandle::new(&config, root.history_db_file())
                .await
                .change_context(DataError::Init)?;

        let (diesel_history_read, diesel_history_read_close) =
            DieselReadHandle::new(&config, root.history_db_file())
                .await
                .change_context(DataError::Init)?;

        // Sqlx

        let (sqlite_write, sqlite_write_close) =
            SqliteWriteHandle::new(&config, root.current_db_file())
                .await
                .change_context(DataError::Init)?;

        let sqlx_sqlite = sqlite_write
            .sqlite_version()
            .await
            .change_context(DataError::Init)?;
        info!("Sqlx SQLite version: {}", sqlx_sqlite);

        if diesel_sqlite != sqlx_sqlite {
            return Err(DataError::SqliteVersionMismatch.report());
        }

        let (sqlite_read, sqlite_read_close) = SqlxReadHandle::new(&config, root.current_db_file())
            .await
            .change_context(DataError::Init)?;

        let (history_write, history_write_close) =
            SqliteWriteHandle::new(&config, root.history_db_file())
                .await
                .change_context(DataError::Init)?;

        let (history_read, history_read_close) =
            SqlxReadHandle::new(&config, root.history_db_file())
                .await
                .change_context(DataError::Init)?;

        let read_commands = SqliteReadCommands::new(&sqlite_read);
        let index = LocationIndexManager::new(config.clone());
        let diesel_current_read = DieselCurrentReadHandle::new(diesel_current_read);
        let diesel_current_write = DieselCurrentWriteHandle::new(diesel_current_write);
        let diesel_history_read = DieselHistoryReadHandle::new(diesel_history_read);
        let diesel_history_write = DieselHistoryWriteHandle::new(diesel_history_write);

        let cache = DatabaseCache::new(
            read_commands,
            diesel_current_read.clone(),
            &index,
            &config,
        )
        .await
        .change_context(DataError::Cache)?;

        let router_write_handle = RouterDatabaseWriteHandle {
            config: config.clone(),
            sqlx_current_write: CurrentDataWriteHandle::new(sqlite_write),
            sqlx_current_read: sqlite_read,
            diesel_current_read: diesel_current_read.clone(),
            diesel_current_write: diesel_current_write.clone(),
            diesel_history_read: diesel_history_read.clone(),
            diesel_history_write: diesel_history_write.clone(),
            sqlx_history_write: HistoryWriteHandle {
                handle: history_write,
            },
            sqlx_history_read: history_read,
            root: root.into(),
            cache: cache.into(),
            location: index.into(),
            media_backup,
            image_processing_queue: Arc::new(tokio::sync::Semaphore::new(
                num_cpus::get(),
            )),
        };

        let sqlite_read = router_write_handle.sqlx_current_read.clone();
        let history_read = router_write_handle.sqlx_history_read.clone();
        let root = router_write_handle.root.clone();
        let cache = router_write_handle.cache.clone();

        let router_read_handle = RouterDatabaseReadHandle {
            sqlite_read,
            history_read,
            diesel_read: diesel_current_read,
            diesel_history_read,
            root,
            cache,
        };

        let database_manager = DatabaseManager {
            sqlite_write_close,
            sqlite_read_close,
            history_write_close,
            history_read_close,
            diesel_current_write_close,
            diesel_current_read_close,
            diesel_history_read_close,
            diesel_history_write_close,
        };

        info!("DatabaseManager created");

        Ok((database_manager, router_read_handle, router_write_handle))
    }

    pub async fn close(self) {
        self.sqlite_read_close.close().await;
        self.sqlite_write_close.close().await;
        self.history_read_close.close().await;
        self.history_write_close.close().await;
        self.diesel_current_read_close.close().await;
        self.diesel_current_write_close.close().await;
        self.diesel_history_read_close.close().await;
        self.diesel_history_write_close.close().await;
    }
}

#[derive(Clone, Debug)]
pub struct RouterDatabaseWriteHandle {
    config: Arc<Config>,
    root: Arc<DatabaseRoot>,
    sqlx_current_write: CurrentDataWriteHandle,
    sqlx_current_read: SqlxReadHandle,
    sqlx_history_write: HistoryWriteHandle,
    sqlx_history_read: SqlxReadHandle,
    diesel_current_write: DieselCurrentWriteHandle,
    diesel_current_read: DieselCurrentReadHandle,
    diesel_history_write: DieselHistoryWriteHandle,
    diesel_history_read: DieselHistoryReadHandle,
    cache: Arc<DatabaseCache>,
    location: Arc<LocationIndexManager>,
    media_backup: MediaBackupHandle,
    image_processing_queue: Arc<tokio::sync::Semaphore>,
}

impl RouterDatabaseWriteHandle {
    pub fn user_write_commands(&self) -> WriteCommands {
        WriteCommands::new(
            &self.config,
            &self.sqlx_current_write,
            &self.sqlx_history_write,
            &self.diesel_current_write,
            &self.diesel_history_write,
            &self.cache,
            &self.root.file_dir,
            &self.location,
            &self.media_backup,
        )
    }

    pub fn user_write_commands_account<'b>(&'b self) -> WriteCommandsConcurrent<'b> {
        WriteCommandsConcurrent::new(
            &self.sqlx_current_write,
            &self.sqlx_history_write,
            &self.cache,
            &self.root.file_dir,
            LocationIndexIteratorHandle::new(&self.location),
            &self.image_processing_queue,
        )
    }

    pub async fn register(
        &self,
        id_light: AccountId,
        sign_in_with_info: SignInWithInfo,
    ) -> Result<AccountIdInternal, DataError> {
        self.user_write_commands()
            .register(id_light, sign_in_with_info)
            .await
    }

    pub fn into_sync_handle(self) -> SyncWriteHandle {
        SyncWriteHandle {
            config: self.config,
            root: self.root,
            sqlx_current_write: self.sqlx_current_write,
            sqlx_current_read: self.sqlx_current_read,
            sqlx_history_write: self.sqlx_history_write,
            sqlx_history_read: self.sqlx_history_read,
            diesel_current_write: self.diesel_current_write,
            diesel_current_read: self.diesel_current_read,
            diesel_history_write: self.diesel_history_write,
            diesel_history_read: self.diesel_history_read,
            cache: self.cache,
            location: self.location,
            media_backup: self.media_backup,
        }
    }
}

/// Handle for writing synchronous write commands.
#[derive(Clone, Debug)]
pub struct SyncWriteHandle {
    config: Arc<Config>,
    root: Arc<DatabaseRoot>,
    sqlx_current_write: CurrentDataWriteHandle,
    sqlx_current_read: SqlxReadHandle,
    sqlx_history_write: HistoryWriteHandle,
    sqlx_history_read: SqlxReadHandle,
    diesel_current_write: DieselCurrentWriteHandle,
    diesel_current_read: DieselCurrentReadHandle,
    diesel_history_write: DieselHistoryWriteHandle,
    diesel_history_read: DieselHistoryReadHandle,
    cache: Arc<DatabaseCache>,
    location: Arc<LocationIndexManager>,
    media_backup: MediaBackupHandle,
}

impl SyncWriteHandle {
    fn cmds(&self) -> WriteCommands {
        WriteCommands::new(
            &self.config,
            &self.sqlx_current_write,
            &self.sqlx_history_write,
            &self.diesel_current_write,
            &self.diesel_history_write,
            &self.cache,
            &self.root.file_dir,
            &self.location,
            &self.media_backup,
        )
    }

    pub fn common(&self) -> WriteCommandsCommon {
        self.cmds().common()
    }

    pub fn account(&self) -> WriteCommandsAccount {
        self.cmds().account()
    }

    pub fn account_admin(&self) -> WriteCommandsAccountAdmin {
        self.cmds().account_admin()
    }

    pub fn media(&self) -> WriteCommandsMedia {
        self.cmds().media()
    }

    pub fn media_admin(&self) -> WriteCommandsMediaAdmin {
        self.cmds().media_admin()
    }

    pub fn profile(&self) -> WriteCommandsProfile {
        self.cmds().profile()
    }

    pub fn profile_admin(&self) -> WriteCommandsProfileAdmin {
        self.cmds().profile_admin()
    }

    pub fn chat(&self) -> WriteCommandsChat {
        self.cmds().chat()
    }

    pub fn chat_admin(&self) -> WriteCommandsChatAdmin {
        self.cmds().chat_admin()
    }

    pub async fn register(
        &self,
        id_light: AccountId,
        sign_in_with_info: SignInWithInfo,
    ) -> Result<AccountIdInternal, DataError> {
        self.cmds().register(id_light, sign_in_with_info).await
    }
}

pub struct RouterDatabaseReadHandle {
    root: Arc<DatabaseRoot>,
    sqlite_read: SqlxReadHandle,
    diesel_read: DieselCurrentReadHandle,
    history_read: SqlxReadHandle,
    diesel_history_read: DieselHistoryReadHandle,
    cache: Arc<DatabaseCache>,
}

impl RouterDatabaseReadHandle {
    pub fn read(&self) -> ReadCommands<'_> {
        ReadCommands::new(
            &self.sqlite_read,
            &self.cache,
            &self.root.file_dir,
            &self.diesel_read,
        )
    }

    pub fn history(&self) -> HistoryReadCommands<'_> {
        HistoryReadCommands::new(&self.history_read)
    }

    pub fn read_files(&self) -> FileReadCommands<'_> {
        FileReadCommands::new(&self.root.file_dir)
    }

    pub fn access_token_manager(&self) -> AccessTokenManager<'_> {
        AccessTokenManager::new(&self.cache)
    }

    pub fn account_id_manager(&self) -> AccountIdManager<'_> {
        AccountIdManager::new(&self.cache, &self.sqlite_read)
    }
}
