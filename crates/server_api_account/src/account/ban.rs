use axum::{extract::State, Extension};
use model::AccountIdInternal;
use model_account::GetAccountBanTimeResult;
use obfuscate_api_macro::obfuscate_api;
use server_api::{app::ReadData, create_open_api_router, S};
use server_data_account::read::GetReadCommandsAccount;
use simple_backend::create_counters;
use utoipa_axum::router::OpenApiRouter;

use crate::utils::{Json, StatusCode};

#[obfuscate_api]
const PATH_GET_ACCOUNT_BAN_TIME: &str = "/account_api/account_ban_time";

#[utoipa::path(
    get,
    path = PATH_GET_ACCOUNT_BAN_TIME,
    responses(
        (status = 200, description = "Successfull.", body = GetAccountBanTimeResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_account_ban_time(
    State(state): State<S>,
    Extension(account): Extension<AccountIdInternal>,
) -> Result<Json<GetAccountBanTimeResult>, StatusCode> {
    ACCOUNT.get_account_ban_time.incr();

    let result = state.read().account().ban().ban_time(account).await?;

    Ok(result.into())
}

pub fn ban_router(s: S) -> OpenApiRouter {
    create_open_api_router!(s, get_account_ban_time,)
}

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_BAN_COUNTERS_LIST,
    get_account_ban_time,
);
