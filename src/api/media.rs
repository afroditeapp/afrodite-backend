pub mod data;
pub mod internal;

use axum::{Json, TypedHeader};
use axum::body::Bytes;
use axum::extract::Path;

use hyper::StatusCode;

use crate::server::database::file::file::ImageSlot;

use self::super::model::SlotId;

use self::data::{ImageFileName, NewModerationRequest, ModerationRequestList};

use super::utils::ApiKeyHeader;
use super::{ReadDatabase, GetApiKeys};
use super::model::AccountIdLight;

pub const PATH_GET_IMAGE: &str = "/media_api/image/:account_id/:image_file";

/// Get profile image
#[utoipa::path(
    get,
    path = "/media_api/image/{account_id}/{image_file}",
    params(AccountIdLight, ImageFileName),
    responses(
        (status = 200, description = "Get image file.", content_type = "image/jpeg"),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("api_key" = [])),
)]
pub async fn get_image<S: ReadDatabase>(
    Path(_account_id): Path<AccountIdLight>,
    Path(_image_file): Path<ImageFileName>,
    _state: S,
) -> Result<(), StatusCode> {

    Ok(())
}

pub const PATH_MODERATION_REQUEST: &str = "/media_api/moderation/request";

/// Get current moderation request.
///
#[utoipa::path(
    get,
    path = "/media_api/moderation/request",
    responses(
        (status = 200, description = "Get moderation request was successfull.", body = ModerationRequest),
        (status = 304, description = "No moderation request found."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("api_key" = [])),
)]
pub async fn get_moderation_request<S: ReadDatabase>(
    Json(moderation_request): Json<NewModerationRequest>,
    _state: S,
) -> Result<(), StatusCode> {
    Err(StatusCode::NOT_MODIFIED)
}

/// Create new or override old moderation request.
///
/// Set images to moderation request slots first.
///
#[utoipa::path(
    put,
    path = "/media_api/moderation/request",
    request_body(content = NewModerationRequest),
    responses(
        (status = 200, description = "Sending or updating new image moderation request was successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 406, description = "Images not found in the slots defined in the request."),
        (status = 500, description = "Internal server error."),
    ),
    security(("api_key" = [])),
)]
pub async fn put_moderation_request<S: ReadDatabase>(
    Json(moderation_request): Json<NewModerationRequest>,
    _state: S,
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
    Err(StatusCode::NOT_ACCEPTABLE)
}

pub const PATH_MODERATION_REQUEST_SLOT: &str = "/media_api/moderation/request/slot/:slot_id";

/// Set image to moderation request slot.
///
/// Slots "camera" and "image1" are available.
///
#[utoipa::path(
    put,
    path = "/media_api/moderation/request/slot/{slot_id}",
    request_body(content = String, content_type = "image/jpeg"),
    responses(
        (status = 200, description = "Sending or updating new image moderation request was successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 406, description = "Unknown slot ID."),
        (status = 500, description = "Internal server error."),
    ),
    security(("api_key" = [])),
)]
pub async fn put_image_to_moderation_slot<S: GetApiKeys>(
    TypedHeader(api_key): TypedHeader<ApiKeyHeader>,
    Path(slot_id): Path<String>,
    image: Bytes,
    state: S,
) -> Result<(), StatusCode> {
    let account_id = state
        .api_keys()
        .api_key_exists(api_key.key())
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let slot = match slot_id.as_str() {
        "slot1" => ImageSlot::Image1,
        "slot2" => ImageSlot::Image2,
        "slot3" => ImageSlot::Image3,
        _ => return Err(StatusCode::NOT_ACCEPTABLE),
    };


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


pub const PATH_ADMIN_MODERATION_PAGE_NEXT: &str =
    "/media_api/admin/moderation/page/next";

/// Get list of next moderation requests in moderation queue.
///
/// ## Access
///
/// Account with `admin_moderate_images` capability is required to access this
/// route.
///
#[utoipa::path(
    get,
    path = "/media_api/admin/moderation/page/next",
    responses(
        (status = 200, description = "Get moderation request list was successfull.", body = ModerationRequestList),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("api_key" = [])),
)]
pub async fn get_moderation_request_list<S: ReadDatabase>(
    _state: S,
) -> Result<Json<ModerationRequestList>, StatusCode> {
    Err(StatusCode::NOT_MODIFIED)
}


pub const PATH_ADMIN_MODERATION_HANDLE_REQUEST: &str =
    "/media_api/admin/moderation/handle_request/:request_id";

/// Handle moderation request.
///
/// ## Access
///
/// Account with `admin_moderate_images` capability is required to access this
/// route.
///
#[utoipa::path(
    post,
    path = "/media_api/admin/moderation/handle_request/{request_id}",
    request_body(content = HandleModerationRequest),
    responses(
        (status = 200, description = "Handling moderation request was successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 404, description = "Request ID does not exists."),
        (status = 406, description = "Already handled."),
        (status = 500, description = "Internal server error."),
    ),
    security(("api_key" = [])),
)]
pub async fn post_handle_moderation_request<S: ReadDatabase>(
    Path(request_id): Path<uuid::Uuid>,
    Json(moderation_request): Json<NewModerationRequest>,
    _state: S,
) -> Result<(), StatusCode> {
    Err(StatusCode::NOT_ACCEPTABLE)
}
