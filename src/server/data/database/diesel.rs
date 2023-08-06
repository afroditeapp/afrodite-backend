use crate::{api::model::AccountIdInternal, server::data::DatabaseError};
use crate::config::{Config, info};

use super::history::read::HistoryReadCommands;
use super::sqlite::{DATABASE_FILE_NAME, HISTORY_FILE_NAME, SqliteDatabaseError};

use async_trait::async_trait;
use deadpool::managed::{HookErrorCause};
use deadpool_diesel::sqlite::{Manager, Pool, Hook};
use diesel::{Connection, RunQueryDsl, ConnectionError, sql_function, OptionalExtension};
use diesel_migrations::{EmbeddedMigrations, embed_migrations, MigrationHarness};
use sqlx::sqlite::SqliteRow;
use sqlx::Row;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::log::{info, error};

use super::history::write::HistoryWriteCommands;
use crate::server::data::database::current::{CurrentDataWriteCommands};

use error_stack::{Result, IntoReport, ResultExt};

use std::time::Duration;
use std::{path::{Path, PathBuf}, sync::Arc, fmt};

use sqlx::{
    sqlite::{self, SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};

use crate::utils::{IntoReportExt, IntoReportFromString};

pub type HookError = deadpool::managed::HookError<deadpool_diesel::Error>;

pub type DieselConnection = diesel::SqliteConnection;
pub type DieselPool = deadpool_diesel::sqlite::Pool;

pub const DIESEL_MIGRATIONS: EmbeddedMigrations = embed_migrations!();

mod sqlite_version {
    use diesel::sql_function;
    sql_function! { fn sqlite_version() -> Text }
}

#[derive(thiserror::Error, Debug)]
pub enum DieselDatabaseError {
    #[error("Connecting to SQLite database failed")]
    Connect,
    #[error("Executing SQL query failed")]
    Execute,
    #[error("Running diesel database migrations failed")]
    Migrate,

    #[error("Connection get failed from connection pool")]
    GetConnection,
    #[error("Interaction with database connection failed")]
    InteractionError,

    #[error("SQLite version query failed")]
    SqliteVersionQuery,

    #[error("Creating in RAM database failed")]
    CreateInRam,
}

pub struct DieselWriteCloseHandle {
    pool: DieselPool,
}

impl DieselWriteCloseHandle {
    /// Call this before closing the server.
    pub async fn close(self) {
        self.pool.close()
    }
}

#[derive(Debug, Clone)]
pub struct DieselCurrentWriteHandle {
    handle: DieselWriteHandle,
}

impl DieselCurrentWriteHandle {
    pub fn new(handle: DieselWriteHandle) -> Self {
        Self {
            handle,
        }
    }
}

impl fmt::Debug for DieselWriteHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DieselWriteHandle")
            .finish()
    }
}

fn create_manager(
    config: &Config,
    db_path: PathBuf,
) -> Result<Manager, DieselDatabaseError> {
    let manager = if config.sqlite_in_ram() {
        let ram_str = if db_path.ends_with(DATABASE_FILE_NAME) {
            "file:current?mode=memory&cache=shared"
        } else if db_path.ends_with(HISTORY_FILE_NAME) {
            "file:history?mode=memory&cache=shared"
        } else {
            return Err(DieselDatabaseError::CreateInRam)
                .into_report()
                .attach_printable("Unknown database file name");
        };

        Manager::new(ram_str, deadpool_diesel::Runtime::Tokio1)
    } else {
        Manager::new(db_path.to_string_lossy(), deadpool_diesel::Runtime::Tokio1)
    };

    Ok(manager)
}

#[derive(Clone)]
pub struct DieselWriteHandle {
    pool: DieselPool,
}

