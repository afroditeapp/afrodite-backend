#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

pub mod data;
pub mod diesel_db;
pub mod sqlx_db;

use std::fmt::Debug;

use diesel_db::{
    DieselReadCloseHandle, DieselReadHandle, DieselWriteCloseHandle, DieselWriteHandle,
};

use diesel_migrations::EmbeddedMigrations;
use error_stack::{Result, ResultExt};
use simple_backend_config::SimpleBackendConfig;
use simple_backend_utils::ContextExt;
use sqlx_db::{SqlxReadCloseHandle, SqlxReadHandle, SqlxWriteCloseHandle, SqlxWriteHandle};

pub type PoolObject = deadpool_diesel::sqlite::Connection;

#[derive(thiserror::Error, Debug)]
pub enum DataError {
    #[error("Diesel error")]
    Diesel,
    #[error("Sqlx error")]
    Sqlx,
    #[error("Matching database not found from config")]
    MatchingDatabaseNotFoundFromConfig,
    #[error("File path creation failed")]
    FilePathCreationFailed,
}

#[derive(Clone, Debug)]
pub struct DbReadHandle {
    sqlx_read: SqlxReadHandle,
    diesel_read: DieselReadHandle,
}

impl DbReadHandle {
    pub fn sqlx(&self) -> &SqlxReadHandle {
        &self.sqlx_read
    }

    pub fn diesel(&self) -> &DieselReadHandle {
        &self.diesel_read
    }
}

pub struct DbReadCloseHandle {
    sqlx_read_close: SqlxReadCloseHandle,
    diesel_read_close: DieselReadCloseHandle,
}

#[derive(Clone, Debug)]
pub struct DbWriteHandle {
    sqlx_write: SqlxWriteHandle,
    diesel_write: DieselWriteHandle,
}

impl DbWriteHandle {
    pub fn sqlx(&self) -> &SqlxWriteHandle {
        &self.sqlx_write
    }

    pub fn diesel(&self) -> &DieselWriteHandle {
        &self.diesel_write
    }

    pub fn to_read_handle(&self) -> DbReadHandle {
        DbReadHandle {
            sqlx_read: self.sqlx_write.to_read_handle(),
            diesel_read: self.diesel_write.to_read_handle(),
        }
    }
}

impl DbReadCloseHandle {
    /// Call this before closing the server.
    pub async fn close(self) {
        self.sqlx_read_close.close().await;
        self.diesel_read_close.close().await
    }
}

pub struct DbWriteCloseHandle {
    sqlx_write_close: SqlxWriteCloseHandle,
    diesel_write_close: DieselWriteCloseHandle,
}

impl DbWriteCloseHandle {
    /// Call this before closing the server.
    pub async fn close(self) {
        self.sqlx_write_close.close().await;
        self.diesel_write_close.close().await
    }
}

pub struct DatabaseHandleCreator {}

impl DatabaseHandleCreator {
    /// Create read handle for database.
    ///
    /// Create the write handle first. Only that runs migrations.
    pub async fn create_read_handle_from_config(
        config: &SimpleBackendConfig,
        name: &'static str,
    ) -> Result<(DbReadHandle, DbReadCloseHandle), DataError> {
        let info = config
            .databases()
            .iter()
            .find(|db| db.file_name() == name)
            .ok_or(DataError::MatchingDatabaseNotFoundFromConfig.report())?;
        if info.file_name() != name {
            return Err(DataError::MatchingDatabaseNotFoundFromConfig.report());
        }

        let info = info.to_sqlite_database();

        let db_file_path = data::create_dirs_and_get_sqlite_database_file_path(config, &info)?;

        let (diesel_read, diesel_read_close) =
            DieselReadHandle::new(&config, &info, db_file_path.clone())
                .await
                .change_context(DataError::Diesel)?;

        let (sqlx_read, sqlx_read_close) = SqlxReadHandle::new(&config, &info, db_file_path)
            .await
            .change_context(DataError::Sqlx)?;

        let read = DbReadHandle {
            sqlx_read,
            diesel_read,
        };
        let close = DbReadCloseHandle {
            sqlx_read_close,
            diesel_read_close,
        };

        Ok((read, close))
    }

    /// Create write handle for database.
    ///
    /// Runs migrations.
    pub async fn create_write_handle_from_config(
        config: &SimpleBackendConfig,
        name: &'static str,
        migrations: EmbeddedMigrations,
    ) -> Result<(DbWriteHandle, DbWriteCloseHandle), DataError> {
        let info = config
            .databases()
            .iter()
            .find(|db| db.file_name() == name)
            .ok_or(DataError::MatchingDatabaseNotFoundFromConfig.report())?;
        if info.file_name() != name {
            return Err(DataError::MatchingDatabaseNotFoundFromConfig.report());
        }

        let info = info.to_sqlite_database();

        let db_file_path = data::create_dirs_and_get_sqlite_database_file_path(config, &info)?;

        let (diesel_write, diesel_write_close) =
            DieselWriteHandle::new(&config, &info, db_file_path.clone(), migrations)
                .await
                .change_context(DataError::Diesel)?;

        let (sqlx_write, sqlx_write_close) = SqlxWriteHandle::new(&config, &info, db_file_path)
            .await
            .change_context(DataError::Sqlx)?;

        let write = DbWriteHandle {
            sqlx_write,
            diesel_write,
        };
        let close = DbWriteCloseHandle {
            sqlx_write_close,
            diesel_write_close,
        };

        Ok((write, close))
    }
}
