use axum::{
    Extension,
    body::Body,
    extract::{Path, Query, State},
};
use axum_extra::TypedHeader;
use headers::{CacheControl, ContentLength, ContentType, ETag, IfNoneMatch};
use model::{
    ContentQualityVariant, EventToClientInternal, NotificationEvent, PendingAppNotificationInternal,
};
use model_media::{
    AccountContent, AccountId, AccountIdInternal, AccountState, ContentId, ContentProcessingState,
    ContentSlot, GetContentQueryParams, NewContentParams, Permissions,
    PutContentToContentSlotResult,
};
use server_api::{
    S,
    app::{ApiLimitsProvider, ApiUsageTrackerProvider, GetConfig},
    create_open_api_router, db_write,
    result::WrappedResultExt,
    utils::{IfNoneMatchExtensions, cache_control_for_images},
};
use server_data::{
    DataError,
    content_processing::ContentProcessingOngoing,
    read::GetReadCommandsCommon,
    write::GetWriteCommandsCommon,
    write_concurrent::{ConcurrentWriteAction, ConcurrentWriteContentHandle},
};
use server_data_media::{read::GetReadMediaCommands, write::GetWriteCommandsMedia};
use simple_backend::create_counters;

use crate::{
    app::{ContentProcessingProvider, GetAccounts, ReadData, WriteData},
    media::{content::quality::ContentQualityHeader, quality::ContentSendingTracker},
    utils::{Json, StatusCode},
};

pub mod quality;

const PATH_GET_CONTENT: &str = "/media_api/content/{aid}/{cid}";

