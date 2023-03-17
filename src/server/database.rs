pub mod file;
pub mod read;
pub mod sqlite;
pub mod current;
pub mod write;
pub mod utils;
pub mod history;

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::de::DeserializeOwned;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

use error_stack::{Result, ResultExt};

use crate::{
    api::model::{AccountId, Profile, Account, AccountSetup, AccountIdLight},
    utils::{AppendErr, ErrorContainer, ErrorConversion},
};

use self::{
    file::{utils::DatabasePath, GitDatabaseOperationHandle, GitError, write::GitDatabaseWriteCommands, file::GitJsonFile},
    read::{ReadCmd},
    sqlite::{
        SqliteDatabasePath, SqliteReadCloseHandle, SqliteReadHandle, SqliteWriteCloseHandle,
        SqliteWriteHandle, DatabaseType,
    },
    write::{WriteCmd, WriteCommands}, utils::GetReadWriteCmd, current::read::SqliteReadCommands, history::read::HistoryReadCommands,
};
use crate::utils::IntoReportExt;

pub const DB_HISTORY_DIR_NAME: &str = "history";
pub const DB_CURRENT_DATA_DIR_NAME: &str = "current";

pub type DatabeseEntryId = String;

#[derive(thiserror::Error, Debug)]
pub enum DatabaseError {
    #[error("Git error")]
    Git,
    #[error("SQLite error")]
    Sqlite,

    // Other errors
    #[error("Database initialization error")]
    Init,
    #[error("Database SQLite and Git integrity check")]
    Integrity,
}

/// Absolsute path to database root directory.
pub struct DatabaseRoot {
    root: PathBuf,
    history: SqliteDatabasePath,
    current: SqliteDatabasePath,
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

        Ok(Self {
            root,
            history,
            current,
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
}

/// Handle Git and SQLite databases
pub struct DatabaseManager {
    sqlite_write_close: SqliteWriteCloseHandle,
    sqlite_read_close: SqliteReadCloseHandle,
    history_write_close: SqliteWriteCloseHandle,
    history_read_close: SqliteReadCloseHandle,
}

impl DatabaseManager {
    /// Runs also some blocking file system code.
    pub async fn new<T: AsRef<Path>>(
        database_dir: T,
    ) -> Result<(Self, RouterDatabaseHandle), DatabaseError> {
        let root = DatabaseRoot::new(database_dir)?;

        let (sqlite_write, sqlite_write_close) =
            SqliteWriteHandle::new(root.current(), DatabaseType::Current)
                .await
                .change_context(DatabaseError::Init)?;

        let (sqlite_read, sqlite_read_close) =
            SqliteReadHandle::new(root.current(), DatabaseType::Current)
                .await
                .change_context(DatabaseError::Init)?;

        let (history_write, history_write_close) =
            SqliteWriteHandle::new(root.current(), DatabaseType::History)
                .await
                .change_context(DatabaseError::Init)?;

        let (history_read, history_read_close) =
            SqliteReadHandle::new(root.current(), DatabaseType::History)
                .await
                .change_context(DatabaseError::Init)?;

        let database_manager = DatabaseManager {
            sqlite_write_close,
            sqlite_read_close,
            history_write_close,
            history_read_close,
        };

        let router_handle = RouterDatabaseHandle {
            sqlite_write,
            sqlite_read,
            history_write,
            history_read,
            root,
        };

        Ok((database_manager, router_handle))
    }

    pub async fn close(self) {
        self.sqlite_read_close.close().await;
        self.sqlite_write_close.close().await;
        self.history_read_close.close().await;
        self.history_write_close.close().await;
    }
}

pub struct RouterDatabaseHandle {
    root: DatabaseRoot,
    sqlite_write: SqliteWriteHandle,
    sqlite_read: SqliteReadHandle,
    history_write: SqliteWriteHandle,
    history_read: SqliteReadHandle,
}

impl RouterDatabaseHandle {
    pub fn user_write_commands(&self, id: AccountIdLight) -> WriteCommands {
        WriteCommands::new(
            id,
            self.sqlite_write.clone(),
            self.history_write.clone(),
        )
    }

    pub fn read(&self) -> SqliteReadCommands<'_> {
        SqliteReadCommands::new(&self.sqlite_read)
    }

    pub fn history(&self) -> HistoryReadCommands<'_> {
        HistoryReadCommands::new(&self.history_read)
    }
}

#[derive(Debug, Clone)]
enum WriteCmdIntegrity {
    GitAccountIdFile(AccountId),
}

impl std::fmt::Display for WriteCmdIntegrity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Integrity write command: {:?}", self))
    }
}

#[derive(Debug, Clone)]
enum ReadCmdIntegrity {
    AccountId(AccountId),
}

impl std::fmt::Display for ReadCmdIntegrity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Read command: {:?}", self))
    }
}
