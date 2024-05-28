use axum::{extract::State, Extension, Router};
use model::{
    AccountId, AccountIdInternal, AccountSetup, AccountState, Capabilities, EmailAddress,
    EventToClientInternal, SignInWithInfo,
};
use simple_backend::create_counters;
use tracing::warn;

use crate::{
    api::{
        db_write, db_write_multiple,
        utils::{Json, StatusCode},
    },
    app::{GetAccessTokens, GetConfig, GetInternalApi, ReadData, WriteData},
    data::write::account::IncrementAdminAccessGrantedCount,
    internal_api,
};

// TODO: Update register and login to support Apple and Google single sign on.

pub async fn register_impl<S: WriteData>(
    state: &S,
    sign_in_with: SignInWithInfo,
    email: Option<EmailAddress>,
) -> Result<AccountIdInternal, StatusCode> {
    // New unique UUID is generated every time so no special handling needed
    // to avoid database collisions.
    let id = AccountId::new(uuid::Uuid::new_v4());

    let id = state
        .write(move |cmds| async move { cmds.register(id, sign_in_with, email).await })
        .await?;

    Ok(id)
}

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
    request_body(content = AccountSetup),
    responses(
        (status = 200, description = "Request successfull."),
        (status = 406, description = "Current state is not initial setup."),
        (status = 401, description = "Unauthorized."),
        (
            status = 500,
            description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_account_setup<S: GetAccessTokens + ReadData + WriteData>(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Json(data): Json<AccountSetup>,
) -> Result<(), StatusCode> {
    ACCOUNT.post_account_setup.incr();
    let account = state.read().common().account(api_caller_account_id).await?;

    if account.state() == AccountState::InitialSetup {
        db_write!(state, move |cmds| cmds
            .account()
            .account_setup(api_caller_account_id, data))
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
pub async fn post_complete_setup<S: ReadData + WriteData + GetInternalApi + GetConfig>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Extension(account_state): Extension<AccountState>,
) -> Result<(), StatusCode> {
    ACCOUNT.post_complete_setup.incr();

    if account_state != AccountState::InitialSetup {
        return Err(StatusCode::NOT_ACCEPTABLE);
    }

    let account_setup = state.read().account().account_setup(id).await?;
    if account_setup.is_invalid() {
        return Err(StatusCode::NOT_ACCEPTABLE);
    }

    // Validate media moderation.
    // Moderation request creation also validates that the initial request
    // contains security content, so there is no possibility that user
    // changes the request to be invalid just after this check.
    internal_api::media::media_check_moderation_request_for_account(&state, id).await?;

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

    let new_account = db_write_multiple!(state, move |cmds| {
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
pub fn register_router(s: crate::app::S) -> Router {
    use axum::routing::{get, post};

    use crate::app::S;

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
