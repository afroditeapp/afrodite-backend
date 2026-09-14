use axum::{
    Extension,
    extract::{Path, State},
};
use model::{AccountId, Permissions};
use model_account::{AccountLockedState, GetAccountLoginSessionInfo};
use server_api::{
    S,
    app::{GetAccounts, ReadData, WriteData},
    create_open_api_router, db_write,
};
use server_data_account::{read::GetReadCommandsAccount, write::GetWriteCommandsAccount};
use simple_backend::create_counters;

use crate::utils::{Json, StatusCode};

const PATH_GET_ACCOUNT_LOCKED_STATE: &str = "/account_api/get_account_locked_state/{aid}";

/// Get account locked state
///
/// # Access
///
/// Permission [model::Permissions::admin_view_login] is required.
#[utoipa::path(
    get,
    path = PATH_GET_ACCOUNT_LOCKED_STATE,
    params(AccountId),
    responses(
        (status = 200, description = "Successful.", body = AccountLockedState),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_account_locked_state(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Path(account_id): Path<AccountId>,
) -> Result<Json<AccountLockedState>, StatusCode> {
    ACCOUNT_ADMIN.get_account_locked_state.incr();

    if !permissions.admin_view_login {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let internal_id = state.get_internal_id(account_id).await?;

    let result = state
        .read()
        .account_admin()
        .login()
        .account_locked_state(internal_id)
        .await?;

    Ok(result.into())
}

const PATH_POST_SET_ACCOUNT_LOCKED_STATE: &str = "/account_api/set_account_locked_state/{aid}";

/// Set account locked state
///
/// # Access
///
/// Permission [model::Permissions::admin_edit_login] is required.
#[utoipa::path(
    post,
    path = PATH_POST_SET_ACCOUNT_LOCKED_STATE,
    params(AccountId),
    request_body = AccountLockedState,
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_set_account_locked_state(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Path(account_id): Path<AccountId>,
    Json(locked_state): Json<AccountLockedState>,
) -> Result<(), StatusCode> {
    ACCOUNT_ADMIN.post_set_account_locked_state.incr();

    if !permissions.admin_edit_login {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let internal_id = state.get_internal_id(account_id).await?;

    db_write!(state, move |cmds| {
        cmds.account_admin()
            .login()
            .set_locked_state(internal_id, locked_state.locked)
            .await
    })?;

    Ok(())
}

const PATH_GET_ACCOUNT_LOGIN_SESSION_INFO: &str =
    "/account_api/get_account_login_session_info/{aid}";

/// Get login session info for specific account.
///
/// This includes the client platform and app attestation details of the
/// last login session.
///
/// # Access
///
/// Permission [model::Permissions::admin_view_login] is required.
#[utoipa::path(
    get,
    path = PATH_GET_ACCOUNT_LOGIN_SESSION_INFO,
    params(AccountId),
    responses(
        (status = 200, description = "Successful.", body = GetAccountLoginSessionInfo),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_account_login_session_info(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Path(account_id): Path<AccountId>,
) -> Result<Json<GetAccountLoginSessionInfo>, StatusCode> {
    ACCOUNT_ADMIN.get_account_login_session_info.incr();

    if !permissions.admin_view_login {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let internal_id = state.get_internal_id(account_id).await?;

    let info = state
        .read()
        .account_admin()
        .login()
        .login_session_info(internal_id)
        .await?;

    Ok(GetAccountLoginSessionInfo { info }.into())
}

create_open_api_router!(
    fn router_admin_login,
    get_account_locked_state,
    post_set_account_locked_state,
    get_account_login_session_info,
);

create_counters!(
    AccountAdminCounters,
    ACCOUNT_ADMIN,
    ACCOUNT_ADMIN_LOGIN_COUNTERS_LIST,
    get_account_locked_state,
    post_set_account_locked_state,
    get_account_login_session_info,
);
