use axum::{Extension};
use model::{
    Account, AccountId, AccountSetup, AccountState, AccessToken, AuthPair, BooleanSetting,
    DeleteStatus, GoogleAccountId, LoginResult, RefreshToken, SignInWithInfo, SignInWithLoginInfo, AccountIdInternal,
};
use tracing::{error};

use super::{
    db_write,
    utils::{Json, StatusCode},
    GetAccessTokens, GetConfig, GetInternalApi, GetAccounts, ReadData, SignInWith, WriteData,
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
    let account = state
        .read()
        .account()
        .account(api_caller_account_id)
        .await?;
    Ok(account.into())
}

pub const PATH_ACCOUNT_SETUP: &str = "/account_api/setup";

/// Setup non-changeable user information during `initial setup` state.
#[utoipa::path(
    post,
    path = "/account_api/setup",
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
    let account = state
        .read()
        .account()
        .account(api_caller_account_id)
        .await?;

    if account.state() == AccountState::InitialSetup {
        db_write!(state, move |cmds| cmds.account().account_setup(api_caller_account_id, data))
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
    S: GetAccessTokens + ReadData + WriteData + GetInternalApi + GetConfig,
>(
    Extension(id): Extension<AccountIdInternal>,
    state: S,
) -> Result<(), StatusCode> {
    let account_setup = state
        .read()
        .account()
        .account_setup(id)
        .await?;

    if account_setup.is_empty() {
        return Err(StatusCode::NOT_ACCEPTABLE);
    }

    state
        .internal_api()
        .media_check_moderation_request_for_account(id)
        .await?;

    let mut account = state
        .read()
        .account()
        .account(id)
        .await?;

    let sign_in_with_info = state
        .read()
        .account()
        .account_sign_in_with_info(id)
        .await?;

    if account.state() == AccountState::InitialSetup {
        account.complete_setup();

        if state.config().debug_mode() {
            if account_setup.email() == state.config().admin_email() {
                account.add_admin_capablities();
            }
        } else {
            if let Some(sign_in_with_config) = state.config().sign_in_with_google_config() {
                if sign_in_with_info.google_account_id
                    == Some(model::GoogleAccountId(
                        sign_in_with_config.admin_google_account_id.clone(),
                    ))
                    && account_setup.email() == state.config().admin_email()
                {
                    account.add_admin_capablities();
                }
            }
        }

        db_write!(state, move |cmds| cmds.account().account(id, account))
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
    S: GetAccessTokens + ReadData + GetInternalApi + GetConfig + WriteData,
>(
    Extension(id): Extension<AccountIdInternal>,
    Json(new_value): Json<BooleanSetting>,
    state: S,
) -> Result<(), StatusCode> {
    let account = state
        .read()
        .account()
        .account(id)
        .await?;

    if account.state() != AccountState::Normal {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

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
    Err(StatusCode::INTERNAL_SERVER_ERROR)
}
