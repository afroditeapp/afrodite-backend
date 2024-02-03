use axum::{extract::State, Extension, Router};
use model::{
    AccountId, AccountIdInternal, AccountSetup, AccountState, EventToClientInternal, SignInWithInfo,
};
use simple_backend::create_counters;
use tracing::error;

use crate::{
    api::{
        db_write,
        utils::{Json, StatusCode},
    },
    app::{EventManagerProvider, GetAccessTokens, GetConfig, GetInternalApi, ReadData, WriteData},
};

// TODO: Update register and login to support Apple and Google single sign on.

pub const PATH_REGISTER: &str = "/account_api/register";

/// Register new account. Returns new account ID which is UUID.
///
/// Available only if server is running in debug mode and
/// bot_login is enabled from config file.
#[utoipa::path(
    post,
    path = "/account_api/register",
    security(),
    responses(
        (status = 200, description = "New profile created.", body = AccountId),
        (status = 500, description = "Internal server error."),
    )
)]
pub async fn post_register<S: WriteData + GetConfig>(
    State(state): State<S>,
) -> Result<Json<AccountId>, StatusCode> {
    ACCOUNT.post_register.incr();
    register_impl(&state, SignInWithInfo::default())
        .await
        .map(|id| id.into())
}

pub async fn register_impl<S: WriteData + GetConfig>(
    state: &S,
    sign_in_with: SignInWithInfo,
) -> Result<AccountId, StatusCode> {
    // New unique UUID is generated every time so no special handling needed
    // to avoid database collisions.
    let id = AccountId::new(uuid::Uuid::new_v4());

    let result = state
        .write(move |cmds| async move { cmds.register(id, sign_in_with).await })
        .await;

    match result {
        Ok(id) => Ok(id.as_id().into()),
        Err(e) => {
            error!("Error: {e:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
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
    let account = state
        .read()
        .account()
        .account(api_caller_account_id)
        .await?;

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
/// Request to this handler will complete if client is in `initial setup`,
/// setup information is set and image moderation request has been made.
///
#[utoipa::path(
    post,
    path = "/account_api/complete_setup",
    responses(
        (status = 200, description = "Request successfull."),
        (status = 406, description = "Current state is not initial setup, AccountSetup is empty or moderation request does not contain camera image."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_complete_setup<
    S: GetAccessTokens + ReadData + WriteData + GetInternalApi + GetConfig + EventManagerProvider,
>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
) -> Result<(), StatusCode> {
    ACCOUNT.post_complete_setup.incr();
    let account_setup = state.read().account().account_setup(id).await?;

    if account_setup.is_empty() {
        return Err(StatusCode::NOT_ACCEPTABLE);
    }

    state
        .internal_api()
        .media_check_moderation_request_for_account(id)
        .await?;

    let mut account = state.read().account().account(id).await?;
    let account_data = state.read().account().account_data(id).await?;

    let sign_in_with_info = state.read().account().account_sign_in_with_info(id).await?;

    if account.state() == AccountState::InitialSetup {
        // Handle profile related initial setup
        state
            .internal_api()
            .profile_initial_setup(id, account_setup.name().to_string())
            .await?;

        // Handle account related initial setup

        account.complete_setup();

        if state.config().debug_mode() {
            if account_data.email == state.config().admin_email() {
                account.add_admin_capablities();
            }
        } else {
            if let Some(sign_in_with_config) =
                state.config().simple_backend().sign_in_with_google_config()
            {
                if sign_in_with_info.google_account_id
                    == Some(model::GoogleAccountId(
                        sign_in_with_config.admin_google_account_id.clone(),
                    ))
                    && account_data.email == state.config().admin_email()
                {
                    account.add_admin_capablities();
                }
            }
        }

        let new_account_copy = account.clone();
        state
            .internal_api()
            .modify_and_sync_account_state(id, |d| {
                *d.state = new_account_copy.state();
                *d.capabilities = new_account_copy.into_capablities();
            })
            .await?;

        state
            .event_manager()
            .send_connected_event(
                id.uuid,
                EventToClientInternal::AccountStateChanged {
                    state: account.state(),
                },
            )
            .await?;

        state
            .event_manager()
            .send_connected_event(
                id.uuid,
                EventToClientInternal::AccountCapabilitiesChanged {
                    capabilities: account.into_capablities(),
                },
            )
            .await?;

        Ok(())
    } else {
        Err(StatusCode::NOT_ACCEPTABLE)
    }
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
    post_register,
    get_account_setup,
    post_account_setup,
    post_complete_setup,
);
