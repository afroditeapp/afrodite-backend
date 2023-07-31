pub mod data;
pub mod internal;

use axum::extract::{BodyStream, Path, Query};

use axum::{Json, TypedHeader, Extension};

use headers::ContentType;
use hyper::StatusCode;

use tracing::error;

use crate::server::database::file::file::ImageSlot;

use self::super::model::SlotId;

use self::data::{
    ContentId, HandleModerationRequest, ModerationList, ModerationRequest, ModerationRequestContent, PrimaryImage, SecurityImage, ImageAccessCheck, NormalImages, MediaContentType,
};

use super::model::{AccountIdLight, AccountIdInternal};
use super::utils::ApiKeyHeader;
use super::{GetApiKeys, GetInternalApi, GetUsers, ReadDatabase, WriteDatabase};

pub const PATH_GET_IMAGE: &str = "/media_api/image/:account_id/:content_id";

// TODO:
//       Security image should only be downloadable for the owner of the image
//       or admin with moderation rights.

/// Get profile image
#[utoipa::path(
    get,
    path = "/media_api/image/{account_id}/{content_id}",
    params(AccountIdLight, ContentId, ImageAccessCheck),
    responses(
        (status = 200, description = "Get image file.", body = Vec<u8>, content_type = "image/jpeg"),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("api_key" = [])),
)]
pub async fn get_image<S: ReadDatabase>(
    Path(account_id): Path<AccountIdLight>,
    Path(content_id): Path<ContentId>,
    Query(access_check): Query<ImageAccessCheck>,
    state: S,
) -> Result<(TypedHeader<ContentType>, Vec<u8>), StatusCode> {
    // TODO: Add access restrictions.

    // TODO: Change to use stream when error handling is improved in future axum
    // version. Or check will the connection be closed if there is an error. And
    // set content lenght? Or use ServeFile service from tower middleware.

    let data = state
        .read_database()
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
    params(AccountIdLight, ImageAccessCheck),
    responses(
        (status = 200, description = "Get primary image info.", body = PrimaryImage),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("api_key" = [])),
)]
pub async fn get_primary_image_info<S: ReadDatabase + GetUsers + GetApiKeys>(
    Path(account_id): Path<AccountIdLight>,
    Query(access_check): Query<ImageAccessCheck>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    state: S,
) -> Result<Json<PrimaryImage>, StatusCode> {
    // TODO: access restrictions

    let internal_id = state
        .users()
        .get_internal_id(account_id)
        .await
        .map_err(|e| {
            error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let internal_current_media = state
        .read_database()
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

pub const PATH_GET_SECURITY_IMAGE_INFO: &str = "/media_api/security_image_info/:account_id";

/// Get current security image for selected profile. Only for admins.
#[utoipa::path(
    get,
    path = "/media_api/security_image_info/{account_id}",
    params(AccountIdLight),
    responses(
        (status = 200, description = "Get security image info.", body = SecurityImage),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("api_key" = [])),
)]
pub async fn get_security_image_info<S: ReadDatabase + GetUsers>(
    Path(account_id): Path<AccountIdLight>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    state: S,
) -> Result<Json<SecurityImage>, StatusCode> {

    // TODO: access restrictions

    let internal_id = state
        .users()
        .get_internal_id(account_id)
        .await
        .map_err(|e| {
            error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let internal_current_media = state
        .read_database()
        .media()
        .current_account_media(internal_id)
        .await
        .map_err(|e| {
            error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let info: SecurityImage = internal_current_media.into();
    Ok(info.into())
}

pub const PATH_GET_ALL_NORMAL_IMAGES_INFO: &str = "/media_api/all_normal_images_info/:account_id";

/// Get list of all normal images on the server for one account.
#[utoipa::path(
    get,
    path = "/media_api/all_normal_images/{account_id}",
    params(AccountIdLight),
    responses(
        (status = 200, description = "Get list of available primary images.", body = NormalImages),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("api_key" = [])),
)]
pub async fn get_all_normal_images<S: ReadDatabase + GetUsers>(
    Path(account_id): Path<AccountIdLight>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    state: S,
) -> Result<Json<NormalImages>, StatusCode> {

    // TODO: access restrictions

    let internal_id = state
        .users()
        .get_internal_id(account_id)
        .await
        .map_err(|e| {
            error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let internal_current_media = state
        .read_database()
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
    security(("api_key" = [])),
)]
pub async fn put_primary_image<S: WriteDatabase>(
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Json(new_image): Json<PrimaryImage>,
    state: S,
) -> Result<(), StatusCode> {
    if new_image.content_id.is_none() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    state
        .write_database()
        .media()
        .update_primary_image(api_caller_account_id, new_image)
        .await
        .map_err(|e| {
            error!("{}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

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
pub async fn get_moderation_request<S: ReadDatabase + GetApiKeys>(
    TypedHeader(api_key): TypedHeader<ApiKeyHeader>,
    state: S,
) -> Result<Json<ModerationRequest>, StatusCode> {
    let account_id = state
        .api_keys()
        .api_key_exists(api_key.key())
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let request = state
        .read_database()
        .moderation_request(account_id)
        .await
        .map_err(|e| {
            error!("{}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
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
    security(("api_key" = [])),
)]
pub async fn put_moderation_request<S: WriteDatabase + GetApiKeys>(
    TypedHeader(api_key): TypedHeader<ApiKeyHeader>,
    Json(moderation_request): Json<ModerationRequestContent>,
    state: S,
) -> Result<(), StatusCode> {
    let account_id = state
        .api_keys()
        .api_key_exists(api_key.key())
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    state
        .write_database()
        .media()
        .set_moderation_request(account_id, moderation_request)
        .await
        .map_err(|e| {
            error!("{:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
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
    security(("api_key" = [])),
)]
pub async fn put_image_to_moderation_slot<S: GetApiKeys + WriteDatabase>(
    TypedHeader(api_key): TypedHeader<ApiKeyHeader>,
    Path(slot_number): Path<SlotId>,
    image: BodyStream,
    state: S,
) -> Result<Json<ContentId>, StatusCode> {
    let account_id = state
        .api_keys()
        .api_key_exists(api_key.key())
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let slot = match slot_number.slot_id {
        0 => ImageSlot::Image1,
        1 => ImageSlot::Image2,
        2 => ImageSlot::Image3,
        _ => return Err(StatusCode::NOT_ACCEPTABLE),
    };

    let content_id = state
        .write_database()
        .save_to_tmp(account_id, image)
        .await
        .map_err(|e| {
            error!("Error: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    state
        .write_database()
        .media()
        .save_to_slot(account_id, content_id, slot)
        .await
        .map_err(|e| {
            error!("Error: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(content_id.into())
}

pub const PATH_ADMIN_MODERATION_PAGE_NEXT: &str = "/media_api/admin/moderation/page/next";

/// Get current list of moderation requests in my moderation queue.
/// Additional requests will be added to my queue if necessary.
///
/// ## Access
///
/// Account with `admin_moderate_images` capability is required to access this
/// route.
///
#[utoipa::path(
    patch,
    path = "/media_api/admin/moderation/page/next",
    responses(
        (status = 200, description = "Get moderation request list was successfull.", body = ModerationList),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("api_key" = [])),
)]
pub async fn patch_moderation_request_list<S: WriteDatabase + GetApiKeys>(
    TypedHeader(api_key): TypedHeader<ApiKeyHeader>,
    state: S,
) -> Result<Json<ModerationList>, StatusCode> {
    let account_id = state
        .api_keys()
        .api_key_exists(api_key.key())
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // TODO: Access restrictions

    let data = state
        .write_database()
        .media()
        .get_moderation_list_and_create_if_necessary(account_id)
        .await
        .map_err(|e| {
            error!("{}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(ModerationList { list: data }.into())
}

pub const PATH_ADMIN_MODERATION_HANDLE_REQUEST: &str =
    "/media_api/admin/moderation/handle_request/:account_id";

/// Handle moderation request of some account.
///
/// ## Access
///
/// Account with `admin_moderate_images` capability is required to access this
/// route.
///
#[utoipa::path(
    post,
    path = "/media_api/admin/moderation/handle_request/{account_id}",
    request_body(content = HandleModerationRequest),
    params(AccountIdLight),
    responses(
        (status = 200, description = "Handling moderation request was successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("api_key" = [])),
)]
pub async fn post_handle_moderation_request<
    S: GetInternalApi + WriteDatabase + GetApiKeys + GetUsers,
>(
    Path(moderation_request_owner_account_id): Path<AccountIdLight>,
    TypedHeader(api_key): TypedHeader<ApiKeyHeader>,
    Json(moderation_decision): Json<HandleModerationRequest>,
    state: S,
) -> Result<(), StatusCode> {
    let admin_account_id = state
        .api_keys()
        .api_key_exists(api_key.key())
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let account = state
        .internal_api()
        .get_account_state(admin_account_id)
        .await
        .map_err(|e| {
            error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if account.capablities().admin_moderate_images {
        let moderation_request_owner = state
            .users()
            .get_internal_id(moderation_request_owner_account_id)
            .await
            .map_err(|e| {
                error!("{e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        state
            .write_database()
            .media()
            .update_moderation(
                admin_account_id,
                moderation_request_owner,
                moderation_decision,
            )
            .await
            .map_err(|e| {
                error!("{e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
