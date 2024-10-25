#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

use std::sync::Arc;

use config::Config;
use server_api::internal_api::InternalApiClient;
use server_common::push_notifications::PushNotificationSender;
use server_data::{
    content_processing::ContentProcessingManagerData, db_manager::RouterDatabaseReadHandle,
    write_commands::WriteCommandRunnerHandle,
};
use server_data_all::demo::DemoModeManager;
use server_data_profile::statistics::ProfileStatisticsCache;
use simple_backend::app::SimpleBackendAppState;

pub mod state_impl;
pub mod state_impl_empty;
pub mod connection_tools_impl;

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

pub(crate) type E = AppStateEmpty;

#[derive(Clone)]
pub struct AppStateEmpty;