/// Get content data
///
/// # Access
///
/// ## Own content
/// Unrestricted access.
///
/// ## Public other content
/// Normal account state required. Only accepted content can be accessed.
///
/// ## Private other content
/// If owner of the requested content is a match and the requested content
/// is in current profile content, then the requested content can be accessed
/// if query parameter `is_match` is set to `true`.
///
/// Only accepted content can be accessed.
///
/// ## Admin access
/// - [Permissions::admin_view_all_profiles]
/// - [Permissions::admin_moderate_media_content]
/// - [Permissions::admin_edit_media_content_face_verified_value]
/// - [Permissions::admin_edit_security_content_verified_value]
/// - [Permissions::admin_process_reports]
///
/// # Content quality
///
/// When content owner or admins requests content, high quality version
/// is returned even if lower quality version is requested.
///
/// For any other case preferred quality is used if API has not
/// too much concurrent access.
///
#[utoipa::path(
    get,
    path = PATH_GET_CONTENT,
    params(AccountId, ContentId, GetContentQueryParams),
    responses(
        (status = 200, description = "Get content file.", body = inline(model::BinaryData), content_type = "application/octet-stream"),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
#[allow(clippy::too_many_arguments)]
pub async fn get_content(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Extension(account_state): Extension<AccountState>,
    Extension(permissions): Extension<Permissions>,
    Path(requested_profile): Path<AccountId>,
    Path(requested_content_id): Path<ContentId>,
    Query(params): Query<GetContentQueryParams>,
    browser_etag: Option<TypedHeader<IfNoneMatch>>,
) -> Result<
    (
        TypedHeader<ETag>,
        TypedHeader<CacheControl>,
        TypedHeader<ContentType>,
        TypedHeader<ContentLength>,
        TypedHeader<ContentQualityHeader>,
        Body,
    ),
    StatusCode,
> {
    MEDIA.get_content.incr();
    state
        .api_usage_tracker()
        .incr(account_id, |u| &u.get_content)
        .await;

    let preferred_quality = params
        .q
        .as_deref()
        .and_then(|q| match q {
            "h" => Some(ContentQualityVariant::High),
            "m" => Some(ContentQualityVariant::Medium),
            "l" => Some(ContentQualityVariant::Low),
            _ => None,
        })
        .unwrap_or(ContentQualityVariant::High);

    if account_id.as_id() == requested_profile {
        return send_content(
            &state,
            ContentQualityVariant::High,
            requested_profile,
            requested_content_id,
            browser_etag,
        )
        .await;
    }

    let is_admin = permissions.admin_view_all_profiles
        || permissions.admin_moderate_media_content
        || permissions.admin_edit_media_content_face_verified_value
        || permissions.admin_edit_security_content_verified_value
        || permissions.admin_process_reports;

    if is_admin {
        return send_content(
            &state,
            ContentQualityVariant::High,
            requested_profile,
            requested_content_id,
            browser_etag,
        )
        .await;
    }

    if account_state != AccountState::Normal {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let requested_profile_internal_id = state.get_internal_id(requested_profile).await?;

    let visibility = state
        .read()
        .common()
        .account(requested_profile_internal_id)
        .await?
        .is_profile_visible();

    let internal = state
        .read()
        .media()
        .current_account_media(requested_profile_internal_id)
        .await?;

    let content = internal
        .iter_current_profile_content()
        .find(|c| c.content_id() == requested_content_id);
    let requested_content_is_profile_content = content.is_some();
    let content_accepted = content.map(|v| v.state().is_accepted()).unwrap_or_default();

    let access_allowed = (visibility && requested_content_is_profile_content && content_accepted)
        || (params.is_match
            && requested_content_is_profile_content
            && content_accepted
            && state
                .data_all_access()
                .is_match(account_id, requested_profile_internal_id)
                .await?);

    if !access_allowed {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let _guard = ContentSendingTracker::track();
    let concurrent = ContentSendingTracker::concurrent_count();
    let limits = state.config().limits_media();

    let max_allowed = if concurrent
        >= limits
            .get_content_low_quality_concurrent_requests_threshold
            .into()
    {
        ContentQualityVariant::Low
    } else if concurrent
        >= limits
            .get_content_medium_quality_concurrent_requests_threshold
            .into()
    {
        ContentQualityVariant::Medium
    } else {
        ContentQualityVariant::High
    };

    let actual_quality = select_quality(preferred_quality, max_allowed);

    send_content(
        &state,
        actual_quality,
        requested_profile,
        requested_content_id,
        browser_etag,
    )
    .await
}

fn select_quality(
    preferred: ContentQualityVariant,
    max_allowed: ContentQualityVariant,
) -> ContentQualityVariant {
    use ContentQualityVariant::*;
    match (preferred, max_allowed) {
        (_, Low) | (Low, _) => Low,
        (_, Medium) | (Medium, High) => Medium,
        (High, High) => High,
    }
}

async fn send_content(
    state: &S,
    quality: ContentQualityVariant,
    requested_profile: AccountId,
    requested_content_id: ContentId,
    browser_etag: Option<TypedHeader<IfNoneMatch>>,
) -> Result<
    (
        TypedHeader<ETag>,
        TypedHeader<CacheControl>,
        TypedHeader<ContentType>,
        TypedHeader<ContentLength>,
        TypedHeader<ContentQualityHeader>,
        Body,
    ),
    StatusCode,
> {
    let data = state.read().media().content_data_variant(
        requested_profile,
        requested_content_id,
        quality,
    )?;

    let (length, stream) = data
        .byte_count_and_read_stream()
        .await
        .change_context(DataError::File)?;

    if browser_etag.matches(state.etag_utils().immutable_content()) {
        return Err(StatusCode::NOT_MODIFIED);
    }

    Ok((
        TypedHeader(state.etag_utils().immutable_content().clone()),
        TypedHeader(cache_control_for_images()),
        TypedHeader(ContentType::octet_stream()),
        TypedHeader(ContentLength(length)),
        TypedHeader(ContentQualityHeader(quality)),
        Body::from_stream(stream),
    ))
}

const PATH_GET_ALL_ACCOUNT_MEDIA_CONTENT: &str = "/media_api/all_account_media_content/{aid}";

/// Get list of all media content on the server for one account.
///
/// # Access
///
/// - Own account
/// - Permission [model::Permissions::admin_moderate_media_content]
/// - Permission [model::Permissions::admin_edit_media_content_face_detected_value]
/// - Permission [model::Permissions::admin_edit_media_content_face_verified_value]
#[utoipa::path(
    get,
    path = PATH_GET_ALL_ACCOUNT_MEDIA_CONTENT,
    params(AccountId),
    responses(
        (status = 200, description = "Successful.", body = AccountContent),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn get_all_account_media_content(
    State(state): State<S>,
    Path(account_id): Path<AccountId>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Extension(api_caller_permissions): Extension<Permissions>,
) -> Result<Json<AccountContent>, StatusCode> {
    MEDIA.get_all_account_media_content.incr();

    let internal_id = state.get_internal_id(account_id).await?;

    let access_allowed = api_caller_account_id == internal_id
        || api_caller_permissions.admin_moderate_media_content
        || api_caller_permissions.admin_edit_media_content_face_detected_value
        || api_caller_permissions.admin_edit_media_content_face_verified_value;
    if !access_allowed {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let internal_current_media = state
        .read()
        .media()
        .all_account_media_content(internal_id)
        .await?;

    let data = internal_current_media
        .into_iter()
        .map(|m| m.into())
        .collect();

    Ok(AccountContent {
        data,
        max_content_count: state.config().limits_media().max_content_count,
        unused_content_wait_seconds: state
            .config()
            .limits_media()
            .unused_content_wait_duration
            .seconds,
    }
    .into())
}

const PATH_PUT_UPLOAD_CONTENT: &str = "/media_api/upload_content";

/// Upload content to server for processing.
///
/// The processed content is saved to content processing
/// slot when account state is [model::AccountState::InitialSetup].
/// In other states the slot number is ignored and content goes
/// directly to moderation. Slots from 0 to 6 are available.
///
/// When no errors are returned, processing of the content
/// will begin. Events about the content processing will be sent
/// to the client.
///
/// One account can only have one content in upload or processing ongoing.
/// Ongoing upload can be cancelled by starting another upload. When processing
/// is ongoing, uploading can't be done.
///
/// Content uploading will fail if content file size exceeds 10 MiB.
/// Content processing will fail if image content resolution width or height
/// value is less than 512.
#[utoipa::path(
    put,
    path = PATH_PUT_UPLOAD_CONTENT,
    params(NewContentParams),
    request_body(content = inline(model::BinaryData), content_type = "application/octet-stream"),
    responses(
        (status = 200, description = "Image upload result.", body = PutContentToContentSlotResult),
        (status = 401, description = "Unauthorized."),
        (status = 429, description = "Too many requests."),
        (status = 406, description = "Unknown slot ID."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn put_upload_content(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Query(new_content_params): Query<NewContentParams>,
    content_data: Body,
) -> Result<Json<PutContentToContentSlotResult>, StatusCode> {
    MEDIA.put_upload_content.incr();
    state
        .api_limits(account_id)
        .media()
        .put_upload_content()
        .await?;

    let slot = TryInto::<ContentSlot>::try_into(new_content_params.slot_id as i16)
        .map_err(|_| StatusCode::NOT_ACCEPTABLE)?;

    let count = state
        .read()
        .media()
        .all_account_media_content_count(account_id)
        .await?;
    if count > state.config().limits_media().max_content_count.into() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let upload_permit = match state.content_processing().begin_upload(account_id).await {
        Ok(v) => v,
        Err(ContentProcessingOngoing) => {
            return Ok(PutContentToContentSlotResult::error_content_processing_ongoing().into());
        }
    };

    let stream = content_data.into_data_stream();

    let content_info = state
        .write_concurrent(account_id.as_id(), move |cmds| async move {
            let out: ConcurrentWriteAction<crate::result::Result<_, DataError>> = cmds
                .accquire_image(move |cmds: ConcurrentWriteContentHandle| {
                    Box::new(
                        async move { cmds.save_to_tmp(account_id, stream, upload_permit).await },
                    )
                })
                .await;
            out
        })
        .await??;

    state
        .content_processing()
        .queue_new_content(account_id, slot, content_info, new_content_params)
        .await;

    Ok(PutContentToContentSlotResult::ok().into())
}

const PATH_GET_CONTENT_PROCESSING_STATE: &str = "/media_api/content_processing_state";

/// Get current content processing state for account.
#[utoipa::path(
    get,
    path = PATH_GET_CONTENT_PROCESSING_STATE,
    responses(
        (status = 200, description = "Successful.", body = ContentProcessingState),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_content_processing_state(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<ContentProcessingState>, StatusCode> {
    MEDIA.get_content_processing_state.incr();

    if let Some(state) = state
        .content_processing()
        .get_current_state(account_id)
        .await
    {
        Ok(state.into())
    } else {
        Ok(ContentProcessingState::default().into())
    }
}

const PATH_DELETE_CONTENT: &str = "/media_api/content/{aid}/{cid}";

/// Delete content data.
///
/// # Own account
/// Content can be deleted after specific time has passed
/// since removing all usage of it (content is not assigned
/// as security or profile content).
///
/// # Admin
/// Admin can remove content without restrictions with
/// permission `admin_delete_media_content`.
#[utoipa::path(
    delete,
    path = PATH_DELETE_CONTENT,
    params(AccountId, ContentId),
    responses(
        (status = 200, description = "Content data deleted."),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn delete_content(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Extension(permissions): Extension<Permissions>,
    Path(content_owner_account_id): Path<AccountId>,
    Path(content_id): Path<ContentId>,
) -> Result<(), StatusCode> {
    MEDIA.delete_content.incr();

    let content_owner_account_id = state.get_internal_id(content_owner_account_id).await?;
    let content_id = state
        .read()
        .media()
        .content_id_internal(content_owner_account_id, content_id)
        .await?;
    let content = state.read().media().content_state(content_id).await?;

    if *content_owner_account_id.as_db_id() != content.account_id {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let owner_deleting_content = content_owner_account_id == api_caller_account_id;
    let admin_access = permissions.admin_delete_media_content;
    let route_access_allowed = owner_deleting_content || admin_access;

    if !route_access_allowed {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    if owner_deleting_content
        && !admin_access
        && !content.removable_by_user(
            state
                .config()
                .limits_media()
                .unused_content_wait_duration
                .seconds,
        )
    {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    if owner_deleting_content {
        MEDIA.delete_content_for_content_owner.incr();
    } else {
        MEDIA.delete_content_for_admin.incr();
    }

    db_write!(state, move |cmds| {
        let r = cmds.media().delete_content(content_id).await?;

        if r.current_media_content_refresh_needed {
            cmds.events()
                .send_connected_event(
                    api_caller_account_id,
                    EventToClientInternal::MediaContentChanged,
                )
                .await?;
        }

        if content.moderation_state.is_in_moderation() {
            // Removed content was in moderation.

            cmds.common()
                .notification()
                .upsert_pending_app_notification(
                    content_id.content_owner(),
                    PendingAppNotificationInternal::MediaContentModerationDeleted,
                )
                .await?;

            cmds.events()
                .send_notification(
                    content_id.content_owner(),
                    NotificationEvent::MediaContentModerationCompleted,
                )
                .await?;
        }

        Ok(())
    })
}

create_open_api_router!(
    fn router_content,
    get_content,
    get_all_account_media_content,
    put_upload_content,
    get_content_processing_state,
    delete_content,
);

create_counters!(
    MediaCounters,
    MEDIA,
    MEDIA_CONTENT_COUNTERS_LIST,
    get_content,
    get_all_account_media_content,
    get_content_processing_state,
    put_upload_content,
    delete_content,
    delete_content_for_content_owner,
    delete_content_for_admin,
);
