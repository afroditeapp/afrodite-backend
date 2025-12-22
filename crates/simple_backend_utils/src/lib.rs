#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

use std::fmt::Display;

use error_stack::{Context, Report, Result, ResultExt};

pub mod db;
pub mod dir;
pub mod file;
pub mod macros;
pub mod string;
pub mod time;
mod uuid;

pub use uuid::{UuidBase64Url, UuidBase64UrlToml};

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
            Err(err) => Err(context.report()).attach_printable(err.to_string()),
        }
    }
}

pub trait ContextExt: Context + Sized {
    #[track_caller]
    fn report(self) -> Report<Self> {
        error_stack::report!(self)
    }
}

impl<E: Context + Sized> ContextExt for E {}

pub fn current_unix_time() -> i64 {
    chrono::Utc::now().timestamp()
}

pub trait ComponentError: Context {
    const COMPONENT_NAME: &'static str;
}
