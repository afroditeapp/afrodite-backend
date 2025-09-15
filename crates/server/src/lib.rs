#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]
#![allow(clippy::while_let_loop)]

pub mod admin_notifications;
pub mod api;
pub mod api_doc;
pub mod content_processing;
pub mod daily_likes;
pub mod data_export;
pub mod dynamic_config;
pub mod email;
pub mod hourly_tasks;
pub mod perf;
pub mod profile_search;
pub mod push_notifications;
pub mod scheduled_tasks;
pub mod shutdown_tasks;
pub mod startup_tasks;
pub mod task_utils;
pub mod unlimited_likes;

use std::sync::Arc;

use api_doc::ApiDoc;
use axum::Router;
use config::Config;
use content_processing::{ContentProcessingManager, ContentProcessingManagerQuitHandle};
use email::ServerEmailDataProvider;
use hourly_tasks::{HourlyTaskManager, HourlyTaskManagerQuitHandle};
use model::{AccountIdInternal, EmailMessages};
use perf::ALL_COUNTERS;
use profile_search::{ProfileSearchManager, ProfileSearchManagerQuitHandle};
use push_notifications::ServerPushNotificationStateProvider;
use scheduled_tasks::{ScheduledTaskManager, ScheduledTaskManagerQuitHandle};
use server_api::app::{DataSignerProvider, GetConfig, WriteDynamicConfig};
use server_common::push_notifications::{
    PushNotificationManager, PushNotificationManagerQuitHandle,
};
use server_data::{
    content_processing::ContentProcessingManagerData,
    data_export::DataExportManagerData,
    db_manager::DatabaseManager,
    write_commands::{WriteCmdWatcher, WriteCommandRunnerHandle},
};
use server_data_all::{app::DataAllUtilsImpl, load::DbDataToCacheLoader};
use server_router_account::{AccountRoutes, CommonRoutes, LocalBotApiRoutes, RemoteBotApiRoutes};
use server_router_chat::ChatRoutes;
use server_router_media::MediaRoutes;
use server_router_profile::ProfileRoutes;
use server_state::{
    AppState, StateForRouterCreation, admin_notifications::AdminNotificationManagerData,
    demo::DemoAccountManager, dynamic_config::DynamicConfigManagerData,
};
use shutdown_tasks::ShutdownTasks;
use simple_backend::{
    BusinessLogic, ServerQuitWatcher,
    app::SimpleBackendAppState,
    email::{EmailManager, EmailManagerQuitHandle},
    perf::counters::AllCounters,
    web_socket::WebSocketManager,
};
use startup_tasks::StartupTasks;
use tracing::{error, warn};
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    admin_notifications::{AdminNotificationManager, AdminNotificationManagerQuitHandle},
    daily_likes::{DailyLikesManager, DailyLikesManagerQuitHandle},
    data_export::{DataExportManager, DataExportManagerQuitHandle},
    dynamic_config::{DynamicConfigManager, DynamicConfigManagerQuitHandle},
    unlimited_likes::{UnlimitedLikesManager, UnlimitedLikesManagerQuitHandle},
};

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
            dynamic_config_manager: None,
            write_cmd_waiter: None,
            database_manager: None,
            content_processing_quit_handle: None,
            admin_notification_quit_handle: None,
            data_export_quit_handle: None,
            push_notifications_quit_handle: None,
            email_manager_quit_handle: None,
            shutdown_tasks: None,
            scheduled_tasks: None,
            hourly_tasks: None,
            profile_search: None,
            unlimited_likes: None,
            daily_likes: None,
        };
        let server = simple_backend::SimpleBackend::new(logic, self.config.simple_backend_arc());
        server.run().await;
    }
}

pub struct DatingAppBusinessLogic {
    config: Arc<Config>,
    dynamic_config_manager: Option<DynamicConfigManagerQuitHandle>,
    write_cmd_waiter: Option<WriteCmdWatcher>,
    database_manager: Option<DatabaseManager>,
    content_processing_quit_handle: Option<ContentProcessingManagerQuitHandle>,
    admin_notification_quit_handle: Option<AdminNotificationManagerQuitHandle>,
    data_export_quit_handle: Option<DataExportManagerQuitHandle>,
    push_notifications_quit_handle: Option<PushNotificationManagerQuitHandle>,
    email_manager_quit_handle: Option<EmailManagerQuitHandle>,
    shutdown_tasks: Option<ShutdownTasks>,
    scheduled_tasks: Option<ScheduledTaskManagerQuitHandle>,
    hourly_tasks: Option<HourlyTaskManagerQuitHandle>,
    profile_search: Option<ProfileSearchManagerQuitHandle>,
    unlimited_likes: Option<UnlimitedLikesManagerQuitHandle>,
    daily_likes: Option<DailyLikesManagerQuitHandle>,
}

