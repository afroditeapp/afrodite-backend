use axum::{
    Extension,
    extract::{Query, State},
};
use model_profile::{AccountIdDbValue, Permissions, ProfileIteratorPage, ProfileIteratorSettings};
use server_api::{S, create_open_api_router};
use server_data_profile::read::GetReadProfileCommands;
use simple_backend::create_counters;

use crate::{
    app::ReadData,
    utils::{Json, StatusCode},
};

const PATH_GET_LATEST_CREATED_ACCOUNT_ID_DB: &str = "/profile_api/get_latest_created_account_id_db";

/// Get latest created account ID DB
///
/// # Access
/// - Permission [model::Permissions::admin_view_all_profiles]
#[utoipa::path(
    get,
    path = PATH_GET_LATEST_CREATED_ACCOUNT_ID_DB,
    responses(
        (status = 200, description = "Successful.", body = AccountIdDbValue),
        (status = 401, description = "Unauthorized."),
        (
            status = 500,
            description = "Internal server error.",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn get_latest_created_account_id_db(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
) -> Result<Json<AccountIdDbValue>, StatusCode> {
    PROFILE.get_latest_created_account_id_db.incr();

    if !permissions.admin_view_all_profiles {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let r = state
        .read()
        .profile_admin()
        .iterator()
        .get_latest_created_account_id_db()
        .await?;

    Ok(r.into())
}

const PATH_GET_ADMIN_PROFILE_ITERATOR_PAGE: &str = "/profile_api/get_admin_profile_iterator_page";

/// Get admin profile iterator page
///
/// # Access
/// - Permission [model::Permissions::admin_view_all_profiles]
#[utoipa::path(
    get,
    path = PATH_GET_ADMIN_PROFILE_ITERATOR_PAGE,
    params(ProfileIteratorSettings),
    responses(
        (status = 200, description = "Successful.", body = ProfileIteratorPage),
        (status = 401, description = "Unauthorized."),
        (
            status = 500,
            description = "Internal server error.",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn get_admin_profile_iterator_page(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Query(settings): Query<ProfileIteratorSettings>,
) -> Result<Json<ProfileIteratorPage>, StatusCode> {
    PROFILE.get_admin_profile_iterator_page.incr();

    if !permissions.admin_view_all_profiles {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let r = state
        .read()
        .profile_admin()
        .iterator()
        .get_profile_page(settings)
        .await?;

    Ok(r.into())
}

create_open_api_router!(
        fn router_admin_iterate_profiles,
        get_latest_created_account_id_db,
        get_admin_profile_iterator_page,
);

create_counters!(
    ProfileCounters,
    PROFILE,
    PROFILE_ADMIN_ITERATE_PROFILES_COUNTERS_LIST,
    get_latest_created_account_id_db,
    get_admin_profile_iterator_page,
);
