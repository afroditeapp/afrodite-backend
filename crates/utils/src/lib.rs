#![deny(unsafe_code)]
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

fn test() -> Result<(), io::Error> {
    let x: io::Error = io::ErrorKind::NotFound.into();
    Err(x).to_next_err(io::ErrorKind::AddrInUse.into())
}

fn test2() -> std::result::Result<(), ErrorWrapper<io::Error>> {
    let x: io::Error = io::ErrorKind::NotFound.into();
    Err(x).to_next_err(io::ErrorKind::AddrInUse.into())
}

pub struct ErrorWrapper<T: Context>(pub error_stack::Report<T>);

// impl <T: Context> From<error_stack::Report<T>> for ErrorWrapper<T> {
//     fn from(value: error_stack::Report<T>) -> Self {
//         Self(value)
//     }
// }

impl<E: Context, Ok> NextErrWrapped<Ok, E, ErrorWrapper<E>> for std::result::Result<Ok, E> {
    fn to_next_err(self, e: E) -> std::result::Result<Ok, ErrorWrapper<E>> {
        self.into_report()
            .change_context(e)
            .map_err(|e| ErrorWrapper(e))
    }
}

impl<E: Context, Ok> NextErrWrapped<Ok, E, error_stack::Report<E>> for std::result::Result<Ok, E> {
    fn to_next_err(self, _e: E) -> std::result::Result<Ok, error_stack::Report<E>> {
        self.into_report()
    }
}

// // ErrConteiner thing does not work as there is two impls

// pub trait NextErr<Ok, NewE> {
//     type ErrContainer;

//     fn to_next_err(self, e: NewE) -> std::result::Result<Ok, Self::ErrContainer>;
// }

// pub trait NextErrWrapped<Ok, NewE> {
//     type ErrContainer;

//     fn to_next_err(self, e: NewE) -> std::result::Result<Ok, Self::ErrContainer>;
// }

// impl <Ok, NewE: Context> NextErrWrapped<Ok, NewE> for std::result::Result<Ok, NewE> {
//     type ErrContainer = ErrorWrapper<NewE>;

//     fn to_next_err(self, e: NewE) -> std::result::Result<Ok, Self::ErrContainer> {
//         self.into_report().map_err(|e| ErrorWrapper(e))
//     }
// }

// impl <Ok, NewE: Context> NextErr<Ok, NewE> for std::result::Result<Ok, NewE> {
//     type ErrContainer = error_stack::Report<NewE>;

//     fn to_next_err(self, e: NewE) -> std::result::Result<Ok, Self::ErrContainer> {
//         self.into_report()
//     }
// }

pub trait ErrPrev {
    type PrevErr;
    type Err: Context;
}

impl<C: Context> ErrPrev for ErrorWrapper<C> {
    type PrevErr = error_stack::Report<C>;
    type Err = C;
}

// pub trait ErrContainer<NextErr>: Into<NextErr> {
//     type CurrentErr: Context;

// }

// impl<Ok, Err: From<>> ErrContainer<Err> for Result<Ok, Err> {}

pub trait NextErrWrapped<Ok, Err: Context, Out>: IntoReport {
    fn to_next_err(self, e: Err) -> std::result::Result<Ok, Out>;
}

pub trait IntoReportExt: IntoReport {
    #[track_caller]
    fn into_error<C: Context>(self, context: C) -> Result<<Self as IntoReport>::Ok, C> {
        self.into_report().change_context(context)
    }

    // #[track_caller]
    // fn into_error2<
    //     C: Context,
    //     Out,
    // >(self, context: C) -> std::result::Result<<Self as IntoReport>::Ok, Out> where
    //     Self: NextErr<Out, Error = C> {
    //     let i: std::result::Result<<Self as IntoReport>::Ok, error_stack::Report<C>> = self.into_report().change_context(context);

    // }

    // TODO: This is not working, because the compiler can't infer the type of E
    // #[track_caller]
    // fn into_error2<
    //     C: Context,
    //     I: ErrContainer<Err = C>,
    //     E: ErrContainer<Err = C>,
    // >(self, context: C) -> std::result::Result<<Self as IntoReport>::Ok, E> where
    //     std::result::Result<<Self as IntoReport>::Ok, error_stack::Report<C>>: std::convert::Into<std::result::Result<<Self as IntoReport>::Ok, I>>,
    //     std::result::Result<<Self as IntoReport>::Ok, I>: std::convert::Into<std::result::Result<<Self as IntoReport>::Ok, E>>  {
    //     let i: std::result::Result<<Self as IntoReport>::Ok, error_stack::Report<C>> = self.into_report().change_context(context);
    //     let convert_to_e = i.into();
    //     convert_to_e.into()
    // }

    // #[track_caller]
    // fn into_error<
    //     C: Context,
    //     E: ErrContainer<Err = C> + From<error_stack::Report<C>>,
    // >(self, context: C) -> std::result::Result<<Self as IntoReport>::Ok, E> where
    //     std::result::Result<<Self as IntoReport>::Ok, error_stack::Report<C>>: std::convert::Into<std::result::Result<<Self as IntoReport>::Ok, E>> {
    //     let i: std::result::Result<<Self as IntoReport>::Ok, error_stack::Report<C>> = self.into_report().change_context(context);
    //     i.map_err(|e| e.into())
    // }

    #[track_caller]
    fn into_wrapped_error<C: Context, E: From<error_stack::Report<C>>>(
        self,
        context: C,
    ) -> std::result::Result<<Self as IntoReport>::Ok, E> {
        let r: std::result::Result<<Self as IntoReport>::Ok, error_stack::Report<C>> =
            self.into_report().change_context(context);
        r.map_err(|e| e.into())
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