impl DatingAppBusinessLogic {
    fn add_obfuscation_supported_routes(
        &self,
        mut router: Router,
        state: StateForRouterCreation,
    ) -> Router {
        router = router.merge(CommonRoutes::routes_with_obfuscation_support(state.clone()));

        router = router.merge(AccountRoutes::routes_with_obfuscation_support(
            state.clone(),
        ));

        router = router.merge(ProfileRoutes::routes_with_obfuscation_support(
            state.clone(),
        ));

        router = router.merge(MediaRoutes::routes_with_obfuscation_support(state.clone()));

        router = router.merge(ChatRoutes::routes_with_obfuscation_support(state.clone()));

        router
    }
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
        disable_api_obfuscation: bool,
    ) -> Router {
        let state = StateForRouterCreation {
            s: state.clone(),
            disable_api_obfuscation,
            allow_only_remote_bots: false,
        };

        let mut router =
            CommonRoutes::routes_without_obfuscation_support(state.clone(), web_socket_manager);

        if !state.s.config().remote_bots().is_empty() {
            router = router.merge(RemoteBotApiRoutes::router(state.s.clone()))
        }
        router = router.merge(AccountRoutes::routes_without_obfuscation_support(
            state.clone(),
        ));

        router = router.merge(ProfileRoutes::routes_without_obfuscation_support(
            state.clone(),
        ));

        router = router.merge(MediaRoutes::routes_without_obfuscation_support(
            state.clone(),
        ));

        router = router.merge(ChatRoutes::routes_without_obfuscation_support(
            state.clone(),
        ));

        router = self.add_obfuscation_supported_routes(router, state.clone());

        // Add unobfuscated API for remote bots if needed

        let api_obfuscation_enabled =
            state.s.config().api_obfuscation_salt().is_some() && !state.disable_api_obfuscation;
        if api_obfuscation_enabled && !state.s.config().remote_bots().is_empty() {
            router = self.add_obfuscation_supported_routes(
                router,
                StateForRouterCreation {
                    s: state.s,
                    disable_api_obfuscation: true,
                    allow_only_remote_bots: true,
                },
            );
        }

        router
    }

    fn local_bot_api_router(
        &self,
        web_socket_manager: WebSocketManager,
        state: &Self::AppState,
    ) -> Router {
        let mut router = Router::new();

        router = router.merge(LocalBotApiRoutes::router(state.clone()));

        router = router.merge(self.public_api_router(web_socket_manager, state, true));

        router
    }

    fn create_swagger_ui(&self, state: &Self::AppState) -> Option<SwaggerUi> {
        const API_DOC_URL: &str = "/api-doc/app_api.json";
        const API_DOC_URL_OBFUSCATION_DISABLED: &str = "/api-doc/app_api_obfuscation_disabled.json";
        let router_state = StateForRouterCreation {
            s: state.clone(),
            disable_api_obfuscation: false,
            allow_only_remote_bots: false,
        };
        let mut swagger =
            SwaggerUi::new("/swagger-ui").url(API_DOC_URL, ApiDoc::all(router_state.clone()));

        let swagger_config = if state.config().api_obfuscation_salt().is_some() {
            let router_state = StateForRouterCreation {
                s: state.clone(),
                disable_api_obfuscation: true,
                allow_only_remote_bots: false,
            };
            swagger = swagger.url(
                API_DOC_URL_OBFUSCATION_DISABLED,
                ApiDoc::all(router_state.clone()),
            );
            utoipa_swagger_ui::Config::new([API_DOC_URL, API_DOC_URL_OBFUSCATION_DISABLED])
        } else {
            utoipa_swagger_ui::Config::new([API_DOC_URL])
        };
        Some(swagger.config(swagger_config.display_operation_id(true)))
    }

    async fn on_before_server_start(
        &mut self,
        simple_state: SimpleBackendAppState,
        server_quit_watcher: ServerQuitWatcher,
    ) -> Self::AppState {
        let (push_notification_sender, push_notification_receiver) =
            server_common::push_notifications::channel();
        let (email_sender, email_receiver) =
            simple_backend::email::channel::<AccountIdInternal, EmailMessages>();
        let (database_manager, router_database_handle, router_database_write_handle) =
            DatabaseManager::new(
                self.config.clone(),
                push_notification_sender.clone(),
                email_sender.clone(),
            )
            .await
            .expect("Database init failed");

        DbDataToCacheLoader::load_to_cache(
            router_database_handle.cache_read_write_access(),
            router_database_handle.read_handle_raw(),
            router_database_write_handle.location_raw(),
        )
        .await
        .expect("Loading data from database to cache failed");

        let (write_cmd_runner_handle, write_cmd_waiter) =
            WriteCommandRunnerHandle::new(router_database_write_handle.into(), &self.config).await;

        let (content_processing, content_processing_receiver) = ContentProcessingManagerData::new();
        let content_processing = Arc::new(content_processing);

        let (admin_notification, admin_notification_receiver) = AdminNotificationManagerData::new();
        let admin_notification = Arc::new(admin_notification);

        let (data_export, data_export_receiver) = DataExportManagerData::new();

        let (dynamic_config_manager, dynamic_config_manager_receiver) =
            DynamicConfigManagerData::new();

        let demo = DemoAccountManager::new(
            self.config
                .demo_account_config()
                .cloned()
                .unwrap_or_default(),
        )
        .expect("Demo account manager init failed");

        let app_state = AppState::create_app_state(
            router_database_handle,
            write_cmd_runner_handle,
            self.config.clone(),
            content_processing.clone(),
            admin_notification.clone(),
            demo,
            push_notification_sender,
            data_export,
            dynamic_config_manager,
            simple_state,
            &DataAllUtilsImpl,
        )
        .await;

        app_state
            .data_signer()
            .load_or_generate_keys(self.config.simple_backend())
            .await
            .expect("Data signer init failed");

        let dynamic_config_manager_quit_handle =
            DynamicConfigManager::new_manager(dynamic_config_manager_receiver, app_state.clone());

        let content_processing_quit_handle = ContentProcessingManager::new_manager(
            content_processing_receiver,
            app_state.clone(),
            server_quit_watcher.resubscribe(),
        );

        let admin_notification_quit_handle = AdminNotificationManager::new_manager(
            admin_notification_receiver,
            app_state.clone(),
            server_quit_watcher.resubscribe(),
        );

        let data_export_quit_handle = DataExportManager::new_manager(
            data_export_receiver,
            app_state.clone(),
            server_quit_watcher.resubscribe(),
        );

        let push_notifications_quit_handle = PushNotificationManager::new_manager(
            self.config.clone(),
            server_quit_watcher.resubscribe(),
            ServerPushNotificationStateProvider::new(app_state.clone()),
            push_notification_receiver,
        )
        .await;

        let email_manager_quit_handle = EmailManager::new_manager(
            self.config.simple_backend(),
            server_quit_watcher.resubscribe(),
            ServerEmailDataProvider::new(app_state.clone()),
            email_receiver,
        )
        .await;

        StartupTasks::new(app_state.clone())
            .run_and_wait_completion(email_sender)
            .await
            .expect("Startup tasks failed");

        let scheduled_tasks =
            ScheduledTaskManager::new_manager(app_state.clone(), server_quit_watcher.resubscribe());
        let hourly_tasks =
            HourlyTaskManager::new_manager(app_state.clone(), server_quit_watcher.resubscribe());
        let profile_search =
            ProfileSearchManager::new_manager(app_state.clone(), server_quit_watcher.resubscribe());
        let unlimited_likes = UnlimitedLikesManager::new_manager(
            app_state.clone(),
            server_quit_watcher.resubscribe(),
        );
        let daily_likes =
            DailyLikesManager::new_manager(app_state.clone(), server_quit_watcher.resubscribe());

        self.database_manager = Some(database_manager);
        self.write_cmd_waiter = Some(write_cmd_waiter);
        self.content_processing_quit_handle = Some(content_processing_quit_handle);
        self.admin_notification_quit_handle = Some(admin_notification_quit_handle);
        self.data_export_quit_handle = Some(data_export_quit_handle);
        self.push_notifications_quit_handle = Some(push_notifications_quit_handle);
        self.email_manager_quit_handle = Some(email_manager_quit_handle);
        self.shutdown_tasks = Some(ShutdownTasks::new(app_state.clone()));
        self.scheduled_tasks = Some(scheduled_tasks);
        self.hourly_tasks = Some(hourly_tasks);
        self.profile_search = Some(profile_search);
        self.unlimited_likes = Some(unlimited_likes);
        self.daily_likes = Some(daily_likes);

        self.dynamic_config_manager = Some(dynamic_config_manager_quit_handle);

        app_state
    }

    async fn on_after_server_start(&mut self, state: &Self::AppState) {
        state.reload_dynamic_config().await;
    }

    async fn on_before_server_quit(&mut self) {
        self.dynamic_config_manager.take().unwrap().quit().await;
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

        // Avoid running tasks simultaneously with shutdown tasks.
        self.daily_likes.expect("Not initialized").wait_quit().await;
        self.unlimited_likes
            .expect("Not initialized")
            .wait_quit()
            .await;
        self.profile_search
            .expect("Not initialized")
            .wait_quit()
            .await;
        self.hourly_tasks
            .expect("Not initialized")
            .wait_quit()
            .await;
        self.scheduled_tasks
            .expect("Not initialized")
            .wait_quit()
            .await;
        self.content_processing_quit_handle
            .expect("Not initialized")
            .wait_quit()
            .await;
        self.admin_notification_quit_handle
            .expect("Not initialized")
            .wait_quit()
            .await;
        self.data_export_quit_handle
            .expect("Not initialized")
            .wait_quit()
            .await;

        let result = self
            .shutdown_tasks
            .expect("Not initialized")
            .run_and_wait_completion()
            .await;
        if let Err(e) = result {
            error!("Running shutdown tasks failed: {:?}", e);
        }

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

// TODO(web): Add Cache-Control header for images as web client should
// use browser cache.
