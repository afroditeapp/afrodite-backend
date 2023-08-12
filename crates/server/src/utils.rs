use error_stack::{Context, Report, Result, ResultExt};
use tokio::sync::oneshot;

use database::{
    ConvertCommandError, DatabaseId, HistoryReadError, HistoryReadResult,
    HistoryWriteError, HistoryWriteResult, ReadError, ReadResult, sqlite::SqliteDatabaseError, WriteError,
    WriteResult,
};
use utils::{ComponentError, ErrorResultExt};

use crate::data::{cache::CacheError, DatabaseError, file::FileError};

/// Sender only used for quit request message sending.
pub type QuitSender = oneshot::Sender<()>;

/// Receiver only used for quit request message receiving.
pub type QuitReceiver = oneshot::Receiver<()>;

pub trait ErrorConversion: ResultExt + Sized {
    type Err: Context;
    const ERROR: Self::Err;

    /// Change error context and add additional info about error.
    #[track_caller]
    fn with_info<I: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static>(
        self,
        info: I,
    ) -> Result<<Self as ResultExt>::Ok, Self::Err> {
        self.change_context_with_info(Self::ERROR, info)
    }

    /// Change error context and add additional info about error. Sets
    /// additional info about error lazily.
    #[track_caller]
    fn with_info_lazy<
        F: FnOnce() -> I,
        I: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
    >(
        self,
        info: F,
    ) -> Result<<Self as ResultExt>::Ok, Self::Err> {
        self.change_context_with_info_lazy(Self::ERROR, info)
    }
}

impl<T> ErrorConversion for Result<T, SqliteDatabaseError> {
    type Err = DatabaseError;
    const ERROR: <Self as ErrorConversion>::Err = DatabaseError::Sqlite;
}

impl<T> ErrorConversion for Result<T, CacheError> {
    type Err = DatabaseError;
    const ERROR: <Self as ErrorConversion>::Err = DatabaseError::Cache;
}

impl<T> ErrorConversion for Result<T, FileError> {
    type Err = DatabaseError;
    const ERROR: <Self as ErrorConversion>::Err = DatabaseError::File;
}

pub(crate) trait ConvertCommandErrorExt<D>: ConvertCommandError<D> {
    /// Use DatabaseId::Empty if there is no real ID.
    #[track_caller]
    fn convert<I: Into<DatabaseId>>(self, id: I) -> Result<D, DatabaseError>;
}

