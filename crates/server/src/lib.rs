#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]
#![allow(clippy::while_let_loop)]

pub mod api;
pub mod api_doc;
pub mod bot;
pub mod content_processing;
pub mod perf;
pub mod scheduled_tasks;
pub mod startup_tasks;
pub mod shutdown_tasks;
pub mod utils;

use std::sync::Arc;

use api_doc::ApiDoc;
use axum::Router;
use config::Config;
use content_processing::{ContentProcessingManager, ContentProcessingManagerQuitHandle};
use model::{AccountIdInternal, EmailMessages};
use perf::ALL_COUNTERS;
use scheduled_tasks::{ScheduledTaskManager, ScheduledTaskManagerQuitHandle};
use server_common::push_notifications::{
    self, PushNotificationManager, PushNotificationManagerQuitHandle,
};
use server_data::{
    content_processing::ContentProcessingManagerData,
    db_manager::DatabaseManager,
    write_commands::{WriteCmdWatcher, WriteCommandRunnerHandle},
};
use server_data_all::{demo::DemoModeManager, load::DbDataToCacheLoader};
use server_state::AppState;
use shutdown_tasks::ShutdownTasks;
use simple_backend::{
    app::SimpleBackendAppState, email::{self, EmailManager, EmailManagerQuitHandle}, media_backup::MediaBackupHandle, perf::AllCounters, web_socket::WebSocketManager, BusinessLogic, ServerQuitWatcher
};
use startup_tasks::StartupTasks;
use tracing::{error, warn};
use utoipa_swagger_ui::SwaggerUi;

use crate::bot::BotClient;

pub struct DatingAppServer {
    config: Arc<Config>,
}

impl DatingAppServer {
    pub fn new(config: Config) -> Self {
        Self {
            config: config.into(),
        }
    }

    pub async fn run(self) {
        let logic = DatingAppBusinessLogic {
            config: self.config.clone(),
            bot_client: None,
            write_cmd_waiter: None,
            database_manager: None,
            content_processing_quit_handle: None,
            push_notifications_quit_handle: None,
            email_manager_quit_handle: None,
            shutdown_tasks: None,
            scheduled_tasks: None,
        };
        let server = simple_backend::SimpleBackend::new(logic, self.config.simple_backend_arc());
        server.run().await;
    }
}

pub struct DatingAppBusinessLogic {
    config: Arc<Config>,
    bot_client: Option<BotClient>,
    write_cmd_waiter: Option<WriteCmdWatcher>,
    database_manager: Option<DatabaseManager>,
    content_processing_quit_handle: Option<ContentProcessingManagerQuitHandle>,
    push_notifications_quit_handle: Option<PushNotificationManagerQuitHandle>,
    email_manager_quit_handle: Option<EmailManagerQuitHandle>,
    shutdown_tasks: Option<ShutdownTasks>,
    scheduled_tasks: Option<ScheduledTaskManagerQuitHandle>,
}

impl BusinessLogic for DatingAppBusinessLogic {
    type AppState = AppState;

    fn all_counters(&self) -> AllCounters {
        ALL_COUNTERS
    }

    fn public_api_router(
        &self,
        web_socket_manager: WebSocketManager,
        state: &Self::AppState,
    ) -> Router {
        let mut router = server_router_account::create_common_server_router(state.clone(), web_socket_manager);

        if self.config.components().account {
            router = router.merge(server_router_account::create_account_server_router(
                state.clone(),
            ))
        }

        if self.config.components().profile {
            router = router.merge(server_router_profile::create_profile_server_router(
                state.clone(),
            ))
        }

        if self.config.components().media {
            router = router.merge(server_router_media::create_media_server_router(
                state.clone(),
            ))
        }

        if self.config.components().chat {
            router = router.merge(server_router_chat::create_chat_server_router(state.clone()))
        }

        router
    }

    fn internal_api_router(&self, state: &Self::AppState) -> Router {
        let mut router = Router::new();
        if self.config.components().account {
            router = router.merge(
                server_router_account::InternalApp::create_account_server_router(state.clone()),
            )
        }

        if self.config.components().media {
            router = router
                .merge(server_router_media::InternalApp::create_media_server_router(state.clone()))
        }

        router
    }

    fn create_swagger_ui(&self) -> Option<SwaggerUi> {
        const API_DOC_URL: &str = "/api-doc/dating-app-api-doc.json";
        Some(
            SwaggerUi::new("/swagger-ui")
                .url(API_DOC_URL, ApiDoc::all())
                .config(
                    utoipa_swagger_ui::Config::from(API_DOC_URL)
                        .display_operation_id(true)
                        .use_base_layout()
                )
        )
    }

