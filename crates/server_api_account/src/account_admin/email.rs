use axum::{
    Extension,
    extract::{Path, State},
};
use model::{AccountId, Permissions};
use model_account::GetEmailLoginEnabled;
use server_api::{S, create_open_api_router};
use server_data_account::read::GetReadCommandsAccount;
use simple_backend::create_counters;

use crate::{
    app::{GetAccounts, ReadData},
    utils::{Json, StatusCode},
};

pub const PATH_GET_EMAIL_LOGIN_ENABLED: &str = "/account_api/get_email_login_enabled/{aid}";

/// Get the current email login enabled status for an account.
///
/// Requires `admin_edit_login` permission.
#[utoipa::path(
    get,
    path = PATH_GET_EMAIL_LOGIN_ENABLED,
    params(AccountId),
    responses(
        (status = 200, description = "Successfull.", body = GetEmailLoginEnabled),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_email_login_enabled(
    State(state): State<S>,
    Extension(api_caller_permissions): Extension<Permissions>,
    Path(target_account): Path<AccountId>,
) -> Result<Json<GetEmailLoginEnabled>, StatusCode> {
    ACCOUNT_ADMIN.get_email_login_enabled.incr();

    if !api_caller_permissions.admin_edit_login {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let target_account_internal = state.get_internal_id(target_account).await?;

    let account_internal = state
        .read()
        .account()
        .account_internal(target_account_internal)
        .await?;

    Ok(Json(GetEmailLoginEnabled {
        enabled: account_internal.email_login_enabled,
    }))
}

create_open_api_router!(
    fn router_admin_email,
    get_email_login_enabled,
);

create_counters!(
    AccountAdminCounters,
    ACCOUNT_ADMIN,
    ACCOUNT_ADMIN_EMAIL_COUNTERS_LIST,
    get_email_login_enabled,
);
