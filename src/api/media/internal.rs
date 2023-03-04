//! Handlers for internal from Server to Server state transfers and messages

use axum::{
    body::Bytes,
    extract::{BodyStream, Path},
    middleware::Next,
    response::Response,
    Json, TypedHeader,
};
use headers::{ContentLength, ContentType, Header, HeaderValue};
use hyper::{header, Request, StatusCode};
use tokio::sync::Mutex;
use utoipa::{
    openapi::security::{ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};

use crate::{server::session::AccountState, api::model::AccountIdLight};

use super::{
    super::profile::data::Profile,
    super::account::data::{ApiKey, AccountId},
};

use super::data::ImageFileName;

use tracing::error;

use super::{
    super::utils::ApiKeyHeader, db_write, GetApiKeys, GetRouterDatabaseHandle, GetUsers,
    ReadDatabase, WriteDatabase,
};

pub const PATH_POST_IMAGE: &str = "/internal/image/:user_id/:image_name";

#[utoipa::path(
    post,
    path = "/internal/image/{user_id}/{image_name}",
    request_body(content = ImageFile, description = "Upload new image", content_type = "image/jpeg"),
    params(AccountId, ImageFileName),
    responses(
        (status = 200, description = "Image upload successfull"),
        (status = 500),
    ),
)]
pub async fn post_image<S>(
    Path(id): Path<AccountIdLight>,
    Path(image_file): Path<ImageFileName>,
    TypedHeader(content_type): TypedHeader<ContentType>,
    TypedHeader(content_lenght): TypedHeader<ContentLength>,
    image_bytes: BodyStream,
    state: S,
) -> Result<StatusCode, StatusCode> {
    // TODO
    Ok(StatusCode::OK)
}

// TODO: Post image handler, setup internal server, implement database image
// reading and writing.
