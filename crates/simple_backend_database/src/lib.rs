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
use simple_backend_config::{Database, SimpleBackendConfig};

pub type PoolObject = diesel_db::PoolObject;

#[derive(thiserror::Error, Debug)]
pub enum SimpleDatabaseError {
    #[error("Diesel error")]
    Diesel,
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
        name: &Database,
    ) -> Result<(DbReadHandle, DbReadCloseHandle), SimpleDatabaseError> {
        let db_file_path = data::create_dirs_and_get_sqlite_database_file_path(config, name)?;

        let (diesel_read, diesel_read_close) =
            DieselReadHandle::new(config, name, db_file_path.clone())
                .await
                .change_context(SimpleDatabaseError::Diesel)?;

        let read = DbReadHandle { diesel_read };
        let close = DbReadCloseHandle { diesel_read_close };

        Ok((read, close))
    }

    /// Create write handle for database.
    ///
    /// Runs migrations.
    pub async fn create_write_handle_from_config(
        config: &SimpleBackendConfig,
        name: &Database,
        migrations: EmbeddedMigrations,
    ) -> Result<(DbWriteHandle, DbWriteCloseHandle), SimpleDatabaseError> {
        let db_file_path = data::create_dirs_and_get_sqlite_database_file_path(config, name)?;

        let (diesel_write, diesel_write_close) =
            DieselWriteHandle::new(config, name, db_file_path.clone(), migrations)
                .await
                .change_context(SimpleDatabaseError::Diesel)?;

        let write = DbWriteHandle { diesel_write };
        let close = DbWriteCloseHandle { diesel_write_close };

        Ok((write, close))
    }
}
