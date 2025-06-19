use axum::{
    Extension,
    extract::{Path, State},
};
use model::{Account, AccountId, Permissions};
use server_api::{
    S,
    app::{GetAccounts, ReadData},
    create_open_api_router,
};
use server_data::read::GetReadCommandsCommon;
use simple_backend::create_counters;

use crate::utils::{Json, StatusCode};

const PATH_GET_ACCOUNT_STATE_ADMIN: &str = "/account_api/get_account_state_admin/{aid}";

/// Get [model::Account] for specific account.
///
/// # Access
///
/// Permission [model::Permissions::admin_view_private_info] is required.
#[utoipa::path(
    get,
    path = PATH_GET_ACCOUNT_STATE_ADMIN,
    params(AccountId),
    responses(
        (status = 200, description = "Successfull.", body = Account),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_account_state_admin(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Path(account_id): Path<AccountId>,
) -> Result<Json<Account>, StatusCode> {
    ACCOUNT_ADMIN.get_account_state_admin.incr();

    if !permissions.admin_view_private_info {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let internal_id = state.get_internal_id(account_id).await?;

    let permissions = state.read().common().account(internal_id).await?;

    Ok(permissions.into())
}

create_open_api_router!(fn router_admin_state, get_account_state_admin,);

create_counters!(
    AccountCounters,
    ACCOUNT_ADMIN,
    ACCOUNT_ADMIN_STATE_COUNTERS_LIST,
    get_account_state_admin,
);
