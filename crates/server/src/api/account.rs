use axum::Extension;
use model::{
    AccessToken, Account, AccountId, AccountIdInternal, AccountSetup, AccountState, AuthPair,
    BooleanSetting, DeleteStatus, GoogleAccountId, LoginResult, RefreshToken, SignInWithInfo,
    SignInWithLoginInfo, EventToClient, EventToClientInternal, AccountData,
};
use tracing::error;

use crate::{event, perf::ACCOUNT};

use super::{
    db_write,
    utils::{Json, StatusCode},
    GetAccessTokens, GetAccounts, GetConfig, GetInternalApi, ReadData, SignInWith, WriteData, EventManagerProvider,
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
    state: S,
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

pub const PATH_LOGIN: &str = "/account_api/login";

/// Get new AccessToken.
///
/// Available only if server is running in debug mode and
/// bot_login is enabled from config file.
#[utoipa::path(
    post,
    path = "/account_api/login",
    security(),
    request_body = AccountId,
    responses(
        (status = 200, description = "Login successful.", body = LoginResult),
        (status = 500, description = "Internal server error."),
    ),
)]
pub async fn post_login<S: GetAccessTokens + WriteData + GetAccounts>(
    Json(id): Json<AccountId>,
    state: S,
) -> Result<Json<LoginResult>, StatusCode> {
    ACCOUNT.post_login.incr();
    login_impl(id, state).await.map(|d| d.into())
}

async fn login_impl<S: GetAccessTokens + WriteData + GetAccounts>(
    id: AccountId,
    state: S,
) -> Result<LoginResult, StatusCode> {
    let id = state.accounts().get_internal_id(id).await?;

    let access = AccessToken::generate_new();
    let refresh = RefreshToken::generate_new();
    let account = AuthPair { access, refresh };
    let account_clone = account.clone();

    db_write!(state, move |cmds| cmds.common().set_new_auth_pair(
        id,
        account_clone,
        None
    ))?;

    // TODO: microservice support

    let result = LoginResult {
        account,
        profile: None,
        media: None,
    };
    Ok(result.into())
}

pub const PATH_SIGN_IN_WITH_LOGIN: &str = "/account_api/sign_in_with_login";

/// Start new session with sign in with Apple or Google. Creates new account if
/// it does not exists.
#[utoipa::path(
    post,
    path = "/account_api/sign_in_with_login",
    security(),
    request_body = SignInWithLoginInfo,
    responses(
        (status = 200, description = "Login or account creation successful.", body = LoginResult),
        (status = 500, description = "Internal server error."),
    ),
)]
pub async fn post_sign_in_with_login<
    S: GetAccessTokens + WriteData + ReadData + GetAccounts + SignInWith + GetConfig,
