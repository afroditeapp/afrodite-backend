use std::sync::Arc;

use config::Config;
use model::{AccountIdInternal, EmailMessages};
use server_data::{
    content_processing::ContentProcessingManagerData, db_manager::DatabaseManager,
    write_commands::WriteCommandRunnerHandle,
};
use server_data_all::app::DataAllUtilsImpl;
use server_state::{demo::DemoModeManager, S};
use simple_backend::{
    app::SimpleBackendAppState, media_backup::MediaBackupHandle, perf::PerfCounterManagerData,
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
    pub fn all(state: S) -> utoipa::openapi::OpenApi {
        let mut doc = ApiDoc::openapi();
        doc.merge(server_api::ApiDocCommon::openapi());
        let common_admin = ApiDoc::openapi()
            .merge_from(server_api::common_admin::perf_router(state.clone()).into_openapi())
            .merge_from(server_api::common_admin::config_router(state.clone()).into_openapi())
            .merge_from(server_api::common_admin::manager_router(state.clone()).into_openapi())
            .tag_routes("common_admin");
        doc.merge(common_admin);
        // Account
        doc.merge(server_api_account::ApiDocAccount::openapi());
        let account = ApiDoc::openapi()
            .merge_from(server_api_account::account::delete_router(state.clone()).into_openapi())
            .merge_from(server_api_account::account::demo_mode_router(state.clone()).into_openapi())
            .merge_from(server_api_account::account::logout_router(state.clone()).into_openapi())
            .merge_from(server_api_account::account::news_router(state.clone()).into_openapi())
            .merge_from(server_api_account::account::register_router(state.clone()).into_openapi())
            .merge_from(server_api_account::account::settings_router(state.clone()).into_openapi())
            .merge_from(server_api_account::account::state_router(state.clone()).into_openapi())
            .tag_routes("account");
        doc.merge(account);
        let account_admin = ApiDoc::openapi()
            .merge_from(
                server_api_account::account_admin::admin_news_router(state.clone()).into_openapi(),
            )
            .tag_routes("account_admin");
        doc.merge(account_admin);
        // Media
        doc.merge(server_api_media::ApiDocMedia::openapi());
        let media = ApiDoc::openapi()
            .merge_from(server_api_media::media::content_router(state.clone()).into_openapi())
            .merge_from(
                server_api_media::media::moderation_request_router(state.clone()).into_openapi(),
            )
            .merge_from(
                server_api_media::media::profile_content_router(state.clone()).into_openapi(),
            )
            .merge_from(
                server_api_media::media::security_content_router(state.clone()).into_openapi(),
            )
            .merge_from(server_api_media::media::tile_map_router(state.clone()).into_openapi())
            .tag_routes("media");
        doc.merge(media);
        let media_admin = ApiDoc::openapi()
            .merge_from(
                server_api_media::media_admin::admin_moderation_router(state.clone())
                    .into_openapi(),
            )
            .tag_routes("media_admin");
        doc.merge(media_admin);
        // Profile
        doc.merge(server_api_profile::ApiDocProfile::openapi());
        let profile = ApiDoc::openapi()
            .merge_from(
                server_api_profile::profile::attributes_router(state.clone()).into_openapi(),
            )
            .merge_from(server_api_profile::profile::benchmark_router(state.clone()).into_openapi())
            .merge_from(server_api_profile::profile::favorite_router(state.clone()).into_openapi())
            .merge_from(
                server_api_profile::profile::iterate_profiles_router(state.clone()).into_openapi(),
            )
            .merge_from(server_api_profile::profile::location_router(state.clone()).into_openapi())
            .merge_from(
                server_api_profile::profile::profile_data_router(state.clone()).into_openapi(),
            )
            .merge_from(
                server_api_profile::profile::statistics_router(state.clone()).into_openapi(),
            )
            .tag_routes("profile");
        doc.merge(profile);
        let profile_admin = ApiDoc::openapi()
            .merge_from(
                server_api_profile::profile_admin::admin_statistics_router(state.clone())
                    .into_openapi(),
            )
            .merge_from(
                server_api_profile::profile_admin::admin_profile_name_allowlist_router(
                    state.clone(),
                )
                .into_openapi(),
            )
            .merge_from(
                server_api_profile::profile_admin::admin_profile_text_router(state.clone())
                    .into_openapi(),
            )
            .tag_routes("profile_admin");
        doc.merge(profile_admin);
        // Chat
        doc.merge(server_api_chat::ApiDocChat::openapi());
        let chat = ApiDoc::openapi()
            .merge_from(server_api_chat::chat::block_router(state.clone()).into_openapi())
            .merge_from(server_api_chat::chat::like_router(state.clone()).into_openapi())
            .merge_from(server_api_chat::chat::match_router(state.clone()).into_openapi())
            .merge_from(server_api_chat::chat::message_router(state.clone()).into_openapi())
            .merge_from(server_api_chat::chat::public_key_router(state.clone()).into_openapi())
            .merge_from(
                server_api_chat::chat::push_notification_router_private(state.clone())
                    .into_openapi(),
            )
            .merge_from(
                server_api_chat::chat::push_notification_router_public(state.clone())
                    .into_openapi(),
            )
            .tag_routes("chat");
        doc.merge(chat);
        doc
    }

    pub async fn open_api_json_string() -> Result<String, serde_json::Error> {
        let config = Arc::new(SimpleBackendConfig::load_from_file_with_in_ram_database());
        let perf_data = PerfCounterManagerData::new(&[]).into();
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

        Self::all(app_state).to_pretty_json()
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