impl DieselWriteHandle {
    pub async fn new(
        config: &Config,
        db_path: PathBuf,
    ) -> Result<(Self, DieselWriteCloseHandle), DieselDatabaseError> {
        let manager = create_manager(config, db_path)?;

        let pool = Pool::builder(manager)
            .max_size(1)
            .post_create(sqlite_setup_hook(&config));

        let pool = if config.sqlite_in_ram() {
            // Prevent all in RAM database from being dropped

            pool
                .runtime(deadpool::Runtime::Tokio1)
                .recycle_timeout(Some(std::time::Duration::MAX))
        } else {
            pool
        };

        let pool = pool.build()
            .into_error(DieselDatabaseError::Connect)?;

        let conn = pool
            .get()
            .await
            .into_error(DieselDatabaseError::GetConnection)?;
        conn.interact(|conn| {
            conn.run_pending_migrations(DIESEL_MIGRATIONS)
                .map(|_| ())
        })
            .await
            .into_error_string(DieselDatabaseError::InteractionError)?
            .into_error_string(DieselDatabaseError::Migrate)?;

        // let pool_clone = pool.clone();
        // tokio::spawn( async move {
        //     loop {
        //         sleep(Duration::from_secs(5)).await;
        //         info!("{:?}", pool_clone.status());
        //     }
        // });

        let write_handle = DieselWriteHandle {
            pool: pool.clone(),
        };

        let close_handle = DieselWriteCloseHandle { pool: pool.clone() };

        Ok((write_handle, close_handle))
    }

    pub fn pool(&self) -> &DieselPool {
        &self.pool
    }

    pub async fn sqlite_version(&self) -> Result<String, DieselDatabaseError> {
        let conn = self.pool
            .get()
            .await
            .into_error(DieselDatabaseError::GetConnection)?;

        let sqlite_version: Vec<String> = conn.interact(move |conn| {
            diesel::select(sqlite_version::sqlite_version()).load(conn)
        })
            .await
            .into_error_string(DieselDatabaseError::Execute)?
            .into_error_string(DieselDatabaseError::Execute)?;

        sqlite_version
            .first()
            .ok_or(DieselDatabaseError::SqliteVersionQuery)
            .into_report()
            .cloned()
    }
}

pub struct DieselReadCloseHandle {
    pool: DieselPool,
}

impl DieselReadCloseHandle {
    /// Call this before closing the server.
    pub async fn close(self) {
        self.pool.close()
    }
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
pub struct DieselReadHandle {
    pool: DieselPool,
}

impl fmt::Debug for DieselReadHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DieselReadHandle")
            .finish()
    }
}

impl DieselReadHandle {
    pub async fn new(
        config: &Config,
        db_path: PathBuf,
    ) -> Result<(Self, DieselReadCloseHandle), DieselDatabaseError> {
        let manager = create_manager(config, db_path)?;
        let pool = Pool::builder(manager)
            .max_size(num_cpus::get())
            .post_create(sqlite_setup_hook(&config));


        let pool = if config.sqlite_in_ram() {
            // Prevent all in RAM database from being dropped

            pool
                .runtime(deadpool::Runtime::Tokio1)
                .recycle_timeout(Some(std::time::Duration::MAX))
        } else {
            pool
        };

        let pool = pool
            .build()
            .into_error(DieselDatabaseError::Connect)?;


        let pool_clone = pool.clone();
        tokio::spawn( async move {
            loop {
                sleep(Duration::from_secs(5)).await;
                info!("{:?}", pool_clone.status());
            }
        });


        let handle = DieselReadHandle {
            pool: pool.clone(),
        };

        let close_handle = DieselReadCloseHandle { pool };

        Ok((handle, close_handle))
    }

    pub fn pool(&self) -> &DieselPool {
        &self.pool
    }
}

#[derive(Debug, Clone)]
pub struct DieselCurrentReadHandle {
    handle: DieselReadHandle,
}

impl DieselCurrentReadHandle {
    pub fn new(handle: DieselReadHandle) -> Self {
        Self {
            handle,
        }
    }
}
