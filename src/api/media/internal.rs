//! Handlers for internal from Server to Server state transfers and messages


use axum::{extract::{Path, BodyStream}, middleware::Next, response::Response, Json, TypedHeader, body::Bytes};
use headers::{Header, HeaderValue, ContentType, ContentLength};
use hyper::{header, Request, StatusCode};
use tokio::sync::Mutex;
use utoipa::{
    openapi::security::{ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};

use crate::server::session::UserState;

use super::{
    super::core::profile::Profile,
    super::core::user::{ApiKey, UserId},
    super::core::SecurityApiTokenDefault,
};

use super::{
    image::ImageFileName,
};

use tracing::error;

use super::{db_write, GetApiKeys, GetRouterDatabaseHandle, GetUsers, ReadDatabase, WriteDatabase, super::core::ApiKeyHeader};

#[derive(OpenApi)]
#[openapi(
    paths(post_image),
    components(schemas(
        super::super::core::user::ApiKey,
        super::super::core::user::UserId,
        super::image::ImageFile,
    )),
)]
pub struct ApiDocMediaInternal;

pub const PATH_POST_IMAGE: &str = "/image/:user_id/:image_name";

#[utoipa::path(
    post,
    path = "/image/{user_id}/{image_name}",
    request_body(content = ImageFile, description = "Upload new image", content_type = "image/jpeg"),
    responses(
        (status = 200, description = "Image upload successfull"),
        (status = 500),
    ),
)]
pub async fn post_image<S>(
    TypedHeader(content_type): TypedHeader<ContentType>,
    TypedHeader(content_lenght): TypedHeader<ContentLength>,
    image_bytes: BodyStream,
    state: S,
) -> Result<StatusCode, StatusCode> {
    Ok(StatusCode::OK)
}

// TODO: Post image handler, setup internal server, implement database image
// reading and writing.
