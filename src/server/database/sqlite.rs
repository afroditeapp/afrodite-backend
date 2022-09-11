use std::path::{PathBuf, Path};

use sqlx::{SqliteConnection, Sqlite, SqlitePool, sqlite::{SqliteConnectOptions, self, SqlitePoolOptions}};

use crate::api::core::user::UserId;


pub const DATABASE_FILE_NAME: &str = "current.db";

#[derive(Debug)]
pub enum SqliteDatabaseError {
    Path,
    Connect(sqlx::Error),
    Execute(sqlx::Error),
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
            db_initial_setup(&pool).await?;
        }

        let write_handle = SqliteWriteHandle {
            pool: pool.clone()
        };

        let close_handle = SqliteWriteCloseHandle {
            pool,
        };

        Ok((write_handle, close_handle))
    }

    pub async fn insert_profile(&self, id: UserId) -> Result<(), SqliteDatabaseError> {
        sqlx::query(
            r#"
            INSERT INTO Profile (id)
            VALUES (?)
            "#
        )
        .bind(id)
        .execute(&self.pool).await.map_err(SqliteDatabaseError::Execute)?;

        Ok(())
    }

    pub async fn update_name(&self, id: UserId, name: &str) -> Result<(), SqliteDatabaseError> {
        sqlx::query(
            r#"
            UPDATE Profile
            SET name = ?
            WHERE id = ?
            "#
        )
        .bind(name)
        .bind(id)
        .execute(&self.pool).await.map_err(SqliteDatabaseError::Execute)?;

        Ok(())
    }
}



/// Creates database if it does not exists
pub async fn db_initial_setup(pool: &SqlitePool) -> Result<(), SqliteDatabaseError> {

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS Profile(
            id          TEXT PRIMARY KEY NOT NULL,
            name   TEXT
        )
        "#
    ).execute(pool).await.map_err(SqliteDatabaseError::Execute)?;



    Ok(())
}
