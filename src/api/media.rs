
pub mod image;
pub mod internal;

use axum::{extract::Path, middleware::Next, response::Response, Json, TypedHeader};
use headers::{Header, HeaderValue};
use hyper::{header, Request, StatusCode};
use tokio::sync::Mutex;
use utoipa::{
    openapi::security::{ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};

use crate::server::session::UserState;

use self::{
    super::core::profile::Profile,
    super::core::user::{ApiKey, UserId},
    super::core::SecurityApiTokenDefault,
};

use self::{
    image::ImageFileName,
};

use tracing::error;

use super::{db_write, GetApiKeys, GetRouterDatabaseHandle, GetUsers, ReadDatabase, WriteDatabase, core::{ApiKeyHeader, API_KEY_HEADER_STR}, GetCoreServerInternalApi};

#[derive(OpenApi)]
#[openapi(
    paths(get_image),
    components(schemas(
        super::core::user::UserId,
        super::core::user::ApiKey,
        image::ImageFileName,
    )),
    modifiers(&SecurityApiTokenDefault),
)]
pub struct ApiDocMedia;

pub const PATH_GET_IMAGE: &str = "/image/:user_id/:image_file";

#[utoipa::path(
    get,
    path = "/image/{user_id}/{image_file}",
    params(UserId, ImageFileName),
    responses(
        (status = 200, description = "Get image file.", content_type = "image/jpeg"),
        (status = 500),
    ),
    security(("api_key" = [])),
)]
pub async fn get_image<S: ReadDatabase>(
    Path(user_id): Path<UserId>,
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


pub async fn authenticate_media_api<T, S: GetApiKeys + GetCoreServerInternalApi>(
    state: S,
    req: Request<T>,
    next: Next<T>,
) -> Result<Response, StatusCode> {
    let header = req
        .headers()
        .get(API_KEY_HEADER_STR)
        .ok_or(StatusCode::BAD_REQUEST)?;
    let key_str = header.to_str().map_err(|_| StatusCode::BAD_REQUEST)?;
    let key = ApiKey::new(key_str.to_string());

    if state.api_keys().read().await.contains_key(&key) {
        Ok(next.run(req).await)
    } else {
        match state.core_server_internal_api().check_api_key(key).await {
            Ok(Some(user_id)) => {
                // TODO: Cache this API key.
                Ok(next.run(req).await)
            },
            Ok(None) => Err(StatusCode::UNAUTHORIZED),
            Err(e) => {
                // TODO: It is probably not good to log this because this can
                // happen often if core server is not available.
                error!("{}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}
