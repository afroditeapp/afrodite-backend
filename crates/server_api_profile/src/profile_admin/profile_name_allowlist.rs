use axum::{
    extract::State,
    Extension,
};
use model::{
    GetProfileNamePendingModerationList, Permissions
};
use obfuscate_api_macro::obfuscate_api;
use server_api::create_open_api_router;
use server_data_profile::read::GetReadProfileCommands;
use simple_backend::create_counters;
use utoipa_axum::router::OpenApiRouter;

use crate::{
    app::{
        ReadData, StateBase,
    },
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
pub async fn get_profile_name_pending_moderation_list<
    S: ReadData,
>(
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

pub fn admin_profile_name_allowlist_router<
    S: StateBase + ReadData,
>(
    s: S,
) -> OpenApiRouter {
    create_open_api_router!(
        s,
        get_profile_name_pending_moderation_list::<S>,
    )
}

create_counters!(
    ProfileCounters,
    PROFILE,
    PROFILE_ADMIN_PROFILE_NAME_ALLOWLIST_COUNTERS_LIST,
    get_profile_name_pending_moderation_list,
);
