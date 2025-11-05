use axum::{
    Extension,
    extract::{Path, State},
};
use model::{AccountId, Permissions};
use server_api::{
    S,
    app::{GetAccounts, WriteData},
    create_open_api_router, db_write,
};
use server_data::write::GetWriteCommandsCommon;
use simple_backend::create_counters;

use crate::utils::StatusCode;

const PATH_POST_ADMIN_LOGOUT: &str = "/account_api/admin_logout/{aid}";

/// Logout any account
///
/// # Access
///
/// Permission [model::Permissions::admin_edit_login] is required.
#[utoipa::path(
    post,
    path = PATH_POST_ADMIN_LOGOUT,
    params(AccountId),
    responses(
        (status = 200, description = "Successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_admin_logout(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Path(account_id): Path<AccountId>,
) -> Result<(), StatusCode> {
    ACCOUNT_ADMIN.post_admin_logout.incr();

    if !permissions.admin_edit_login {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let internal_id = state.get_internal_id(account_id).await?;

    db_write!(state, move |cmds| {
        cmds.common().logout(internal_id).await
    })?;

    Ok(())
}

create_open_api_router!(fn router_admin_logout, post_admin_logout,);

create_counters!(
    AccountAdminCounters,
    ACCOUNT_ADMIN,
    ACCOUNT_ADMIN_LOGOUT_COUNTERS_LIST,
    post_admin_logout,
);
