use std::fmt::Write;

use axum::{
    extract::{BodyStream, Path, Query, State},
    Extension, TypedHeader,
};
use headers::ContentType;
use model::{
    AccountId, AccountIdInternal, ContentId, ContentAccessCheck, ContentSlot, MapTileX, MapTileY,
    MapTileZ, ModerationRequest, ModerationRequestContent, AccountContent,
    ProfileContent, SlotId, NewContentParams, ContentProcessingId, ContentProcessingState, SetProfileContent, PendingProfileContent, SecurityImage, PendingSecurityImage,
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
        write_concurrent::{ConcurrentWriteAction, ConcurrentWriteContentHandle},
        DataError,
    },
    perf::MEDIA, app::ContentProcessingProvider,
};

pub const PATH_GET_CONTENT: &str = "/media_api/content/:account_id/:content_id";

/// Get content data
#[utoipa::path(
    get,
    path = "/media_api/content/{account_id}/{content_id}",
    params(AccountId, ContentId, ContentAccessCheck),
    responses(
        (status = 200, description = "Get content file.", body = Vec<u8>, content_type = "application/octet-stream"),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn get_content<S: ReadData>(
    State(state): State<S>,
    Path(account_id): Path<AccountId>,
    Path(content_id): Path<ContentId>,
    Query(_access_check): Query<ContentAccessCheck>,
) -> Result<(TypedHeader<ContentType>, Vec<u8>), StatusCode> {
    MEDIA.get_content.incr();

    // TODO: Add access restrictions.

    // TODO: Change to use stream when error handling is improved in future axum
    // version. Or check will the connection be closed if there is an error. And
    // set content lenght? Or use ServeFile service from tower middleware.

    let data = state
        .read()
        .media()
        .content_data(account_id, content_id)
        .await?;

    Ok((TypedHeader(ContentType::octet_stream()), data))
}

pub const PATH_GET_PROFILE_CONTENT_INFO: &str = "/media_api/profile_content_info/:account_id";

/// Get current profile content for selected profile
#[utoipa::path(
    get,
    path = "/media_api/profile_content_info/{account_id}",
    params(AccountId, ContentAccessCheck),
    responses(
        (status = 200, description = "Get profile content info.", body = ProfileContent),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn get_profile_content_info<S: ReadData + GetAccounts>(
    State(state): State<S>,
    Path(account_id): Path<AccountId>,
    Query(_access_check): Query<ContentAccessCheck>,
    Extension(_api_caller_account_id): Extension<AccountIdInternal>,
) -> Result<Json<ProfileContent>, StatusCode> {
    MEDIA.get_profile_content_info.incr();

    // TODO: access restrictions

    let internal_id = state
        .accounts()
        .get_internal_id(account_id)
        .await?;

    let internal_current_media = state
        .read()
        .media()
        .current_account_media(internal_id)
        .await?;

    let info: ProfileContent = internal_current_media.into();
    Ok(info.into())
}

pub const PATH_GET_ALL_ACCOUNT_MEDIA_CONTENT: &str = "/media_api/all_account_media_content/:account_id";

/// Get list of all media content on the server for one account.
#[utoipa::path(
    get,
    path = "/media_api/all_account_media_content/{account_id}",
    params(AccountId),
    responses(
        (status = 200, description = "Successful.", body = AccountContent),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn get_all_account_media_content<S: ReadData + GetAccounts>(
    State(state): State<S>,
    Path(account_id): Path<AccountId>,
    Extension(_api_caller_account_id): Extension<AccountIdInternal>,
) -> Result<Json<AccountContent>, StatusCode> {
    MEDIA.get_all_account_media_content.incr();

    // TODO: access restrictions

    let internal_id = state
        .accounts()
        .get_internal_id(account_id)
        .await?;

    let internal_current_media =
        state
            .read()
            .all_account_media_content(internal_id)
            .await?;

    let data = internal_current_media
        .into_iter()
        .map(|m| m.into())
        .collect();

    Ok(AccountContent { data }.into())
}

pub const PATH_PUT_PROFILE_CONTENT: &str = "/media_api/profile_content";

/// Set new profile content for current account.
///
/// # Restrictions
/// - All content must be moderated as accepted.
/// - All content must be owned by the account.
/// - All content must be images.
#[utoipa::path(
    put,
    path = "/media_api/profile_content",
    request_body(content = SetProfileContent),
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn put_profile_content<S: WriteData>(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Json(new): Json<SetProfileContent>,
) -> Result<(), StatusCode> {
    MEDIA.put_profile_content.incr();

    db_write!(state, move |cmds| cmds
        .media()
        .update_profile_content(api_caller_account_id, new))
}

pub const PATH_GET_PENDING_PROFILE_CONTENT_INFO: &str = "/media_api/pending_profile_content_info/:account_id";

/// Get pending profile content for selected profile
#[utoipa::path(
    get,
    path = "/media_api/pending_profile_content_info/{account_id}",
    params(AccountId),
    responses(
        (status = 200, description = "Successful.", body = PendingProfileContent),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn get_pending_profile_content_info<S: ReadData + GetAccounts>(
    State(state): State<S>,
    Path(account_id): Path<AccountId>,
    Extension(_api_caller_account_id): Extension<AccountIdInternal>,
) -> Result<Json<PendingProfileContent>, StatusCode> {
    MEDIA.get_pending_profile_content_info.incr();

    // TODO: access restrictions

    let internal_id = state
        .accounts()
        .get_internal_id(account_id)
        .await?;

    let internal_current_media = state
        .read()
        .media()
        .current_account_media(internal_id)
        .await?;

    let info: PendingProfileContent = internal_current_media.into();
    Ok(info.into())
}

pub const PATH_PUT_PENDING_PROFILE_CONTENT: &str = "/media_api/pending_profile_content";

/// Set new pending profile content for current account.
/// Server will switch to pending content when next moderation request is
/// accepted.
///
/// # Restrictions
/// - All content must not be moderated as denied.
/// - All content must be owned by the account.
/// - All content must be images.
#[utoipa::path(
    put,
    path = "/media_api/pending_profile_content",
    request_body(content = SetProfileContent),
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn put_pending_profile_content<S: WriteData>(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Json(new): Json<SetProfileContent>,
) -> Result<(), StatusCode> {
    MEDIA.put_pending_profile_content.incr();

    db_write!(state, move |cmds| cmds
        .media()
        .update_or_delete_pending_profile_content(api_caller_account_id, Some(new)))
}

pub const PATH_DELETE_PENDING_PROFILE_CONTENT: &str = "/media_api/pending_profile_content";

/// Delete new pending profile content for current account.
/// Server will not switch to pending content when next moderation request is
/// accepted.
#[utoipa::path(
    delete,
    path = "/media_api/pending_profile_content",
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn delete_pending_profile_content<S: WriteData>(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
) -> Result<(), StatusCode> {
    MEDIA.delete_pending_profile_content.incr();

    db_write!(state, move |cmds| cmds
        .media()
        .update_or_delete_pending_profile_content(api_caller_account_id, None))
}

pub const PATH_GET_SECURITY_IMAGE_INFO: &str = "/media_api/security_image_info/:account_id";

/// Get current security image for selected profile.
#[utoipa::path(
    get,
    path = "/media_api/security_image_info/{account_id}",
    params(AccountId),
    responses(
        (status = 200, description = "Successful.", body = SecurityImage),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn get_security_image_info<S: ReadData + GetAccounts>(
    State(state): State<S>,
    Path(requested_account_id): Path<AccountId>,
    Extension(_api_caller_account_id): Extension<AccountIdInternal>,
) -> Result<Json<SecurityImage>, StatusCode> {
    MEDIA.get_security_image_info.incr();

    // TODO: access restrictions

    let internal_id = state
        .accounts()
        .get_internal_id(requested_account_id)
        .await?;

    let internal_current_media = state
        .read()
        .media()
        .current_account_media(internal_id)
        .await?;

    let info: SecurityImage = internal_current_media.into();
    Ok(info.into())
}

pub const PATH_GET_PENDING_SECURITY_IMAGE_INFO: &str = "/media_api/pending_security_image_info/:account_id";

/// Get pending security image for selected profile.
#[utoipa::path(
    get,
    path = "/media_api/pending_security_image_info/{account_id}",
    params(AccountId),
    responses(
        (status = 200, description = "Successful.", body = PendingSecurityImage),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn get_pending_security_image_info<S: ReadData + GetAccounts>(
    State(state): State<S>,
    Path(requested_account_id): Path<AccountId>,
    Extension(_api_caller_account_id): Extension<AccountIdInternal>,
) -> Result<Json<PendingSecurityImage>, StatusCode> {
    MEDIA.get_pending_security_image_info.incr();

    // TODO: access restrictions

    let internal_id = state
        .accounts()
        .get_internal_id(requested_account_id)
        .await?;

    let internal_current_media = state
        .read()
        .media()
        .current_account_media(internal_id)
        .await?;

    let info: PendingSecurityImage = internal_current_media.into();
    Ok(info.into())
}

pub const PATH_PUT_SECURITY_IMAGE_INFO: &str = "/media_api/security_image_info";

/// Set current security image content for current account.
///
/// # Restrictions
/// - The content must be moderated as accepted.
/// - The content must be owned by the account.
/// - The content must be an image.
/// - The content must be captured by client.
#[utoipa::path(
    put,
    path = "/media_api/security_image_info",
    request_body = ContentId,
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn put_security_image_info<S: WriteData>(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Json(content_id): Json<ContentId>,
) -> Result<(), StatusCode> {
    MEDIA.put_security_image_info.incr();

    db_write!(state, move |cmds| cmds
        .media()
        .update_security_image(api_caller_account_id, content_id))
}

pub const PATH_PUT_PENDING_SECURITY_IMAGE_INFO: &str = "/media_api/pending_security_image_info";

/// Set pending security image for current account.
#[utoipa::path(
    put,
    path = "/media_api/pending_security_image_info",
    request_body = ContentId,
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn put_pending_security_image_info<S: WriteData>(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Json(content_id): Json<ContentId>,
) -> Result<(), StatusCode> {
    MEDIA.put_pending_security_image_info.incr();

    db_write!(state, move |cmds| cmds
        .media()
        .update_or_delete_pending_security_image(api_caller_account_id, Some(content_id)))
}

pub const DELETE_PENDING_SECURITY_IMAGE_INFO: &str = "/media_api/pending_security_image_info";

/// Delete pending security image for current account.
/// Server will not change the security image when next moderation request
/// is moderated as accepted.
#[utoipa::path(
    delete,
    path = "/media_api/pending_security_image_info",
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn delete_pending_security_image_info<S: WriteData>(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
) -> Result<(), StatusCode> {
    MEDIA.put_pending_security_image_info.incr();

    db_write!(state, move |cmds| cmds
        .media()
        .update_or_delete_pending_security_image(api_caller_account_id, None))
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
pub async fn get_moderation_request<S: ReadData>(
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
pub async fn put_moderation_request<S: WriteData>(
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

/// Delete current moderation request which is not yet in moderation.
#[utoipa::path(
    delete,
    path = "/media_api/moderation/request",
    responses(
        (status = 200, description = "Successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn delete_moderation_request<S: WriteData>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<(), StatusCode> {
    MEDIA.delete_moderation_request.incr();

    db_write!(state, move |cmds| {
        cmds.media()
            .delete_moderation_request_if_possible(account_id)
    })
}

pub const PATH_PUT_CONTENT_TO_CONTENT_SLOT: &str = "/media_api/content_slot/:slot_id";

/// Set content to content processing slot.
/// Processing ID will be returned and processing of the content
/// will begin.
/// Events about the content processing will be sent to the client.
///
/// The state of the processing can be also queired. The querying is
/// required to receive the content ID.
///
/// Slots from 0 to 6 are available.
///
/// One account can only have one content in upload or processing state.
/// New upload might potentially delete the previous if processing of it is
/// not complete.
///
#[utoipa::path(
    put,
    path = "/media_api/content_slot/{slot_id}",
    params(SlotId, NewContentParams),
    request_body(content = Vec<u8>, content_type = "image/jpeg"),
    responses(
        (status = 200, description = "Image upload was successful.", body = ContentProcessingId),
        (status = 401, description = "Unauthorized."),
        (status = 406, description = "Unknown slot ID."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn put_content_to_content_slot<S: WriteData + ContentProcessingProvider>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Path(slot_number): Path<SlotId>,
    Query(new_content_params): Query<NewContentParams>,
    content_data: BodyStream,
) -> Result<Json<ContentProcessingId>, StatusCode> {
    MEDIA.put_content_to_content_slot.incr();

    let slot = TryInto::<ContentSlot>::try_into(slot_number.slot_id as i64)
        .map_err(|_| StatusCode::NOT_ACCEPTABLE)?;

    let content_info = state
        .write_concurrent(account_id.as_id(), move |cmds| async move {
            let out: ConcurrentWriteAction<error_stack::Result<_, DataError>> = cmds
                .accquire_image(move |cmds: ConcurrentWriteContentHandle| {
                    Box::new(async move { cmds.save_to_tmp(account_id, content_data).await })
                })
                .await;
            out
        })
        .await??;

    state.content_processing().queue_new_content(
        account_id,
        slot,
        content_info.clone(),
        new_content_params
    ).await;

    Ok(content_info.processing_id.into())
}

pub const PATH_GET_CONTENT_SLOT_STATE: &str = "/media_api/content_slot/:slot_id";

/// Get state of content slot.
///
/// Slots from 0 to 6 are available.
///
#[utoipa::path(
    get,
    path = "/media_api/content_slot/{slot_id}",
    params(SlotId),
    responses(
        (status = 200, description = "Successful.", body = ContentProcessingState),
        (status = 401, description = "Unauthorized."),
        (status = 406, description = "Unknown slot ID."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_content_slot_state<S: ContentProcessingProvider>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Path(slot_number): Path<SlotId>,
) -> Result<Json<ContentProcessingState>, StatusCode> {
    MEDIA.get_content_slot_state.incr();

    let slot = TryInto::<ContentSlot>::try_into(slot_number.slot_id as i64)
        .map_err(|_| StatusCode::NOT_ACCEPTABLE)?;

    if let Some(state) = state.content_processing().get_state(account_id, slot).await {
        Ok(state.into())
    } else {
        Ok(ContentProcessingState::empty().into())
    }
}

pub const PATH_DELETE_CONTENT: &str = "/media_api/content/:account_id/:content_id";

/// Delete content data. Content can be removed after specific time has passed
/// since removing all usage from it (content is not a security image or profile
/// content).
#[utoipa::path(
    delete,
    path = "/media_api/content/{account_id}/{content_id}",
    params(AccountId, ContentId),
    responses(
        (status = 200, description = "Content data deleted."),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn delete_content<S: WriteData + GetAccounts>(
    State(state): State<S>,
    Path(account_id): Path<AccountId>,
    Path(content_id): Path<ContentId>,
) -> Result<(), StatusCode> {
    MEDIA.delete_content.incr();

    // TODO: Add access restrictions.

    // TODO: Add database support for keeping track of content usage.

    let internal_id = state
        .accounts()
        .get_internal_id(account_id)
        .await?;

    db_write!(state, move |cmds| cmds
        .media()
        .delete_content(internal_id, content_id))
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
