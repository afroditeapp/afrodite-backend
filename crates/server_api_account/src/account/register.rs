use axum::{extract::State, Extension};
use model_account::{AccountIdInternal, AccountSetup, AccountState, SetAccountSetup};
use server_api::{create_open_api_router, S};
use server_data_account::{read::GetReadCommandsAccount, write::GetWriteCommandsAccount};
use simple_backend::create_counters;

use crate::{
    app::{ReadData, WriteData},
    db_write_multiple, internal_api,
    utils::{Json, StatusCode},
};

// TODO: Update register and login to support Apple and Google single sign on.

const PATH_GET_ACCOUNT_SETUP: &str = "/account_api/account_setup";

/// Get non-changeable user information to account.
#[utoipa::path(
    get,
    path = PATH_GET_ACCOUNT_SETUP,
    responses(
        (status = 200, description = "Request successfull.", body = AccountSetup),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_account_setup(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
) -> Result<Json<AccountSetup>, StatusCode> {
    ACCOUNT.get_account_setup.incr();
    let data = state
        .read()
        .account()
        .account_setup(api_caller_account_id)
        .await?;
    Ok(data.into())
}

const PATH_POST_ACCOUNT_SETUP: &str = "/account_api/account_setup";

/// Setup non-changeable user information during `initial setup` state.
#[utoipa::path(
    post,
    path = PATH_POST_ACCOUNT_SETUP,
    request_body(content = SetAccountSetup),
    responses(
        (status = 200, description = "Request successfull."),
        (status = 406, description = "Current state is not initial setup or setup data is invalid"),
        (status = 401, description = "Unauthorized."),
        (
            status = 500,
            description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_account_setup(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Extension(account_state): Extension<AccountState>,
    Json(data): Json<SetAccountSetup>,
) -> Result<(), StatusCode> {
    ACCOUNT.post_account_setup.incr();

    if account_state == AccountState::InitialSetup {
        if !data.is_valid() {
            return Err(StatusCode::NOT_ACCEPTABLE);
        }

        // TODO(microservice): Add mutex to avoid data races
        internal_api::common::sync_birthdate(&state, id).await?;

        db_write_multiple!(state, move |cmds| {
            cmds.account().account_setup(id, data).await
        })?;

        Ok(())
    } else {
        Err(StatusCode::NOT_ACCEPTABLE)
    }
}

const PATH_ACCOUNT_COMPLETE_SETUP: &str = "/account_api/complete_setup";

/// Complete initial setup.
///
/// Requirements:
///  - Account must be in `InitialSetup` state.
///  - Account must have a valid AccountSetup info set.
///
#[utoipa::path(
    post,
    path = PATH_ACCOUNT_COMPLETE_SETUP,
    responses(
        (status = 200, description = "Request successfull."),
        (status = 406, description = "Current state is not initial setup."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error or current state is invalid for Normal state."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_complete_setup(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Extension(account_state): Extension<AccountState>,
) -> Result<(), StatusCode> {
    ACCOUNT.post_complete_setup.incr();

    // Initial account state check. The complete setup implementation has
    // another which handles race conditions.
    if account_state != AccountState::InitialSetup {
        return Err(StatusCode::NOT_ACCEPTABLE);
    }

    let new_account = state.data_all_access().complete_initial_setup(id).await?;

    // TODO(microservice): initial setup completed time sync
    internal_api::common::sync_account_state(&state, id, new_account).await?;

    Ok(())
}

create_open_api_router!(
        /// Contains only routes which require authentication.
        fn router_register,
        get_account_setup,
        post_account_setup,
        post_complete_setup,
);

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_REGISTER_COUNTERS_LIST,
    get_account_setup,
    post_account_setup,
    post_complete_setup,
);
