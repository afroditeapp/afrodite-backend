pub mod data;
pub mod internal;

use axum::{extract::Path, middleware::Next, response::Response, Json, TypedHeader};
use headers::{Header, HeaderValue};
use hyper::{header, Request, StatusCode};
use tokio::sync::Mutex;
use utoipa::{
    openapi::security::{ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};

use crate::server::session::AccountStateInRam;

use self::{
    super::model::{ApiKey, AccountId, Profile},
};

use self::data::ImageFileName;

use tracing::error;

use super::{
    utils::{ApiKeyHeader, API_KEY_HEADER_STR},
    db_write, GetApiKeys, GetCoreServerInternalApi, GetRouterDatabaseHandle, GetUsers,
    ReadDatabase, WriteDatabase,
};

pub const PATH_GET_IMAGE: &str = "/image/:user_id/:image_file";

#[utoipa::path(
    get,
    path = "/image/{user_id}/{image_file}",
    params(AccountId, ImageFileName),
    responses(
        (status = 200, description = "Get image file.", content_type = "image/jpeg"),
        (status = 500),
    ),
    security(("api_key" = [])),
)]
pub async fn get_image<S: ReadDatabase>(
    Path(user_id): Path<AccountId>,
    Path(image_file): Path<ImageFileName>,
    state: S,
) -> Result<(), StatusCode> {
    // TODO: Validate user id
    // state
    //     .read_database()
    //     .user_profile(&user_id)
    //     .await
    //     .map(|profile| ()) // TODO: Read and send image.
    //     .map_err(|e| {
    //         error!("Get profile error: {e:?}");
    //         StatusCode::INTERNAL_SERVER_ERROR // Database reading failed.
    //     })
    Ok(())
}
