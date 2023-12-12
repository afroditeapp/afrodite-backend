use std::{
    fmt,
    path::{Path, PathBuf},
};

use diesel::sqlite::Sqlite;
use simple_backend_config::{SimpleBackendConfig, file::{DatabaseInfo, SqliteDatabase}};
use error_stack::{ResultExt, Result};
use sqlx::{
    sqlite::{self, SqliteConnectOptions, SqlitePoolOptions, SqliteRow},
    Row, SqlitePool, database,
};
use tracing::log::error;
use simple_backend_utils::ComponentError;

pub const DATABASE_FILE_EXTENSION: &str = ".db";

impl ComponentError for SqliteDatabaseError {
    const COMPONENT_NAME: &'static str = "Sqlx";
}

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
    #[error("Starting transaction failed")]
    TransactionBegin,
    #[error("Rollbacking transaction failed")]
    TransactionRollback,
    #[error("Commiting transaction failed")]
    TransactionCommit,

    #[error("Deserialization error")]
    SerdeDeserialize,
    #[error("Serialization error")]
    SerdeSerialize,

    #[error("Time parsing error")]
    TimeParsing,

    #[error("TryFrom error")]
    TryFromError,
    #[error("Data format conversion error")]
    DataFormatConversion,

    #[error("Content slot not empty")]
    ContentSlotNotEmpty,
    #[error("Content slot is empty")]
    ContentSlotEmpty,
    #[error("ModerationRequestContentIsInvalid")]
    ModerationRequestContentInvalid,

    #[error("Creating in RAM config for sqlx failed")]
    CreateInRamOptions,
}

// /// Path to directory which contains Sqlite files.
// #[derive(Debug, Clone)]
// pub struct SqliteDatabasePath {
//     database_dir: PathBuf,
// }

// impl SqliteDatabasePath {
//     pub fn new(database_dir: PathBuf) -> Self {
//         Self { database_dir }
//     }

//     pub fn path(&self) -> &Path {
//         &self.database_dir
//     }
// }

pub struct SqlxWriteCloseHandle {
    pool: SqlitePool,
}

impl SqlxWriteCloseHandle {
    /// Call this before closing the server.
    pub async fn close(self) {
        self.pool.close().await
    }
}

#[derive(Debug, Clone)]
pub struct DbWriteHandle {
    database_info: SqliteDatabase,
    handle: SqlxWriteHandle,
    read_handle: SqlxReadHandle,
}

impl DbWriteHandle {
    pub fn new(
        database_info: SqliteDatabase,
        handle: SqlxWriteHandle,
    ) -> Self {
        Self {
            database_info,
            read_handle: SqlxReadHandle {
                pool: handle.pool().clone(),
            },
            handle,
        }
    }

    pub fn pool(&self) -> &SqlitePool {
        self.handle.pool()
    }

    pub fn read(&self) -> &SqlxReadHandle {
        &self.read_handle
    }
}

fn create_sqlite_connect_options(
    config: &SimpleBackendConfig,
    database_info: &SqliteDatabase,
    db_path: &Path,
    create_if_missing: bool,
) -> Result<SqliteConnectOptions, SqliteDatabaseError> {
    if config.sqlite_in_ram() {
        // TODO: validata database name
        let ram_str = format!(
            "sqlite:file:{}?mode=memory&cache=shared",
            database_info.name
        );

        let options = ram_str
            .parse::<SqliteConnectOptions>()
            .change_context(SqliteDatabaseError::CreateInRamOptions)?
            .foreign_keys(true);
        return Ok(options);
    }

    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(create_if_missing)
        .foreign_keys(true)
        .journal_mode(sqlite::SqliteJournalMode::Wal)
        // Synchronous Normal should be ok in WAL mode
        .synchronous(sqlite::SqliteSynchronous::Normal);

    let options = if config.litestream().is_some() {
        options
            // Litestream docs recommend 5 second timeout
            .busy_timeout(std::time::Duration::from_secs(5))
            // Prevent backend from removing WAL files
            .pragma("wal_autocheckpoint", "0")
    } else {
        options
    };

    Ok(options)
}

impl fmt::Debug for SqlxWriteHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SqliteWriteHandle").finish()
    }
}

#[derive(Clone)]
pub struct SqlxWriteHandle {
    pool: SqlitePool,
}

impl SqlxWriteHandle {
    pub async fn new(
        config: &SimpleBackendConfig,
        database_info: &SqliteDatabase,
        db_path: PathBuf,
    ) -> Result<(Self, SqlxWriteCloseHandle), SqliteDatabaseError> {
        let pool = SqlitePoolOptions::new().max_connections(1);

        let pool = if config.sqlite_in_ram() {
            pool.max_lifetime(None).idle_timeout(None)
        } else {
            pool.max_lifetime(None).idle_timeout(None)
        };

        let pool = pool
            .connect_with(create_sqlite_connect_options(config, database_info, &db_path, true)?)
            .await
            .change_context(SqliteDatabaseError::Connect)?;

        let write_handle = SqlxWriteHandle { pool: pool.clone() };

        let close_handle = SqlxWriteCloseHandle { pool };

        Ok((write_handle, close_handle))
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub async fn sqlite_version(&self) -> Result<String, SqliteDatabaseError> {
        let version = sqlx::query("SELECT sqlite_version()")
            .map(|x: SqliteRow| {
                let r: String = x.get(0);
                r
            })
            .fetch_one(&self.pool)
            .await
            .change_context(SqliteDatabaseError::Execute)?;
        Ok(version)
    }

    pub fn to_read_handle(&self) -> SqlxReadHandle {
        SqlxReadHandle {
            pool: self.pool.clone(),
        }
    }
}

pub struct SqlxReadCloseHandle {
    pool: SqlitePool,
}

impl SqlxReadCloseHandle {
    /// Call this before closing the server.
    pub async fn close(self) {
        self.pool.close().await
    }
}

#[derive(Clone)]
pub struct SqlxReadHandle {
    pool: SqlitePool,
}

impl fmt::Debug for SqlxReadHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SqliteReadHandle").finish()
    }
}

impl SqlxReadHandle {
    pub async fn new(
        config: &SimpleBackendConfig,
        database_info: &SqliteDatabase,
        db_path: PathBuf,
    ) -> Result<(Self, SqlxReadCloseHandle), SqliteDatabaseError> {
        let pool = SqlitePoolOptions::new().max_connections(num_cpus::get() as u32);

        let pool = if config.sqlite_in_ram() {
            pool.max_lifetime(None).idle_timeout(None)
        } else {
            pool.max_lifetime(None).idle_timeout(None)
        };

        let pool = pool
            .connect_with(create_sqlite_connect_options(&config, database_info, &db_path, false)?)
            .await
            .change_context(SqliteDatabaseError::Connect)?;

        let handle = SqlxReadHandle { pool: pool.clone() };

        let close_handle = SqlxReadCloseHandle { pool };

        Ok((handle, close_handle))
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}
