use std::fmt::Write;

use axum::{
    extract::{BodyStream, Path, Query, State},
    Extension, TypedHeader, Router,
};
use headers::ContentType;
use model::{
    AccountId, AccountIdInternal, ContentId, ContentAccessCheck, ContentSlot, MapTileX, MapTileY,
    MapTileZ, ModerationRequest, ModerationRequestContent, AccountContent,
    ProfileContent, SlotId, NewContentParams, ContentProcessingId, ContentProcessingState, SetProfileContent, PendingProfileContent, SecurityImage, PendingSecurityImage, HandleModerationRequest, ModerationList,
};
use simple_backend::{app::GetTileMap, create_counters};
use tracing::error;

use crate::{app::{GetAccessTokens, GetAccounts, ReadData, WriteData, GetInternalApi, GetConfig}};
use crate::api::{
    db_write,
    utils::{Json, StatusCode},
};
use crate::{
    data::{
        write_concurrent::{ConcurrentWriteAction, ConcurrentWriteContentHandle},
        DataError,
    }, app::ContentProcessingProvider,
};


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
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<ModerationList>, StatusCode> {
    MEDIA_ADMIN.patch_moderation_request_list.incr();

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
    State(state): State<S>,
    Path(moderation_request_owner_account_id): Path<AccountId>,
    Extension(admin_account_id): Extension<AccountIdInternal>,
    Json(moderation_decision): Json<HandleModerationRequest>,
) -> Result<(), StatusCode> {
    MEDIA_ADMIN.post_handle_moderation_request.incr();

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
            cmds.media_admin().update_moderation(
                admin_account_id,
                moderation_request_owner,
                moderation_decision,
            )
        })
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}


pub fn admin_moderation_router(s: crate::app::S) -> Router {
    use crate::app::S;
    use axum::routing::{patch, post};

    Router::new()
        .route(PATH_ADMIN_MODERATION_PAGE_NEXT, patch(patch_moderation_request_list::<S>))
        .route(PATH_ADMIN_MODERATION_HANDLE_REQUEST, post(post_handle_moderation_request::<S>))
        .with_state(s)
}

create_counters!(
    MediaAdminCounters,
    MEDIA_ADMIN,
    MEDIA_ADMIN_MODERATION_COUNTERS_LIST,
    patch_moderation_request_list,
    post_handle_moderation_request,
);
