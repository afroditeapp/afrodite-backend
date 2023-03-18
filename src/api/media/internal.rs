//! Handlers for internal from Server to Server state transfers and messages

use axum::{
    extract::{BodyStream, Path},
    TypedHeader,
};
use headers::{ContentLength, ContentType};
use hyper::StatusCode;

use crate::api::model::AccountIdInternal;

use super::super::account::data::AccountId;

use super::data::ImageFileName;

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
    Path(_id): Path<AccountIdInternal>,
    Path(_image_file): Path<ImageFileName>,
    TypedHeader(_content_type): TypedHeader<ContentType>,
    TypedHeader(_content_lenght): TypedHeader<ContentLength>,
    _image_bytes: BodyStream,
    _state: S,
) -> Result<StatusCode, StatusCode> {
    // TODO
    Ok(StatusCode::OK)
}

// TODO: Post image handler, setup internal server, implement database image
// reading and writing.
