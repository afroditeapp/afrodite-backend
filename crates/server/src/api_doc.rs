use std::sync::Arc;

use config::Config;
use model::{AccountIdInternal, EmailMessages};
use server_data::{
    content_processing::ContentProcessingManagerData, db_manager::DatabaseManager,
    write_commands::WriteCommandRunnerHandle,
};
use server_data_all::app::DataAllUtilsImpl;
use server_state::{demo::DemoModeManager, StateForRouterCreation, S};
use simple_backend::{
    app::SimpleBackendAppState, media_backup::MediaBackupHandle, perf::PerfMetricsManagerData,
};
use simple_backend_config::SimpleBackendConfig;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(info(
    title = "dating-app-backend",
    description = "Dating app backend API",
    version = "0.1.0",
    license(name = "", url = "https://example.com"),
))]
pub struct ApiDoc;

impl ApiDoc {
    pub fn all(state: StateForRouterCreation) -> utoipa::openapi::OpenApi {
        let mut doc = ApiDoc::openapi();
        doc.merge(server_api::ApiDocCommon::openapi());
        let common_admin = ApiDoc::openapi()
            .merge_from(server_api::common_admin::router_perf(state.clone()).into_openapi())
            .merge_from(server_api::common_admin::router_config(state.clone()).into_openapi())
            .merge_from(server_api::common_admin::router_manager(state.clone()).into_openapi())
            .tag_routes("common_admin");
        doc.merge(common_admin);
        // Account
        doc.merge(server_api_account::ApiDocAccount::openapi());
        let account = ApiDoc::openapi()
            .merge_from(server_api_account::account::router_ban(state.clone()).into_openapi())
            .merge_from(server_api_account::account::router_delete(state.clone()).into_openapi())
            .merge_from(server_api_account::account::router_demo_mode(state.clone()).into_openapi())
            .merge_from(server_api_account::account::router_logout(state.clone()).into_openapi())
            .merge_from(server_api_account::account::router_news(state.clone()).into_openapi())
            .merge_from(server_api_account::account::router_register(state.clone()).into_openapi())
            .merge_from(server_api_account::account::router_settings(state.clone()).into_openapi())
            .merge_from(server_api_account::account::router_state(state.clone()).into_openapi())
            .tag_routes("account");
        doc.merge(account);
        let account_admin = ApiDoc::openapi()
            .merge_from(
                server_api_account::account_admin::router_admin_ban(state.clone()).into_openapi(),
            )
            .merge_from(
                server_api_account::account_admin::router_admin_delete(state.clone()).into_openapi(),
            )
            .merge_from(
                server_api_account::account_admin::router_admin_news(state.clone()).into_openapi(),
            )
            .merge_from(
                server_api_account::account_admin::router_admin_search(state.clone()).into_openapi(),
            )
            .merge_from(
                server_api_account::account_admin::router_admin_permissions(state.clone()).into_openapi(),
            )
            .merge_from(
                server_api_account::account_admin::router_admin_state(state.clone()).into_openapi(),
            )
            .tag_routes("account_admin");
        doc.merge(account_admin);
        // Media
        doc.merge(server_api_media::ApiDocMedia::openapi());
        let media = ApiDoc::openapi()
            .merge_from(server_api_media::media::router_content(state.clone()).into_openapi())
            .merge_from(
                server_api_media::media::router_media_content(state.clone()).into_openapi(),
            )
            .merge_from(
                server_api_media::media::router_profile_content(state.clone()).into_openapi(),
            )
            .merge_from(
                server_api_media::media::router_security_content(state.clone()).into_openapi(),
            )
            .merge_from(server_api_media::media::router_tile_map(state.clone()).into_openapi())
            .tag_routes("media");
        doc.merge(media);
        let media_admin = ApiDoc::openapi()
            .merge_from(
                server_api_media::media_admin::router_admin_moderation(state.clone())
                    .into_openapi(),
            )
            .tag_routes("media_admin");
        doc.merge(media_admin);
        // Profile
        doc.merge(server_api_profile::ApiDocProfile::openapi());
        let profile = ApiDoc::openapi()
            .merge_from(
                server_api_profile::profile::router_filters(state.clone()).into_openapi(),
            )
            .merge_from(server_api_profile::profile::router_benchmark(state.clone()).into_openapi())
            .merge_from(server_api_profile::profile::router_favorite(state.clone()).into_openapi())
            .merge_from(
                server_api_profile::profile::router_iterate_profiles(state.clone()).into_openapi(),
            )
            .merge_from(server_api_profile::profile::router_location(state.clone()).into_openapi())
            .merge_from(
                server_api_profile::profile::router_profile_data(state.clone()).into_openapi(),
            )
            .merge_from(
                server_api_profile::profile::router_statistics(state.clone()).into_openapi(),
            )
            .tag_routes("profile");
        doc.merge(profile);
        let profile_admin = ApiDoc::openapi()
            .merge_from(
                server_api_profile::profile_admin::router_admin_statistics(state.clone())
                    .into_openapi(),
            )
            .merge_from(
                server_api_profile::profile_admin::router_admin_profile_data(state.clone())
                    .into_openapi(),
            )
            .merge_from(
                server_api_profile::profile_admin::router_admin_profile_name_allowlist(
                    state.clone(),
                )
                .into_openapi(),
            )
            .merge_from(
                server_api_profile::profile_admin::router_admin_profile_text(state.clone())
                    .into_openapi(),
            )
            .tag_routes("profile_admin");
        doc.merge(profile_admin);
        // Chat
        doc.merge(server_api_chat::ApiDocChat::openapi());
        let chat = ApiDoc::openapi()
            .merge_from(server_api_chat::chat::router_block(state.clone()).into_openapi())
            .merge_from(server_api_chat::chat::router_like(state.clone()).into_openapi())
            .merge_from(server_api_chat::chat::router_match(state.clone()).into_openapi())
            .merge_from(server_api_chat::chat::router_message(state.clone()).into_openapi())
            .merge_from(server_api_chat::chat::router_public_key(state.clone()).into_openapi())
            .merge_from(
                server_api_chat::chat::router_push_notification_private(state.clone())
                    .into_openapi(),
            )
            .merge_from(
                server_api_chat::chat::router_push_notification_public(state.clone())
                    .into_openapi(),
            )
            .tag_routes("chat");
        doc.merge(chat);
        doc
    }

