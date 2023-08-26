use axum::{extract::Path, Extension};
use model::{
    AccountIdInternal, AccountId, HandleModerationRequest, ModerationList, SecurityImage,
};


use super::{
    utils::{Json, StatusCode},
    GetAccessTokens, GetConfig, GetInternalApi, GetAccounts, ReadData, WriteData, db_write,
};

pub const PATH_GET_SECURITY_IMAGE_INFO: &str = "/media_api/security_image_info/:account_id";

/// Get current security image for selected profile. Only for admins.
#[utoipa::path(
    get,
    path = "/media_api/security_image_info/{account_id}",
    params(AccountId),
    responses(
        (status = 200, description = "Get security image info.", body = SecurityImage),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn get_security_image_info<S: ReadData + GetAccounts>(
    Path(requested_account_id): Path<AccountId>,
    Extension(_api_caller_account_id): Extension<AccountIdInternal>,
    state: S,
) -> Result<Json<SecurityImage>, StatusCode> {
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
    security(("access_token" = [])),
)]
pub async fn patch_moderation_request_list<S: WriteData + GetAccessTokens>(
    Extension(account_id): Extension<AccountIdInternal>,
    state: S,
) -> Result<Json<ModerationList>, StatusCode> {

    // TODO: Access restrictions

    let data = db_write!(state, move |cmds| {
        cmds.media_admin()
            .moderation_get_list_and_create_new_if_necessary(account_id)
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
    params(AccountId),
    responses(
        (status = 200, description = "Handling moderation request was successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_handle_moderation_request<
    S: GetInternalApi + WriteData + GetAccessTokens + GetAccounts + GetConfig + ReadData,
>(
    Path(moderation_request_owner_account_id): Path<AccountId>,
    Extension(admin_account_id): Extension<AccountIdInternal>,
    Json(moderation_decision): Json<HandleModerationRequest>,
    state: S,
) -> Result<(), StatusCode> {
    let account = state
        .internal_api()
        .get_account_state(admin_account_id)
        .await?;

    if account.capablities().admin_moderate_images {
        let moderation_request_owner = state
            .accounts()
            .get_internal_id(moderation_request_owner_account_id)
            .await?;

        db_write!(state, move |cmds| {
            cmds.media_admin()
                .update_moderation(
                    admin_account_id,
                    moderation_request_owner,
                    moderation_decision,
                )
        })
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
