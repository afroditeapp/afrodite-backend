#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

pub mod current;
pub mod history;

use std::{fmt::Debug, marker::PhantomData};

use diesel_migrations::{embed_migrations, EmbeddedMigrations};
use error_stack::{Context, Result, ResultExt};
pub use model::schema;
use model::IsLoggingAllowed;
use simple_backend_config::RUNNING_IN_DEBUG_MODE;
use simple_backend_database::{diesel_db::DieselDatabaseError, DbReadHandle, DbWriteHandle};

pub const DIESEL_MIGRATIONS: EmbeddedMigrations = embed_migrations!();

/// Write handle for current database.
#[derive(Clone, Debug)]
pub struct CurrentWriteHandle(pub DbWriteHandle);

impl CurrentWriteHandle {
    pub fn to_read_handle(&self) -> CurrentReadHandle {
        CurrentReadHandle(self.0.to_read_handle())
    }
}

/// Read handle for current database.
#[derive(Debug, Clone)]
pub struct CurrentReadHandle(pub DbReadHandle);

/// Write handle for current database.
#[derive(Clone, Debug)]
pub struct HistoryWriteHandle(pub DbWriteHandle);

/// Read handle for current database.
#[derive(Clone, Debug)]
pub struct HistoryReadHandle(pub DbReadHandle);

pub struct ErrorContext<T, Ok> {
    pub force_debug_print: bool,
    pub context_value: T,
    /// Makes the type printable
    pub context_type: PhantomData<T>,
    /// Makes the type printable
    pub ok_type: PhantomData<Ok>,
}

impl<T, Ok> ErrorContext<T, Ok> {
    pub fn new(e: T) -> Self {
        Self {
            force_debug_print: RUNNING_IN_DEBUG_MODE.value(),
            context_value: e,
            context_type: PhantomData,
            ok_type: PhantomData,
        }
    }
}

impl<T: IsLoggingAllowed + std::fmt::Debug, Ok> ErrorContext<T, Ok> {
    pub fn printable(&self) -> String {
        format!("{:#?}", self)
    }
}

impl<T: IsLoggingAllowed + std::fmt::Debug, Ok> std::fmt::Debug for ErrorContext<T, Ok> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        struct Printer<'a, T> {
            value: &'a T,
        }
        impl<'a, T: IsLoggingAllowed> Debug for Printer<'a, T> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.value.fmt_loggable(f)
            }
        }

        let printer = Printer {
            value: &self.context_value,
        };

        let printable = if self.force_debug_print {
            &self.context_value
        } else {
            &printer as &dyn Debug
        };

        f.debug_struct("ErrorContext")
            .field("context_value", printable)
            .field("context_type", &self.context_type)
            .field("ok_type", &self.ok_type)
            .finish()
    }
}

trait IntoDatabaseError<Err: Context>: ResultExt + Sized {
    const DEFAULT_NEW_ERROR: Self::NewError;
    type NewError: Context;

    #[track_caller]
    fn into_db_error<T: Debug + IsLoggingAllowed>(
        self,
        request_context: T,
    ) -> Result<Self::Ok, Self::NewError> {
        self.change_context(Self::DEFAULT_NEW_ERROR)
            .attach_printable_lazy(move || {
                let context = ErrorContext::<T, Self::Ok>::new(request_context);

                format!("{:#?}", context)
            })
    }
}

trait IntoDatabaseErrorExt<Err: Context>: ResultExt + Sized {
    #[track_caller]
    fn with_info<T: Debug + IsLoggingAllowed>(
        self,
        request_context: T,
    ) -> Result<Self::Ok, <Self as ResultExt>::Context> {
        self.attach_printable_lazy(move || {
            let context = ErrorContext::<T, Self::Ok>::new(request_context);

            format!("{:#?}", context)
        })
    }
}

impl<Ok> IntoDatabaseError<DieselDatabaseError>
    for std::result::Result<Ok, ::diesel::result::Error>
{
    const DEFAULT_NEW_ERROR: Self::NewError = DieselDatabaseError::DieselError;
    type NewError = DieselDatabaseError;
}

impl<Ok> IntoDatabaseErrorExt<DieselDatabaseError>
    for std::result::Result<Ok, ::serde_json::error::Error>
{
}

impl<Ok> IntoDatabaseErrorExt<DieselDatabaseError>
    for std::result::Result<Ok, DieselDatabaseError>
{
}

impl<Ok> IntoDatabaseErrorExt<DieselDatabaseError>
    for std::result::Result<Ok, model::account::AccountStateError>
{
}

// Workaround because it is not possible to implement From<diesel::result::Error>
// to error_stack::Report from here.
pub struct TransactionError(error_stack::Report<DieselDatabaseError>);

impl TransactionError {
    pub fn into_report(self) -> error_stack::Report<DieselDatabaseError> {
        self.0
    }
}

impl <E: std::error::Error> From<error_stack::Report<E>> for TransactionError {
    fn from(value: error_stack::Report<E>) -> Self {
        Self(value.change_context(DieselDatabaseError::FromStdErrorToTransactionError))
    }
}

impl From<::diesel::result::Error> for TransactionError {
    fn from(value: ::diesel::result::Error) -> Self {
        TransactionError(
            error_stack::report!(value)
                .change_context(DieselDatabaseError::FromDieselErrorToTransactionError),
        )
    }
}
