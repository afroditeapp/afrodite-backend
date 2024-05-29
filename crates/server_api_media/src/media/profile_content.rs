use axum::{
    extract::{Path, Query, State},
    Extension, Router,
};
use model::{
    AccountId, AccountIdInternal, ContentAccessCheck, PendingProfileContent, ProfileContent,
    SetProfileContent,
};
use simple_backend::create_counters;

use crate::{
    app::{GetAccounts, ReadData, StateBase, WriteData},
    db_write,
    utils::{Json, StatusCode},
};

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

    let internal_id = state.get_internal_id(account_id).await?;

    let internal_current_media = state
        .read()
        .media()
        .current_account_media(internal_id)
        .await?;

    let info: ProfileContent = internal_current_media.into();
    Ok(info.into())
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

pub const PATH_GET_PENDING_PROFILE_CONTENT_INFO: &str =
    "/media_api/pending_profile_content_info/:account_id";

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

    let internal_id = state.get_internal_id(account_id).await?;

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
/// - All content must not be moderated as rejected.
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
        .update_or_delete_pending_profile_content(
            api_caller_account_id,
            Some(new)
        ))
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
        .update_or_delete_pending_profile_content(
            api_caller_account_id,
            None
        ))
}

pub fn profile_content_router<S: StateBase + WriteData + ReadData + GetAccounts>(s: S) -> Router {
    use axum::routing::{delete, get, put};

    Router::new()
        .route(
            PATH_GET_PROFILE_CONTENT_INFO,
            get(get_profile_content_info::<S>),
        )
        .route(PATH_PUT_PROFILE_CONTENT, put(put_profile_content::<S>))
        .route(
            PATH_GET_PENDING_PROFILE_CONTENT_INFO,
            get(get_pending_profile_content_info::<S>),
        )
        .route(
            PATH_PUT_PENDING_PROFILE_CONTENT,
            put(put_pending_profile_content::<S>),
        )
        .route(
            PATH_DELETE_PENDING_PROFILE_CONTENT,
            delete(delete_pending_profile_content::<S>),
        )
        .with_state(s)
}

create_counters!(
    MediaCounters,
    MEDIA,
    MEDIA_PROFILE_CONTENT_COUNTERS_LIST,
    get_profile_content_info,
    put_profile_content,
    get_pending_profile_content_info,
    put_pending_profile_content,
    delete_pending_profile_content,
);
