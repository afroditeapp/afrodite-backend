use axum::{
    extract::{Path, Query, State},
    Extension,
};
use model_media::{
    AccountId, AccountIdInternal, EventToClientInternal, HandleModerationRequest, ModerationList,
    ModerationQueueTypeParam, NotificationEvent, Permissions,
};
use obfuscate_api_macro::obfuscate_api;
use server_api::{create_open_api_router, S};
use server_data_media::write::GetWriteCommandsMedia;
use simple_backend::create_counters;
use utoipa_axum::router::OpenApiRouter;

use crate::{
    app::{GetAccounts, WriteData},
    db_write, db_write_multiple,
    utils::{Json, StatusCode},
};

// TODO: Add moderation content moderation weight to account and use it when moderating.
//       Moderation should have some value which keeps track how much moderation
//       request has moderation weight added. Perhaps this should not be in MVP?

#[obfuscate_api]
const PATH_ADMIN_MODERATION_PAGE_NEXT: &str = "/media_api/admin/moderation/page/next";

/// Get current list of moderation requests in my moderation queue.
/// Additional requests will be added to my queue if necessary.
///
/// ## Access
///
/// Account with `admin_moderate_images` permission is required to access this
/// route.
///
#[utoipa::path(
    patch,
    path = PATH_ADMIN_MODERATION_PAGE_NEXT,
    params(ModerationQueueTypeParam),
    responses(
        (status = 200, description = "Get moderation request list was successfull.", body = ModerationList),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn patch_moderation_request_list(
    State(state): State<S>,
    Query(queue_type): Query<ModerationQueueTypeParam>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<ModerationList>, StatusCode> {
    MEDIA_ADMIN.patch_moderation_request_list.incr();

    // TODO: Access restrictions

    let data = db_write!(state, move |cmds| {
        cmds.media_admin()
            .moderation_get_list_and_create_new_if_necessary(account_id, queue_type.queue)
    })?;

    Ok(ModerationList { list: data }.into())
}

// TODO(prod): Check that make, get and moderate requests in both moderation
//             queues.

#[obfuscate_api]
const PATH_ADMIN_MODERATION_HANDLE_REQUEST: &str =
    "/media_api/admin/moderation/handle_request/{aid}";

/// Handle moderation request of some account.
///
/// ## Access
///
/// Account with `admin_moderate_images` permission is required to access this
/// route.
///
#[utoipa::path(
    post,
    path = PATH_ADMIN_MODERATION_HANDLE_REQUEST,
    request_body(content = HandleModerationRequest),
    params(AccountId),
    responses(
        (status = 200, description = "Handling moderation request was successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_handle_moderation_request(
    State(state): State<S>,
    Path(moderation_request_owner_account_id): Path<AccountId>,
    Extension(admin_account_id): Extension<AccountIdInternal>,
    Extension(api_caller_permissions): Extension<Permissions>,
    Json(moderation_decision): Json<HandleModerationRequest>,
) -> Result<(), StatusCode> {
    MEDIA_ADMIN.post_handle_moderation_request.incr();

    if !api_caller_permissions.admin_moderate_images {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let moderation_request_owner = state
        .get_internal_id(moderation_request_owner_account_id)
        .await?;

    db_write_multiple!(state, move |cmds| {
        let info = cmds
            .media_admin()
            .update_moderation(
                admin_account_id,
                moderation_request_owner,
                moderation_decision,
            )
            .await?;

        if cmds.config().components().account {
            if let Some(new_visibility) = info.new_visibility {
                cmds.events()
                    .send_connected_event(
                        moderation_request_owner,
                        EventToClientInternal::ProfileVisibilityChanged(new_visibility),
                    )
                    .await?;
            }
        }

        cmds.events()
            .send_notification(
                moderation_request_owner,
                NotificationEvent::ContentModerationRequestCompleted,
            )
            .await?;

        Ok(())
    })?;

    // TODO(microservice): Add profile visibility change notification
    //                     to account internal API.

    Ok(())
}

pub fn admin_moderation_router(s: S) -> OpenApiRouter {
    create_open_api_router!(
        s,
        patch_moderation_request_list,
        post_handle_moderation_request,
    )
}

create_counters!(
    MediaAdminCounters,
    MEDIA_ADMIN,
    MEDIA_ADMIN_MODERATION_COUNTERS_LIST,
    patch_moderation_request_list,
    post_handle_moderation_request,
);
