use axum::{
    extract::{Path, Query, State},
    Extension,
};
use model::AccountId;
use model_profile::{
    AccountIdInternal, EventToClientInternal, GetProfileTextPendingModerationList, GetProfileTextPendingModerationParams, GetProfileTextState, Permissions, PostModerateProfileText
};
use obfuscate_api_macro::obfuscate_api;
use server_api::{
    app::{GetAccounts, WriteData},
    create_open_api_router, db_write_multiple, S,
};
use server_data_profile::{read::GetReadProfileCommands, write::GetWriteCommandsProfile};
use simple_backend::create_counters;
use utoipa_axum::router::OpenApiRouter;

use crate::{
    app::ReadData,
    utils::{Json, StatusCode},
};

#[obfuscate_api]
const PATH_GET_PROFILE_TEXT_PENDING_MODERATION_LIST: &str =
    "/profile_api/admin/profile_text_pending_moderation";

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
pub async fn get_profile_text_pending_moderation_list(
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
pub async fn post_moderate_profile_text(
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

    let text_owner_id = state.get_internal_id(data.id).await?;

    db_write_multiple!(state, move |cmds| {
        cmds.profile_admin()
            .profile_text()
            .moderate_profile_text(
                moderator_id,
                text_owner_id,
                data.text,
                data.accept,
                data.rejected_category,
                data.rejected_details,
                data.move_to_human.unwrap_or_default(),
            )
            .await?;

        cmds.events()
            .send_connected_event(text_owner_id, EventToClientInternal::ProfileChanged)
            .await?;

        Ok(())
    })?;

    Ok(())
}

#[obfuscate_api]
const PATH_GET_PROFILE_TEXT_STATE: &str = "/profile_api/get_profile_text_state/{aid}";

/// Get profile text state
///
/// # Access
/// - Permission [model::Permissions::admin_moderate_profile_texts]
#[utoipa::path(
    get,
    path = PATH_GET_PROFILE_TEXT_STATE,
    params(AccountId),
    responses(
        (status = 200, description = "Successful.", body = GetProfileTextState),
        (status = 401, description = "Unauthorized."),
        (
            status = 500,
            description = "Internal server error.",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn get_profile_text_state(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Path(account_id): Path<AccountId>,
) -> Result<Json<GetProfileTextState>, StatusCode> {
    PROFILE.get_profile_text_state.incr();

    if !permissions.admin_moderate_profile_texts {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let text_owner_id = state.get_internal_id(account_id).await?;

    let r = state.read().profile().my_profile(text_owner_id).await?;
    let r = GetProfileTextState {
        text: r.p.ptext,
        moderation_info: r.text_moderation_info,
    };

    Ok(r.into())
}

pub fn admin_profile_text_router(s: S) -> OpenApiRouter {
    create_open_api_router!(
        s,
        get_profile_text_pending_moderation_list,
        post_moderate_profile_text,
        get_profile_text_state,
    )
}

create_counters!(
    ProfileCounters,
    PROFILE,
    PROFILE_ADMIN_PROFILE_TEXT_COUNTERS_LIST,
    get_profile_text_pending_moderation_list,
    post_moderate_profile_text,
    get_profile_text_state,
);
