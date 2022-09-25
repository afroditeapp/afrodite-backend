pub mod read;
pub mod write;

use std::path::{PathBuf, Path};

use sqlx::{SqliteConnection, Sqlite, SqlitePool, sqlite::{SqliteConnectOptions, self, SqlitePoolOptions}};

use crate::api::core::user::UserId;


pub const DATABASE_FILE_NAME: &str = "current.db";

#[derive(Debug)]
pub enum SqliteDatabaseError {
    Path,
    Connect(sqlx::Error),
    Execute(sqlx::Error),
    Migrate(sqlx::migrate::MigrateError),
}


/// Path to directory which contains Sqlite files.
#[derive(Debug, Clone)]
pub struct SqliteDatabasePath {
    database_dir: PathBuf,
}

impl SqliteDatabasePath {
    pub fn new<T: ToOwned<Owned = PathBuf>>(database_dir: T) -> Self {
        Self {
            database_dir: database_dir.to_owned(),
        }
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
pub struct SqliteWriteHandle {
    pool: SqlitePool,
}

impl SqliteWriteHandle {
    pub async fn new(dir: SqliteDatabasePath) -> Result<(Self, SqliteWriteCloseHandle), SqliteDatabaseError> {

        let db_path = dir.path().join(DATABASE_FILE_NAME);

        let run_initial_setup = !db_path.exists();

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(
                SqliteConnectOptions::new()
                    .filename(db_path)
                    .create_if_missing(true)
                    .journal_mode(sqlite::SqliteJournalMode::Wal)
        ).await.map_err(SqliteDatabaseError::Connect)?;

        if run_initial_setup {
            sqlx::migrate!().run(&pool).await.map_err(SqliteDatabaseError::Migrate)?;
        }

        let write_handle = SqliteWriteHandle {
            pool: pool.clone()
        };

        let close_handle = SqliteWriteCloseHandle {
            pool,
        };

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
    pub async fn new(dir: SqliteDatabasePath) -> Result<(Self, SqliteReadCloseHandle), SqliteDatabaseError> {

        let db_path = dir.path().join(DATABASE_FILE_NAME);

        let pool = SqlitePoolOptions::new()
            .max_connections(16)
            .connect_with(
                SqliteConnectOptions::new()
                    .filename(db_path)
                    .create_if_missing(false)
                    .journal_mode(sqlite::SqliteJournalMode::Wal)
        ).await.map_err(SqliteDatabaseError::Connect)?;

        let handle = SqliteReadHandle {
            pool: pool.clone()
        };

        let close_handle = SqliteReadCloseHandle {
            pool,
        };

        Ok((handle, close_handle))
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}
