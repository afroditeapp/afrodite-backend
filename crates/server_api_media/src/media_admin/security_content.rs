use axum::{
    Extension,
    extract::{Path, State},
};
use model_media::{
    AccountId, AccountIdInternal, GetSecurityContentVerificationQueueNextItemResult, Permissions,
    PostSecurityContentVerificationQueueRemoveNextItem, PostSecurityContentVerifiedValue,
    SecurityContentAdminInfo, SecurityContentVerificationQueueAdminItem,
};
use server_api::{
    DataError, S,
    app::{GetAccounts, ReadData, SecurityContentVerificationQueueProvider},
    create_open_api_router,
    result::WrappedContextExt,
};
use server_data_media::{read::GetReadMediaCommands, write::GetWriteCommandsMedia};
use simple_backend::create_counters;

use crate::{
    app::WriteData,
    db_write,
    utils::{Json, StatusCode},
};

const PATH_GET_SECURITY_CONTENT_ADMIN_INFO: &str = "/media_api/security_content_admin_info/{aid}";

/// Get current security content for selected profile.
///
/// # Access
///
/// - Permission [model::Permissions::admin_moderate_media_content]
/// - Permission [model::Permissions::admin_edit_media_content_face_verified_value]
/// - Permission [model::Permissions::admin_edit_security_content_verified_value]
#[utoipa::path(
    get,
    path = PATH_GET_SECURITY_CONTENT_ADMIN_INFO,
    params(AccountId),
    responses(
        (status = 200, description = "Successful.", body = SecurityContentAdminInfo),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn get_security_content_admin_info(
    State(state): State<S>,
    Path(requested_account_id): Path<AccountId>,
    Extension(permissions): Extension<Permissions>,
) -> Result<Json<SecurityContentAdminInfo>, StatusCode> {
    MEDIA_ADMIN.get_security_content_admin_info.incr();

    let access_allowed = permissions.admin_moderate_media_content
        || permissions.admin_edit_media_content_face_verified_value
        || permissions.admin_edit_security_content_verified_value;

    if !access_allowed {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let internal_id = state.get_internal_id(requested_account_id).await?;

    let internal_current_media = state
        .read()
        .media()
        .current_account_media(internal_id)
        .await?;

    let info: SecurityContentAdminInfo = SecurityContentAdminInfo::new(internal_current_media);
    Ok(info.into())
}

const PATH_POST_SECURITY_CONTENT_VERIFIED_VALUE: &str =
    "/media_api/security_content_verified_value";

/// Change security content verified value
///
/// Bot account sets automatic value and human admin account sets manual override value.
///
/// # Access
/// * Permission [model::Permissions::admin_edit_security_content_verified_value]
#[utoipa::path(
    post,
    path = PATH_POST_SECURITY_CONTENT_VERIFIED_VALUE,
    request_body = PostSecurityContentVerifiedValue,
    responses(
        (status = 200, description = "Successful"),
        (status = 401, description = "Unauthorized"),
        (
            status = 500,
            description = "Internal server error",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn post_security_content_verified_value(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Extension(moderator_id): Extension<AccountIdInternal>,
    Json(data): Json<PostSecurityContentVerifiedValue>,
) -> Result<(), StatusCode> {
    MEDIA_ADMIN.post_security_content_verified_value.incr();

    if !permissions.admin_edit_security_content_verified_value {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let content_owner = state.get_internal_id(data.account_id).await?;

    db_write!(state, move |cmds| {
        let current_security_content = cmds
            .read()
            .media()
            .current_account_media(content_owner)
            .await?
            .security_content_id
            .map(|v| v.content_id());

        if current_security_content != Some(data.security_content) {
            return Err(DataError::NotAllowed.report());
        }

        cmds.media_admin()
            .content()
            .change_security_content_verified_value(moderator_id, content_owner, data.value)
            .await?;

        Ok(())
    })?;

    Ok(())
}

const PATH_GET_SECURITY_CONTENT_VERIFICATION_QUEUE_NEXT_ITEM: &str =
    "/media_api/security_content_verification_queue_admin_next";

/// Get next item in security content verification queue.
///
/// # Access
/// * Permission [model::Permissions::admin_edit_security_content_verified_value]
#[utoipa::path(
    get,
    path = PATH_GET_SECURITY_CONTENT_VERIFICATION_QUEUE_NEXT_ITEM,
    responses(
        (status = 200, description = "Successful", body = GetSecurityContentVerificationQueueNextItemResult),
        (status = 401, description = "Unauthorized"),
        (
            status = 500,
            description = "Internal server error",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn get_security_content_verification_queue_next_item(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
) -> Result<Json<GetSecurityContentVerificationQueueNextItemResult>, StatusCode> {
    MEDIA_ADMIN
        .get_security_content_verification_queue_next_item
        .incr();

    if !permissions.admin_edit_security_content_verified_value {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let item = state
        .security_content_verification_queue()
        .next_item()
        .await
        .map(
            |(account_id, value)| SecurityContentVerificationQueueAdminItem {
                account_id,
                security_content: value.security_content,
                verification_method: value.verification_method,
                verification_data: value.verification_data,
            },
        );

    Ok(GetSecurityContentVerificationQueueNextItemResult { item }.into())
}

const PATH_POST_SECURITY_CONTENT_VERIFICATION_QUEUE_REMOVE_NEXT_ITEM: &str =
    "/media_api/security_content_verification_queue_admin_next_remove";

/// Remove next item from security content verification queue if possible.
///
/// Removal succeeds only when the provided account id matches queue head item owner.
/// No error is returned is there is a mismatch.
///
/// # Access
/// * Permission [model::Permissions::admin_edit_security_content_verified_value]
#[utoipa::path(
    post,
    path = PATH_POST_SECURITY_CONTENT_VERIFICATION_QUEUE_REMOVE_NEXT_ITEM,
    request_body = PostSecurityContentVerificationQueueRemoveNextItem,
    responses(
        (status = 200, description = "Successful"),
        (status = 401, description = "Unauthorized"),
        (
            status = 500,
            description = "Internal server error",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn post_security_content_verification_queue_remove_next_item(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Json(data): Json<PostSecurityContentVerificationQueueRemoveNextItem>,
) -> Result<(), StatusCode> {
    MEDIA_ADMIN
        .post_security_content_verification_queue_remove_next_item
        .incr();

    if !permissions.admin_edit_security_content_verified_value {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let expected_account_id = state.get_internal_id(data.account_id).await?;

    let _ = state
        .security_content_verification_queue()
        .remove_next_item(expected_account_id)
        .await;

    Ok(())
}
create_open_api_router!(
    fn router_admin_security_content,
    get_security_content_admin_info,
    post_security_content_verified_value,
    get_security_content_verification_queue_next_item,
    post_security_content_verification_queue_remove_next_item,
);

create_counters!(
    MediaAdminCounters,
    MEDIA_ADMIN,
    MEDIA_ADMIN_SECURITY_CONTENT_COUNTERS_LIST,
    get_security_content_admin_info,
    post_security_content_verified_value,
    get_security_content_verification_queue_next_item,
    post_security_content_verification_queue_remove_next_item,
);
