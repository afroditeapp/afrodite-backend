use std::{fmt, path::PathBuf};

use diesel::{QueryableByName, RunQueryDsl, SqliteConnection, sql_types::BigInt};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};
use error_stack::{Result, ResultExt};
use simple_backend_config::{Database, SimpleBackendConfig};
use simple_backend_utils::{ContextExt, IntoReportFromString};
use tracing::error;

mod connection;

pub use simple_backend_utils::db::{DieselDatabaseError, MyDbConnection};

pub type DieselConnection = MyDbConnection;
pub type DieselPool = deadpool::unmanaged::Pool<DieselConnection>;
pub type PoolObject = deadpool::unmanaged::Object<DieselConnection>;

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
    fn interact<F: FnOnce(&mut MyDbConnection) -> R + Send + 'static, R: Send + 'static>(
        self,
        action: F,
    ) -> impl std::future::Future<Output = Result<R, DieselDatabaseError>> + Send;
}

impl ObjectExtensions<MyDbConnection> for PoolObject {
    async fn interact<F: FnOnce(&mut MyDbConnection) -> R + Send + 'static, R: Send + 'static>(
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
    database_info: &Database,
    db_path: PathBuf,
    connection_count: usize,
) -> Result<DieselPool, DieselDatabaseError> {
    let pool = deadpool::unmanaged::Pool::new(connection_count);
    for _ in 0..connection_count {
        let conn = connection::create_connection(config, database_info, db_path.clone()).await?;
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
        database_info: &Database,
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

        let conn = pool
            .get()
            .await
            .change_context(DieselDatabaseError::GetConnection)?;
        let db_name = database_info.sqlite_name();
        conn.interact(move |conn| run_sqlite_wal_checkpoint(conn, db_name))
            .await?
            .into_error_string(DieselDatabaseError::Execute)?;

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

        conn.interact(|conn| conn.sqlite_version())
            .await?
            .and_then(|opt| opt.ok_or(DieselDatabaseError::SqliteVersionQuery.report()))
    }

    pub fn to_read_handle(&self) -> DieselReadHandle {
        DieselReadHandle {
            pool: self.pool.clone(),
        }
    }
}

/// Move data from WAL to DB
fn run_sqlite_wal_checkpoint(
    conn: &mut MyDbConnection,
    db_name: &str,
) -> Result<(), DieselDatabaseError> {
    if let MyDbConnection::Sqlite(sqlite_conn) = conn {
        #[derive(QueryableByName)]
        struct WalCheckpointResult {
            #[diesel(sql_type = BigInt)]
            busy: i64,
        }

        let result: diesel::QueryResult<WalCheckpointResult> =
            diesel::sql_query("PRAGMA wal_checkpoint(TRUNCATE);").get_result(sqlite_conn);

        match result {
            Ok(checkpoint_result) => {
                if checkpoint_result.busy != 0 {
                    error!(
                        "WAL checkpoint returned non-zero busy value: {}, DB: '{}'",
                        checkpoint_result.busy, db_name,
                    );
                }
            }
            Err(e) => {
                error!(
                    "Failed to execute WAL checkpoint: {:?}, DB: '{}'",
                    e, db_name
                );
            }
        }
    }
    Ok(())
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

pub fn sqlite_setup_connection(conn: &mut SqliteConnection) -> Result<(), DieselDatabaseError> {
    let pragmas = &[
        "PRAGMA journal_mode=WAL;",
        "PRAGMA synchronous=NORMAL;",
        "PRAGMA foreign_keys=ON;",
        "PRAGMA secure_delete=ON;",
    ];

    for pragma_str in pragmas.iter() {
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
        database_info: &Database,
        db_path: PathBuf,
    ) -> Result<(Self, DieselReadCloseHandle), DieselDatabaseError> {
        let connections = num_cpus::get();
        let pool = create_pool(config, database_info, db_path, connections).await?;

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
