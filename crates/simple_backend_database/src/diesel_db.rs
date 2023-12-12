use std::{fmt, path::PathBuf};

use deadpool::managed::HookErrorCause;
use deadpool_diesel::sqlite::{Hook, Manager, Pool};
use diesel::RunQueryDsl;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};
use error_stack::{Result, ResultExt};
use simple_backend_config::{file::SqliteDatabase, SimpleBackendConfig};
use simple_backend_utils::{ComponentError, ContextExt, IntoReportFromString};
use tracing::log::error;

pub type HookError = deadpool::managed::HookError<deadpool_diesel::Error>;

pub type DieselConnection = diesel::SqliteConnection;
pub type DieselPool = deadpool_diesel::sqlite::Pool;

mod sqlite_version {
    use diesel::sql_function;
    sql_function! { fn sqlite_version() -> Text }
}

impl ComponentError for DieselDatabaseError {
    const COMPONENT_NAME: &'static str = "Diesel";
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

    #[error("Deserializing failed")]
    SerdeDeserialize,
    #[error("Serializing failed")]
    SerdeSerialize,

    #[error("Content slot not empty")]
    ContentSlotNotEmpty,
    #[error("Content slot empty")]
    ContentSlotEmpty,
    #[error("Moderation request content invalid")]
    ModerationRequestContentInvalid,
    #[error("Moderation request is missing")]
    MissingModerationRequest,

    #[error("Operation is not allowed")]
    NotAllowed,

    #[error("Data format conversion failed")]
    DataFormatConversion,

    #[error("Transaction failed")]
    FromDieselErrorToTransactionError,

    #[error("Connection pool locking error")]
    LockConnectionFailed,

    #[error("File operation failed")]
    File,
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

impl fmt::Debug for DieselWriteHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DieselWriteHandle").finish()
    }
}

fn create_manager(
    config: &SimpleBackendConfig,
    database_info: &SqliteDatabase,
    db_path: PathBuf,
) -> Result<Manager, DieselDatabaseError> {
    let manager = if config.sqlite_in_ram() {
        // TODO: validate name?
        let ram_str = format!("file:{}?mode=memory&cache=shared", database_info.name);

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
    /// Create new connection pool
    ///
    /// pub const DIESEL_MIGRATIONS: EmbeddedMigrations = embed_migrations!();
    pub async fn new(
        config: &SimpleBackendConfig,
        database_info: &SqliteDatabase,
        db_path: PathBuf,
        migrations: EmbeddedMigrations,
    ) -> Result<(Self, DieselWriteCloseHandle), DieselDatabaseError> {
        let manager = create_manager(config, database_info, db_path)?;

        let pool = Pool::builder(manager)
            .max_size(1)
            .post_create(sqlite_setup_hook(&config));

        let pool = if config.sqlite_in_ram() {
            // Prevent all in RAM database from being dropped

            pool.runtime(deadpool::Runtime::Tokio1)
                .recycle_timeout(Some(std::time::Duration::MAX))
        } else {
            pool
        };

        let pool = pool.build().change_context(DieselDatabaseError::Connect)?;

        let conn = pool
            .get()
            .await
            .change_context(DieselDatabaseError::GetConnection)?;
        conn.interact(|conn| conn.run_pending_migrations(migrations).map(|_| ()))
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

        let write_handle = DieselWriteHandle { pool: pool.clone() };

        let close_handle = DieselWriteCloseHandle { pool: pool.clone() };

        Ok((write_handle, close_handle))
    }

    pub fn pool(&self) -> &DieselPool {
        &self.pool
    }

    pub async fn sqlite_version(&self) -> Result<String, DieselDatabaseError> {
        let conn = self
            .pool
            .get()
            .await
            .change_context(DieselDatabaseError::GetConnection)?;

        let sqlite_version: Vec<String> = conn
            .interact(move |conn| diesel::select(sqlite_version::sqlite_version()).load(conn))
            .await
            .into_error_string(DieselDatabaseError::Execute)?
            .into_error_string(DieselDatabaseError::Execute)?;

        sqlite_version
            .first()
            .ok_or(DieselDatabaseError::SqliteVersionQuery.report())
            .cloned()
    }

    pub fn to_read_handle(&self) -> DieselReadHandle {
        DieselReadHandle {
            pool: self.pool.clone(),
        }
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

pub fn sqlite_setup_hook(config: &SimpleBackendConfig) -> Hook {
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
        Box::pin(async move {
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
        })
    })
}

#[derive(Clone)]
pub struct DieselReadHandle {
    pool: DieselPool,
}

impl fmt::Debug for DieselReadHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DieselReadHandle").finish()
    }
}

impl DieselReadHandle {
    pub async fn new(
        config: &SimpleBackendConfig,
        database_info: &SqliteDatabase,
        db_path: PathBuf,
    ) -> Result<(Self, DieselReadCloseHandle), DieselDatabaseError> {
        let manager = create_manager(config, database_info, db_path)?;
        let pool = Pool::builder(manager)
            .max_size(num_cpus::get())
            .post_create(sqlite_setup_hook(&config));

        let pool = if config.sqlite_in_ram() {
            // Prevent all in RAM database from being dropped

            pool.runtime(deadpool::Runtime::Tokio1)
                .recycle_timeout(Some(std::time::Duration::MAX))
        } else {
            pool
        };

        let pool = pool.build().change_context(DieselDatabaseError::Connect)?;

        // let pool_clone = pool.clone();
        // tokio::spawn(async move {
        //     loop {
        //         sleep(Duration::from_secs(5)).await;
        //         info!("{:?}", pool_clone.status());
        //     }
        // });

        let handle = DieselReadHandle { pool: pool.clone() };

        let close_handle = DieselReadCloseHandle { pool };

        Ok((handle, close_handle))
    }

    pub fn pool(&self) -> &DieselPool {
        &self.pool
    }
}

pub trait ConnectionProvider {
    fn conn(&mut self) -> &mut DieselConnection;
    // fn read(&mut self) -> crate::current::read::CurrentSyncReadCommands<&mut DieselConnection> {
    //     crate::current::read::CurrentSyncReadCommands::new(self.conn())
    // }
}

impl ConnectionProvider for &mut DieselConnection {
    fn conn(&mut self) -> &mut DieselConnection {
        self
    }
}
