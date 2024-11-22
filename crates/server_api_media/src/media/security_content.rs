use axum::{
    extract::{Path, State},
    Extension,
};
use model::{AccountId, AccountIdInternal, ContentId, PendingSecurityContent, SecurityContent};
use obfuscate_api_macro::obfuscate_api;
use server_api::create_open_api_router;
use server_data_media::{read::GetReadMediaCommands, write::GetWriteCommandsMedia};
use simple_backend::create_counters;
use utoipa_axum::router::OpenApiRouter;

use crate::{
    app::{GetAccounts, ReadData, StateBase, WriteData},
    db_write,
    utils::{Json, StatusCode},
};

#[obfuscate_api]
const PATH_GET_SECURITY_CONTENT_INFO: &str = "/media_api/security_content_info/{aid}";

/// Get current security content for selected profile.
#[utoipa::path(
    get,
    path = PATH_GET_SECURITY_CONTENT_INFO,
    params(AccountId),
    responses(
        (status = 200, description = "Successful.", body = SecurityContent),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn get_security_content_info<S: ReadData + GetAccounts>(
    State(state): State<S>,
    Path(requested_account_id): Path<AccountId>,
    Extension(_api_caller_account_id): Extension<AccountIdInternal>,
) -> Result<Json<SecurityContent>, StatusCode> {
    MEDIA.get_security_content_info.incr();

    // TODO: access restrictions

    let internal_id = state.get_internal_id(requested_account_id).await?;

    let internal_current_media = state
        .read()
        .media()
        .current_account_media(internal_id)
        .await?;

    let info: SecurityContent = internal_current_media.into();
    Ok(info.into())
}

#[obfuscate_api]
const PATH_PUT_SECURITY_CONTENT_INFO: &str = "/media_api/security_content_info";

/// Set current security content content for current account.
///
/// # Restrictions
/// - The content must be moderated as accepted.
/// - The content must be owned by the account.
/// - The content must be an image.
/// - The content must be captured by client.
/// - The content must have face detected.
#[utoipa::path(
    put,
    path = PATH_PUT_SECURITY_CONTENT_INFO,
    request_body = ContentId,
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn put_security_content_info<S: WriteData>(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Json(content_id): Json<ContentId>,
) -> Result<(), StatusCode> {
    MEDIA.put_security_content_info.incr();

    db_write!(state, move |cmds| cmds
        .media()
        .update_security_content(api_caller_account_id, content_id))
}

#[obfuscate_api]
const PATH_GET_PENDING_SECURITY_CONTENT_INFO: &str =
    "/media_api/pending_security_content_info/{aid}";

/// Get pending security content for selected profile.
#[utoipa::path(
    get,
    path = PATH_GET_PENDING_SECURITY_CONTENT_INFO,
    params(AccountId),
    responses(
        (status = 200, description = "Successful.", body = PendingSecurityContent),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn get_pending_security_content_info<S: ReadData + GetAccounts>(
    State(state): State<S>,
    Path(requested_account_id): Path<AccountId>,
    Extension(_api_caller_account_id): Extension<AccountIdInternal>,
) -> Result<Json<PendingSecurityContent>, StatusCode> {
    MEDIA.get_pending_security_content_info.incr();

    // TODO: access restrictions

    let internal_id = state.get_internal_id(requested_account_id).await?;

    let internal_current_media = state
        .read()
        .media()
        .current_account_media(internal_id)
        .await?;

    let info: PendingSecurityContent = internal_current_media.into();
    Ok(info.into())
}

#[obfuscate_api]
const PATH_PUT_PENDING_SECURITY_CONTENT_INFO: &str = "/media_api/pending_security_content_info";

/// Set pending security content for current account.
///
/// Requires that the content has face detected.
#[utoipa::path(
    put,
    path = PATH_PUT_PENDING_SECURITY_CONTENT_INFO,
    request_body = ContentId,
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn put_pending_security_content_info<S: WriteData>(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Json(content_id): Json<ContentId>,
) -> Result<(), StatusCode> {
    MEDIA.put_pending_security_content_info.incr();

    db_write!(state, move |cmds| cmds
        .media()
        .update_or_delete_pending_security_content(
            api_caller_account_id,
            Some(content_id)
        ))
}

#[obfuscate_api]
const DELETE_PENDING_SECURITY_CONTENT_INFO: &str = "/media_api/pending_security_content_info";

/// Delete pending security content for current account.
/// Server will not change the security content when next moderation request
/// is moderated as accepted.
#[utoipa::path(
    delete,
    path = DELETE_PENDING_SECURITY_CONTENT_INFO,
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn delete_pending_security_content_info<S: WriteData>(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
) -> Result<(), StatusCode> {
    MEDIA.put_pending_security_content_info.incr();

    db_write!(state, move |cmds| cmds
        .media()
        .update_or_delete_pending_security_content(
            api_caller_account_id,
            None
        ))
}

pub fn security_content_router<S: StateBase + WriteData + ReadData + GetAccounts>(s: S) -> OpenApiRouter {
    create_open_api_router!(
        s,
        get_security_content_info::<S>,
        put_security_content_info::<S>,
        get_pending_security_content_info::<S>,
        put_pending_security_content_info::<S>,
        delete_pending_security_content_info::<S>,
    )
}

create_counters!(
    MediaCounters,
    MEDIA,
    MEDIA_SECURITY_CONTENT_COUNTERS_LIST,
    get_security_content_info,
    put_security_content_info,
    get_pending_security_content_info,
    put_pending_security_content_info,
);
