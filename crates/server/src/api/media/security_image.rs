use axum::{
    extract::{Path, State},
    Extension, Router,
};
use model::{AccountId, AccountIdInternal, ContentId, PendingSecurityImage, SecurityImage};
use simple_backend::create_counters;

use crate::{
    api::{
        db_write,
        utils::{Json, StatusCode},
    },
    app::{GetAccounts, ReadData, WriteData},
};

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
        .update_security_content(api_caller_account_id, content_id))
}

pub const PATH_GET_PENDING_SECURITY_IMAGE_INFO: &str =
    "/media_api/pending_security_image_info/:account_id";

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
        .update_or_delete_pending_security_content(
            api_caller_account_id,
            Some(content_id)
        ))
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
        .update_or_delete_pending_security_content(
            api_caller_account_id,
            None
        ))
}

pub fn security_image_router(s: crate::app::S) -> Router {
    use axum::routing::{delete, get, put};

    use crate::app::S;

    Router::new()
        .route(
            PATH_GET_SECURITY_IMAGE_INFO,
            get(get_security_image_info::<S>),
        )
        .route(
            PATH_PUT_SECURITY_IMAGE_INFO,
            put(put_security_image_info::<S>),
        )
        .route(
            PATH_GET_PENDING_SECURITY_IMAGE_INFO,
            get(get_pending_security_image_info::<S>),
        )
        .route(
            PATH_PUT_PENDING_SECURITY_IMAGE_INFO,
            put(put_pending_security_image_info::<S>),
        )
        .route(
            DELETE_PENDING_SECURITY_IMAGE_INFO,
            delete(delete_pending_security_image_info::<S>),
        )
        .with_state(s)
}

create_counters!(
    MediaCounters,
    MEDIA,
    MEDIA_SECURITY_IMAGE_COUNTERS_LIST,
    get_security_image_info,
    put_security_image_info,
    get_pending_security_image_info,
    put_pending_security_image_info,
);
