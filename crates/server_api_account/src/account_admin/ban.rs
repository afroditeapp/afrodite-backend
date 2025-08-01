use axum::{Extension, extract::State};
use model::{AccountIdInternal, EventToClientInternal, Permissions};
use model_account::SetAccountBanState;
use server_api::{
    S,
    app::{GetAccounts, WriteData},
    create_open_api_router, db_write,
};
use server_data_account::write::GetWriteCommandsAccount;
use simple_backend::create_counters;

use crate::utils::{Json, StatusCode};

const PATH_POST_SET_BAN_STATE: &str = "/account_api/set_ban_state";

/// Ban or unban account
///
/// # Access
///
/// Permission [model_account::Permissions::admin_ban_account] is required.
#[utoipa::path(
    post,
    path = PATH_POST_SET_BAN_STATE,
    request_body = SetAccountBanState,
    responses(
        (status = 200, description = "Successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_set_ban_state(
    State(state): State<S>,
    Extension(api_caller_id): Extension<AccountIdInternal>,
    Extension(permissions): Extension<Permissions>,
    Json(ban_info): Json<SetAccountBanState>,
) -> Result<(), StatusCode> {
    ACCOUNT_ADMIN.post_set_ban_state.incr();

    if !permissions.admin_ban_account {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let internal_id = state.get_internal_id(ban_info.account).await?;

    db_write!(state, move |cmds| {
        let new_account = cmds
            .account_admin()
            .ban()
            .set_account_ban_state(
                internal_id,
                Some(api_caller_id),
                ban_info.ban_until,
                ban_info.reason_category,
                ban_info.reason_details,
            )
            .await?;

        if new_account.is_some() {
            cmds.events()
                .send_connected_event(internal_id.uuid, EventToClientInternal::AccountStateChanged)
                .await?;
        }

        Ok(())
    })?;

    Ok(())
}

create_open_api_router!(fn router_admin_ban, post_set_ban_state,);

create_counters!(
    AccountCounters,
    ACCOUNT_ADMIN,
    ACCOUNT_ADMIN_BAN_COUNTERS_LIST,
    post_set_ban_state,
);
