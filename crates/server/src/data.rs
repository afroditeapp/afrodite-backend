use std::{
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use simple_backend::media_backup::MediaBackupHandle;
use simple_backend_database::{DbReadCloseHandle, DbWriteCloseHandle, DbWriteHandle, DatabaseHandleCreator, diesel_db::DieselWriteHandle};
use simple_backend_utils::ContextExt;
use config::Config;
use database::{
    current::{read::CurrentReadCommands, self},
    // diesel::{
    //     DieselCurrentReadHandle, DieselCurrentWriteHandle, DieselHistoryReadHandle,
    //     DieselHistoryWriteHandle, DieselReadCloseHandle, DieselReadHandle, DieselWriteCloseHandle,
    //     DieselWriteHandle,
    // },
    history::read::HistoryReadCommands,
    // sqlite::{
    //     CurrentDataWriteHandle, DatabaseType, HistoryWriteHandle, SqliteDatabasePath,
    //     SqliteWriteCloseHandle, SqliteWriteHandle, SqlxReadCloseHandle, SqlxReadHandle,
    // },
    ErrorContext, CurrentReadHandle, CurrentWriteHandle, HistoryWriteHandle, HistoryReadHandle,
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
use crate::{internal::InternalApiError};

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
    #[error("Tried to do something that is not allowed")]
    NotAllowed,
    #[error("Action is already done")]
    AlreadyDone,

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
    file_dir: FileDir,
}

impl DatabaseRoot {
    pub fn new<T: AsRef<Path>>(path: T) -> Result<Self, DataError> {
        let root = path.as_ref().to_path_buf();
        if !root.exists() {
            fs::create_dir(&root).change_context(DataError::Init)?;
        }

        // let history = root.join(DB_HISTORY_DIR_NAME);
        // if !history.exists() {
        //     fs::create_dir(&history).change_context(DataError::Init)?;
        // }
        // let history = SqliteDatabasePath::new(history);

        // let current = root.join(DB_CURRENT_DATA_DIR_NAME);
        // if !current.exists() {
        //     fs::create_dir(&current).change_context(DataError::Init)?;
        // }
        // let current = SqliteDatabasePath::new(current);

        let file_dir = root.join(DB_FILE_DIR_NAME);
        if !file_dir.exists() {
            fs::create_dir(&file_dir).change_context(DataError::Init)?;
        }
        let file_dir = FileDir::new(file_dir);

        Ok(Self {
            root,
            // history,
            // current,
            file_dir,
        })
    }

    // /// History Sqlite database path
    // pub fn history(&self) -> SqliteDatabasePath {
    //     self.history.clone()
    // }

    // pub fn history_ref(&self) -> &SqliteDatabasePath {
    //     &self.history
    // }

    // /// Sqlite database path
    // pub fn current(&self) -> SqliteDatabasePath {
    //     self.current.clone()
    // }

    // pub fn current_ref(&self) -> &SqliteDatabasePath {
    //     &self.current
    // }

    pub fn file_dir(&self) -> &FileDir {
        &self.file_dir
    }

    // pub fn current_db_file(&self) -> PathBuf {
    //     self.current
    //         .clone()
    //         .path()
    //         .join(DatabaseType::Current.to_file_name())
    // }

    // pub fn history_db_file(&self) -> PathBuf {
    //     self.history
    //         .clone()
    //         .path()
    //         .join(DatabaseType::History.to_file_name())
    // }
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
    ) -> Result<(Self, RouterDatabaseReadHandle, RouterDatabaseWriteHandle), DataError> {
        info!("Creating DatabaseManager");

        let root = DatabaseRoot::new(database_dir)?;

        // Write handles

        let (current_write, current_write_close) = DatabaseHandleCreator::create_write_handle_from_config(
            config.simple_backend(),
            "current",
            database::DIESEL_MIGRATIONS,
        )
            .await
            .change_context(DataError::Init)?;

        let diesel_sqlite = current_write
            .diesel()
            .sqlite_version()
            .await
            .change_context(DataError::Sqlite)?;
        info!("Diesel SQLite version: {}", diesel_sqlite);

        let sqlx_sqlite = current_write
            .sqlx()
            .sqlite_version()
            .await
            .change_context(DataError::Init)?;
        info!("Sqlx SQLite version: {}", sqlx_sqlite);

        if diesel_sqlite != sqlx_sqlite {
            return Err(DataError::SqliteVersionMismatch.report());
        }

        let (history_write, history_write_close) = DatabaseHandleCreator::create_write_handle_from_config(
            config.simple_backend(),
            "history",
            database::DIESEL_MIGRATIONS,
        )
            .await
            .change_context(DataError::Init)?;

        // Read handles

        let (current_read, current_read_close) = DatabaseHandleCreator::create_read_handle_from_config(
            config.simple_backend(),
            "current",
        )
            .await
            .change_context(DataError::Init)?;

        let (history_read, history_read_close) = DatabaseHandleCreator::create_read_handle_from_config(
            config.simple_backend(),
            "history",
        )
            .await
            .change_context(DataError::Init)?;

        // Sqlx

        // let read_commands = CurrentReadCommands::new(current_read.sqlx());
        let index = LocationIndexManager::new(config.clone());
        let current_read_handle = CurrentReadHandle(current_read);
        let current_write_handle = CurrentWriteHandle(current_write);
        let history_read_handle = HistoryReadHandle(history_read);
        let history_write_handle = HistoryWriteHandle(history_write);

        let cache = DatabaseCache::new(
            &current_read_handle,
            &index,
            &config,
        )
        .await
        .change_context(DataError::Cache)?;

        let router_write_handle = RouterDatabaseWriteHandle {
            config: config.clone(),
            current_read_handle: current_read_handle.clone(),
            current_write_handle: current_write_handle.clone(),
            history_read_handle: history_read_handle.clone(),
            history_write_handle: history_write_handle.clone(),
            root: root.into(),
            cache: cache.into(),
            location: index.into(),
            media_backup,
            image_processing_queue: Arc::new(tokio::sync::Semaphore::new(
                num_cpus::get(),
            )),
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
    current_read_handle: CurrentReadHandle,
    current_write_handle: CurrentWriteHandle,
    history_read_handle: HistoryReadHandle,
    history_write_handle: HistoryWriteHandle,
    cache: Arc<DatabaseCache>,
    location: Arc<LocationIndexManager>,
    media_backup: MediaBackupHandle,
    image_processing_queue: Arc<tokio::sync::Semaphore>,
}

impl RouterDatabaseWriteHandle {
    pub fn user_write_commands(&self) -> WriteCommands {
        WriteCommands::new(
            &self.config,
            &self.current_write_handle,
            &self.history_write_handle,
            &self.cache,
            &self.root.file_dir,
            &self.location,
            &self.media_backup,
        )
    }

    pub fn user_write_commands_account<'b>(&'b self) -> WriteCommandsConcurrent<'b> {
        WriteCommandsConcurrent::new(
            &self.current_write_handle,
            &self.history_write_handle,
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
            current_write_handle: self.current_write_handle,
            history_write_handle: self.history_write_handle,
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
    current_write_handle: CurrentWriteHandle,
    history_write_handle: HistoryWriteHandle,
    cache: Arc<DatabaseCache>,
    location: Arc<LocationIndexManager>,
    media_backup: MediaBackupHandle,
}

impl SyncWriteHandle {
    fn cmds(&self) -> WriteCommands {
        WriteCommands::new(
            &self.config,
            &self.current_write_handle,
            &self.history_write_handle,
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
    current_read_handle: CurrentReadHandle,
    history_read_handle: HistoryReadHandle,
    cache: Arc<DatabaseCache>,
}

impl RouterDatabaseReadHandle {
    pub fn read(&self) -> ReadCommands<'_> {
        ReadCommands::new(
            &self.current_read_handle,
            &self.cache,
            &self.root.file_dir
        )
    }

    pub fn history(&self) -> HistoryReadCommands<'_> {
        HistoryReadCommands::new(&self.history_read_handle)
    }

    pub fn read_files(&self) -> FileReadCommands<'_> {
        FileReadCommands::new(&self.root.file_dir)
    }

    pub fn access_token_manager(&self) -> AccessTokenManager<'_> {
        AccessTokenManager::new(&self.cache)
    }

    pub fn account_id_manager(&self) -> AccountIdManager<'_> {
        AccountIdManager::new(&self.cache)
    }
}
