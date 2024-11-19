use axum::{
    extract::{Query, State},
    Extension,
};
use model::{
    AccountIdInternal, EventToClientInternal, GetProfileTextPendingModerationList, GetProfileTextPendingModerationParams, Permissions, PostModerateProfileText
};
use obfuscate_api_macro::obfuscate_api;
use server_api::{app::{GetAccounts, WriteData}, create_open_api_router, db_write_multiple};
use server_data_profile::{read::GetReadProfileCommands, write::GetWriteCommandsProfile};
use simple_backend::create_counters;
use utoipa_axum::router::OpenApiRouter;

use crate::{
    app::{
        ReadData, StateBase,
    },
    utils::{Json, StatusCode},
};

#[obfuscate_api]
const PATH_GET_PROFILE_TEXT_PENDING_MODERATION_LIST: &str = "/profile_api/admin/profile_text_pending_moderation";


/// Get first page of pending profile text moderations. Oldest item is first and count 25.
#[utoipa::path(
    get,
    path = PATH_GET_PROFILE_TEXT_PENDING_MODERATION_LIST,
    params(GetProfileTextPendingModerationParams),
    responses(
        (status = 200, description = "Successful", body = GetProfileTextPendingModerationList),
        (status = 401, description = "Unauthorized"),
        (
            status = 500,
            description = "Internal server error",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn get_profile_text_pending_moderation_list<
    S: ReadData,
>(
    State(state): State<S>,
    Extension(moderator_id): Extension<AccountIdInternal>,
    Extension(permissions): Extension<Permissions>,
    Query(params): Query<GetProfileTextPendingModerationParams>,
) -> Result<Json<GetProfileTextPendingModerationList>, StatusCode> {
    PROFILE.get_profile_text_pending_moderation_list.incr();

    if !permissions.admin_moderate_profile_texts {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let r = state
        .read()
        .profile_admin()
        .profile_text()
        .profile_text_pending_moderation_list(moderator_id, params)
        .await?;

    Ok(r.into())
}

#[obfuscate_api]
const PATH_POST_MODERATE_PROFILE_TEXT: &str = "/profile_api/admin/moderate_profile_text";


/// Rejected category and details can be set only when the text is rejected.
///
/// This route will fail if the text is already moderated or the users's
/// profile text is not the same text that was moderated.
#[utoipa::path(
    post,
    path = PATH_POST_MODERATE_PROFILE_TEXT,
    request_body = PostModerateProfileText,
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
pub async fn post_moderate_profile_text<
    S: WriteData + GetAccounts,
>(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Extension(moderator_id): Extension<AccountIdInternal>,
    Json(data): Json<PostModerateProfileText>,
) -> Result<(), StatusCode> {
    PROFILE.post_moderate_profile_text.incr();

    if !permissions.admin_moderate_profile_texts {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    if data.accept && (data.rejected_category.is_some() || data.rejected_details.is_some()) {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let name_owner_id = state.get_internal_id(data.id).await?;

    db_write_multiple!(state, move |cmds| {
        cmds
            .profile_admin()
            .profile_text()
            .moderate_profile_text(
                moderator_id,
                name_owner_id,
                data.text,
                data.accept,
                data.rejected_category,
                data.rejected_details,
            ).await?;

        if data.accept {
            cmds.events()
                .send_connected_event(name_owner_id, EventToClientInternal::ProfileChanged)
                .await?;
        }

        Ok(())
    })?;

    Ok(())
}

pub fn admin_profile_text_router<
    S: StateBase + ReadData + WriteData + GetAccounts,
>(
    s: S,
) -> OpenApiRouter {
    create_open_api_router!(
        s,
        get_profile_text_pending_moderation_list::<S>,
        post_moderate_profile_text::<S>,
    )
}

create_counters!(
    ProfileCounters,
    PROFILE,
    PROFILE_ADMIN_PROFILE_TEXT_COUNTERS_LIST,
    get_profile_text_pending_moderation_list,
    post_moderate_profile_text,
);
