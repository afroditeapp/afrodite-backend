#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

use std::sync::Arc;

use config::Config;
use self::internal_api::InternalApiClient;
use server_common::push_notifications::PushNotificationSender;
use server_data::{
    content_processing::ContentProcessingManagerData, db_manager::RouterDatabaseReadHandle,
    write_commands::WriteCommandRunnerHandle,
};
use server_data_all::demo::DemoModeManager;
use server_data_profile::statistics::ProfileStatisticsCache;
use simple_backend::app::SimpleBackendAppState;

pub mod state_impl;
pub mod connection_tools_impl;
pub mod internal_api;
pub mod websocket;
pub mod app;
pub mod utils;

pub use server_common::{data::DataError, result};

/// State type for route handlers.
pub type S = AppState;

#[derive(Clone)]
pub struct AppState {
    database: Arc<RouterDatabaseReadHandle>,
    write_queue: Arc<WriteCommandRunnerHandle>,
    internal_api: Arc<InternalApiClient>,
    config: Arc<Config>,
    content_processing: Arc<ContentProcessingManagerData>,
    demo_mode: DemoModeManager,
    push_notification_sender: PushNotificationSender,
    simple_backend_state: SimpleBackendAppState,
    profile_statistics_cache: Arc<ProfileStatisticsCache>,
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub async fn create_app_state(
        database_handle: RouterDatabaseReadHandle,
        write_queue: WriteCommandRunnerHandle,
        config: Arc<Config>,
        content_processing: Arc<ContentProcessingManagerData>,
        demo_mode: DemoModeManager,
        push_notification_sender: PushNotificationSender,
        simple_backend_state: SimpleBackendAppState,
    ) -> AppState {
        let database = Arc::new(database_handle);
        let state = AppState {
            config: config.clone(),
            database: database.clone(),
            write_queue: Arc::new(write_queue),
            internal_api: InternalApiClient::new(config.external_service_urls().clone()).into(),
            content_processing,
            demo_mode,
            push_notification_sender,
            simple_backend_state,
            profile_statistics_cache: ProfileStatisticsCache::default().into()
        };

        state
    }
}

#[derive(Clone)]
pub struct AppStateEmpty;

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
/// use server_api::S;
/// pub async fn axum_route_handler(
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
            $path:ident,
        )*
    ) => {
        {
            utoipa_axum::router::OpenApiRouter::new()
            $(
                .merge(utoipa_axum::router::OpenApiRouter::new().routes(utoipa_axum::routes!($path)))
            )*
            .with_state($state_instance)
        }
    };
}