    pub async fn open_api_json_string() -> Result<String, serde_json::Error> {
        let config = Arc::new(SimpleBackendConfig::load_from_file_with_in_ram_database());
        let perf_data = PerfMetricsManagerData::new(&[]).into();
        let simple_state = SimpleBackendAppState::new(config.clone(), perf_data)
            .await
            .unwrap();

        let config = Arc::new(Config::minimal_config_for_api_doc_json(config.clone()));

        let (push_notification_sender, _) = server_common::push_notifications::channel();
        let (email_sender, _) =
            simple_backend::email::channel::<AccountIdInternal, EmailMessages>();
        let (_, router_database_handle, router_database_write_handle) = DatabaseManager::new(
            config.simple_backend().data_dir().to_path_buf(),
            config.clone(),
            MediaBackupHandle::broken_handle_for_api_doc_json(),
            push_notification_sender.clone(),
            email_sender.clone(),
        )
        .await
        .expect("Database init failed");

        let (write_cmd_runner_handle, _) =
            WriteCommandRunnerHandle::new(router_database_write_handle.into(), &config).await;

        let (content_processing, _) = ContentProcessingManagerData::new();
        let content_processing = Arc::new(content_processing);

        let demo_mode =
            DemoModeManager::new(config.demo_mode_config().cloned().unwrap_or_default())
                .expect("Demo mode manager init failed");

        let app_state = S::create_app_state(
            router_database_handle,
            write_cmd_runner_handle,
            config.clone(),
            content_processing.clone(),
            demo_mode,
            push_notification_sender,
            simple_state,
            &DataAllUtilsImpl,
        )
        .await;

        let state = StateForRouterCreation {
            s: app_state,
            disable_api_obfuscation: true,
        };

        Self::all(state).to_pretty_json()
    }
}

trait OpenApiExtensions: Sized {
    fn tag_routes(self, tag: &str) -> Self;
}

impl OpenApiExtensions for utoipa::openapi::OpenApi {
    fn tag_routes(mut self, tag: &str) -> Self {
        let handle_operation = |operation: Option<&mut utoipa::openapi::path::Operation>| {
            if let Some(operation) = operation {
                let mut tags = operation.tags.take().unwrap_or_default();
                tags.clear();
                tags.push(tag.to_string());
                operation.tags = Some(tags);
            }
        };

        for (_, item) in self.paths.paths.iter_mut() {
            handle_operation(item.get.as_mut());
            handle_operation(item.put.as_mut());
            handle_operation(item.post.as_mut());
            handle_operation(item.delete.as_mut());
            handle_operation(item.options.as_mut());
            handle_operation(item.head.as_mut());
            handle_operation(item.patch.as_mut());
            handle_operation(item.trace.as_mut());
        }
        self
    }
}
