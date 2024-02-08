use std::ops::ShlAssign;

use simple_backend_database::{diesel_db::DieselDatabaseError};

use error_stack::{Context, Report, ResultExt};

use crate::data::{cache::CacheError, file::FileError, index::IndexError, DataError};

pub type Result<Ok, Err> = std::result::Result<Ok, WrappedReport<Report<Err>>>;

/// A wrapper around `error_stack::Report` that allows automatic
/// type conversions.
pub struct WrappedReport<E> {
    report: E,
}

impl <E> WrappedReport<Report<E>> {
    #[track_caller]
    pub fn change_context<C: error_stack::Context>(self, context: C) -> WrappedReport<Report<C>> {
        WrappedReport {
            report: self.report.change_context(context)
        }
    }

    pub fn attach_printable<
        A: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
    >(self, attachment: A) -> WrappedReport<Report<E>> {
        WrappedReport {
            report: self.report.attach_printable(attachment)
        }
    }
}

impl <E> std::fmt::Debug for WrappedReport<Report<E>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.report)
    }
}

impl <E> std::fmt::Display for WrappedReport<Report<E>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.report)
    }
}

impl From<Report<DieselDatabaseError>> for WrappedReport<Report<DataError>> {
    #[track_caller]
    fn from(error: Report<DieselDatabaseError>) -> Self {
        Self {
            report: error.change_context(DataError::Diesel)
        }
    }
}

impl From<Report<FileError>> for WrappedReport<Report<DataError>> {
    #[track_caller]
    fn from(error: Report<FileError>) -> Self {
        Self {
            report: error.change_context(DataError::File)
        }
    }
}

impl From<Report<CacheError>> for WrappedReport<Report<DataError>> {
    #[track_caller]
    fn from(error: Report<CacheError>) -> Self {
        Self {
            report: error.change_context(DataError::Cache)
        }
    }
}

impl From<Report<IndexError>> for WrappedReport<Report<DataError>> {
    #[track_caller]
    fn from(error: Report<IndexError>) -> Self {
        Self {
            report: error.change_context(DataError::ProfileIndex)
        }
    }
}

impl From<Report<simple_backend_database::DataError>> for WrappedReport<Report<DataError>> {
    #[track_caller]
    fn from(error: Report<simple_backend_database::DataError>) -> Self {
        Self {
            report: error.change_context(DataError::Diesel)
        }
    }
}

impl From<Report<simple_backend_database::sqlx_db::SqliteDatabaseError>> for WrappedReport<Report<DataError>> {
    #[track_caller]
    fn from(error: Report<simple_backend_database::sqlx_db::SqliteDatabaseError>) -> Self {
        Self {
            report: error.change_context(DataError::Sqlite)
        }
    }
}

impl From<std::io::Error> for WrappedReport<Report<DataError>> {
    #[track_caller]
    fn from(error: std::io::Error) -> Self {
        Self {
            report: Report::from(error).change_context(DataError::Io)
        }
    }
}


/// Create wrapped Reports
pub trait WrappedContextExt: error_stack::Context + Sized {
    #[track_caller]
    fn report(self) -> WrappedReport<Report<Self>> {
        WrappedReport {
            report: error_stack::report!(self),
        }
    }
}

impl <E: Context> WrappedContextExt for E {}

pub trait WrappedResultExt<Ok>: Sized {
    #[track_caller]
    fn change_context<C: Context>(self, context: C) -> std::result::Result<Ok, WrappedReport<Report<C>>>;
}

impl <Ok, Err: Context> WrappedResultExt<Ok> for std::result::Result<Ok, Report<Err>> {
    #[track_caller]
    fn change_context<C: Context>(self, context: C) -> std::result::Result<Ok, WrappedReport<Report<C>>> {
        self.map_err(|e| {
            WrappedReport {
                report: e.change_context(context),
            }
        })
    }
}

impl <Ok, Err: Context> WrappedResultExt<Ok> for std::result::Result<Ok, WrappedReport<Report<Err>>> {
    #[track_caller]
    fn change_context<C: Context>(self, context: C) -> std::result::Result<Ok, WrappedReport<Report<C>>> {
        match self {
            Ok(ok) => Ok(ok),
            Err(err) => Err(err.change_context(context)),
        }
    }
}

/// WrappedResultExt2 is same as WrappedResultExt but different trait is needed
/// as it was not possible to implement WrappedResultExt for both
/// std::result::Result<Ok, Report<Err>> and std::result::Result<Ok, Err>.
pub trait WrappedResultExt2<Ok>: Sized {
    #[track_caller]
    fn change_context<C: Context>(self, context: C) -> std::result::Result<Ok, WrappedReport<Report<C>>>;
}

impl <Ok, Err: Context> WrappedResultExt2<Ok> for std::result::Result<Ok, Err> {
    #[track_caller]
    fn change_context<C: Context>(self, context: C) -> std::result::Result<Ok, WrappedReport<Report<C>>> {
        match self {
            Ok(ok) => Ok(ok),
            Err(err) => Err(
                WrappedReport {
                    report: Report::from(err).change_context(context),
                }
            ),
        }
    }
}
