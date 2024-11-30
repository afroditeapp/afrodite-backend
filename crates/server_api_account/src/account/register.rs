use axum::{extract::State, Extension};
use model_account::{AccountIdInternal, AccountSetup, AccountState, SetAccountSetup};
use obfuscate_api_macro::obfuscate_api;
use server_api::{app::ValidateModerationRequest, create_open_api_router, S};
use server_data_account::{read::GetReadCommandsAccount, write::GetWriteCommandsAccount};
use simple_backend::create_counters;
use utoipa_axum::router::OpenApiRouter;

use crate::{
    app::{ReadData, WriteData},
    db_write_multiple, internal_api,
    utils::{Json, StatusCode},
};

// TODO: Update register and login to support Apple and Google single sign on.

#[obfuscate_api]
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

#[obfuscate_api]
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

#[obfuscate_api]
const PATH_ACCOUNT_COMPLETE_SETUP: &str = "/account_api/complete_setup";

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

    // Validate media moderation.
    // Moderation request creation also validates that the initial request
    // contains security content, so there is no possibility that user
    // changes the request to be invalid just after this check.
    state.media_check_moderation_request_for_account(id).await?;

    let new_account = state.data_all_access().complete_initial_setup(id).await?;

    internal_api::common::sync_account_state(&state, id, new_account).await?;

    Ok(())
}

/// Contains only routes which require authentication.
pub fn register_router(s: S) -> OpenApiRouter {
    create_open_api_router!(
        s,
        get_account_setup,
        post_account_setup,
        post_complete_setup,
    )
}

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_REGISTER_COUNTERS_LIST,
    get_account_setup,
    post_account_setup,
    post_complete_setup,
);
