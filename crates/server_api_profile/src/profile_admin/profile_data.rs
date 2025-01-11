use axum::{
    extract::{Path, State},
    Extension,
};
use model::AccountId;
use model_profile::{
    GetProfileAgeAndName, Permissions,
};
use obfuscate_api_macro::obfuscate_api;
use server_api::{
    app::GetAccounts,
    create_open_api_router, S,
};
use server_data_profile::read::GetReadProfileCommands;
use simple_backend::create_counters;
use utoipa_axum::router::OpenApiRouter;

use crate::{
    app::ReadData,
    utils::{Json, StatusCode},
};

#[obfuscate_api]
const PATH_GET_PROFILE_AGE_AND_NAME: &str = "/profile_api/get_profile_age_and_name/{aid}";

/// Get profile age and name
///
/// # Access
/// - Permission [model::Permissions::admin_find_account_by_email]
/// - Permission [model::Permissions::admin_view_permissions]
#[utoipa::path(
    get,
    path = PATH_GET_PROFILE_AGE_AND_NAME,
    params(AccountId),
    responses(
        (status = 200, description = "Successful.", body = GetProfileAgeAndName),
        (status = 401, description = "Unauthorized."),
        (
            status = 500,
            description = "Internal server error.",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn get_profile_age_and_name(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Path(account_id): Path<AccountId>,
) -> Result<Json<GetProfileAgeAndName>, StatusCode> {
    PROFILE.get_profile_age_and_name.incr();

    if !(permissions.admin_find_account_by_email || permissions.admin_view_permissions) {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let profile_owner_id = state.get_internal_id(account_id).await?;

    let r = state.read().profile().profile(profile_owner_id).await?;
    let r = GetProfileAgeAndName {
        age: r.profile.age,
        name: r.profile.name,
    };

    Ok(r.into())
}

pub fn admin_profile_data_router(s: S) -> OpenApiRouter {
    create_open_api_router!(
        s,
        get_profile_age_and_name,
    )
}

create_counters!(
    ProfileCounters,
    PROFILE,
    PROFILE_ADMIN_PROFILE_DATA_COUNTERS_LIST,
    get_profile_age_and_name,
);
