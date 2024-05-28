#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

pub mod data;
pub mod diesel_db;

use std::fmt::Debug;

use diesel_db::{
    DieselReadCloseHandle, DieselReadHandle, DieselWriteCloseHandle, DieselWriteHandle,
};

use diesel_migrations::EmbeddedMigrations;
use error_stack::{Result, ResultExt};
use simple_backend_config::SimpleBackendConfig;
use simple_backend_utils::ContextExt;

pub type PoolObject = deadpool_diesel::sqlite::Connection;

#[derive(thiserror::Error, Debug)]
pub enum DataError {
    #[error("Diesel error")]
    Diesel,
    #[error("Matching database not found from config")]
    MatchingDatabaseNotFoundFromConfig,
    #[error("File path creation failed")]
    FilePathCreationFailed,
}

#[derive(Clone, Debug)]
pub struct DbReadHandle {
    diesel_read: DieselReadHandle,
}

impl DbReadHandle {
    pub fn diesel(&self) -> &DieselReadHandle {
        &self.diesel_read
    }
}

pub struct DbReadCloseHandle {
    diesel_read_close: DieselReadCloseHandle,
}

#[derive(Clone, Debug)]
pub struct DbWriteHandle {
    diesel_write: DieselWriteHandle,
}

impl DbWriteHandle {
    pub fn diesel(&self) -> &DieselWriteHandle {
        &self.diesel_write
    }

    pub fn to_read_handle(&self) -> DbReadHandle {
        DbReadHandle {
            diesel_read: self.diesel_write.to_read_handle(),
        }
    }
}

impl DbReadCloseHandle {
    /// Call this before closing the server.
    pub async fn close(self) {
        self.diesel_read_close.close().await
    }
}

pub struct DbWriteCloseHandle {
    diesel_write_close: DieselWriteCloseHandle,
}

impl DbWriteCloseHandle {
    /// Call this before closing the server.
    pub async fn close(self) {
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

        let read = DbReadHandle {
            diesel_read,
        };
        let close = DbReadCloseHandle {
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

        let write = DbWriteHandle {
            diesel_write,
        };
        let close = DbWriteCloseHandle {
            diesel_write_close,
        };

        Ok((write, close))
    }
}
