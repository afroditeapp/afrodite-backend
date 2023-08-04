use crate::{api::model::AccountIdInternal, server::data::DatabaseError};
use crate::config::Config;

use super::history::read::HistoryReadCommands;

use async_trait::async_trait;
use deadpool::managed::{HookErrorCause};
use deadpool_diesel::sqlite::{Manager, Pool, Hook};
use diesel::{Connection, RunQueryDsl, ConnectionError};
use diesel_migrations::{EmbeddedMigrations, embed_migrations, MigrationHarness};
use sqlx::sqlite::SqliteRow;
use sqlx::Row;
use tokio::sync::Mutex;
use tracing::log::{info, error};

use super::history::write::HistoryWriteCommands;
use crate::server::data::database::current::{CurrentDataWriteCommands, SqliteReadCommands};

use error_stack::Result;

use std::{path::{Path, PathBuf}, sync::Arc, fmt};

use sqlx::{
    sqlite::{self, SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};

use crate::utils::{IntoReportExt, IntoReportFromString};

pub type HookError = deadpool::managed::HookError<deadpool_diesel::Error>;


pub const DIESEL_MIGRATIONS: EmbeddedMigrations = embed_migrations!();

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

    #[error("Connection get failed from connection pool")]
    GetConnection,
    #[error("Interaction with database connection failed")]
    InteractionError,
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
    pub fn new(handle: SqliteWriteHandle, read_handle: SqliteReadHandle) -> Self {
        Self {
            handle,
            read_handle,        // TODO: use write handle for reading?
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

fn create_sqlite_connect_options(
    config: &Config,
    db_path: &Path,
    create_if_missing: bool,
) -> SqliteConnectOptions {
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

    options
}

impl fmt::Debug for SqliteWriteHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SqliteWriteHandle")
            .finish()
    }
}

#[derive(Clone)]
pub struct SqliteWriteHandle {
    pool: SqlitePool,
    diesel_pool: deadpool_diesel::Pool<Manager>,
}

impl SqliteWriteHandle {
    pub async fn new(
        config: &Config,
        db_path: PathBuf,
    ) -> Result<(Self, SqliteWriteCloseHandle), SqliteDatabaseError> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(create_sqlite_connect_options(config, &db_path, true))
            .await
            .into_error(SqliteDatabaseError::Connect)?;

        sqlx::migrate!()
            .run(&pool)
            .await
            .into_error(SqliteDatabaseError::Migrate)?;


        let manager = Manager::new(db_path.to_string_lossy(), deadpool_diesel::Runtime::Tokio1);
        let diesel_pool = Pool::builder(manager)
            .max_size(1)
            .post_create(sqlite_setup_hook(&config))
            .build()
            .into_error(SqliteDatabaseError::Connect)?;

        let connection = diesel_pool
            .get()
            .await
            .into_error(SqliteDatabaseError::GetConnection)?;
        connection.interact(|connection| {
            connection.run_pending_migrations(DIESEL_MIGRATIONS)
                .map(|_| ())
        })
            .await
            .into_error_string(SqliteDatabaseError::InteractionError)?
            .into_error_string(SqliteDatabaseError::Migrate)?;

        let write_handle = SqliteWriteHandle {
            pool: pool.clone(),
            diesel_pool,
        };

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

pub fn diesel_open_sqlite_for_reading(
    config: &Config,
    db_path: PathBuf,
) -> diesel::SqliteConnection {
    diesel::SqliteConnection::establish(&db_path.to_string_lossy())
        .unwrap_or_else(|_| panic!("Error connecting to {}", db_path.to_string_lossy()))
}

pub fn sqlite_setup_hook(config: &Config) -> Hook {
    let pragmas = &[
        "PRAGMA journal_mode=WAL;",
        "PRAGMA synchronous=NORMAL;",
        "PRAGMA foreign_keys=ON;",
    ];

    let litestram_pragmas = if config.litestream().is_some() {
        &[
            // Litestream docs recommend 5 second timeout
            "PRAGMA busy_timeout=5000;",
            // Prevent backend from removing WAL files
            "PRAGMA wal_autocheckpoint=0;",
        ]
    } else {
        [].as_slice()
    };

    Hook::async_fn(move |pool, _| {
        Box::pin(
            async move {
                pool.interact(move |connection| {
                    for pragma_str in pragmas.iter().chain(litestram_pragmas) {
                        diesel::sql_query(*pragma_str).execute(connection)?;
                    }

                    Ok(())
                })
                .await
                .map_err(|e| {
                    error!("Error: {}", e);
                    HookError::Abort(HookErrorCause::Message(e.to_string()))
                })?
                .map_err(|e: diesel::result::Error| {
                    error!("Error: {}", e);
                    HookError::Abort(HookErrorCause::Backend(e.into()))
                })
            }
        )
    })
}

#[derive(Clone)]
pub struct SqliteReadHandle {
    pool: SqlitePool,
    pub diesel_pool: deadpool_diesel::Pool<Manager>,
}

impl fmt::Debug for SqliteReadHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SqliteReadHandle")
            .finish()
    }
}

impl SqliteReadHandle {
    pub async fn new(
        config: &Config,
        db_path: PathBuf,
    ) -> Result<(Self, SqliteReadCloseHandle), SqliteDatabaseError> {
        let pool = SqlitePoolOptions::new()
            .max_connections(16)
            .connect_with(create_sqlite_connect_options(&config, &db_path, false))
            .await
            .into_error(SqliteDatabaseError::Connect)?;

        let manager = Manager::new(db_path.to_string_lossy(), deadpool_diesel::Runtime::Tokio1);
        let diesel_pool = Pool::builder(manager)
            .max_size(8)
            .post_create(sqlite_setup_hook(&config))
            .build()
            .into_error(SqliteDatabaseError::Connect)?;

        let handle = SqliteReadHandle {
            pool: pool.clone(),
            diesel_pool,
        };

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

pub async fn print_sqlite_version(pool: &SqlitePool) -> Result<(), SqliteDatabaseError> {
    let q = sqlx::query("SELECT sqlite_version()")
        .map(|x: SqliteRow| {
            let r: String = x.get(0);
            r
        })
        .fetch_one(pool)
        .await
        .into_error(SqliteDatabaseError::Execute)?;

    info!("SQLite version: {}", q);
    Ok(())
}
