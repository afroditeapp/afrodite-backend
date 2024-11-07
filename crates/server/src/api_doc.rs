use server_state::AppStateEmpty;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(info(
    title = "dating-app-backend",
    description = "Dating app backend API",
    version = "0.1.0",
    license(
        name = "",
        url = "https://example.com"
    ),
))]
pub struct ApiDoc;

impl ApiDoc {
    pub fn all() -> utoipa::openapi::OpenApi {
        let state = AppStateEmpty;
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
            .merge_from(server_api_account::account::news_router(state.clone()).into_openapi())
            .merge_from(server_api_account::account::register_router(state.clone()).into_openapi())
            .merge_from(server_api_account::account::settings_router(state.clone()).into_openapi())
            .merge_from(server_api_account::account::state_router(state.clone()).into_openapi())
            .tag_routes("account");
        doc.merge(account);
        let account_admin = ApiDoc::openapi()
            .merge_from(server_api_account::account_admin::admin_news_router(state.clone()).into_openapi())
            .tag_routes("account_admin");
        doc.merge(account_admin);
        // Media
        doc.merge(server_api_media::ApiDocMedia::openapi());
        let media = ApiDoc::openapi()
            .merge_from(server_api_media::media::content_router(state.clone()).into_openapi())
            .merge_from(server_api_media::media::moderation_request_router(state.clone()).into_openapi())
            .merge_from(server_api_media::media::profile_content_router(state.clone()).into_openapi())
            .merge_from(server_api_media::media::security_content_router(state.clone()).into_openapi())
            .merge_from(server_api_media::media::tile_map_router(state.clone()).into_openapi())
            .tag_routes("media");
        doc.merge(media);
        let media_admin = ApiDoc::openapi()
            .merge_from(server_api_media::media_admin::admin_moderation_router(state.clone()).into_openapi())
            .tag_routes("media_admin");
        doc.merge(media_admin);
        // Profile
        doc.merge(server_api_profile::ApiDocProfile::openapi());
        let profile = ApiDoc::openapi()
            .merge_from(server_api_profile::profile::attributes_router(state.clone()).into_openapi())
            .merge_from(server_api_profile::profile::benchmark_router(state.clone()).into_openapi())
            .merge_from(server_api_profile::profile::favorite_router(state.clone()).into_openapi())
            .merge_from(server_api_profile::profile::iterate_profiles_router(state.clone()).into_openapi())
            .merge_from(server_api_profile::profile::location_router(state.clone()).into_openapi())
            .merge_from(server_api_profile::profile::profile_data_router(state.clone()).into_openapi())
            .merge_from(server_api_profile::profile::statistics_router(state.clone()).into_openapi())
            .tag_routes("profile");
        doc.merge(profile);
        let profile_admin = ApiDoc::openapi()
            .merge_from(server_api_profile::profile_admin::admin_statistics_router(state.clone()).into_openapi())
            .merge_from(server_api_profile::profile_admin::admin_profile_name_allowlist_router(state.clone()).into_openapi())
            .merge_from(server_api_profile::profile_admin::admin_profile_text_router(state.clone()).into_openapi())
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
            .merge_from(server_api_chat::chat::push_notification_router_private(state.clone()).into_openapi())
            .merge_from(server_api_chat::chat::push_notification_router_public(state.clone()).into_openapi())
            .tag_routes("chat");
        doc.merge(chat);
        doc
    }

    pub fn open_api_json_string() -> Result<String, serde_json::Error> {
        Self::all().to_pretty_json()
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
