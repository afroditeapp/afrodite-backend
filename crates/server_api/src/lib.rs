#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

//! HTTP API types and request handlers for all servers.

use utoipa::OpenApi;

use self::utils::SecurityApiAccessTokenDefault;

// Routes
pub mod common;
pub mod common_admin;
pub mod common_internal;

pub mod app;
pub mod internal_api;
pub mod utils;

pub use server_common::{data::DataError, result};

// API docs

#[derive(OpenApi)]
#[openapi(
    paths(
        // Common
        common::get_version,
        common::get_connect_websocket,
    ),
    components(schemas(
        // Common
        model::common::EventToClient,
        // Manager
        manager_model::SoftwareOptions,
    )),
    modifiers(&SecurityApiAccessTokenDefault),
)]
pub struct ApiDocCommon;

/// Macro for writing data with different code style.
/// Makes "async move" and "await" keywords unnecessary.
/// The macro "closure" should work like a real closure.
///
/// This macro will guarantee that contents of the closure will run
/// completely even if HTTP connection fails when closure is running.
///
/// Converts crate::data::DataError to crate::api::utils::StatusCode.
///
/// Example usage:
///
/// ```
/// use server_api::db_write;
/// use server_api::utils::StatusCode;
/// use server_api::app::WriteData;
/// pub async fn axum_route_handler<S: WriteData>(
///     state: S,
/// ) -> std::result::Result<(), StatusCode> {
///     db_write!(state, move |cmds|
///         async move { Ok(()) }
///     )
/// }
/// ```
#[macro_export]
macro_rules! db_write {
    ($state:expr, move |$cmds:ident| $commands:expr) => {{
        let r = async {
            let r: $crate::result::Result<_, server_data::DataError> = $state
                .write(move |$cmds| async move {
                    ($commands).await
                })
                .await;
            r
        }
        .await;

        use $crate::utils::ConvertDataErrorToStatusCode;
        r.convert_data_error_to_status_code()
    }};
}

/// Same as db_write! but allows multiple commands to be executed because the
/// commands are not automatically awaited.
#[macro_export]
macro_rules! db_write_multiple {
    ($state:expr, move |$cmds:ident| $commands:expr) => {{
        let r = async {
            let r: $crate::result::Result<_, $crate::DataError> = $state
                .write(move |$cmds| async move {
                    ($commands)
                })
                .await;
            r
        }
        .await;

        use $crate::utils::ConvertDataErrorToStatusCode;
        r.convert_data_error_to_status_code()
    }};
}

/// This is should be used outside axum route handlers.
#[macro_export]
macro_rules! db_write_raw {
    ($state:expr, move |$cmds:ident| $commands:expr) => {{
        async {
            let r: $crate::result::Result<_, $crate::DataError> = $state
                .write(move |$cmds| async move {
                    ($commands)
                })
                .await;
            r
        }
    }};
}

#[macro_export]
macro_rules! create_open_api_router {
    (
        $state_instance:ident,
        $(
            $path:ident::<$state_type:ty>,
        )*
    ) => {
        {
            $(
                let $path = $path::<$state_type>;
            )*
            utoipa_axum::router::OpenApiRouter::new()
            $(
                .merge(utoipa_axum::router::OpenApiRouter::new().routes(utoipa_axum::routes!($path)))
            )*
            .with_state($state_instance)
        }
    };
}
