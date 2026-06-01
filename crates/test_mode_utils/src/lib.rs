#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

pub use model::AccountVerificationErrorFlags;

use crate::client::TestError;

pub mod client;
pub mod dir;
pub mod server;
pub mod state;
pub mod websocket_protocol;

pub struct ServerConfigEditor<'a> {
    pub server: &'a mut config::file::ConfigFile,
    pub simple_backend: &'a mut simple_backend_config::file::SimpleBackendConfigFile,
}

/// Workaround for api_client error type conversion to
/// avoid change_context calls.
pub struct ServerTestError {
    pub error: error_stack::Report<TestError>,
}

impl ServerTestError {
    pub fn new(error: error_stack::Report<crate::client::TestError>) -> Self {
        Self { error }
    }
}

impl From<error_stack::Report<crate::client::TestError>> for ServerTestError {
    #[track_caller]
    fn from(error: error_stack::Report<crate::client::TestError>) -> Self {
        Self {
            error: error.change_context(TestError::ServerTestFailed),
        }
    }
}

impl<T> From<api_client::apis::Error<T>> for ServerTestError
where
    api_client::apis::Error<T>: error_stack::Context,
{
    #[track_caller]
    fn from(error: api_client::apis::Error<T>) -> Self {
        Self {
            error: error_stack::Report::from(error).change_context(TestError::ServerTestFailed),
        }
    }
}