impl<D, CmdContext, E: ComponentError> ConvertCommandErrorExt<D> for WriteResult<D, E, CmdContext>
where
    Result<D, E>: ErrorConversion<Err = DatabaseError> + ResultExt<Ok = D>,
{
    #[track_caller]
    fn convert<I: Into<DatabaseId>>(self, id: I) -> Result<D, DatabaseError> {
        match self {
            Ok(d) => Ok(d),
            Err(WriteError { e, t }) => Err(e).with_info_lazy(|| {
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

// impl<D, CmdContext> ConvertCommandErrorExt<D> for WriteResult<D, CacheError, CmdContext> {
//     type Err = CacheError;

//     #[track_caller]
//     fn convert<I: Into<DatabaseId>>(self, id: I) -> Result<D, DatabaseError> {
//         match self {
//             Ok(d) => Ok(d),
//             Err(WriteError { e, t }) => Err(e)
//                 .with_info_lazy(|| format!("Cache write command: {:?}, id: {:?}", t, id.into())),
//         }
//     }

//     #[track_caller]
//     fn attach<I: Into<DatabaseId>>(self, id: I) -> Result<D, CacheError> {
//         match self {
//             Ok(d) => Ok(d),
//             Err(WriteError { e, t }) => Err(e).attach_printable_lazy(|| {
//                 format!("Cache write command: {:?}, id: {:?}", t, id.into())
//             }),
//         }
//     }
// }

impl<D, CmdContext, E: ComponentError> ConvertCommandErrorExt<D>
    for HistoryWriteResult<D, E, CmdContext>
where
    Result<D, E>: ErrorConversion<Err = DatabaseError> + ResultExt<Ok = D>,
{
    #[track_caller]
    fn convert<I: Into<DatabaseId>>(self, id: I) -> Result<D, DatabaseError> {
        match self {
            Ok(d) => Ok(d),
            Err(HistoryWriteError { e, t }) => Err(e).with_info_lazy(|| {
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

impl<D, CmdContext, E: ComponentError> ConvertCommandErrorExt<D> for ReadResult<D, E, CmdContext>
where
    Result<D, E>: ErrorConversion<Err = DatabaseError> + ResultExt<Ok = D>,
{
    #[track_caller]
    fn convert<I: Into<DatabaseId>>(self, id: I) -> Result<D, DatabaseError> {
        match self {
            Ok(d) => Ok(d),
            Err(ReadError { e, t }) => Err(e).with_info_lazy(|| {
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

// impl<D, CmdContext> ConvertCommandErrorExt<D> for ReadResult<D, CacheError, CmdContext> {
//     type Err = CacheError;

//     #[track_caller]
//     fn attach<I: Into<DatabaseId>>(self, id: I) -> Result<D, CacheError> {
//         match self {
//             Ok(d) => Ok(d),
//             Err(ReadError { e, t }) => Err(e).attach_printable_lazy(|| {
//                 format!("Cache read command: {:?}, id: {:?}", t, id.into())
//             }),
//         }
//     }

//     #[track_caller]
//     fn convert<I: Into<DatabaseId>>(self, id: I) -> Result<D, DatabaseError> {
//         match self {
//             Ok(d) => Ok(d),
//             Err(ReadError { e, t }) => Err(e)
//                 .with_info_lazy(|| format!("Cache read command: {:?}, id: {:?}", t, id.into())),
//         }
//     }
// }

// impl<D, CmdContext> ConvertCommandErrorExt<D> for ReadResult<D, FileError, CmdContext> {
//     type Err = FileError;

//     #[track_caller]
//     fn attach<I: Into<DatabaseId>>(self, id: I) -> Result<D, FileError> {
//         match self {
//             Ok(d) => Ok(d),
//             Err(ReadError { e, t }) => Err(e).attach_printable_lazy(|| {
//                 format!("File read command: {:?}, id: {:?}", t, id.into())
//             }),
//         }
//     }

//     #[track_caller]
//     fn convert<I: Into<DatabaseId>>(self, id: I) -> Result<D, DatabaseError> {
//         match self {
//             Ok(d) => Ok(d),
//             Err(ReadError { e, t }) => {
//                 Err(e).with_info_lazy(|| format!("File read command: {:?}, id: {:?}", t, id.into()))
//             }
//         }
//     }
// }

impl<D, CmdContext, E: ComponentError> ConvertCommandErrorExt<D>
    for HistoryReadResult<D, E, CmdContext>
where
    Result<D, E>: ErrorConversion<Err = DatabaseError> + ResultExt<Ok = D>,
{
    #[track_caller]
    fn convert<I: Into<DatabaseId>>(self, id: I) -> Result<D, DatabaseError> {
        match self {
            Ok(d) => Ok(d),
            Err(HistoryReadError { e, t }) => Err(e).with_info_lazy(|| {
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

pub type ErrorContainer<E> = Option<Report<E>>;

pub trait AppendErr: Sized {
    type E: Context;

    fn append(&mut self, e: Report<Self::E>);
    fn into_result(self) -> Result<(), Self::E>;
}

impl AppendErr for ErrorContainer<DatabaseError> {
    type E = DatabaseError;

    fn append(&mut self, e: Report<Self::E>) {
        if let Some(error) = self.as_mut() {
            error.extend_one(e);
        } else {
            *self = Some(e);
        }
    }

    fn into_result(self) -> Result<(), Self::E> {
        match self {
            None => Ok(()),
            Some(e) => Err(e),
        }
    }
}

pub trait AppendErrorTo<Err>: Sized {
    fn append_to_and_ignore(self, container: &mut ErrorContainer<Err>);
    fn append_to_and_return_container(self, container: &mut ErrorContainer<Err>)
        -> Result<(), Err>;
}

impl<Ok, Err: Context> AppendErrorTo<Err> for Result<Ok, Err>
where
    ErrorContainer<Err>: AppendErr<E = Err>,
{
    fn append_to_and_ignore(self, container: &mut ErrorContainer<Err>) {
        if let Err(e) = self {
            container.append(e)
        }
    }

    fn append_to_and_return_container(
        self,
        container: &mut ErrorContainer<Err>,
    ) -> Result<(), Err> {
        if let Err(e) = self {
            container.append(e);
            container.take().into_result()
        } else {
            Ok(())
        }
    }
}
