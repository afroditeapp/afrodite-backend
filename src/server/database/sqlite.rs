use crate::api::model::{AccountIdInternal, AccountIdLight};

use super::history::read::HistoryReadCommands;
use super::read::ReadCommands;

use async_trait::async_trait;

use super::current::{read::SqliteReadCommands, write::CurrentDataWriteCommands};
use super::history::write::HistoryWriteCommands;

use error_stack::Result;

use std::path::{Path, PathBuf};

use sqlx::{
    sqlite::{self, SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};

use crate::utils::IntoReportExt;

pub const DATABASE_FILE_NAME: &str = "current.db";
pub const HISTORY_FILE_NAME: &str = "history.db";

#[derive(thiserror::Error, Debug)]
pub enum SqliteDatabaseError {
    #[error("Connecting to SQLite database failed")]
    Connect,
    #[error("Executing SQL query failed")]
    Execute,
    #[error("Error when streaming data from SQL query")]
    Fetch,
    #[error("Running sqlx database migrations failed")]
    Migrate,

    #[error("Deserialization error")]
    SerdeDeserialize,
    #[error("Serialization error")]
    SerdeSerialize,

    #[error("Time parsing error")]
    TimeParsing,

    #[error("TryFrom error")]
    TryFromError,
}

/// Path to directory which contains Sqlite files.
#[derive(Debug, Clone)]
pub struct SqliteDatabasePath {
    database_dir: PathBuf,
}

impl SqliteDatabasePath {
    pub fn new(database_dir: PathBuf) -> Self {
        Self { database_dir }
    }

    pub fn path(&self) -> &Path {
        &self.database_dir
    }
}

pub struct SqliteWriteCloseHandle {
    pool: SqlitePool,
}

impl SqliteWriteCloseHandle {
    /// Call this before closing the server.
    pub async fn close(self) {
        self.pool.close().await
    }
}

#[derive(Debug, Clone)]
pub struct CurrentDataWriteHandle {
    handle: SqliteWriteHandle,
    read_handle: SqliteReadHandle,
}

impl CurrentDataWriteHandle {
    pub fn new(handle: SqliteWriteHandle) -> Self {
        Self {
            read_handle: SqliteReadHandle { pool: handle.pool.clone() },
            handle,
        }
    }

    pub fn pool(&self) -> &SqlitePool {
        self.handle.pool()
    }

    pub fn read(&self) -> SqliteReadCommands<'_> {
        SqliteReadCommands::new(&self.read_handle)
    }
}

#[derive(Debug, Clone)]
pub struct HistoryWriteHandle {
    pub handle: SqliteWriteHandle,
}

impl HistoryWriteHandle {
    pub fn pool(&self) -> &SqlitePool {
        self.handle.pool()
    }
}

#[derive(Debug, Clone)]
pub struct SqliteWriteHandle {
    pool: SqlitePool,
}

impl SqliteWriteHandle {
    pub async fn new(
        dir: SqliteDatabasePath,
        db_type: DatabaseType,
    ) -> Result<(Self, SqliteWriteCloseHandle), SqliteDatabaseError> {
        let db_path = dir.path().join(db_type.to_file_name());

        let run_initial_setup = !db_path.exists();

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(
                SqliteConnectOptions::new()
                    .filename(db_path)
                    .create_if_missing(true)
                    .foreign_keys(true)
                    .journal_mode(sqlite::SqliteJournalMode::Wal),
            )
            .await
            .into_error(SqliteDatabaseError::Connect)?;

        if run_initial_setup {
            sqlx::migrate!()
                .run(&pool)
                .await
                .into_error(SqliteDatabaseError::Migrate)?;
        }

        let write_handle = SqliteWriteHandle { pool: pool.clone() };

        let close_handle = SqliteWriteCloseHandle { pool };

        Ok((write_handle, close_handle))
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

pub struct SqliteReadCloseHandle {
    pool: SqlitePool,
}

impl SqliteReadCloseHandle {
    /// Call this before closing the server.
    pub async fn close(self) {
        self.pool.close().await
    }
}

#[derive(Debug, Clone)]
pub struct SqliteReadHandle {
    pool: SqlitePool,
}

impl SqliteReadHandle {
    pub async fn new(
        dir: SqliteDatabasePath,
        db_type: DatabaseType,
    ) -> Result<(Self, SqliteReadCloseHandle), SqliteDatabaseError> {
        let db_path = dir.path().join(db_type.to_file_name());

        let pool = SqlitePoolOptions::new()
            .max_connections(16)
            .connect_with(
                SqliteConnectOptions::new()
                    .filename(db_path)
                    .create_if_missing(false)
                    .foreign_keys(true)
                    .journal_mode(sqlite::SqliteJournalMode::Wal),
            )
            .await
            .into_error(SqliteDatabaseError::Connect)?;

        let handle = SqliteReadHandle { pool: pool.clone() };

        let close_handle = SqliteReadCloseHandle { pool };

        Ok((handle, close_handle))
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

#[derive(Debug, Clone)]
pub enum DatabaseType {
    Current,
    History,
}

impl DatabaseType {
    pub fn to_file_name(&self) -> &str {
        match self {
            DatabaseType::Current => DATABASE_FILE_NAME,
            DatabaseType::History => HISTORY_FILE_NAME,
        }
    }
}

#[async_trait]
pub trait SqliteUpdateJson {
    async fn update_json(
        &self,
        id: AccountIdInternal,
        write: &CurrentDataWriteCommands,
    ) -> Result<(), SqliteDatabaseError>;
}

#[async_trait]
pub trait SqliteSelectJson: Sized {
    async fn select_json(
        id: AccountIdInternal,
        read: &SqliteReadCommands,
    ) -> Result<Self, SqliteDatabaseError>;
}

#[async_trait]
pub trait HistoryUpdateJson {
    async fn history_update_json(
        &self,
        id: AccountIdInternal,
        write: &HistoryWriteCommands,
    ) -> Result<(), SqliteDatabaseError>;
}

#[async_trait]
pub trait HistorySelectJson: Sized {
    async fn history_select_json(
        id: AccountIdInternal,
        read: &HistoryReadCommands,
    ) -> Result<Self, SqliteDatabaseError>;
}
