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
use model::{AccountIdInternal, AccountIdLight, ContentId, IsLoggingAllowed};
use utils::ComponentError;

use crate::diesel::DieselDatabaseError;

pub type PoolObject = deadpool_diesel::sqlite::Connection;

// #[derive(thiserror::Error, Debug)]
// pub enum DatabaseError {
//     #[error("Git error")]
//     Git,
//     #[error("SQLite error")]
//     Sqlite,
//     #[error("Cache error")]
//     Cache,
//     #[error("File error")]
//     File,
//     #[error("Media backup error")]
//     MediaBackup,

//     #[error("Diesel error")]
//     Diesel,

//     #[error("Database command sending failed")]
//     CommandSendingFailed,
//     #[error("Database command result receiving failed")]
//     CommandResultReceivingFailed,

//     // Other errors
//     #[error("Database initialization error")]
//     Init,
//     #[error("Database SQLite and Git integrity check")]
//     Integrity,
//     #[error("Feature disabled from config file")]
//     FeatureDisabled,

//     #[error("Command runner quit too early")]
//     CommandRunnerQuit,

//     #[error("Different SQLite versions detected between diesel and sqlx")]
//     SqliteVersionMismatch,
// }

// pub trait Printable: fmt::Debug {}

// impl <T: fmt::Debug> Printable for T {}

// pub trait IsDbId: Printable {}

// impl IsDbId for DatabaseId {}

// pub trait ContextData {
//     /// If true, don't print to logs when debug is disabled.
//     const SENSITIVE: bool;
// }

// impl <T> ContextData for T {
//     const SENSITIVE: bool = true;
// }

// impl ContextData for DatabaseId {
//     const SENSITIVE: bool = false;
// }

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

// Workaround that it is not possible to implement From<diesel::result::Error>
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

// impl <Ok> IntoDatabaseError<Ok, crate::sqlite::SqliteDatabaseError> for std::result::Result<Ok, ::diesel::result::Error> {
//     const DB_ERROR: DatabaseError = DatabaseError::Diesel;
// }

pub type WriteResult<T, Err, WriteContext = T> =
    std::result::Result<T, WriteError<error_stack::Report<Err>, WriteContext>>;
pub type HistoryWriteResult<T, Err, WriteContext = T> =
    std::result::Result<T, HistoryWriteError<error_stack::Report<Err>, WriteContext>>;

#[derive(Debug)]
pub struct WriteError<Err, Target = ()> {
    pub e: Err,
    pub t: PhantomData<Target>,
}

impl<Target, E: ComponentError> From<error_stack::Report<E>>
    for WriteError<error_stack::Report<E>, Target>
{
    fn from(value: error_stack::Report<E>) -> Self {
        Self {
            t: PhantomData,
            e: value,
        }
    }
}

impl<Target, E: ComponentError> From<E> for WriteError<error_stack::Report<E>, Target> {
    fn from(value: E) -> Self {
        Self {
            t: PhantomData,
            e: value.into(),
        }
    }
}

#[derive(Debug)]
pub struct HistoryWriteError<Err, Target = ()> {
    pub e: Err,
    pub t: PhantomData<Target>,
}

impl<Target, E: ComponentError> From<error_stack::Report<E>>
    for HistoryWriteError<error_stack::Report<E>, Target>
{
    fn from(value: error_stack::Report<E>) -> Self {
        Self {
            t: PhantomData,
            e: value,
        }
    }
}

impl<Target, E: ComponentError> From<E> for HistoryWriteError<error_stack::Report<E>, Target> {
    fn from(value: E) -> Self {
        Self {
            t: PhantomData,
            e: value.into(),
        }
    }
}

pub type ReadResult<T, Err, WriteContext = T> =
    std::result::Result<T, ReadError<error_stack::Report<Err>, WriteContext>>;
pub type HistoryReadResult<T, Err, WriteContext = T> =
    std::result::Result<T, HistoryReadError<error_stack::Report<Err>, WriteContext>>;

#[derive(Debug)]
pub struct ReadError<Err, Target = ()> {
    pub e: Err,
    pub t: PhantomData<Target>,
}

impl<Target, E: ComponentError> From<error_stack::Report<E>>
    for ReadError<error_stack::Report<E>, Target>
{
    fn from(value: error_stack::Report<E>) -> Self {
        Self {
            t: PhantomData,
            e: value,
        }
    }
}

