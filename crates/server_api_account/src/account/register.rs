use axum::{extract::State, Extension, Router};
use model::{AccountIdInternal, AccountSetup, AccountState, SetAccountSetup};
use server_api::app::{CompleteInitialSetupCmd, ValidateModerationRequest};
use server_data_account::{
    read::GetReadCommandsAccount,
    write::GetWriteCommandsAccount,
};
use simple_backend::create_counters;

use crate::{
    app::{GetAccessTokens, GetConfig, GetInternalApi, ReadData, StateBase, WriteData},
    db_write_multiple, internal_api,
    utils::{Json, StatusCode},
};

// TODO: Update register and login to support Apple and Google single sign on.

pub const PATH_GET_ACCOUNT_SETUP: &str = "/account_api/account_setup";

/// Get non-changeable user information to account.
#[utoipa::path(
    get,
    path = "/account_api/account_setup",
    responses(
        (status = 200, description = "Request successfull.", body = AccountSetup),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_account_setup<S: GetAccessTokens + ReadData + WriteData>(
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

pub const PATH_POST_ACCOUNT_SETUP: &str = "/account_api/account_setup";

/// Setup non-changeable user information during `initial setup` state.
#[utoipa::path(
    post,
    path = "/account_api/account_setup",
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
pub async fn post_account_setup<S: GetConfig + GetInternalApi + WriteData>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Extension(account_state): Extension<AccountState>,
    Json(data): Json<SetAccountSetup>,
) -> Result<(), StatusCode> {
    ACCOUNT.post_account_setup.incr();

    if account_state == AccountState::InitialSetup {
        if data.is_invalid() {
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

pub const PATH_ACCOUNT_COMPLETE_SETUP: &str = "/account_api/complete_setup";

/// Complete initial setup.
///
/// Requirements:
///  - Account must be in `InitialSetup` state.
///  - Account must have a valid AccountSetup info set.
///  - Account must have a moderation request.
///  - The current or pending security image of the account is in the request.
///  - The current or pending first profile image of the account is in the
///    request.
///
#[utoipa::path(
    post,
    path = "/account_api/complete_setup",
    responses(
        (status = 200, description = "Request successfull."),
        (status = 406, description = "Current state is not initial setup."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error or current state is invalid for Normal state."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_complete_setup<
    S: CompleteInitialSetupCmd,
>(
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

    state.complete_initial_setup(id).await?;

    Ok(())
}

/// Contains only routes which require authentication.
pub fn register_router<
    S: StateBase
        + ReadData
        + WriteData
        + GetInternalApi
        + GetConfig
        + GetAccessTokens
        + ValidateModerationRequest
        + CompleteInitialSetupCmd,
>(
    s: S,
) -> Router {
    use axum::routing::{get, post};

    Router::new()
        // Skip PATH_REGISTER because it does not need authentication.
        .route(PATH_GET_ACCOUNT_SETUP, get(get_account_setup::<S>))
        .route(PATH_POST_ACCOUNT_SETUP, post(post_account_setup::<S>))
        .route(PATH_ACCOUNT_COMPLETE_SETUP, post(post_complete_setup::<S>))
        .with_state(s)
}

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_REGISTER_COUNTERS_LIST,
    get_account_setup,
    post_account_setup,
    post_complete_setup,
);
