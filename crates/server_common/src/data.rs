use std::fmt::Debug;

use database::ErrorContext;
use error_stack::Context;
use model::markers::IsLoggingAllowed;

use crate::{internal_api::InternalApiError, result::WrappedReport};

pub mod cache;
pub mod file;
pub mod index;

#[derive(thiserror::Error, Debug)]
pub enum DataError {
    #[error("SQLite error")]
    Sqlite,
    #[error("Cache error")]
    Cache,
    #[error("File error")]
    File,
    #[error("I/O error")]
    Io,
    #[error("Profile index error")]
    ProfileIndex,
    #[error("Diesel error")]
    Diesel,
    #[error("Event sender access failed")]
    EventSenderAccessFailed,
    #[error("Time error")]
    Time,
    #[error("Database command result receiving failed")]
    CommandResultReceivingFailed,
    #[error("Feature disabled from config file")]
    FeatureDisabled,
    #[error("Not found")]
    NotFound,
    #[error("Tried to do something that is not allowed")]
    NotAllowed,
    #[error("Server closing in progress")]
    ServerClosingInProgress,
}

/// Attach more info to current error
///
/// This trait is for error container error_stack::Report<Err>
pub trait WithInfo<Ok, Err: Context>: Sized {
    fn into_error_without_context(self) -> std::result::Result<Ok, error_stack::Report<Err>>;

    #[track_caller]
    fn with_info<T: Debug + IsLoggingAllowed>(
        self,
        request_context: T,
    ) -> std::result::Result<Ok, error_stack::Report<Err>> {
        self.into_error_without_context().map_err(|e| {
            e.attach_printable(ErrorContext::<T, Ok>::new(request_context).printable())
        })
    }
}

impl<Ok, Err: Context> WithInfo<Ok, Err> for std::result::Result<Ok, error_stack::Report<Err>> {
    #[track_caller]
    fn into_error_without_context(self) -> std::result::Result<Ok, error_stack::Report<Err>> {
        self
    }
}

/// Attach more info to current error.
///
/// This trait is for error container WrappedReport<error_stack::Report<Err>>
pub trait WrappedWithInfo<Ok, Err: Context>: Sized {
    fn into_error_without_context(
        self,
    ) -> std::result::Result<Ok, WrappedReport<error_stack::Report<Err>>>;

    #[track_caller]
    fn with_info<T: Debug + IsLoggingAllowed>(
        self,
        request_context: T,
    ) -> std::result::Result<Ok, WrappedReport<error_stack::Report<Err>>> {
        self.into_error_without_context().map_err(|e| {
            e.attach_printable(ErrorContext::<T, Ok>::new(request_context).printable())
        })
    }
}

impl<Ok, Err: Context> WrappedWithInfo<Ok, Err>
    for std::result::Result<Ok, WrappedReport<error_stack::Report<Err>>>
{
    #[track_caller]
    fn into_error_without_context(
        self,
    ) -> std::result::Result<Ok, WrappedReport<error_stack::Report<Err>>> {
        self
    }
}

impl<Ok> WrappedWithInfo<Ok, InternalApiError> for std::result::Result<Ok, InternalApiError> {
    #[track_caller]
    fn into_error_without_context(
        self,
    ) -> std::result::Result<Ok, WrappedReport<error_stack::Report<InternalApiError>>> {
        let value = self?;
        Ok(value)
    }
}

/// Convert to DataError and attach more info to current error
pub trait IntoDataError<Ok, Err: Context>: Sized {
    fn into_data_error_without_context(
        self,
    ) -> std::result::Result<Ok, WrappedReport<error_stack::Report<Err>>>;

    #[track_caller]
    fn into_data_error<T: Debug + IsLoggingAllowed>(
        self,
        request_context: T,
    ) -> std::result::Result<Ok, WrappedReport<error_stack::Report<Err>>> {
        self.into_data_error_without_context().map_err(|e| {
            e.attach_printable(ErrorContext::<T, Ok>::new(request_context).printable())
        })
    }

    #[track_caller]
    fn into_error(self) -> std::result::Result<Ok, WrappedReport<error_stack::Report<Err>>> {
        self.into_data_error_without_context()
    }
}

impl<Ok> IntoDataError<Ok, simple_backend_database::SimpleDatabaseError>
    for error_stack::Result<Ok, simple_backend_database::SimpleDatabaseError>
{
    #[track_caller]
    fn into_data_error_without_context(
        self,
    ) -> std::result::Result<
        Ok,
        WrappedReport<error_stack::Report<simple_backend_database::SimpleDatabaseError>>,
    > {
        let value = self?;
        Ok(value)
    }
}

impl<Ok> IntoDataError<Ok, DataError> for error_stack::Result<Ok, crate::data::file::FileError> {
    #[track_caller]
    fn into_data_error_without_context(
        self,
    ) -> std::result::Result<Ok, WrappedReport<error_stack::Report<DataError>>> {
        let value = self?;
        Ok(value)
    }
}

impl<Ok> IntoDataError<Ok, DataError> for error_stack::Result<Ok, crate::data::cache::CacheError> {
    #[track_caller]
    fn into_data_error_without_context(
        self,
    ) -> std::result::Result<Ok, WrappedReport<error_stack::Report<DataError>>> {
        let value = self?;
        Ok(value)
    }
}

impl<Ok> IntoDataError<Ok, DataError>
    for error_stack::Result<Ok, simple_backend_database::diesel_db::DieselDatabaseError>
{
    #[track_caller]
    fn into_data_error_without_context(
        self,
    ) -> std::result::Result<Ok, WrappedReport<error_stack::Report<DataError>>> {
        let value = self?;
        Ok(value)
    }
}
