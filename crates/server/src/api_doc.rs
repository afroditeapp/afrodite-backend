use server_state::AppStateEmpty;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(info(
    title = "pihka-backend",
    description = "Pihka backend API",
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
        doc.merge(server_api::common_admin::perf_router(state.clone()).into_openapi());
        doc.merge(server_api::common_admin::config_router(state.clone()).into_openapi());
        doc.merge(server_api::common_admin::manager_router(state.clone()).into_openapi());
        // Account
        doc.merge(server_api_account::ApiDocAccount::openapi());
        doc.merge(server_api_account::account::delete_router(state.clone()).into_openapi());
        doc.merge(server_api_account::account::demo_mode_router(state.clone()).into_openapi());
        doc.merge(server_api_account::account::news_router(state.clone()).into_openapi());
        doc.merge(server_api_account::account::register_router(state.clone()).into_openapi());
        doc.merge(server_api_account::account::settings_router(state.clone()).into_openapi());
        doc.merge(server_api_account::account::state_router(state.clone()).into_openapi());
        // Media
        doc.merge(server_api_media::ApiDocMedia::openapi());
        // Profile
        doc.merge(server_api_profile::ApiDocProfile::openapi());
        // Chat
        doc.merge(server_api_chat::ApiDocChat::openapi());
        doc
    }

    pub fn open_api_json_string() -> Result<String, serde_json::Error> {
        Self::all().to_pretty_json()
    }
}
