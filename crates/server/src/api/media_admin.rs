use axum::extract::{Path};

use axum::{Extension, Json, TypedHeader};


use hyper::StatusCode;

use tracing::error;





use model::{
    HandleModerationRequest, ModerationList, SecurityImage,
};

use model::{AccountIdInternal, AccountIdLight};
use super::utils::ApiKeyHeader;
use super::{GetApiKeys, GetInternalApi, GetUsers, ReadDatabase, WriteData};



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
    Extension(_api_caller_account_id): Extension<AccountIdInternal>,
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
pub async fn patch_moderation_request_list<S: WriteData + GetApiKeys>(
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
        .write(move |cmds| async move {
            cmds.media_admin()
                .moderation_get_list_and_create_new_if_necessary(account_id)
                .await
        })
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
    S: GetInternalApi + WriteData + GetApiKeys + GetUsers,
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
            .write(move |cmds| async move {
                cmds.media_admin()
                    .update_moderation(
                        admin_account_id,
                        moderation_request_owner,
                        moderation_decision,
                    )
                    .await
            })
            .await
            .map_err(|e| {
                error!("{e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
