use axum::{extract::State, Extension};
use model_account::{Account, AccountIdInternal, ClientId, LatestBirthdate};
use server_api::{app::WriteData, create_open_api_router, db_write, S};
use server_data::read::GetReadCommandsCommon;
use server_data_account::write::GetWriteCommandsAccount;
use simple_backend::create_counters;

use crate::{
    app::ReadData,
    utils::{Json, StatusCode},
};

const PATH_ACCOUNT_STATE: &str = "/account_api/state";

/// Get current account state.
#[utoipa::path(
    get,
    path = PATH_ACCOUNT_STATE,
    responses(
        (status = 200, description = "Request successfull.", body = Account),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_account_state(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
) -> Result<Json<Account>, StatusCode> {
    ACCOUNT.get_account_state.incr();
    let account = state.read().common().account(api_caller_account_id).await?;
    Ok(account.into())
}

const PATH_LATEST_BIRTHDATE: &str = "/account_api/latest_birthdate";

#[utoipa::path(
    get,
    path = PATH_LATEST_BIRTHDATE,
    responses(
        (status = 200, description = "Request successfull.", body = LatestBirthdate),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_latest_birthdate(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
) -> Result<Json<LatestBirthdate>, StatusCode> {
    ACCOUNT.get_latest_birthdate.incr();
    let birthdate = state.read().common().latest_birthdate(id).await?;
    let birthdate = LatestBirthdate { birthdate };
    Ok(birthdate.into())
}

const PATH_POST_GET_NEXT_CLIENT_ID: &str = "/account_api/next_client_id";

#[utoipa::path(
    post,
    path = PATH_POST_GET_NEXT_CLIENT_ID,
    responses(
        (status = 200, description = "Successfull.", body = ClientId),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_get_next_client_id(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
) -> Result<Json<ClientId>, StatusCode> {
    ACCOUNT.post_get_next_client_id.incr();

    let client_id = db_write!(state, move |cmds| { cmds.account().get_next_client_id(id) })?;

    Ok(client_id.into())
}

create_open_api_router!(
        fn router_state,
        get_account_state,
        get_latest_birthdate,
        post_get_next_client_id,
);

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_STATE_COUNTERS_LIST,
    get_account_state,
    get_latest_birthdate,
    post_get_next_client_id,
);
