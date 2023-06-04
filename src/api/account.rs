pub mod data;
pub mod internal;

use axum::{Json, TypedHeader};

use futures::FutureExt;
use hyper::StatusCode;

use self::data::{
    Account, AccountIdLight, AccountSetup, AccountState, ApiKey, AuthPair, BooleanSetting,
    DeleteStatus, GoogleAccountId, LoginResult, RefreshToken, SignInWithInfo, SignInWithLoginInfo,
};

use super::{GetConfig, GetInternalApi, SignInWith};

use tracing::error;

use super::{utils::ApiKeyHeader, GetApiKeys, GetUsers, ReadDatabase, WriteDatabase};

use tokio_stream::StreamExt;

// TODO: Update register and login to support Apple and Google single sign on.

pub const PATH_REGISTER: &str = "/account_api/register";

/// Register new account. Returns new account ID which is UUID.
#[utoipa::path(
    post,
    path = "/account_api/register",
    security(),
    responses(
        (status = 200, description = "New profile created.", body = AccountIdLight),
        (status = 500, description = "Internal server error."),
    )
)]
pub async fn post_register<S: WriteDatabase + GetConfig>(
    state: S,
) -> Result<Json<AccountIdLight>, StatusCode> {
    register_impl(&state, SignInWithInfo::default())
        .await
        .map(|id| id.into())
}

