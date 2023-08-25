#![deny(unsafe_code)]
#![warn(unused_crate_dependencies)]

pub mod current;
pub mod diesel;
pub mod history;
pub mod sqlite;

use std::{fmt::Debug, marker::PhantomData};

use config::RUNNING_IN_DEBUG_MODE;
use error_stack::{Context, IntoReport, Result, ResultExt};
pub use model::schema;
use model::{AccountIdInternal, AccountId, ContentId, IsLoggingAllowed};
use utils::ComponentError;

use crate::diesel::DieselDatabaseError;

pub type PoolObject = deadpool_diesel::sqlite::Connection;

pub struct ErrorContext<T, Ok> {
    pub force_debug_print: bool,
    pub context_value: T,
    /// Makes the type printable
    pub context_type: PhantomData<T>,
    /// Makes the type printable
    pub ok_type: PhantomData<Ok>,
}

impl<T, Ok> ErrorContext<T, Ok> {
    pub fn new(e: T, force_debug_print: bool) -> Self {
        Self {
            force_debug_print,
            context_value: e,
            context_type: PhantomData,
            ok_type: PhantomData,
        }
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

pub trait IntoDatabaseError<Err: Context>: IntoReport {
    #[track_caller]
    fn into_db_error<T: Debug + IsLoggingAllowed>(
        self,
        e: Err,
        request_context: T,
    ) -> Result<Self::Ok, Err> {
        self.into_report()
            .change_context(e)
            .attach_printable_lazy(move || {
                let context = ErrorContext::<T, Self::Ok>::new(
                    request_context,
                    RUNNING_IN_DEBUG_MODE.value(),
                );

                format!("{:#?}", context)
            })
    }

    #[track_caller]
    fn into_transaction_error<T: Debug + IsLoggingAllowed>(
        self,
        e: Err,
        request_context: T,
    ) -> std::result::Result<Self::Ok, TransactionError<Err>> {
        self.into_db_error(e, request_context)
            .map_err(TransactionError)
    }
}

impl<E> From<error_stack::Report<E>> for TransactionError<E> {
    fn from(value: error_stack::Report<E>) -> Self {
        Self(value)
    }
}

impl From<::diesel::result::Error> for TransactionError<DieselDatabaseError> {
    fn from(value: ::diesel::result::Error) -> Self {
        TransactionError(
            error_stack::report!(value)
                .change_context(DieselDatabaseError::FromDieselErrorToTransactionError),
        )
    }
}

impl<E> From<TransactionError<E>> for error_stack::Report<E> {
    fn from(value: TransactionError<E>) -> Self {
        value.0
    }
}

// Workaround because it is not possible to implement From<diesel::result::Error>
// to error_stack::Report from here.
pub struct TransactionError<E>(error_stack::Report<E>);

impl<Ok> IntoDatabaseError<crate::diesel::DieselDatabaseError>
    for std::result::Result<Ok, ::diesel::result::Error>
{
}
impl<Ok> IntoDatabaseError<crate::diesel::DieselDatabaseError>
    for std::result::Result<Ok, ::serde_json::error::Error>
{
}

impl<Ok> IntoDatabaseError<crate::sqlite::SqliteDatabaseError>
    for std::result::Result<Ok, ::sqlx::Error>
{
}
