use axum::{Extension, extract::State};
use model::Permissions;
use model_profile::{AccountIdDbValue, ProfileIteratorPage, ProfileIteratorSettings};
use server_api::{S, create_open_api_router};
use server_data_profile::read::GetReadProfileCommands;
use simple_backend::{app::MaxMindDbDataProvider, create_counters};

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
/// HTTP method is POST to allow JSON request body for filters and extra
/// info settings.
///
/// Filter values and extra info which the caller does not have permission
/// to use are silently ignored.
///
/// # Access
/// - Permission [model::Permissions::admin_view_all_profiles]
#[utoipa::path(
    post,
    path = PATH_GET_ADMIN_PROFILE_ITERATOR_PAGE,
    request_body = ProfileIteratorSettings,
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
    Json(settings): Json<ProfileIteratorSettings>,
) -> Result<Json<ProfileIteratorPage>, StatusCode> {
    PROFILE.get_admin_profile_iterator_page.incr();

    if !permissions.admin_view_all_profiles {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let mut settings = settings;

    // Remove filters and extra info which the caller does not have
    // permission to use.

    let can_view_account_state = permissions.admin_view_account_state;
    let can_view_email = permissions.admin_view_email_address;
    let can_view_login = permissions.admin_view_login;
    let can_view_ip_usage = permissions.admin_view_account_ip_address_usage;

    if let Some(filters) = settings.filters.as_mut() {
        if !can_view_account_state {
            filters.account_state = None;
            filters.profile_visibility = None;
            filters.account_online = None;
        }
        if !can_view_login {
            filters.login_session_basic_app_attestation_validity = None;
        }
        if !can_view_ip_usage {
            filters.login_session_ip_country_is_not = None;
        }
    }

    if let Some(extra_info) = settings.extra_info.as_mut() {
        if !can_view_account_state {
            extra_info.account_state = false;
            extra_info.account_online = false;
            extra_info.profile_visibility = false;
        }
        if !can_view_email {
            extra_info.email_address = false;
        }
        if !can_view_login {
            extra_info.login_session_client_type = false;
            extra_info.login_session_login_method = false;
            extra_info.login_session_app_attestation_info = false;
        }
        if !can_view_ip_usage {
            extra_info.login_session_ip_country = false;
        }
    }

    let ip_db = if can_view_ip_usage
        && (settings.extra_info().login_session_ip_country
            || settings
                .filters
                .as_ref()
                .and_then(|f| f.login_session_ip_country_is_not.as_ref())
                .is_some())
    {
        state.maxmind_db().current_db().await
    } else {
        None
    };

    let r = state
        .read()
        .profile_admin()
        .iterator()
        .get_profile_page(settings, ip_db)
        .await?;

    Ok(Json(r))
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