impl<Target, E: ComponentError> From<E> for ReadError<error_stack::Report<E>, Target> {
    fn from(value: E) -> Self {
        Self {
            t: PhantomData,
            e: value.into(),
        }
    }
}

#[derive(Debug)]
pub struct HistoryReadError<Err, Target = ()> {
    pub e: Err,
    pub t: PhantomData<Target>,
}

impl<Target, E: ComponentError> From<error_stack::Report<E>>
    for HistoryReadError<error_stack::Report<E>, Target>
{
    fn from(value: error_stack::Report<E>) -> Self {
        Self {
            t: PhantomData,
            e: value,
        }
    }
}

impl<Target, E: ComponentError> From<E> for HistoryReadError<error_stack::Report<E>, Target> {
    fn from(value: E) -> Self {
        Self {
            t: PhantomData,
            e: value.into(),
        }
    }
}

pub struct NoId;

#[derive(Debug, Clone, Copy)]
pub enum DatabaseId {
    Light(AccountIdLight),
    Internal(AccountIdInternal),
    Content(AccountIdLight, ContentId),
    Empty,
}

impl From<AccountIdLight> for DatabaseId {
    fn from(value: AccountIdLight) -> Self {
        DatabaseId::Light(value)
    }
}

impl From<AccountIdInternal> for DatabaseId {
    fn from(value: AccountIdInternal) -> Self {
        DatabaseId::Internal(value)
    }
}

impl From<(AccountIdLight, ContentId)> for DatabaseId {
    fn from(value: (AccountIdLight, ContentId)) -> Self {
        DatabaseId::Content(value.0, value.1)
    }
}

impl From<NoId> for DatabaseId {
    fn from(_: NoId) -> Self {
        DatabaseId::Empty
    }
}

pub trait ConvertCommandError<D>: Sized {
    type Err: ComponentError;

    #[track_caller]
    fn attach<I: Into<DatabaseId>>(self, id: I) -> Result<D, Self::Err>;
}

impl<D, CmdContext, E: ComponentError> ConvertCommandError<D> for WriteResult<D, E, CmdContext> {
    type Err = E;

    #[track_caller]
    fn attach<I: Into<DatabaseId>>(self, id: I) -> Result<D, E> {
        match self {
            Ok(d) => Ok(d),
            Err(WriteError { e, t }) => Err(e).attach_printable_lazy(|| {
                format!(
                    "{} write command: {:?}, id: {:?}",
                    E::COMPONENT_NAME,
                    t,
                    id.into()
                )
            }),
        }
    }
}

impl<D, CmdContext, E: ComponentError> ConvertCommandError<D>
    for HistoryWriteResult<D, E, CmdContext>
{
    type Err = E;

    #[track_caller]
    fn attach<I: Into<DatabaseId>>(self, id: I) -> Result<D, E> {
        match self {
            Ok(d) => Ok(d),
            Err(HistoryWriteError { e, t }) => Err(e).attach_printable_lazy(|| {
                format!(
                    "{} history write command: {:?}, id: {:?}",
                    E::COMPONENT_NAME,
                    t,
                    id.into()
                )
            }),
        }
    }
}

impl<D, CmdContext, E: ComponentError> ConvertCommandError<D> for ReadResult<D, E, CmdContext> {
    type Err = E;

    #[track_caller]
    fn attach<I: Into<DatabaseId>>(self, id: I) -> Result<D, E> {
        match self {
            Ok(d) => Ok(d),
            Err(ReadError { e, t }) => Err(e).attach_printable_lazy(|| {
                format!(
                    "{} read command: {:?}, id: {:?}",
                    E::COMPONENT_NAME,
                    t,
                    id.into()
                )
            }),
        }
    }
}

impl<D, CmdContext, E: ComponentError> ConvertCommandError<D>
    for HistoryReadResult<D, E, CmdContext>
{
    type Err = E;

    #[track_caller]
    fn attach<I: Into<DatabaseId>>(self, id: I) -> Result<D, E> {
        match self {
            Ok(d) => Ok(d),
            Err(HistoryReadError { e, t }) => Err(e).attach_printable_lazy(|| {
                format!(
                    "{} history read command: {:?}, id: {:?}",
                    E::COMPONENT_NAME,
                    t,
                    id.into()
                )
            }),
        }
    }
}
