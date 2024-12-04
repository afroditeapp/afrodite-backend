use std::{fmt, path::PathBuf};

use diesel::{Connection, RunQueryDsl, SqliteConnection};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};
use error_stack::{Result, ResultExt};
use simple_backend_config::{file::SqliteDatabase, SimpleBackendConfig};
use simple_backend_utils::{ComponentError, ContextExt, IntoReportFromString};
use tracing::log::error;

pub type DieselConnection = diesel::SqliteConnection;
pub type DieselPool = deadpool::unmanaged::Pool<DieselConnection>;
pub type PoolObject = deadpool::unmanaged::Object<DieselConnection>;

mod sqlite_version {
    use diesel::define_sql_function;
    define_sql_function! { fn sqlite_version() -> Text }
}

impl ComponentError for DieselDatabaseError {
    const COMPONENT_NAME: &'static str = "Diesel";
}

#[derive(thiserror::Error, Debug)]
pub enum DieselDatabaseError {
    #[error("Connecting to SQLite database failed")]
    Connect,
    #[error("SQLite connection setup failed")]
    Setup,
    #[error("Executing SQL query failed")]
    Execute,
    #[error("Running diesel database migrations failed")]
    Migrate,

    #[error("Running an action failed")]
    RunAction,
    #[error("Add connection to pool failed")]
    AddConnection,
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

    #[error("Not found")]
    NotFound,
    #[error("Operation is not allowed")]
    NotAllowed,
    #[error("Action is already done")]
    AlreadyDone,
    #[error("No available IDs")]
    NoAvailableIds,

    #[error("Data format conversion failed")]
    DataFormatConversion,

    #[error("Transaction failed")]
    FromDieselErrorToTransactionError,

    #[error("File operation failed")]
    File,

    #[error("Diesel error")]
    DieselError,

    #[error("Transaction error")]
    FromStdErrorToTransactionError,
}

async fn close_connections(pool: &DieselPool, connections: usize) {
    for _ in 0..connections {
        let result = pool.remove().await;
        match result {
            Ok(conn) => drop(conn),
            Err(_) => error!("Failed to remove connection from pool"),
        }
    }
}

pub struct DieselWriteCloseHandle {
    pool: DieselPool,
    connections: usize,
}

impl DieselWriteCloseHandle {
    /// Call this before closing the server.
    pub async fn close(self) {
        close_connections(&self.pool, self.connections).await;
        self.pool.close()
    }
}

impl fmt::Debug for DieselWriteHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DieselWriteHandle").finish()
    }
}

pub trait ObjectExtensions<T>: Sized {
    fn interact<F: FnOnce(&mut SqliteConnection) -> R + Send + 'static, R: Send + 'static>(
        self,
        action: F,
    ) -> impl std::future::Future<Output = Result<R, DieselDatabaseError>> + Send;
}

impl ObjectExtensions<SqliteConnection> for PoolObject {
    async fn interact<F: FnOnce(&mut SqliteConnection) -> R + Send + 'static, R: Send + 'static>(
        mut self,
        action: F,
    ) -> Result<R, DieselDatabaseError> {
        let handle = tokio::task::spawn_blocking(move || {
            let conn = self.as_mut();
            action(conn)
        });
        match handle.await {
            Ok(value) => Ok(value),
            Err(e) => Err(e.report()).change_context(DieselDatabaseError::RunAction),
        }
    }
}

async fn create_pool(
    config: &SimpleBackendConfig,
    database_info: &SqliteDatabase,
    db_path: PathBuf,
    connection_count: usize,
) -> Result<DieselPool, DieselDatabaseError> {
    let db_str = if config.sqlite_in_ram() {
        // TODO: validate name?
        format!("file:{}?mode=memory&cache=shared", database_info.name)
    } else {
        db_path.to_string_lossy().to_string()
    };

    let pool = deadpool::unmanaged::Pool::new(connection_count);
    for _ in 0..connection_count {
        let mut conn =
            SqliteConnection::establish(&db_str).change_context(DieselDatabaseError::Connect)?;
        sqlite_setup_connection(config, &mut conn)?;
        pool.add(conn)
            .await
            .map_err(|(_, e)| e)
            .change_context(DieselDatabaseError::AddConnection)?;
    }

    Ok(pool)
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
        let connections = 1;
        let pool = create_pool(config, database_info, db_path, connections).await?;

        let conn = pool
            .get()
            .await
            .change_context(DieselDatabaseError::GetConnection)?;
        conn.interact(|conn| conn.run_pending_migrations(migrations).map(|_| ()))
            .await?
            .into_error_string(DieselDatabaseError::Migrate)?;

        // let pool_clone = pool.clone();
        // std::thread::spawn(move || {
        //     loop {
        //         std::thread::sleep(std::time::Duration::from_secs(5));
        //         tracing::info!("write pool: {:?}", pool_clone.status());
        //     }
        // });

        let write_handle = DieselWriteHandle { pool: pool.clone() };

        let close_handle = DieselWriteCloseHandle {
            pool: pool.clone(),
            connections,
        };

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
            .await?
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
    connections: usize,
}

impl DieselReadCloseHandle {
    /// Call this before closing the server.
    pub async fn close(self) {
        close_connections(&self.pool, self.connections).await;
        self.pool.close()
    }
}

pub fn sqlite_setup_connection(
    config: &SimpleBackendConfig,
    conn: &mut SqliteConnection,
) -> Result<(), DieselDatabaseError> {
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

    for pragma_str in pragmas.iter().chain(litestram_pragmas) {
        diesel::sql_query(*pragma_str)
            .execute(conn)
            .change_context(DieselDatabaseError::Setup)?;
    }

    Ok(())
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
        let connections = num_cpus::get();
        let pool = create_pool(config, database_info, db_path, connections).await?;

        // let pool_clone = pool.clone();
        // std::thread::spawn(move || {
        //     loop {
        //         std::thread::sleep(std::time::Duration::from_secs(5));
        //         tracing::info!("read pool: {:?}", pool_clone.status());
        //     }
        // });

        let handle = DieselReadHandle { pool: pool.clone() };

        let close_handle = DieselReadCloseHandle { pool, connections };

        Ok((handle, close_handle))
    }

    pub fn pool(&self) -> &DieselPool {
        &self.pool
    }
}

pub trait ConnectionProvider {
    fn conn(&mut self) -> &mut DieselConnection;
}

impl ConnectionProvider for &mut DieselConnection {
    fn conn(&mut self) -> &mut DieselConnection {
        self
    }
}
