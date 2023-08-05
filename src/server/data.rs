pub mod cache;
pub mod database;
pub mod file;
pub mod index;
pub mod read;
pub mod utils;
pub mod write;
pub mod write_concurrent;
pub mod write_commands;

use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc, fmt::Debug,
};

use error_stack::{Result, ResultExt};

use crate::server::data::database::current::read::SqliteReadCommands;
use tracing::info;

use crate::{
    api::model::{AccountIdInternal, AccountIdLight, SignInWithInfo},
    config::Config,
    media_backup::MediaBackupHandle,
    server::data::{database::sqlite::print_sqlite_version},
};

use self::{
    cache::{DatabaseCache, WriteCacheJson},
    database::{history::read::HistoryReadCommands, sqlite::{HistoryUpdateJson, SqliteUpdateJson}},
    database::sqlite::{
        CurrentDataWriteHandle, DatabaseType, HistoryWriteHandle, SqliteDatabasePath,
        SqlxReadCloseHandle, SqlxReadHandle, SqliteWriteCloseHandle, SqliteWriteHandle,
    },
    file::{read::FileReadCommands, utils::FileDir, FileError},
    index::{LocationIndexIteratorGetter, LocationIndexManager, LocationIndexWriterGetter},
    read::ReadCommands,
    utils::{AccountIdManager, ApiKeyManager},
    write::{WriteCommands, common::WriteCommandsCommon, account::WriteCommandsAccount, account_admin::WriteCommandsAccountAdmin, media::WriteCommandsMedia, media_admin::WriteCommandsMediaAdmin, profile::WriteCommandsProfile, profile_admin::WriteCommandsProfileAdmin, chat::WriteCommandsChat, chat_admin::WriteCommandsChatAdmin},
    write_concurrent::{WriteCommandsConcurrent},
};
use crate::utils::IntoReportExt;

pub const DB_HISTORY_DIR_NAME: &str = "history";
pub const DB_CURRENT_DATA_DIR_NAME: &str = "current";
pub const DB_FILE_DIR_NAME: &str = "files";

pub type DatabeseEntryId = String;

#[derive(thiserror::Error, Debug)]
pub enum DatabaseError {
    #[error("Git error")]
    Git,
    #[error("SQLite error")]
    Sqlite,
    #[error("Cache error")]
    Cache,
    #[error("File error")]
    File,
    #[error("Media backup error")]
    MediaBackup,

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

    #[error("Command runner quit too early")]
    CommandRunnerQuit,
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
    pub fn new<T: AsRef<Path>>(path: T) -> Result<Self, DatabaseError> {
        let root = path.as_ref().to_path_buf();
        if !root.exists() {
            fs::create_dir(&root).into_error(DatabaseError::Init)?;
        }

        let history = root.join(DB_HISTORY_DIR_NAME);
        if !history.exists() {
            fs::create_dir(&history).into_error(DatabaseError::Init)?;
        }
        let history = SqliteDatabasePath::new(history);

        let current = root.join(DB_CURRENT_DATA_DIR_NAME);
        if !current.exists() {
            fs::create_dir(&current).into_error(DatabaseError::Init)?;
        }
        let current = SqliteDatabasePath::new(current);

        let file_dir = root.join(DB_FILE_DIR_NAME);
        if !file_dir.exists() {
            fs::create_dir(&file_dir).into_error(DatabaseError::Init)?;
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
}

impl DatabaseManager {
    /// Runs also some blocking file system code.
    pub async fn new<T: AsRef<Path>>(
        database_dir: T,
        config: Arc<Config>,
        media_backup: MediaBackupHandle,
    ) -> Result<(Self, RouterDatabaseReadHandle, RouterDatabaseWriteHandle), DatabaseError> {
        info!("Creating DatabaseManager");

        let root = DatabaseRoot::new(database_dir)?;

        let (sqlite_write, sqlite_write_close) =
            SqliteWriteHandle::new(&config, root.current_db_file())
                .await
                .change_context(DatabaseError::Init)?;

        print_sqlite_version(sqlite_write.pool())
            .await
            .change_context(DatabaseError::Init)?;

        let (sqlite_read, sqlite_read_close) =
            SqlxReadHandle::new(&config, root.current_db_file())
                .await
                .change_context(DatabaseError::Init)?;

        let (history_write, history_write_close) =
            SqliteWriteHandle::new(&config, root.history_db_file())
                .await
                .change_context(DatabaseError::Init)?;

        let (history_read, history_read_close) =
            SqlxReadHandle::new(&config, root.history_db_file())
                .await
                .change_context(DatabaseError::Init)?;

        let read_commands = SqliteReadCommands::new(&sqlite_read);
        let index = LocationIndexManager::new(config.clone());
        let cache = DatabaseCache::new(
            read_commands,
            LocationIndexIteratorGetter::new(&index),
            LocationIndexWriterGetter::new(&index),
            &config,
        )
        .await
        .change_context(DatabaseError::Cache)?;

        let router_write_handle = RouterDatabaseWriteHandle {
            config: config.clone(),
            sqlite_write: CurrentDataWriteHandle::new(sqlite_write),
            sqlite_read,
            history_write: HistoryWriteHandle {
                handle: history_write,
            },
            history_read,
            root: root.into(),
            cache: cache.into(),
            location: index.into(),
            media_backup,
        };

        let sqlite_read = router_write_handle.sqlite_read.clone();
        let history_read = router_write_handle.history_read.clone();
        let root = router_write_handle.root.clone();
        let cache = router_write_handle.cache.clone();

        let router_read_handle = RouterDatabaseReadHandle {
            sqlite_read,
            history_read,
            root,
            cache,
        };

        let database_manager = DatabaseManager {
            sqlite_write_close,
            sqlite_read_close,
            history_write_close,
            history_read_close,
        };

        info!("DatabaseManager created");

        Ok((database_manager, router_read_handle, router_write_handle))
    }

    pub async fn close(self) {
        self.sqlite_read_close.close().await;
        self.sqlite_write_close.close().await;
        self.history_read_close.close().await;
        self.history_write_close.close().await;
    }
}

#[derive(Clone, Debug)]
pub struct RouterDatabaseWriteHandle {
    config: Arc<Config>,
    root: Arc<DatabaseRoot>,
    sqlite_write: CurrentDataWriteHandle,
    sqlite_read: SqlxReadHandle,
    history_write: HistoryWriteHandle,
    history_read: SqlxReadHandle,
    cache: Arc<DatabaseCache>,
    location: Arc<LocationIndexManager>,
    media_backup: MediaBackupHandle,
}

impl RouterDatabaseWriteHandle {
    pub fn user_write_commands(&self) -> WriteCommands {
        WriteCommands::new(
            &self.config,
            &self.sqlite_write,
            &self.history_write,
            &self.cache,
            &self.root.file_dir,
            LocationIndexWriterGetter::new(&self.location),
            &self.media_backup,
        )
    }

    pub fn user_write_commands_account<'b>(&'b self) -> WriteCommandsConcurrent<'b> {
        WriteCommandsConcurrent::new(
            &self.sqlite_write,
            &self.history_write,
            &self.cache,
            &self.root.file_dir,
            LocationIndexIteratorGetter::new(&self.location),
        )
    }

