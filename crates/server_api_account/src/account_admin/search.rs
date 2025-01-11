use axum::{extract::{Path, State}, Extension};
use model::Permissions;
use model_account::{GetAccountIdFromEmailParams, GetAccountIdFromEmailResult};
use obfuscate_api_macro::obfuscate_api;
use server_api::{app::ReadData, create_open_api_router, S};
use server_data_account::read::GetReadCommandsAccount;
use simple_backend::create_counters;

use crate::utils::{Json, StatusCode};

#[obfuscate_api]
const PATH_GET_ACCOUNT_ID_FROM_EMAIL: &str = "/account_api/get_account_id_from_email/{email}";

/// Get account ID from email
///
/// # Access
///
/// Permission [model_account::Permissions::admin_find_account_by_email] is required.
#[utoipa::path(
    get,
    path = PATH_GET_ACCOUNT_ID_FROM_EMAIL,
    params(GetAccountIdFromEmailParams),
    responses(
        (status = 200, description = "Successfull.", body = GetAccountIdFromEmailResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_account_id_from_email(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Path(email): Path<GetAccountIdFromEmailParams>,
) -> Result<Json<GetAccountIdFromEmailResult>, StatusCode> {
    ACCOUNT_ADMIN.get_account_id_from_email.incr();

    if !permissions.admin_find_account_by_email {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let r = state.read().account_admin().search().account_id_from_email(email.email).await?;
    Ok(r.into())
}

create_open_api_router!(fn router_admin_search, get_account_id_from_email,);

create_counters!(
    AccountCounters,
    ACCOUNT_ADMIN,
    ACCOUNT_ADMIN_SEARCH_COUNTERS_LIST,
    get_account_id_from_email,
);
