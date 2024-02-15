use axum::{
    extract::{Path, Query, State},
    Extension, Router,
};
use model::{schema::shared_state::profile_visibility_state_number, AccountId, AccountIdInternal, EventToClientInternal, HandleModerationRequest, ModerationList, ModerationQueueType, ModerationQueueTypeParam};
use simple_backend::create_counters;

use crate::{
    api::{
        db_write, media::moderation_request, utils::{Json, StatusCode}
    },
    app::{EventManagerProvider, GetAccessTokens, GetAccounts, GetConfig, GetInternalApi, ReadData, WriteData}, internal_api,
};

// TODO: Add moderation content moderation weight to account and use it when moderating.
//       Moderation should have some value which keeps track how much moderation
//       request has moderation weight added. Perhaps this should not be in MVP?

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
    params(ModerationQueueTypeParam),
    responses(
        (status = 200, description = "Get moderation request list was successfull.", body = ModerationList),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn patch_moderation_request_list<S: WriteData + GetAccessTokens>(
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
    S: GetInternalApi + WriteData + GetAccounts + GetConfig + ReadData + EventManagerProvider,
>(
    State(state): State<S>,
    Path(moderation_request_owner_account_id): Path<AccountId>,
    Extension(admin_account_id): Extension<AccountIdInternal>,
    Json(moderation_decision): Json<HandleModerationRequest>,
) -> Result<(), StatusCode> {
    MEDIA_ADMIN.post_handle_moderation_request.incr();

    let account = state.read()
        .common()
        .account(admin_account_id)
        .await?;

    if account.capablities().admin_moderate_images {
        let moderation_request_owner = state
            .accounts()
            .get_internal_id(moderation_request_owner_account_id)
            .await?;

        let profile_visibility_changed = db_write!(state, move |cmds| {
            cmds.media_admin().update_moderation(
                admin_account_id,
                moderation_request_owner,
                moderation_decision,
            )
        })?;

        if profile_visibility_changed.is_some() &&
            state.config().components().account {
                let moderation_request_owner_account = state
                    .read()
                    .common()
                    .account(moderation_request_owner)
                    .await?;
                state
                    .event_manager()
                    .send_connected_event(
                        moderation_request_owner,
                        EventToClientInternal::ProfileVisibilityChanged(
                            moderation_request_owner_account.profile_visibility(),
                        ),
                    )
                    .await?;
        } else {
            // TODO(microservice): Add profile visibility change notification
            //                     to account internal API.
        }

        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub fn admin_moderation_router(s: crate::app::S) -> Router {
    use axum::routing::{patch, post};

    use crate::app::S;

    Router::new()
        .route(
            PATH_ADMIN_MODERATION_PAGE_NEXT,
            patch(patch_moderation_request_list::<S>),
        )
        .route(
            PATH_ADMIN_MODERATION_HANDLE_REQUEST,
            post(post_handle_moderation_request::<S>),
        )
        .with_state(s)
}

create_counters!(
    MediaAdminCounters,
    MEDIA_ADMIN,
    MEDIA_ADMIN_MODERATION_COUNTERS_LIST,
    patch_moderation_request_list,
    post_handle_moderation_request,
);
