#![deny(unsafe_code)]
#![warn(unused_crate_dependencies)]

use std::fmt::Display;

use error_stack::{Context, IntoReport, Result, ResultExt};

pub trait IntoReportFromString {
    type Ok;
    type Err: Display;

    #[track_caller]
    fn into_error_string<C: Context>(self, context: C) -> Result<Self::Ok, C>;
}

impl<Ok, Err: Display> IntoReportFromString for std::result::Result<Ok, Err> {
    type Ok = Ok;
    type Err = Err;

    fn into_error_string<C: Context>(
        self,
        context: C,
    ) -> Result<<Self as IntoReportFromString>::Ok, C> {
        match self {
            Ok(ok) => Ok(ok),
            Err(err) => Err(context).into_report().attach_printable(err.to_string()),
        }
    }
}

pub trait IntoReportExt: IntoReport {
    #[track_caller]
    fn into_error<C: Context>(self, context: C) -> Result<<Self as IntoReport>::Ok, C> {
        self.into_report().change_context(context)
    }

    #[track_caller]
    fn into_error_with_info<
        C: Context,
        I: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
    >(
        self,
        context: C,
        info: I,
    ) -> Result<<Self as IntoReport>::Ok, C> {
        self.into_report()
            .change_context(context)
            .attach_printable(info)
    }

    #[track_caller]
    fn into_error_with_info_lazy<
        C: Context,
        F: FnOnce() -> I,
        I: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
    >(
        self,
        context: C,
        info: F,
    ) -> Result<<Self as IntoReport>::Ok, C> {
        self.into_report()
            .change_context(context)
            .attach_printable_lazy(info)
    }
}

impl<T: IntoReport> IntoReportExt for T {}

pub trait ErrorResultExt: ResultExt + Sized {
    #[track_caller]
    fn change_context_with_info<
        C: Context,
        I: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
    >(
        self,
        context: C,
        info: I,
    ) -> Result<<Self as ResultExt>::Ok, C> {
        self.change_context(context).attach_printable(info)
    }

    #[track_caller]
    fn change_context_with_info_lazy<
        C: Context,
        F: FnOnce() -> I,
        I: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
    >(
        self,
        context: C,
        info: F,
    ) -> Result<<Self as ResultExt>::Ok, C> {
        self.change_context(context).attach_printable_lazy(info)
    }
}

impl<T: ResultExt + Sized> ErrorResultExt for T {}

pub fn current_unix_time() -> i64 {
    time::OffsetDateTime::now_utc().unix_timestamp()
}

pub trait ComponentError: Context {
    const COMPONENT_NAME: &'static str;
}
