use axum::{
    Extension,
    extract::{Path, State},
};
use model::{AccountId, AccountIdInternal, Permissions};
use model_account::GetAccountBanTimeResult;
use server_api::{
    S,
    app::{GetAccounts, ReadData},
    create_open_api_router,
};
use server_data_account::read::GetReadCommandsAccount;
use simple_backend::create_counters;

use crate::utils::{Json, StatusCode};

const PATH_GET_ACCOUNT_BAN_TIME: &str = "/account_api/account_ban_time/{aid}";

/// Get account ban time
///
/// # Access
/// - Account owner
/// - Permission [model::Permissions::admin_ban_account]
#[utoipa::path(
    get,
    path = PATH_GET_ACCOUNT_BAN_TIME,
    params(AccountId),
    responses(
        (status = 200, description = "Successfull.", body = GetAccountBanTimeResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_account_ban_time(
    State(state): State<S>,
    Extension(api_caller): Extension<AccountIdInternal>,
    Extension(permissions): Extension<Permissions>,
    Path(account): Path<AccountId>,
) -> Result<Json<GetAccountBanTimeResult>, StatusCode> {
    ACCOUNT.get_account_ban_time.incr();

    if account != api_caller.as_id() && !permissions.admin_ban_account {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let internal_id = state.get_internal_id(account).await?;

    let result = state.read().account().ban().ban_time(internal_id).await?;

    Ok(result.into())
}

create_open_api_router!(fn router_ban, get_account_ban_time,);

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_BAN_COUNTERS_LIST,
    get_account_ban_time,
);