pub async fn register_impl<S: WriteDatabase + GetConfig>(
    state: &S,
    sign_in_with: SignInWithInfo,
) -> Result<AccountIdLight, StatusCode> {
    // New unique UUID is generated every time so no special handling needed
    // to avoid database collisions.
    let id = AccountIdLight::new(uuid::Uuid::new_v4());

    let register = state.write_database().register(id, sign_in_with);
    match register.await {
        Ok(id) => Ok(id.as_light().into()),
        Err(e) => {
            error!("Error: {e:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub const PATH_LOGIN: &str = "/account_api/login";

/// Get new ApiKey.
#[utoipa::path(
    post,
    path = "/account_api/login",
    security(),
    request_body = AccountIdLight,
    responses(
        (status = 200, description = "Login successful.", body = LoginResult),
        (status = 500, description = "Internal server error."),
    ),
)]
pub async fn post_login<S: GetApiKeys + WriteDatabase + GetUsers>(
    Json(id): Json<AccountIdLight>,
    state: S,
) -> Result<Json<LoginResult>, StatusCode> {
    login_impl(id, state).await.map(|d| d.into())
}

async fn login_impl<S: GetApiKeys + WriteDatabase + GetUsers>(
    id: AccountIdLight,
    state: S,
) -> Result<LoginResult, StatusCode> {
    let access = ApiKey::generate_new();
    let refresh = RefreshToken::generate_new();

    let id = state.users().get_internal_id(id).await.map_err(|e| {
        error!("Login error: {e:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let account = AuthPair { access, refresh };

    state
        .write_database()
        .set_new_auth_pair(id, account.clone(), None)
        .await
        .map_err(|e| {
            error!("Login error: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR // Database writing failed.
        })?;

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
    S: GetApiKeys + WriteDatabase + GetUsers + SignInWith + GetConfig,
>(
    Json(tokens): Json<SignInWithLoginInfo>,
    state: S,
) -> Result<Json<LoginResult>, StatusCode> {
    if let Some(google) = tokens.google_token {
        let info = state
            .sign_in_with_manager()
            .validate_google_token(google)
            .await
            .map_err(|e| {
                error!("{e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        let google_id = GoogleAccountId(info.id);
        let already_existing_account = state
            .users()
            .get_account_with_google_account_id(google_id.clone())
            .await
            .map_err(|e| {
                error!("{e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        if let Some(already_existing_account) = already_existing_account {
            login_impl(already_existing_account.as_light(), state)
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
        let info = state
            .sign_in_with_manager()
            .validate_apple_token(apple)
            .await
            .map_err(|e| {
                error!("{e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        // if validate_sign_in_with_apple_token(apple).await.unwrap() {
        //     let key = ApiKey::generate_new();
        //     Ok(key.into())
        // } else {
        //     Err(StatusCode::INTERNAL_SERVER_ERROR)
        // }
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

// pub const PATH_REFRESH_API_KEY: &str = "/account/refresh_api_key";

// /// Get app refresh token which is used for getting new API keys.
// #[utoipa::path(
//     post,
//     path = "/account/refresh_api_key",
//     request_body = AuthPair,
//     responses(
//         (status = 200, description = "Login successful.", body = ApiKey),
//         (status = 401, description = "Invalid API key."),
//         (status = 500, description = "Internal server error."),
//     ),
//     security(("api_key" = [])),
// )]
// pub async fn post_refresh_api_key<S: GetApiKeys + WriteDatabase + GetUsers>(
//     Json(id): Json<AccountIdLight>,
//     state: S,
// ) -> Result<Json<ApiKey>, StatusCode> {
//     let key = ApiKey::generate_new();
//     let account_state = get_account(&state, id, |account| account.clone()).await?;

//     todo!()
// }

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
    security(("api_key" = [])),
)]
pub async fn get_account_state<S: GetApiKeys + ReadDatabase>(
    TypedHeader(api_key): TypedHeader<ApiKeyHeader>,
    state: S,
) -> Result<Json<Account>, StatusCode> {
    let id = state
        .api_keys()
        .api_key_exists(api_key.key())
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    state
        .read_database()
        .read_json::<Account>(id)
        .await
        .map(|account| account.into())
        .map_err(|e| {
            error!("Get profile error: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR // Database reading failed.
        })
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
    security(("api_key" = [])),
)]
pub async fn post_account_setup<S: GetApiKeys + ReadDatabase + WriteDatabase>(
    TypedHeader(api_key): TypedHeader<ApiKeyHeader>,
    Json(data): Json<AccountSetup>,
    state: S,
) -> Result<(), StatusCode> {
    let id = state
        .api_keys()
        .api_key_exists(api_key.key())
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let account = state
        .read_database()
        .read_json::<Account>(id)
        .await
        .map_err(|e| {
            error!("Get profile error: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR // Database reading failed.
        })?;

    if account.state() == AccountState::InitialSetup {
        state
            .write_database()
            .update_account_setup(id, data)
            .await
            .map_err(|e| {
                error!("Write database error: {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR // Database writing failed.
            })
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
    security(("api_key" = [])),
)]
pub async fn post_complete_setup<
    S: GetApiKeys + ReadDatabase + WriteDatabase + GetInternalApi + GetConfig,
>(
    TypedHeader(api_key): TypedHeader<ApiKeyHeader>,
    state: S,
) -> Result<(), StatusCode> {
    let id = state
        .api_keys()
        .api_key_exists(api_key.key())
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let account_setup = state
        .read_database()
        .read_json::<AccountSetup>(id)
        .await
        .map_err(|e| {
            error!("Complete setup error: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR // Database reading failed.
        })?;

    if account_setup.is_empty() {
        return Err(StatusCode::NOT_ACCEPTABLE);
    }

    state
        .internal_api()
        .media_check_moderation_request_for_account(id)
        .await
        .map_err(|e| {
            error!("Complete setup error: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut account = state
        .read_database()
        .read_json::<Account>(id)
        .await
        .map_err(|e| {
            error!("Complete setup error: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR // Database reading failed.
        })?;

    if account.state() == AccountState::InitialSetup {
        account.complete_setup();

        if account_setup.email() == state.config().admin_email() {
            account.add_admin_capablities();
        }

        state
            .write_database()
            .update_account(id, account)
            .await
            .map_err(|e| {
                error!("Write database error: {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR // Database writing failed.
            })
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
    security(("api_key" = [])),
)]
pub async fn put_setting_profile_visiblity<
    S: GetApiKeys + WriteDatabase + ReadDatabase + GetInternalApi,
>(
    TypedHeader(api_key): TypedHeader<ApiKeyHeader>,
    Json(new_value): Json<BooleanSetting>,
    state: S,
) -> Result<(), StatusCode> {
    let id = state
        .api_keys()
        .api_key_exists(api_key.key())
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let account = state
        .read_database()
        .read_json::<Account>(id)
        .await
        .map_err(|e| {
            error!("put_setting_profile_visiblity: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if account.state() != AccountState::Normal {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    state
        .internal_api()
        .profile_api_set_profile_visiblity(id, new_value)
        .await
        .map_err(|e| {
            error!("put_setting_profile_visiblity: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

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
    security(("api_key" = [])),
)]
pub async fn post_delete<S: GetApiKeys + WriteDatabase + ReadDatabase>(
    _state: S,
) -> Result<(), StatusCode> {
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
    security(("api_key" = [])),
)]
pub async fn get_deletion_status<S: GetApiKeys + WriteDatabase + ReadDatabase>(
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
    security(("api_key" = [])),
)]
pub async fn delete_cancel_deletion<S: GetApiKeys + WriteDatabase + ReadDatabase>(
    _state: S,
) -> Result<DeleteStatus, StatusCode> {
    Err(StatusCode::INTERNAL_SERVER_ERROR)
}
