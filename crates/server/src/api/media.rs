use axum::{
    extract::{BodyStream, Path, Query},
    Extension, TypedHeader,
};
use headers::ContentType;
use model::{
    AccountId, AccountIdInternal, ContentId, ImageAccessCheck, ImageSlot, MediaContentType,
    ModerationRequest, ModerationRequestContent, NormalImages, PrimaryImage, SlotId,
};
use tracing::error;

use super::{
    db_write,
    utils::{Json, StatusCode},
    GetAccessTokens, GetAccounts, ReadData, WriteData,
};

pub const PATH_GET_IMAGE: &str = "/media_api/image/:account_id/:content_id";

// TODO:
//       Security image should only be downloadable for the owner of the image
//       or admin with moderation rights.

/// Get profile image
#[utoipa::path(
    get,
    path = "/media_api/image/{account_id}/{content_id}",
    params(AccountId, ContentId, ImageAccessCheck),
    responses(
        (status = 200, description = "Get image file.", body = Vec<u8>, content_type = "image/jpeg"),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn get_image<S: ReadData>(
    Path(account_id): Path<AccountId>,
    Path(content_id): Path<ContentId>,
    Query(_access_check): Query<ImageAccessCheck>,
    state: S,
) -> Result<(TypedHeader<ContentType>, Vec<u8>), StatusCode> {
    // TODO: Add access restrictions.

    // TODO: Change to use stream when error handling is improved in future axum
    // version. Or check will the connection be closed if there is an error. And
    // set content lenght? Or use ServeFile service from tower middleware.

    let data = state
        .read()
        .media()
        .image(account_id, content_id)
        .await
        .map_err(|e| {
            error!("{:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok((TypedHeader(ContentType::jpeg()), data))
}

pub const PATH_GET_PRIMARY_IMAGE_INFO: &str = "/media_api/primary_image_info/:account_id";

/// Get current public image for selected profile
#[utoipa::path(
    get,
    path = "/media_api/primary_image_info/{account_id}",
    params(AccountId, ImageAccessCheck),
    responses(
        (status = 200, description = "Get primary image info.", body = PrimaryImage),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn get_primary_image_info<S: ReadData + GetAccounts + GetAccessTokens>(
    Path(account_id): Path<AccountId>,
    Query(_access_check): Query<ImageAccessCheck>,
    Extension(_api_caller_account_id): Extension<AccountIdInternal>,
    state: S,
) -> Result<Json<PrimaryImage>, StatusCode> {
    // TODO: access restrictions

    let internal_id = state
        .accounts()
        .get_internal_id(account_id)
        .await
        .map_err(|e| {
            error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let internal_current_media = state
        .read()
        .media()
        .current_account_media(internal_id)
        .await
        .map_err(|e| {
            error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let info: PrimaryImage = internal_current_media.into();
    Ok(info.into())
}

pub const PATH_GET_ALL_NORMAL_IMAGES_INFO: &str = "/media_api/all_normal_images_info/:account_id";

/// Get list of all normal images on the server for one account.
#[utoipa::path(
    get,
    path = "/media_api/all_normal_images/{account_id}",
    params(AccountId),
    responses(
        (status = 200, description = "Get list of available primary images.", body = NormalImages),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn get_all_normal_images<S: ReadData + GetAccounts>(
    Path(account_id): Path<AccountId>,
    Extension(_api_caller_account_id): Extension<AccountIdInternal>,
    state: S,
) -> Result<Json<NormalImages>, StatusCode> {
    // TODO: access restrictions

    let internal_id = state
        .accounts()
        .get_internal_id(account_id)
        .await
        .map_err(|e| {
            error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let internal_current_media =
        state
            .read()
            .all_account_media(internal_id)
            .await
            .map_err(|e| {
                error!("{e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
    let data = internal_current_media
        .into_iter()
        .filter_map(|m| {
            if m.content_type == MediaContentType::Normal {
                Some(m.content_id.as_content_id())
            } else {
                None
            }
        })
        .collect();

    Ok(NormalImages { data }.into())
}

pub const PATH_PUT_PRIMARY_IMAGE: &str = "/media_api/primary_image";

/// Set primary image for account. Image content ID can not be empty.
#[utoipa::path(
    put,
    path = "/media_api/primary_image",
    request_body(content = PrimaryImage),
    responses(
        (status = 200, description = "Primary image update successfull"),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn put_primary_image<S: WriteData>(
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Json(new_image): Json<PrimaryImage>,
    state: S,
) -> Result<(), StatusCode> {
    if new_image.content_id.is_none() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    db_write!(state, move |cmds| cmds
        .media()
        .update_primary_image(api_caller_account_id, new_image))
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
    security(("access_token" = [])),
)]
pub async fn get_moderation_request<S: ReadData + GetAccessTokens>(
    Extension(account_id): Extension<AccountIdInternal>,
    state: S,
) -> Result<Json<ModerationRequest>, StatusCode> {
    let request = state
        .read()
        .moderation_request(account_id)
        .await?
        .ok_or(StatusCode::NOT_MODIFIED)?;

    Ok(request.into())
}

/// Create new or override old moderation request.
///
/// Make sure that moderation request has content IDs which points to your own
/// image slots.
///
#[utoipa::path(
    put,
    path = "/media_api/moderation/request",
    request_body(content = ModerationRequestContent),
    responses(
        (status = 200, description = "Sending or updating new image moderation request was successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error or request content was invalid."),
    ),
    security(("access_token" = [])),
)]
pub async fn put_moderation_request<S: WriteData + GetAccessTokens>(
    Extension(account_id): Extension<AccountIdInternal>,
    Json(moderation_request): Json<ModerationRequestContent>,
    state: S,
) -> Result<(), StatusCode> {
    db_write!(state, move |cmds| {
        cmds.media()
            .set_moderation_request(account_id, moderation_request)
    })
}

pub const PATH_MODERATION_REQUEST_SLOT: &str = "/media_api/moderation/request/slot/:slot_id";

/// Set image to moderation request slot.
///
/// Slots from 0 to 2 are available.
///
/// TODO: resize and check images at some point
///
#[utoipa::path(
    put,
    path = "/media_api/moderation/request/slot/{slot_id}",
    params(SlotId),
    request_body(content = Vec<u8>, content_type = "image/jpeg"),
    responses(
        (status = 200, description = "Sending or updating new image moderation request was successfull.", body = ContentId),
        (status = 401, description = "Unauthorized."),
        (status = 406, description = "Unknown slot ID."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn put_image_to_moderation_slot<S: GetAccessTokens + WriteData>(
    Extension(account_id): Extension<AccountIdInternal>,
    Path(slot_number): Path<SlotId>,
    image: BodyStream,
    state: S,
) -> Result<Json<ContentId>, StatusCode> {
    let slot = match slot_number.slot_id {
        0 => ImageSlot::Image1,
        1 => ImageSlot::Image2,
        2 => ImageSlot::Image3,
        _ => return Err(StatusCode::NOT_ACCEPTABLE),
    };

    let content_id = state
        .write_concurrent(account_id.as_id(), move |cmds| async move {
            cmds.save_to_tmp(account_id, image).await
        })
        .await
        .map_err(|e| {
            error!("Error: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    state
        .write(move |cmds| async move {
            cmds.media()
                .save_to_slot(account_id, content_id, slot)
                .await
        })
        .await
        .map_err(|e| {
            error!("Error: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(content_id.into())
}
