use axum::{
    Extension,
    extract::{Path, State},
};
use model::{AccountId, Permissions};
use server_api::{
    S,
    app::{GetAccounts, WriteData},
    create_open_api_router, db_write_multiple,
};
use server_data_account::write::GetWriteCommandsAccount;
use simple_backend::create_counters;

use crate::utils::StatusCode;

const PATH_POST_DELETE_ACCOUNT: &str = "/account_api/delete_account/{aid}";

/// Delete account instantly
///
/// # Access
///
/// Permission [model_account::Permissions::admin_delete_account] is required.
#[utoipa::path(
    post,
    path = PATH_POST_DELETE_ACCOUNT,
    params(AccountId),
    responses(
        (status = 200, description = "Successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_delete_account(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Path(account): Path<AccountId>,
) -> Result<(), StatusCode> {
    ACCOUNT_ADMIN.post_delete_account.incr();

    if !permissions.admin_delete_account {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let internal_id = state.get_internal_id(account).await?;

    db_write_multiple!(state, move |cmds| {
        cmds.account().delete().delete_account(internal_id).await?;
        Ok(())
    })?;

    Ok(())
}

create_open_api_router!(fn router_admin_delete, post_delete_account,);

create_counters!(
    AccountCounters,
    ACCOUNT_ADMIN,
    ACCOUNT_ADMIN_DELETE_COUNTERS_LIST,
    post_delete_account,
);
