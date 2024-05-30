use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(info(
    title = "pihka-backend",
    description = "Pihka backend API",
    version = "0.1.0"
))]
pub struct ApiDoc;

impl ApiDoc {
    pub fn all() -> utoipa::openapi::OpenApi {
        let mut doc = ApiDoc::openapi();
        doc.merge(server_api::ApiDocCommon::openapi());
        doc.merge(server_api_all::ApiDocConnection::openapi());
        doc.merge(server_api_account::ApiDocAccount::openapi());
        doc.merge(server_api_media::ApiDocMedia::openapi());
        doc.merge(server_api_profile::ApiDocProfile::openapi());
        doc.merge(server_api_chat::ApiDocChat::openapi());
        doc
    }

    pub fn open_api_json_string() -> Result<String, serde_json::Error> {
        Self::all().to_pretty_json()
    }
}
