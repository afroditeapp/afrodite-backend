#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

use std::sync::Arc;

use api_usage::ApiUsageTracker;
use axum::extract::ws::WebSocket;
use client_version::ClientVersionTracker;
use config::Config;
use data_signer::DataSigner;
use ip_address::IpAddressUsageTracker;
use model::{
    Account, AccountIdInternal, PendingNotification, PendingNotificationWithData,
    SyncDataVersionFromClient,
};
use model_chat::SignInWithInfo;
use model_server_data::EmailAddress;
use server_common::{push_notifications::PushNotificationSender, websocket::WebSocketError};
use server_data::{
    app::{DataAllUtils, GetConfig}, content_processing::ContentProcessingManagerData,
    db_manager::RouterDatabaseReadHandle, statistics::ProfileStatisticsCache,
    write_commands::WriteCommandRunnerHandle,
};
use simple_backend::app::SimpleBackendAppState;

use self::internal_api::InternalApiClient;
use crate::demo::DemoModeManager;

pub mod api_usage;
pub mod app;
pub mod demo;
pub mod client_version;
pub mod internal_api;
pub mod ip_address;
pub mod state_impl;
pub mod data_signer;
pub mod utils;

pub use server_common::{data::DataError, result};
pub use utoipa_axum::router::OpenApiRouter;

/// State type for route handlers.
pub type S = AppState;

#[derive(Clone)]
pub struct AppState {
    state: Arc<AppStateInternal>
}

struct AppStateInternal {
    database: Arc<RouterDatabaseReadHandle>,
    write_queue: Arc<WriteCommandRunnerHandle>,
    internal_api: Arc<InternalApiClient>,
    config: Arc<Config>,
    content_processing: Arc<ContentProcessingManagerData>,
    demo_mode: DemoModeManager,
    push_notification_sender: PushNotificationSender,
    simple_backend_state: SimpleBackendAppState,
    profile_statistics_cache: Arc<ProfileStatisticsCache>,
    data_all_utils: &'static dyn DataAllUtils,
    client_version_tracker: ClientVersionTracker,
    api_usage_tracker: ApiUsageTracker,
    ip_address_usage_tracker: IpAddressUsageTracker,
    data_signer: DataSigner,
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
        data_all_utils: &'static dyn DataAllUtils,
    ) -> AppState {
        let database = Arc::new(database_handle);
        let state = AppStateInternal {
            config: config.clone(),
            database: database.clone(),
            write_queue: Arc::new(write_queue),
            internal_api: InternalApiClient::new(config.external_service_urls().clone()).into(),
            content_processing,
            demo_mode,
            push_notification_sender,
            simple_backend_state,
            profile_statistics_cache: ProfileStatisticsCache::default().into(),
            data_all_utils,
            client_version_tracker: ClientVersionTracker::new(),
            api_usage_tracker: ApiUsageTracker::new(),
            ip_address_usage_tracker: IpAddressUsageTracker::new(),
            data_signer: DataSigner::new(),
        };

        AppState {
            state: state.into(),
        }
    }

    pub fn demo_mode(&self) -> &DemoModeManager {
        &self.state.demo_mode
    }

    pub fn data_all_access(&self) -> DataAllAccess {
        DataAllAccess { state: &self.state }
    }
}

pub struct DataAllAccess<'a> {
    state: &'a AppStateInternal,
}

impl DataAllAccess<'_> {
    fn config(&self) -> &Config {
        &self.state.config
    }

    fn read(&self) -> &RouterDatabaseReadHandle {
        &self.state.database
    }

    fn write(&self) -> &WriteCommandRunnerHandle {
        &self.state.write_queue
    }

    fn utils(&self) -> &'static dyn DataAllUtils {
        self.state.data_all_utils
    }

    pub async fn update_unlimited_likes(
        &self,
        id: AccountIdInternal,
        unlimited_likes: bool,
    ) -> server_common::result::Result<(), DataError> {
        let cmd = self
            .utils()
            .update_unlimited_likes(self.write(), id, unlimited_likes);
        cmd.await
    }

    pub async fn register_impl(
        &self,
        sign_in_with: SignInWithInfo,
        email: Option<EmailAddress>,
    ) -> server_common::result::Result<AccountIdInternal, DataError> {
        let cmd = self
            .utils()
            .register_impl(self.write(), sign_in_with, email);
        cmd.await
    }

    pub async fn handle_new_websocket_connection(
        &self,
        socket: &mut WebSocket,
        id: AccountIdInternal,
        sync_versions: Vec<SyncDataVersionFromClient>,
    ) -> server_common::result::Result<(), WebSocketError> {
        let cmd = self.utils().handle_new_websocket_connection(
            self.config(),
            self.read(),
            self.write(),
            &self.state.simple_backend_state.manager_api,
            socket,
            id,
            sync_versions,
        );
        cmd.await
    }

    pub async fn get_push_notification_data(
        &self,
        id: AccountIdInternal,
        notification_value: PendingNotification,
    ) -> PendingNotificationWithData {
        let cmd = self
            .utils()
            .get_push_notification_data(self.read(), id, notification_value);
        cmd.await
    }

    pub async fn complete_initial_setup(
        &self,
        id: AccountIdInternal,
    ) -> server_common::result::Result<Account, DataError> {
        let cmd = self
            .utils()
            .complete_initial_setup(self.config(), self.read(), self.write(), id);
        cmd.await
    }

    pub async fn is_match(
        &self,
        account0: AccountIdInternal,
        account1: AccountIdInternal,
    ) -> server_common::result::Result<bool, DataError> {
        let cmd = self.utils().is_match(self.read(), account0, account1);
        cmd.await
    }
}

