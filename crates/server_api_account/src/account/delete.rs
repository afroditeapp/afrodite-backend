use axum::{extract::{Path, State}, Extension};
use model::{AccountId, AccountIdInternal, EventToClientInternal, Permissions};
use model_account::{BooleanSetting, GetAccountDeletionRequestResult};
use obfuscate_api_macro::obfuscate_api;
use server_api::{app::{GetAccounts, WriteData, ReadData}, create_open_api_router, db_write_multiple, S};
use server_data_account::{read::GetReadCommandsAccount, write::GetWriteCommandsAccount};
use simple_backend::create_counters;

use crate::utils::{Json, StatusCode};

#[obfuscate_api]
const PATH_POST_SET_ACCOUNT_DELETION_REQUEST_STATE: &str = "/account_api/set_account_deletion_request_state/{aid}";

/// Request account deletion or cancel the deletion
///
/// # Access
/// - Account owner
/// - Permission [model_account::Permissions::admin_request_account_deletion]
#[utoipa::path(
    post,
    path = PATH_POST_SET_ACCOUNT_DELETION_REQUEST_STATE,
    request_body = BooleanSetting,
    params(AccountId),
    responses(
        (status = 200, description = "Successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_set_account_deletion_request_state(
    State(state): State<S>,
    Extension(api_caller): Extension<AccountIdInternal>,
    Extension(permissions): Extension<Permissions>,
    Path(account): Path<AccountId>,
    Json(value): Json<BooleanSetting>,
) -> Result<(), StatusCode> {
    ACCOUNT.post_set_account_deletion_request_state.incr();

    if account != api_caller.as_id() && !permissions.admin_request_account_deletion {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let internal_id = state.get_internal_id(account).await?;

    db_write_multiple!(state, move |cmds| {
        let new_account = cmds.account().delete().set_account_deletion_request_state(internal_id, value.value).await?;

        if new_account.is_some() {
            cmds.events()
                .send_connected_event(
                    internal_id.uuid,
                    EventToClientInternal::AccountStateChanged,
                )
                .await?;
        }

        Ok(())
    })?;

    Ok(())
}

#[obfuscate_api]
const PATH_GET_ACCOUNT_DELETION_REQUEST_STATE: &str = "/account_api/get_account_deletion_request_state/{aid}";

/// Get account deletion request state
///
/// # Access
/// - Account owner
/// - Permission [model_account::Permissions::admin_request_account_deletion]
#[utoipa::path(
    get,
    path = PATH_GET_ACCOUNT_DELETION_REQUEST_STATE,
    params(AccountId),
    responses(
        (status = 200, description = "Successfull.", body = GetAccountDeletionRequestResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_account_deletion_request_state(
    State(state): State<S>,
    Extension(api_caller): Extension<AccountIdInternal>,
    Extension(permissions): Extension<Permissions>,
    Path(account): Path<AccountId>,
) -> Result<Json<GetAccountDeletionRequestResult>, StatusCode> {
    ACCOUNT.get_account_deletion_request_state.incr();

    if account != api_caller.as_id() && !permissions.admin_request_account_deletion {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let internal_id = state.get_internal_id(account).await?;

    let result = state.read().account().delete().account_deleteion_state(internal_id).await?;

    Ok(result.into())
}

create_open_api_router!(fn router_delete, post_set_account_deletion_request_state, get_account_deletion_request_state,);

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_DELETE_COUNTERS_LIST,
    post_set_account_deletion_request_state,
    get_account_deletion_request_state,
);
