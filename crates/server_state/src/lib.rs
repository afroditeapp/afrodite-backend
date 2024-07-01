#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

use std::sync::Arc;

use config::Config;
use model::{AccountIdInternal, EmailMessages};
use server_api::internal_api::InternalApiClient;
use server_common::push_notifications::PushNotificationSender;
use server_data::{
    content_processing::ContentProcessingManagerData, db_manager::RouterDatabaseReadHandle,
    write_commands::WriteCommandRunnerHandle,
};
use server_data_all::demo::DemoModeManager;
use simple_backend::{app::SimpleBackendAppState, email::EmailSender};

pub mod state_impl;
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
    email_sender: EmailSender<AccountIdInternal, EmailMessages>,
    simple_backend_state: SimpleBackendAppState,
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
        email_sender: EmailSender<AccountIdInternal, EmailMessages>,
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
            email_sender,
            simple_backend_state,
        };

        state
    }
}