    pub async fn register(
        &self,
        id_light: AccountIdLight,
        sign_in_with_info: SignInWithInfo,
    ) -> Result<AccountIdInternal, DatabaseError> {
        self.user_write_commands().register(id_light, sign_in_with_info).await
    }

    pub fn into_sync_handle(self) -> SyncWriteHandle {
        SyncWriteHandle {
            config: self.config,
            root: self.root,
            sqlite_write: self.sqlite_write,
            sqlite_read: self.sqlite_read,
            history_write: self.history_write,
            history_read: self.history_read,
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
    sqlite_write: CurrentDataWriteHandle,
    sqlite_read: SqlxReadHandle,
    history_write: HistoryWriteHandle,
    history_read: SqlxReadHandle,
    cache: Arc<DatabaseCache>,
    location: Arc<LocationIndexManager>,
    media_backup: MediaBackupHandle,
}

impl SyncWriteHandle {
    fn cmds(&self) -> WriteCommands {
        WriteCommands::new(
            &self.config,
            &self.sqlite_write,
            &self.history_write,
            &self.cache,
            &self.root.file_dir,
            LocationIndexWriterGetter::new(&self.location),
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
        id_light: AccountIdLight,
        sign_in_with_info: SignInWithInfo,
    ) -> Result<AccountIdInternal, DatabaseError> {
        self.cmds().register(
            id_light,
            sign_in_with_info,
        ).await
    }

    pub async fn update_data<
        T: Clone
            + Debug
            + Send
            + SqliteUpdateJson
            + HistoryUpdateJson
            + WriteCacheJson
            + Sync
            + 'static,
    >(
        &self,
        id: AccountIdInternal,
        data: &T,
    ) -> Result<(), DatabaseError> {
        self.cmds().update_data(id, data).await
    }
}




pub struct RouterDatabaseReadHandle {
    root: Arc<DatabaseRoot>,
    sqlite_read: SqlxReadHandle,
    history_read: SqlxReadHandle,
    cache: Arc<DatabaseCache>,
}

impl RouterDatabaseReadHandle {
    pub fn read(&self) -> ReadCommands<'_> {
        ReadCommands::new(&self.sqlite_read, &self.cache, &self.root.file_dir)
    }

    pub fn history(&self) -> HistoryReadCommands<'_> {
        HistoryReadCommands::new(&self.history_read)
    }

    pub fn read_files(&self) -> FileReadCommands<'_> {
        FileReadCommands::new(&self.root.file_dir)
    }

    pub fn api_key_manager(&self) -> ApiKeyManager<'_> {
        ApiKeyManager::new(&self.cache)
    }

    pub fn account_id_manager(&self) -> AccountIdManager<'_> {
        AccountIdManager::new(&self.cache, &self.sqlite_read)
    }
}
