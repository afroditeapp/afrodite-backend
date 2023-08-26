#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

pub mod api;

use std::{fmt::Display, io};

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
}

impl<T: IntoReport> IntoReportExt for T {}

pub fn current_unix_time() -> i64 {
    time::OffsetDateTime::now_utc().unix_timestamp()
}

pub trait ComponentError: Context {
    const COMPONENT_NAME: &'static str;
}
