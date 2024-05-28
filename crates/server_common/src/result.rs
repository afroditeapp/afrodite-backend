use std::fmt::Debug;

use database::ErrorContext;
use error_stack::{Context, Report};
use model::IsLoggingAllowed;
use simple_backend_database::diesel_db::DieselDatabaseError;

use crate::{
    data::{cache::CacheError, file::FileError, index::IndexError, DataError},
    internal_api::InternalApiError,
};

pub type Result<Ok, Err> = std::result::Result<Ok, WrappedReport<Report<Err>>>;

/// A wrapper around `error_stack::Report` that allows automatic
/// type conversions.
pub struct WrappedReport<E> {
    report: E,
}

impl<E> WrappedReport<Report<E>> {
    pub fn new(report: Report<E>) -> Self {
        Self { report }
    }

    #[track_caller]
    pub fn change_context<C: error_stack::Context>(self, context: C) -> WrappedReport<Report<C>> {
        WrappedReport {
            report: self.report.change_context(context),
        }
    }

    pub fn attach_printable<A: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static>(
        self,
        attachment: A,
    ) -> WrappedReport<Report<E>> {
        WrappedReport {
            report: self.report.attach_printable(attachment),
        }
    }

    pub fn into_report(self) -> Report<E> {
        self.report
    }
}

impl<E> std::fmt::Debug for WrappedReport<Report<E>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.report)
    }
}

impl<E> std::fmt::Display for WrappedReport<Report<E>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.report)
    }
}

impl<C: Context> From<Report<C>> for WrappedReport<Report<C>> {
    #[track_caller]
    fn from(error: Report<C>) -> Self {
        Self { report: error }
    }
}

impl From<Report<DieselDatabaseError>> for WrappedReport<Report<DataError>> {
    #[track_caller]
    fn from(error: Report<DieselDatabaseError>) -> Self {
        Self {
            report: error.change_context(DataError::Diesel),
        }
    }
}

impl From<Report<FileError>> for WrappedReport<Report<DataError>> {
    #[track_caller]
    fn from(error: Report<FileError>) -> Self {
        Self {
            report: error.change_context(DataError::File),
        }
    }
}

impl From<Report<CacheError>> for WrappedReport<Report<DataError>> {
    #[track_caller]
    fn from(error: Report<CacheError>) -> Self {
        Self {
            report: error.change_context(DataError::Cache),
        }
    }
}

impl From<Report<IndexError>> for WrappedReport<Report<DataError>> {
    #[track_caller]
    fn from(error: Report<IndexError>) -> Self {
        Self {
            report: error.change_context(DataError::ProfileIndex),
        }
    }
}

impl From<Report<simple_backend_database::SimpleDatabaseError>>
    for WrappedReport<Report<DataError>>
{
    #[track_caller]
    fn from(error: Report<simple_backend_database::SimpleDatabaseError>) -> Self {
        Self {
            report: error.change_context(DataError::Diesel),
        }
    }
}

impl From<std::io::Error> for WrappedReport<Report<DataError>> {
    #[track_caller]
    fn from(error: std::io::Error) -> Self {
        Self {
            report: Report::from(error).change_context(DataError::Io),
        }
    }
}

impl From<InternalApiError> for WrappedReport<Report<InternalApiError>> {
    #[track_caller]
    fn from(error: InternalApiError) -> Self {
        Self {
            report: Report::from(error),
        }
    }
}

/// Convert errors to WrappedReports or Reports.
pub trait WrappedContextExt<ReportAndError>: Context + Sized {
    #[track_caller]
    fn report(self) -> ReportAndError;
}

impl<E: Context> WrappedContextExt<WrappedReport<Report<E>>> for E {
    #[track_caller]
    fn report(self) -> WrappedReport<Report<E>> {
        WrappedReport {
            report: error_stack::report!(self),
        }
    }
}

pub trait WrappedResultExt<
    Ok,
    InContext: Context,
    OutContext: Context,
    InReportAndError,
    OutReportAndError,
>: Sized
{
    #[track_caller]
    fn change_context(self, context: OutContext) -> std::result::Result<Ok, OutReportAndError>;

    #[track_caller]
    fn change_context_with_info<T: Debug + IsLoggingAllowed>(
        self,
        context: OutContext,
        info: T,
    ) -> std::result::Result<Ok, OutReportAndError>;
}

impl<Ok, InContext: Context, OutContext: Context>
    WrappedResultExt<
        Ok,
        InContext,
        OutContext,
        WrappedReport<Report<InContext>>,
        WrappedReport<Report<OutContext>>,
    > for std::result::Result<Ok, WrappedReport<Report<InContext>>>
{
    #[track_caller]
    fn change_context(
        self,
        context: OutContext,
    ) -> std::result::Result<Ok, WrappedReport<Report<OutContext>>> {
        match self {
            Ok(ok) => Ok(ok),
            Err(err) => Err(WrappedReport {
                report: err.report.change_context(context),
            }),
        }
    }

    #[track_caller]
    fn change_context_with_info<T: Debug + IsLoggingAllowed>(
        self,
        context: OutContext,
        info: T,
    ) -> std::result::Result<Ok, WrappedReport<Report<OutContext>>> {
        match self.change_context(context) {
            Ok(ok) => Ok(ok),
            Err(err) => Err(err.attach_printable(ErrorContext::<T, Ok>::new(info).printable())),
        }
    }
}

impl<Ok, InContext: Context, OutContext: Context>
    WrappedResultExt<
        Ok,
        InContext,
        OutContext,
        Report<InContext>,
        WrappedReport<Report<OutContext>>,
    > for std::result::Result<Ok, Report<InContext>>
{
    #[track_caller]
    fn change_context(
        self,
        context: OutContext,
    ) -> std::result::Result<Ok, WrappedReport<Report<OutContext>>> {
        match self {
            Ok(ok) => Ok(ok),
            Err(err) => Err(WrappedReport {
                report: err.change_context(context),
            }),
        }
    }

    #[track_caller]
    fn change_context_with_info<T: Debug + IsLoggingAllowed>(
        self,
        context: OutContext,
        info: T,
    ) -> std::result::Result<Ok, WrappedReport<Report<OutContext>>> {
        match self.change_context(context) {
            Ok(ok) => Ok(ok),
            Err(err) => Err(err.attach_printable(ErrorContext::<T, Ok>::new(info).printable())),
        }
    }
}

impl<Ok, InContext: Context, OutContext: Context>
    WrappedResultExt<Ok, InContext, OutContext, InContext, WrappedReport<Report<OutContext>>>
    for std::result::Result<Ok, InContext>
{
    #[track_caller]
    fn change_context(
        self,
        context: OutContext,
    ) -> std::result::Result<Ok, WrappedReport<Report<OutContext>>> {
        match self {
            Ok(ok) => Ok(ok),
            Err(err) => Err(WrappedReport {
                report: Report::from(err).change_context(context),
            }),
        }
    }

    #[track_caller]
    fn change_context_with_info<T: Debug + IsLoggingAllowed>(
        self,
        context: OutContext,
        info: T,
    ) -> std::result::Result<Ok, WrappedReport<Report<OutContext>>> {
        match self.change_context(context) {
            Ok(ok) => Ok(ok),
            Err(err) => Err(err.attach_printable(ErrorContext::<T, Ok>::new(info).printable())),
        }
    }
}