>(
    Json(tokens): Json<SignInWithLoginInfo>,
    state: S,
) -> Result<Json<LoginResult>, StatusCode> {
    ACCOUNT.post_sign_in_with_login.incr();
    if let Some(google) = tokens.google_token {
        let info = state
            .sign_in_with_manager()
            .validate_google_token(google)
            .await?;
        let google_id = GoogleAccountId(info.id);
        let already_existing_account = state
            .read()
            .account()
            .google_account_id_to_account_id(google_id.clone())
            .await?;

        if let Some(already_existing_account) = already_existing_account {
            login_impl(already_existing_account.as_id(), state)
                .await
                .map(|d| d.into())
        } else {
            let id = register_impl(
                &state,
                SignInWithInfo {
                    google_account_id: Some(google_id),
                },
            )
            .await?;
            login_impl(id, state).await.map(|d| d.into())
        }
    } else if let Some(apple) = tokens.apple_token {
        let _info = state
            .sign_in_with_manager()
            .validate_apple_token(apple)
            .await?;

        // if validate_sign_in_with_apple_token(apple).await.unwrap() {
        //     let key = AccessToken::generate_new();
        //     Ok(key.into())
        // } else {
        //     Err(StatusCode::INTERNAL_SERVER_ERROR)
        // }
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

pub const PATH_ACCOUNT_STATE: &str = "/account_api/state";

/// Get current account state.
#[utoipa::path(
    get,
    path = "/account_api/state",
    responses(
        (status = 200, description = "Request successfull.", body = Account),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_account_state<S: GetAccessTokens + ReadData>(
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    state: S,
) -> Result<Json<Account>, StatusCode> {
    ACCOUNT.get_account_state.incr();
    let account = state
        .read()
        .account()
        .account(api_caller_account_id)
        .await?;
    Ok(account.into())
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
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    state: S,
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
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Json(data): Json<AccountSetup>,
    state: S,
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

pub const PATH_GET_ACCOUNT_DATA: &str = "/account_api/account_data";

/// Get changeable user information to account.
#[utoipa::path(
    get,
    path = "/account_api/account_data",
    responses(
        (status = 200, description = "Request successfull.", body = AccountData),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_account_data<S: GetAccessTokens + ReadData + WriteData>(
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    state: S,
) -> Result<Json<AccountData>, StatusCode> {
    ACCOUNT.get_account_data.incr();
    let data = state
        .read()
        .account()
        .account_data(api_caller_account_id)
        .await?;
    Ok(data.into())
}

pub const PATH_POST_ACCOUNT_DATA: &str = "/account_api/account_data";

/// Set changeable user information to account.
#[utoipa::path(
    post,
    path = "/account_api/account_data",
    request_body(content = AccountData),
    responses(
        (status = 200, description = "Request successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_account_data<S: GetAccessTokens + ReadData + WriteData>(
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Json(data): Json<AccountData>,
    state: S,
) -> Result<(), StatusCode> {
    ACCOUNT.post_account_data.incr();
    // TODO: API limits to prevent DoS attacks

    // TODO: Manual email setting should be removed probably and just
    // use the email from sign in with Google or Apple.

    db_write!(state, move |cmds| cmds
        .account()
        .account_data(api_caller_account_id, data))
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
    Extension(id): Extension<AccountIdInternal>,
    state: S,
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
            if let Some(sign_in_with_config) = state.config().sign_in_with_google_config() {
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
        state.internal_api()
            .modify_and_sync_account_state(
                id,
                |d| {
                    *d.state = new_account_copy.state();
                    *d.capabilities = new_account_copy.into_capablities();
                }
            ).await?;

        state
            .event_manager()
            .send_connected_event(
                id.uuid,
                EventToClientInternal::AccountStateChanged { state: account.state() },
            ).await?;

        state
            .event_manager()
            .send_connected_event(
                id.uuid,
                EventToClientInternal::AccountCapabilitiesChanged { capabilities: account.into_capablities() },
            ).await?;

        Ok(())
    } else {
        Err(StatusCode::NOT_ACCEPTABLE)
    }
}

pub const PATH_SETTING_PROFILE_VISIBILITY: &str = "/account_api/settings/profile_visibility";

/// Update profile visiblity value.
///
/// This will check that the first image moderation request has been moderated
/// before this turns the profile public.
///
/// Sets capablity `view_public_profiles` on or off depending on the value.
#[utoipa::path(
    put,
    path = "/account_api/settings/profile_visibility",
    request_body(content = BooleanSetting),
    responses(
        (status = 200, description = "Update successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn put_setting_profile_visiblity<
    S: GetAccessTokens + ReadData + GetInternalApi + GetConfig + WriteData + EventManagerProvider,
>(
    Extension(id): Extension<AccountIdInternal>,
    Json(new_value): Json<BooleanSetting>,
    state: S,
) -> Result<(), StatusCode> {
    ACCOUNT.put_setting_profile_visiblity.incr();
    let account = state.read().account().account(id).await?;

    if account.state() != AccountState::Normal {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let new_capabilities = state
        .internal_api()
        .modify_and_sync_account_state(id, |d| {
            d.capabilities.user_view_public_profiles = new_value.value;
            *d.is_profile_public = new_value.value;
        })
        .await?;

    state
        .event_manager()
        .send_connected_event(
            id.uuid,
            EventToClientInternal::AccountCapabilitiesChanged { capabilities: new_capabilities },
        ).await?;

    // TODO could this be removed, because there is already the sync call above?
    state
        .internal_api()
        .profile_api_set_profile_visiblity(id, new_value)
        .await?;

    Ok(())
}

pub const PATH_POST_DELETE: &str = "/account_api/delete";

/// Delete account.
///
/// Changes account state to `pending deletion` from all possible states.
/// Previous state will be saved, so it will be possible to stop automatic
/// deletion process.
#[utoipa::path(
    put,
    path = "/account_api/delete",
    responses(
        (status = 200, description = "State changed to 'pending deletion' successfully."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_delete<S: GetAccessTokens + ReadData>(_state: S) -> Result<(), StatusCode> {
    ACCOUNT.post_delete.incr();
    // TODO
    Ok(())
}

pub const PATH_GET_DELETION_STATUS: &str = "/account_api/delete";

/// Get deletion status.
///
/// Get information when account will be really deleted.
#[utoipa::path(
    get,
    path = "/account_api/delete",
    responses(
        (status = 200, description = "Get was successfull.", body = DeleteStatus),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_deletion_status<S: GetAccessTokens + ReadData>(
    _state: S,
) -> Result<DeleteStatus, StatusCode> {
    ACCOUNT.get_deletion_status.incr();
    // TODO
    Err(StatusCode::INTERNAL_SERVER_ERROR)
}

pub const PATH_CANCEL_DELETION: &str = "/account_api/delete";

/// Cancel account deletion.
///
/// Account state will move to previous state.
#[utoipa::path(
    delete,
    path = "/account_api/delete",
    responses(
        (status = 200, description = "Successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn delete_cancel_deletion<S: GetAccessTokens + ReadData>(
    _state: S,
) -> Result<DeleteStatus, StatusCode> {
    ACCOUNT.delete_cancel_deletion.incr();
    // TODO
    Err(StatusCode::INTERNAL_SERVER_ERROR)
}
