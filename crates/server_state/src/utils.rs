use axum::response::{IntoResponse, Response};
use config::file::ConfigFileError;
use server_common::{data::cache::CacheError, internal_api::InternalApiError};
use server_data::{content_processing::ContentProcessingError, event::EventError};
use simple_backend::{
    manager_client::ManagerClientError,
    sign_in_with::{apple::SignInWithAppleError, google::SignInWithGoogleError},
};

use crate::DataError;

#[allow(non_camel_case_types)]
pub enum StatusCode {
    /// 400
    BAD_REQUEST,
    /// 401
    UNAUTHORIZED,
    /// 500
    INTERNAL_SERVER_ERROR,
    /// 406
    NOT_ACCEPTABLE,
    /// 404
    NOT_FOUND,
    /// 304
    NOT_MODIFIED,
}

impl From<StatusCode> for hyper::StatusCode {
    fn from(value: StatusCode) -> Self {
        match value {
            StatusCode::BAD_REQUEST => hyper::StatusCode::BAD_REQUEST,
            StatusCode::UNAUTHORIZED => hyper::StatusCode::UNAUTHORIZED,
            StatusCode::INTERNAL_SERVER_ERROR => hyper::StatusCode::INTERNAL_SERVER_ERROR,
            StatusCode::NOT_ACCEPTABLE => hyper::StatusCode::NOT_ACCEPTABLE,
            StatusCode::NOT_FOUND => hyper::StatusCode::NOT_FOUND,
            StatusCode::NOT_MODIFIED => hyper::StatusCode::NOT_MODIFIED,
        }
    }
}

impl IntoResponse for StatusCode {
    fn into_response(self) -> Response {
        let status: hyper::StatusCode = self.into();
        status.into_response()
    }
}

#[derive(thiserror::Error, Debug)]
enum RequestError {
    #[error("Data reading or writing failed")]
    Data,
    #[error("Cache reading or writing failed")]
    Cache,
    #[error("Sign in with Google error")]
    SignInWithGoogle,
    #[error("Sign in with Apple error")]
    SignInWithApple,
    #[error("Internal API error")]
    InternalApiError,
    #[error("Manager client error")]
    ManagerClientError,
    #[error("Config file error")]
    ConfigFileError,
    #[error("Event error")]
    EventError,
    #[error("Content processing error")]
    ContentProcessingError,
}

/// Convert error to status code. This is workaround for track_caller seems
/// to not work when converting using Into::into. Early return with ? seems
/// to have the correct caller location. This fixes error location printed
/// from db_write macro.
///
pub trait ConvertDataErrorToStatusCode<Ok>: Sized {
    #[track_caller]
    fn convert_data_error_to_status_code(self)
        -> std::result::Result<Ok, crate::utils::StatusCode>;

    #[track_caller]
    fn ignore_and_log_error(self) {
        let _ = self.convert_data_error_to_status_code();
    }
}

macro_rules! impl_error_to_status_code {
    ($err_type:ty, $err_expr:expr) => {
        impl From<$crate::result::WrappedReport<error_stack::Report<$err_type>>> for StatusCode {
            #[track_caller]
            fn from(value: $crate::result::WrappedReport<error_stack::Report<$err_type>>) -> Self {
                tracing::error!("{:?}", value.change_context($err_expr));
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }

        impl From<error_stack::Report<$err_type>> for StatusCode {
            #[track_caller]
            fn from(value: error_stack::Report<$err_type>) -> Self {
                tracing::error!("{:?}", value.change_context($err_expr));
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }

        impl<Ok> ConvertDataErrorToStatusCode<Ok>
            for Result<Ok, $crate::result::WrappedReport<error_stack::Report<$err_type>>>
        {
            #[track_caller]
            fn convert_data_error_to_status_code(
                self,
            ) -> std::result::Result<Ok, crate::utils::StatusCode> {
                use $crate::result::WrappedResultExt;
                let result = self.change_context($err_expr);
                match result {
                    Ok(ok) => Ok(ok),
                    Err(err) => {
                        tracing::error!("{:?}", err);
                        Err(StatusCode::INTERNAL_SERVER_ERROR)
                    }
                }
            }
        }
    };
}

impl_error_to_status_code!(DataError, RequestError::Data);
impl_error_to_status_code!(CacheError, RequestError::Cache);
impl_error_to_status_code!(SignInWithGoogleError, RequestError::SignInWithGoogle);
impl_error_to_status_code!(SignInWithAppleError, RequestError::SignInWithApple);
impl_error_to_status_code!(InternalApiError, RequestError::InternalApiError);
impl_error_to_status_code!(ManagerClientError, RequestError::ManagerClientError);
impl_error_to_status_code!(ConfigFileError, RequestError::ConfigFileError);
impl_error_to_status_code!(EventError, RequestError::EventError);
impl_error_to_status_code!(ContentProcessingError, RequestError::ContentProcessingError);
