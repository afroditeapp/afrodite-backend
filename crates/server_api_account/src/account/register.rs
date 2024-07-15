use axum::{extract::State, Extension, Router};
use model::{AccountIdInternal, AccountSetup, AccountState, Capabilities, EmailMessages, EventToClientInternal, SetAccountSetup};
use server_api::{app::ValidateModerationRequest, result::WrappedContextExt, DataError};
use server_data_account::{
    read::GetReadCommandsAccount,
    write::{account::IncrementAdminAccessGrantedCount, GetWriteCommandsAccount},
};
use simple_backend::create_counters;
use tracing::warn;

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
    S: ReadData + WriteData + GetInternalApi + GetConfig + ValidateModerationRequest,
>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Extension(account_state): Extension<AccountState>,
) -> Result<(), StatusCode> {
    ACCOUNT.post_complete_setup.incr();

    // Initial account state check
    if account_state != AccountState::InitialSetup {
        return Err(StatusCode::NOT_ACCEPTABLE);
    }

    // Validate media moderation.
    // Moderation request creation also validates that the initial request
    // contains security content, so there is no possibility that user
    // changes the request to be invalid just after this check.
    state.media_check_moderation_request_for_account(id).await?;

    let account_data = state.read().account().account_data(id).await?;
    let sign_in_with_info = state.read().account().account_sign_in_with_info(id).await?;
    let (matches_with_grant_admin_access_config, grant_admin_access_more_than_once) =
        if let Some(grant_admin_access_config) = state.config().grant_admin_access_config() {
            let matches = match (
                grant_admin_access_config.email.as_ref(),
                grant_admin_access_config.google_account_id.as_ref(),
            ) {
                (wanted_email @ Some(_), Some(wanted_google_account_id)) => {
                    wanted_email == account_data.email.as_ref()
                        && sign_in_with_info
                            .google_account_id_matches_with(wanted_google_account_id)
                }
                (wanted_email @ Some(_), None) => wanted_email == account_data.email.as_ref(),
                (None, Some(wanted_google_account_id)) => {
                    sign_in_with_info.google_account_id_matches_with(wanted_google_account_id)
                }
                (None, None) => false,
            };

            (
                matches,
                grant_admin_access_config.for_every_matching_new_account,
            )
        } else {
            (false, false)
        };

    let is_bot_account = state.read().account().is_bot_account(id).await?;

    let new_account = db_write_multiple!(state, move |cmds| {
        // Second account state check as db_write quarantees synchronous
        // access.
        let account_state = cmds.read().common().account(id).await?.state();
        if account_state != AccountState::InitialSetup {
            return Err(DataError::NotAllowed.report());
        }

        let account_setup = cmds.read().account().account_setup(id).await?;
        if account_setup.is_invalid() {
            return Err(DataError::NotAllowed.report());
        }

        let global_state = cmds.read().account().global_state().await?;
        let enable_all_capabilities = if matches_with_grant_admin_access_config
            && (global_state.admin_access_granted_count == 0 || grant_admin_access_more_than_once)
        {
            Some(IncrementAdminAccessGrantedCount)
        } else {
            None
        };

        let new_account = cmds
            .account()
            .update_syncable_account_data(
                id,
                enable_all_capabilities,
                move |state, capabilities, _| {
                    if *state == AccountState::InitialSetup {
                        *state = AccountState::Normal;
                        if enable_all_capabilities.is_some() {
                            warn!("Account detected as admin account. Enabling all capabilities");
                            *capabilities = Capabilities::all_enabled();
                        }
                    }
                    Ok(())
                },
            )
            .await?;

        if !is_bot_account && !sign_in_with_info.some_sign_in_with_method_is_set() {
            // Account registered email is not yet sent if email address
            // was provided manually and not from some sign in with method.
            cmds.account().email().send_email_if_not_already_sent(id, EmailMessages::AccountRegistered).await?;
        }

        cmds.events()
            .send_connected_event(
                id.uuid,
                EventToClientInternal::AccountStateChanged(new_account.state()),
            )
            .await?;

        cmds.events()
            .send_connected_event(
                id.uuid,
                EventToClientInternal::AccountCapabilitiesChanged(new_account.capablities()),
            )
            .await?;

        Ok(new_account)
    })?;

    internal_api::common::sync_account_state(&state, id, new_account).await?;

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
        + ValidateModerationRequest,
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