// TODO(future): Change write method to have async closure parameter
//               and remove db_write macros.

/// Macro for writing data with different code style.
/// Makes "async move" and "await" keywords unnecessary.
/// The macro "closure" should work like a real closure.
///
/// This macro will guarantee that contents of the closure will run
/// completely even if HTTP connection fails when closure is running.
///
/// Converts crate::data::DataError to crate::api::utils::StatusCode.
///
/// Does not work in Rust 2024 edition because of drop order changes.
///
/// Example usage:
///
/// ```
/// use server_state::db_write;
/// use server_state::utils::StatusCode;
/// use server_state::app::WriteData;
/// use server_state::S;
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
                .write(move |$cmds| async move { ($commands) })
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
            let r: $crate::result::Result<_, $crate::DataError> =
                $state.write(move |$cmds| async move { ($commands) }).await;
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
            let r: $crate::result::Result<_, $crate::DataError> =
                $state.write(move |$cmds| async move { ($commands) }).await;
            r
        }
    }};
}

#[derive(Clone)]
pub struct StateForRouterCreation {
    pub s: S,
    pub disable_api_obfuscation: bool,
}

#[macro_export]
macro_rules! create_open_api_router {
    (
        $( #[doc = $text:literal] )*
        fn $fn_name:ident,
        $(
            $path:ident,
        )*
    ) => {
        $(#[doc = $text])?
        pub fn $fn_name(state: $crate::StateForRouterCreation) -> $crate::OpenApiRouter {
            utoipa_axum::router::OpenApiRouter::new()
            $(
                .merge(utoipa_axum::router::OpenApiRouter::new().routes($crate::__route!(state, $path)))
            )*
            .with_state(state.s)
        }
    };
}

/// Modified version of [utoipa_axum::routes] macro for only one route
/// and runtime API path obfuscation supported.
#[macro_export]
macro_rules! __route {
    ($state:ident, $route_name:ident) => {
        {
            use utoipa_axum::PathItemExt;
            let mut paths = utoipa::openapi::path::Paths::new();
            let mut schemas = Vec::<(String, utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>)>::new();
            let (path, item, types) = utoipa_axum::routes!(@resolve_types $route_name : schemas);
            let path = $crate::obfuscate_api_path(&$state, path);
            #[allow(unused_mut)]
            let mut method_router = types.iter().by_ref().fold(axum::routing::MethodRouter::new(), |router, path_type| {
                router.on(path_type.to_method_filter(), $route_name)
            });
            paths.add_path_operation(&path, types, item);
            (schemas, paths, method_router)
        }
    }
}

pub fn obfuscate_api_path(
    state: &StateForRouterCreation,
    path: String,
) -> String {
    if state.disable_api_obfuscation {
        return path;
    }

    if let Some(salt) = state.s.config().api_obfuscation_salt() {
        obfuscate_path(&path, salt)
    } else {
        path
    }
}

fn obfuscate_path(path: &str, salt: &str) -> String {
    match path.split_once("/{") {
        Some((first, second)) => {
            format!("/{}/{{{}", obfuscate(first, salt), second)
        }
        None => {
            format!("/{}", obfuscate(path, salt))
        }
    }
}

fn obfuscate(text: &str, salt: &str) -> String {
    use sha1::Digest;
    use base64::Engine;

    let mut hasher = sha1::Sha1::new();
    hasher.update(text.as_bytes());
    hasher.update(salt.as_bytes());
    let hash = hasher.finalize();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hash)
}
