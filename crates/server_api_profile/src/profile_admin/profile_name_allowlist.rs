use axum::{
    extract::State,
    Extension,
};
use model_profile::{
    AccountIdInternal, EventToClientInternal, GetProfileNamePendingModerationList, Permissions, PostModerateProfileName
};
use obfuscate_api_macro::obfuscate_api;
use server_api::S;
use server_api::{app::{GetAccounts, WriteData}, create_open_api_router, db_write_multiple};
use server_data_profile::{read::GetReadProfileCommands, write::GetWriteCommandsProfile};
use simple_backend::create_counters;
use utoipa_axum::router::OpenApiRouter;

use crate::{
    app::ReadData,
    utils::{Json, StatusCode},
};

#[obfuscate_api]
const PATH_GET_PROFILE_NAME_PENDING_MODERATION_LIST: &str = "/profile_api/admin/profile_name_pending_moderation";

#[utoipa::path(
    get,
    path = PATH_GET_PROFILE_NAME_PENDING_MODERATION_LIST,
    responses(
        (status = 200, description = "Successful", body = GetProfileNamePendingModerationList),
        (status = 401, description = "Unauthorized"),
        (
            status = 500,
            description = "Internal server error",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn get_profile_name_pending_moderation_list(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
) -> Result<Json<GetProfileNamePendingModerationList>, StatusCode> {
    PROFILE.get_profile_name_pending_moderation_list.incr();

    if !permissions.admin_moderate_profile_names {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let r = state
        .read()
        .profile_admin()
        .profile_name_allowlist()
        .profile_name_pending_moderation_list()
        .await?;

    Ok(r.into())
}

#[obfuscate_api]
const PATH_POST_MODERATE_PROFILE_NAME: &str = "/profile_api/admin/moderate_profile_name";

#[utoipa::path(
    post,
    path = PATH_POST_MODERATE_PROFILE_NAME,
    request_body = PostModerateProfileName,
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
pub async fn post_moderate_profile_name(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Extension(moderator_id): Extension<AccountIdInternal>,
    Json(data): Json<PostModerateProfileName>,
) -> Result<(), StatusCode> {
    PROFILE.post_moderate_profile_name.incr();

    if !permissions.admin_moderate_profile_names {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let name_owner_id = state.get_internal_id(data.id).await?;

    db_write_multiple!(state, move |cmds| {
        cmds
            .profile_admin()
            .profile_name_allowlist()
            .moderate_profile_name(
                moderator_id,
                name_owner_id,
                data.name,
                data.accept,
            ).await?;

        cmds.events()
            .send_connected_event(name_owner_id, EventToClientInternal::ProfileChanged)
            .await?;

        Ok(())
    })?;

    Ok(())
}

pub fn admin_profile_name_allowlist_router(
    s: S,
) -> OpenApiRouter {
    create_open_api_router!(
        s,
        get_profile_name_pending_moderation_list,
        post_moderate_profile_name,
    )
}

create_counters!(
    ProfileCounters,
    PROFILE,
    PROFILE_ADMIN_PROFILE_NAME_ALLOWLIST_COUNTERS_LIST,
    get_profile_name_pending_moderation_list,
    post_moderate_profile_name,
);
