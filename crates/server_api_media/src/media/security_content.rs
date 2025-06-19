use axum::{
    Extension,
    extract::{Path, State},
};
use model::Permissions;
use model_media::{AccountId, AccountIdInternal, ContentId, SecurityContent};
use server_api::{S, create_open_api_router, db_write_multiple};
use server_data_media::{read::GetReadMediaCommands, write::GetWriteCommandsMedia};
use simple_backend::create_counters;

use crate::{
    app::{GetAccounts, ReadData, WriteData},
    utils::{Json, StatusCode},
};

const PATH_GET_SECURITY_CONTENT_INFO: &str = "/media_api/security_content_info/{aid}";

/// Get current security content for selected profile.
///
/// # Access
///
/// - Own account
/// - Permission [model::Permissions::admin_moderate_media_content]
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
pub async fn get_security_content_info(
    State(state): State<S>,
    Path(requested_account_id): Path<AccountId>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Extension(permissions): Extension<Permissions>,
) -> Result<Json<SecurityContent>, StatusCode> {
    MEDIA.get_security_content_info.incr();

    let internal_id = state.get_internal_id(requested_account_id).await?;

    let access_allowed =
        internal_id == api_caller_account_id || permissions.admin_moderate_media_content;

    if !access_allowed {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let internal_current_media = state
        .read()
        .media()
        .current_account_media(internal_id)
        .await?;

    let info: SecurityContent = SecurityContent::new(internal_current_media);
    Ok(info.into())
}

const PATH_PUT_SECURITY_CONTENT_INFO: &str = "/media_api/security_content_info";

/// Set current security content for current account.
///
/// This also moves the content to moderation if it is not already
/// in moderation or moderated.
///
/// # Restrictions
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
pub async fn put_security_content_info(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Json(content_id): Json<ContentId>,
) -> Result<(), StatusCode> {
    MEDIA.put_security_content_info.incr();

    db_write_multiple!(state, move |cmds| {
        let content_id = cmds
            .read()
            .media()
            .content_id_internal(api_caller_account_id, content_id)
            .await?;
        cmds.media().update_security_content(content_id).await
    })
}

create_open_api_router!(
        fn router_security_content,
        get_security_content_info,
        put_security_content_info,
);

create_counters!(
    MediaCounters,
    MEDIA,
    MEDIA_SECURITY_CONTENT_COUNTERS_LIST,
    get_security_content_info,
    put_security_content_info,
);
