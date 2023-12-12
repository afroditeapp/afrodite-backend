#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

pub mod diesel_db;
pub mod sqlx_db;
pub mod data;

use std::{fmt::Debug, marker::PhantomData};

use diesel_db::{DieselReadHandle, DieselReadCloseHandle, DieselWriteHandle, DieselWriteCloseHandle};
// use ::diesel::migration::MigrationSource;
// use diesel::{DieselReadHandle, DieselWriteHandle, DieselReadCloseHandle, DieselWriteCloseHandle};
use diesel_migrations::EmbeddedMigrations;
use simple_backend_config::{RUNNING_IN_DEBUG_MODE, SimpleBackendConfig};
use error_stack::{Context, ResultExt, Result};
use simple_backend_utils::{markers::IsLoggingAllowed, ContextExt};
use sqlx_db::{SqliteDatabaseError, SqlxReadHandle, SqlxReadCloseHandle, SqlxWriteCloseHandle, SqlxWriteHandle};
use sqlx::migrate::Migration;

use crate::diesel_db::DieselDatabaseError;

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

// pub struct ErrorContext<T, Ok> {
//     pub force_debug_print: bool,
//     pub context_value: T,
//     /// Makes the type printable
//     pub context_type: PhantomData<T>,
//     /// Makes the type printable
//     pub ok_type: PhantomData<Ok>,
// }

// impl<T, Ok> ErrorContext<T, Ok> {
//     pub fn new(e: T) -> Self {
//         Self {
//             force_debug_print: RUNNING_IN_DEBUG_MODE.value(),
//             context_value: e,
//             context_type: PhantomData,
//             ok_type: PhantomData,
//         }
//     }
// }

// impl<T: IsLoggingAllowed + std::fmt::Debug, Ok> std::fmt::Debug for ErrorContext<T, Ok> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         struct Printer<'a, T> {
//             value: &'a T,
//         }
//         impl<'a, T: IsLoggingAllowed> Debug for Printer<'a, T> {
//             fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//                 self.value.fmt_loggable(f)
//             }
//         }

//         let printer = Printer {
//             value: &self.context_value,
//         };

//         let printable = if self.force_debug_print {
//             &self.context_value
//         } else {
//             &printer as &dyn Debug
//         };

//         f.debug_struct("ErrorContext")
//             .field("context_value", printable)
//             .field("context_type", &self.context_type)
//             .field("ok_type", &self.ok_type)
//             .finish()
//     }
// }

// pub trait IntoDatabaseError<Err: Context>: ResultExt + Sized {
//     #[track_caller]
//     fn into_db_error<T: Debug + IsLoggingAllowed>(
//         self,
//         e: Err,
//         request_context: T,
//     ) -> Result<Self::Ok, Err> {
//         self.change_context(e)
//             .attach_printable_lazy(move || {
//                 let context = ErrorContext::<T, Self::Ok>::new(request_context);

//                 format!("{:#?}", context)
//             })
//     }

//     #[track_caller]
//     fn into_transaction_error<T: Debug + IsLoggingAllowed>(
//         self,
//         e: Err,
//         request_context: T,
//     ) -> std::result::Result<Self::Ok, TransactionError<Err>> {
//         self.into_db_error(e, request_context)
//             .map_err(TransactionError)
//     }

//     #[track_caller]
//     fn with_info<T: Debug + IsLoggingAllowed>(
//         self,
//         request_context: T,
//     ) -> Result<Self::Ok, <Self as ResultExt>::Context> {
//         self.attach_printable_lazy(move || {
//             let context = ErrorContext::<T, Self::Ok>::new(request_context);

//             format!("{:#?}", context)
//         })
//     }
// }

// impl<Ok> IntoDatabaseError<crate::diesel_db::DieselDatabaseError>
//     for std::result::Result<Ok, ::diesel::result::Error>
// {
// }

// impl<Ok> IntoDatabaseError<crate::diesel_db::DieselDatabaseError>
//     for std::result::Result<Ok, ::serde_json::error::Error>
// {
// }

// impl<Ok> IntoDatabaseError<crate::diesel_db::DieselDatabaseError>
//     for std::result::Result<Ok, crate::diesel_db::DieselDatabaseError>
// {
// }

// impl<Ok> IntoDatabaseError<crate::sqlx_db::SqliteDatabaseError>
//     for std::result::Result<Ok, ::sqlx::Error>
// {
// }

// // Workaround because it is not possible to implement From<diesel::result::Error>
// // to error_stack::Report from here.
// pub struct TransactionError<E>(error_stack::Report<E>);

// impl<E> From<error_stack::Report<E>> for TransactionError<E> {
//     fn from(value: error_stack::Report<E>) -> Self {
//         Self(value)
//     }
// }

// impl From<::diesel::result::Error> for TransactionError<DieselDatabaseError> {
//     fn from(value: ::diesel::result::Error) -> Self {
//         TransactionError(
//             error_stack::report!(value)
//                 .change_context(DieselDatabaseError::FromDieselErrorToTransactionError),
//         )
//     }
// }

// impl<E> From<TransactionError<E>> for error_stack::Report<E> {
//     fn from(value: TransactionError<E>) -> Self {
//         value.0
//     }
// }


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

pub struct DatabaseHandleCreator {

}

impl DatabaseHandleCreator {
    /// Create read handle for database.
    ///
    /// Create the write handle first. Only that runs migrations.
    pub async fn create_read_handle_from_config(
        config: &SimpleBackendConfig,
        name: &'static str
    ) -> Result<(DbReadHandle, DbReadCloseHandle), DataError> {
        let info = config.databases().iter().find(|db| db.file_name() == name).ok_or(DataError::MatchingDatabaseNotFoundFromConfig.report())?;
        if info.file_name() != name {
            return Err(DataError::MatchingDatabaseNotFoundFromConfig.report());
        }

        let info = info.to_sqlite_database();

        let db_file_path = data::create_dirs_and_get_sqlite_database_file_path(config, &info)?;

        let (diesel_read, diesel_read_close) = DieselReadHandle::new(&config, &info, db_file_path.clone())
            .await.change_context(DataError::Diesel)?;

        let (sqlx_read, sqlx_read_close) = SqlxReadHandle::new(&config, &info, db_file_path)
            .await.change_context(DataError::Sqlx)?;

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
        let info = config.databases().iter().find(|db| db.file_name() == name).ok_or(DataError::MatchingDatabaseNotFoundFromConfig.report())?;
        if info.file_name() != name {
            return Err(DataError::MatchingDatabaseNotFoundFromConfig.report());
        }

        let info = info.to_sqlite_database();

        let db_file_path = data::create_dirs_and_get_sqlite_database_file_path(config, &info)?;

        let (diesel_write, diesel_write_close) = DieselWriteHandle::new(
            &config,
            &info,
            db_file_path.clone(),
            migrations,
        )
            .await.change_context(DataError::Diesel)?;

        let (sqlx_write, sqlx_write_close) = SqlxWriteHandle::new(&config, &info, db_file_path)
            .await.change_context(DataError::Sqlx)?;

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