    async fn on_before_server_start(
        &mut self,
        simple_state: SimpleBackendAppState,
        media_backup_handle: MediaBackupHandle,
        server_quit_watcher: ServerQuitWatcher,
    ) -> Self::AppState {
        let (push_notification_sender, push_notification_receiver) = push_notifications::channel();
        let (email_sender, email_receiver) = email::channel::<AccountIdInternal, EmailMessages>();
        let (database_manager, router_database_handle, router_database_write_handle) =
            DatabaseManager::new(
                self.config.simple_backend().data_dir().to_path_buf(),
                self.config.clone(),
                media_backup_handle,
                push_notification_sender.clone(),
                email_sender.clone(),
            )
            .await
            .expect("Database init failed");

        DbDataToCacheLoader::load_to_cache(
            router_database_handle.cache(),
            router_database_handle.read_handle_raw(),
            router_database_write_handle.location_raw(),
            &self.config,
        )
        .await
        .expect("Loading data from database to cache failed");

        let (write_cmd_runner_handle, write_cmd_waiter) =
            WriteCommandRunnerHandle::new(router_database_write_handle, &self.config).await;

        let (content_processing, content_processing_receiver) = ContentProcessingManagerData::new();
        let content_processing = Arc::new(content_processing);

        let demo_mode =
            DemoModeManager::new(self.config.demo_mode_config().cloned().unwrap_or_default())
                .expect("Demo mode manager init failed");

        let app_state = AppState::create_app_state(
            router_database_handle,
            write_cmd_runner_handle,
            self.config.clone(),
            content_processing.clone(),
            demo_mode,
            push_notification_sender,
            simple_state,
        )
        .await;

        let content_processing_quit_handle = ContentProcessingManager::new_manager(
            content_processing_receiver,
            app_state.clone(),
            server_quit_watcher.resubscribe(),
        );

        let push_notifications_quit_handle = PushNotificationManager::new_manager(
            self.config.simple_backend(),
            server_quit_watcher.resubscribe(),
            app_state.clone(),
            push_notification_receiver,
        )
        .await;

        let email_manager_quit_handle = EmailManager::new_manager(
            self.config.simple_backend(),
            server_quit_watcher.resubscribe(),
            app_state.clone(),
            email_receiver,
        )
        .await;

        StartupTasks::new(app_state.clone())
            .run_and_wait_completion(
                email_sender,
            )
            .await
            .expect("Startup tasks failed");

        let scheduled_tasks = ScheduledTaskManager::new_manager(
            app_state.clone(),
            server_quit_watcher.resubscribe(),
        );

        self.database_manager = Some(database_manager);
        self.write_cmd_waiter = Some(write_cmd_waiter);
        self.content_processing_quit_handle = Some(content_processing_quit_handle);
        self.push_notifications_quit_handle = Some(push_notifications_quit_handle);
        self.email_manager_quit_handle = Some(email_manager_quit_handle);
        self.shutdown_tasks = Some(ShutdownTasks::new(app_state.clone()));
        self.scheduled_tasks = Some(scheduled_tasks);
        app_state
    }

    async fn on_after_server_start(&mut self) {
        let bot_client = if let Some(bot_config) = self.config.bot_config() {
            let result = BotClient::start_bots(&self.config, bot_config).await;

            match result {
                Ok(bot_manager) => Some(bot_manager),
                Err(e) => {
                    error!("Bot client start failed: {:?}", e);
                    None
                }
            }
        } else {
            None
        };

        self.bot_client = bot_client;
    }

    async fn on_before_server_quit(&mut self) {
        if let Some(bot_client) = self.bot_client.take() {
            match bot_client.stop_bots().await {
                Ok(()) => (),
                Err(e) => error!("Bot client stop failed: {:?}", e),
            }
        }
    }

    async fn on_after_server_quit(self) {
        // Email and push notifications have internal shutdown tasks.
        // Wait those to finish first.
        self.email_manager_quit_handle
            .expect("Not initialized")
            .wait_quit()
            .await;
        self.push_notifications_quit_handle
            .expect("Not initialized")
            .wait_quit()
            .await;

        // Avoid running scheduled tasks simultaneously with shutdown tasks.
        self.scheduled_tasks
            .expect("Not initialized")
            .wait_quit()
            .await;

        let result = self.shutdown_tasks
            .expect("Not initialized")
            .run_and_wait_completion()
            .await;
        if let Err(e) = result {
            error!("Running shutdown tasks failed: {:?}", e);
        }

        self.content_processing_quit_handle
            .expect("Not initialized")
            .wait_quit()
            .await;
        self.write_cmd_waiter
            .expect("Not initialized")
            .wait_untill_all_writing_ends()
            .await;
        self.database_manager
            .expect("Not initialized")
            .close()
            .await;
    }
}

// TODO(prod): Add Cache-Control header for images as web client should
// use browser cache.
