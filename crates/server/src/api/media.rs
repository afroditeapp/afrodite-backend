use axum::{
    extract::{BodyStream, Path, Query, State},
    Extension, TypedHeader,
};
use headers::ContentType;
use model::{
    AccountId, AccountIdInternal, ContentId, ImageAccessCheck, ImageSlot, MapTileX, MapTileY,
    MapTileZ, ModerationRequest, ModerationRequestContent, NormalImages,
    PrimaryImage, SlotId,
};
use simple_backend::app::GetTileMap;
use tracing::error;

use super::{
    super::app::{GetAccessTokens, GetAccounts, ReadData, WriteData},
    db_write,
    utils::{Json, StatusCode},
};
use crate::{
    data::{
        write_concurrent::{ConcurrentWriteAction, ConcurrentWriteImageHandle},
        DataError,
    },
    perf::MEDIA,
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
    State(state): State<S>,
    Path(account_id): Path<AccountId>,
    Path(content_id): Path<ContentId>,
    Query(_access_check): Query<ImageAccessCheck>,
) -> Result<(TypedHeader<ContentType>, Vec<u8>), StatusCode> {
    MEDIA.get_image.incr();

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
    State(state): State<S>,
    Path(account_id): Path<AccountId>,
    Query(_access_check): Query<ImageAccessCheck>,
    Extension(_api_caller_account_id): Extension<AccountIdInternal>,
) -> Result<Json<PrimaryImage>, StatusCode> {
    MEDIA.get_primary_image_info.incr();

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
    State(state): State<S>,
    Path(account_id): Path<AccountId>,
    Extension(_api_caller_account_id): Extension<AccountIdInternal>,
) -> Result<Json<NormalImages>, StatusCode> {
    MEDIA.get_all_normal_images.incr();

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
            if !m.secure_capture {
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
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Json(new_image): Json<PrimaryImage>,
) -> Result<(), StatusCode> {
    MEDIA.put_primary_image.incr();

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
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<ModerationRequest>, StatusCode> {
    MEDIA.get_moderation_request.incr();

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
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(moderation_request): Json<ModerationRequestContent>,
) -> Result<(), StatusCode> {
    MEDIA.put_moderation_request.incr();

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
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Path(slot_number): Path<SlotId>,
    image: BodyStream,
) -> Result<Json<ContentId>, StatusCode> {
    MEDIA.put_image_to_moderation_slot.incr();

    let slot = match slot_number.slot_id {
        0 => ImageSlot::Image1,
        1 => ImageSlot::Image2,
        2 => ImageSlot::Image3,
        _ => return Err(StatusCode::NOT_ACCEPTABLE),
    };

    let content_id = state
        .write_concurrent(account_id.as_id(), move |cmds| async move {
            let out: ConcurrentWriteAction<error_stack::Result<_, DataError>> = cmds
                .accquire_image(move |cmds: ConcurrentWriteImageHandle| {
                    Box::new(async move { cmds.save_to_tmp(account_id, image).await })
                })
                .await;
            out
        })
        .await??;

    state
        .write(move |cmds| async move {
            cmds.media()
                .save_to_slot(account_id, content_id, slot)
                .await
        })
        .await?;

    Ok(content_id.into())
}

pub const PATH_GET_MAP_TILE: &str = "/media_api/map_tile/:z/:x/:y";

/// Get map tile PNG file.
///
/// Returns a .png even if the URL does not have it.
#[utoipa::path(
    get,
    path = "/media_api/map_tile/{z}/{x}/{y}",
    params(MapTileZ, MapTileX, MapTileY),
    responses(
        (status = 200, description = "Get map tile PNG file.", body = Vec<u8>, content_type = "image/png"),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn get_map_tile<S: GetTileMap>(
    State(state): State<S>,
    Path(z): Path<MapTileZ>,
    Path(x): Path<MapTileX>,
    Path(y): Path<MapTileY>,
) -> Result<(TypedHeader<ContentType>, Vec<u8>), StatusCode> {
    MEDIA.get_map_tile.incr();

    let y_string = y.y.trim_end_matches(".png");
    let y = y_string
        .parse::<u32>()
        .map_err(|_| StatusCode::NOT_ACCEPTABLE)?;

    let data = state
        .tile_map()
        .load_map_tile(z.z, x.x, y)
        .await
        .map_err(|e| {
            error!("{:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    match data {
        Some(data) => Ok((TypedHeader(ContentType::png()), data)),
        None => Err(StatusCode::NOT_FOUND),
    }
}
