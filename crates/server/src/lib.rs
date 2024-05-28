#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

#![allow(clippy::while_let_loop)]

pub mod api;
pub mod app;
pub mod bot;
pub mod content_processing;
pub mod data;
pub mod demo;
pub mod event;
pub mod internal_api;
pub mod perf;
pub mod push_notifications;
pub mod result;
pub mod startup_tasks;
pub mod utils;

use std::sync::Arc;

use app::AppState;
use async_trait::async_trait;
use axum::Router;
use config::Config;
use content_processing::{
    ContentProcessingManager, ContentProcessingManagerData, ContentProcessingManagerQuitHandle,
};
use data::write_commands::WriteCmdWatcher;
use demo::DemoModeManager;
use perf::ALL_COUNTERS;
use push_notifications::{PushNotificationManager, PushNotificationManagerQuitHandle};
use simple_backend::{
    app::{SimpleBackendAppState, StateBuilder},
    media_backup::MediaBackupHandle,
    perf::AllCounters,
    web_socket::WebSocketManager,
    BusinessLogic, ServerQuitWatcher,
};
use startup_tasks::StartupTasks;
use tracing::{error, warn};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use self::{
    app::{routes_internal::InternalApp, App},
    data::{write_commands::WriteCommandRunnerHandle, DatabaseManager},
};
use crate::{api::ApiDoc, bot::BotClient};

pub struct PihkaServer {
    config: Arc<Config>,
}

impl PihkaServer {
    pub fn new(config: Config) -> Self {
        Self {
            config: config.into(),
        }
    }

    pub async fn run(self) {
        let logic = PihkaBusinessLogic {
            config: self.config.clone(),
            bot_client: None,
            write_cmd_waiter: None,
            database_manager: None,
            content_processing_quit_handle: None,
            push_notifications_quit_handle: None,
        };
        let server = simple_backend::SimpleBackend::new(logic, self.config.simple_backend_arc());
        server.run().await;
    }
}

pub struct PihkaBusinessLogic {
    config: Arc<Config>,
    bot_client: Option<BotClient>,
    write_cmd_waiter: Option<WriteCmdWatcher>,
    database_manager: Option<DatabaseManager>,
    content_processing_quit_handle: Option<ContentProcessingManagerQuitHandle>,
    push_notifications_quit_handle: Option<PushNotificationManagerQuitHandle>,
}

#[async_trait]
impl BusinessLogic for PihkaBusinessLogic {
    type AppState = AppState;

    fn all_counters(&self) -> AllCounters {
        ALL_COUNTERS
    }

    fn public_api_router(
        &self,
        web_socket_manager: WebSocketManager,
        state: &SimpleBackendAppState<Self::AppState>,
    ) -> Router {
        let mut app = App::new(state.clone(), web_socket_manager);
        let mut router = app.create_common_server_router();

        if self.config.components().account {
            router = router.merge(app.create_account_server_router())
        }

        if self.config.components().profile {
            router = router.merge(app.create_profile_server_router())
        }

        if self.config.components().media {
            router = router.merge(app.create_media_server_router())
        }

        if self.config.components().chat {
            router = router.merge(app.create_chat_server_router())
        }

        router
    }

    fn internal_api_router(&self, state: &SimpleBackendAppState<Self::AppState>) -> Router {
        let mut router = Router::new();
        if self.config.components().account {
            router = router.merge(InternalApp::create_account_server_router(state.clone()))
        }

        if self.config.components().profile {
            router = router.merge(InternalApp::create_profile_server_router(state.clone()))
        }

        if self.config.components().media {
            router = router.merge(InternalApp::create_media_server_router(state.clone()))
        }

        if self.config.components().chat {
            router = router.merge(InternalApp::create_chat_server_router(state.clone()))
        }

        router
    }

    fn create_swagger_ui(&self) -> Option<SwaggerUi> {
        Some(SwaggerUi::new("/swagger-ui").url("/api-doc/pihka_api.json", ApiDoc::openapi()))
    }

    async fn on_before_server_start(
        &mut self,
        state_builder: StateBuilder,
        media_backup_handle: MediaBackupHandle,
        server_quit_watcher: ServerQuitWatcher,
    ) -> SimpleBackendAppState<Self::AppState> {
        let (push_notification_sender, push_notification_receiver) =
            PushNotificationManager::channel();
        let (database_manager, router_database_handle, router_database_write_handle) =
            DatabaseManager::new(
                self.config.simple_backend().data_dir().to_path_buf(),
                self.config.clone(),
                media_backup_handle,
                push_notification_sender.clone(),
            )
            .await
            .expect("Database init failed");

        let (write_cmd_runner_handle, write_cmd_waiter) =
            WriteCommandRunnerHandle::new(router_database_write_handle, &self.config).await;

        let (content_processing, content_processing_receiver) = ContentProcessingManagerData::new();
        let content_processing = Arc::new(content_processing);

        let demo_mode =
            DemoModeManager::new(self.config.demo_mode_config().cloned().unwrap_or_default())
                .expect("Demo mode manager init failed");
        let app_state = App::create_app_state(
            router_database_handle,
            write_cmd_runner_handle,
            self.config.clone(),
            content_processing.clone(),
            demo_mode,
            push_notification_sender,
        )
        .await;

        let state = state_builder.build(app_state.clone());

        let content_processing_quit_handle = ContentProcessingManager::new_manager(
            content_processing_receiver,
            state.clone(),
            server_quit_watcher.resubscribe(),
        );

        let push_notifications_quit_handle = PushNotificationManager::new_manager(
            self.config.simple_backend(),
            server_quit_watcher.resubscribe(),
            state.clone(),
            push_notification_receiver,
        )
        .await;

        StartupTasks::new(state.clone())
            .run_and_wait_completion()
            .await
            .expect("Startup tasks failed");

        self.database_manager = Some(database_manager);
        self.write_cmd_waiter = Some(write_cmd_waiter);
        self.content_processing_quit_handle = Some(content_processing_quit_handle);
        self.push_notifications_quit_handle = Some(push_notifications_quit_handle);
        state
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
        self.push_notifications_quit_handle
            .expect("Not initialized")
            .wait_quit()
            .await;
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
