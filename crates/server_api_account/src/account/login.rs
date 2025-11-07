use std::{collections::HashMap, net::SocketAddr, time::Instant};

use axum::{
    Form,
    extract::{ConnectInfo, State},
    response::Redirect,
};
use base64::Engine;
use model::AccountIdInternal;
use model_account::{
    AccessToken, AccountId, AppleAccountId, AuthPair, EmailAddress, EmailLoginToken,
    GoogleAccountId, LoginResult, RefreshToken, RequestEmailLoginToken,
    RequestEmailLoginTokenResult, SignInWithInfo, SignInWithLoginInfo,
};
use server_api::{S, app::GetConfig, db_write};
use server_data::{IntoDataError, db_manager::InternalReading, write::GetWriteCommandsCommon};
use server_data_account::{read::GetReadCommandsAccount, write::GetWriteCommandsAccount};
use simple_backend::{
    app::SignInWith,
    create_counters,
    sign_in_with::{apple::AppleAccountInfo, google::GoogleAccountInfo},
};
use tokio::time::{Duration, timeout};

use crate::{
    app::{GetAccounts, ReadData, WriteData},
    utils::{Json, StatusCode},
};

pub async fn login_impl(
    id: AccountId,
    address: SocketAddr,
    state: &S,
) -> Result<LoginResult, StatusCode> {
    let id = state.get_internal_id(id).await?;

    let locked = state
        .read()
        .account_admin()
        .login()
        .account_locked_state(id)
        .await?;
    if locked.locked {
        return Ok(LoginResult::error_account_locked());
    }

    let email = state.read().account().email_address_state(id).await?;

    let access = AccessToken::generate_new();
    let refresh = RefreshToken::generate_new();
    let tokens = AuthPair { access, refresh };
    let tokens_clone = tokens.clone();

    db_write!(state, move |cmds| {
        cmds.common()
            .push_notification()
            .remove_push_notification_device_token_and_encryption_key(id)
            .await?;
        cmds.cache()
            .websocket_cache_cmds()
            .init_login_session(id.into(), tokens_clone, address, false)
            .await
            .into_error()?;
        Ok(())
    })?;

    Ok(LoginResult::ok(tokens, id.as_id(), email.email))
}

pub const PATH_SIGN_IN_WITH_LOGIN: &str = "/account_api/sign_in_with_login";

// TODO(prod): Add error for unverified email address? Or add
//             email verification to initial setup?

trait SignInWithInfoTrait {
    fn email(&self) -> String;
    fn email_verified(&self) -> bool;
    fn sign_in_with_info(&self) -> SignInWithInfo;
    async fn already_existing_account(
        &self,
        state: &S,
    ) -> Result<Option<AccountIdInternal>, StatusCode>;
}

impl SignInWithInfoTrait for GoogleAccountInfo {
    fn email(&self) -> String {
        self.email.clone()
    }

    fn email_verified(&self) -> bool {
        self.email_verified
    }

    fn sign_in_with_info(&self) -> SignInWithInfo {
        SignInWithInfo {
            google_account_id: Some(GoogleAccountId(self.id.clone())),
            ..Default::default()
        }
    }

    async fn already_existing_account(
        &self,
        state: &S,
    ) -> Result<Option<AccountIdInternal>, StatusCode> {
        let already_existing_account = state
            .read()
            .account()
            .google_account_id_to_account_id(GoogleAccountId(self.id.clone()))
            .await?;

        Ok(already_existing_account)
    }
}

impl SignInWithInfoTrait for AppleAccountInfo {
    fn email(&self) -> String {
        self.email.clone()
    }

    fn email_verified(&self) -> bool {
        self.email_verified
    }

    fn sign_in_with_info(&self) -> SignInWithInfo {
        SignInWithInfo {
            apple_account_id: Some(AppleAccountId(self.id.clone())),
            ..Default::default()
        }
    }

    async fn already_existing_account(
        &self,
        state: &S,
    ) -> Result<Option<AccountIdInternal>, StatusCode> {
        let already_existing_account = state
            .read()
            .account()
            .apple_account_id_to_account_id(AppleAccountId(self.id.clone()))
            .await?;

        Ok(already_existing_account)
    }
}

/// Start new session with sign in with Apple or Google.
///
/// Registers new account if it does not exists. That can be disabled
/// using [SignInWithLoginInfo::disable_registering].
#[utoipa::path(
    post,
    path = PATH_SIGN_IN_WITH_LOGIN,
    security(),
    request_body = SignInWithLoginInfo,
    responses(
        (status = 200, description = "Login or account creation successful.", body = LoginResult),
        (status = 500, description = "Internal server error."),
    ),
)]
pub async fn post_sign_in_with_login(
    State(state): State<S>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    Json(tokens): Json<SignInWithLoginInfo>,
) -> Result<Json<LoginResult>, StatusCode> {
    ACCOUNT.post_sign_in_with_login.incr();

    if let Some(min_version) = state.config().min_client_version() {
        if !min_version.received_version_is_accepted(tokens.client_info.client_version) {
            return Ok(LoginResult::error_unsupported_client().into());
        }
    }

    let r = if let Some(apple) = tokens.apple {
        let nonce_bytes = base64::engine::general_purpose::URL_SAFE
            .decode(apple.nonce)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let info = state
            .sign_in_with_manager()
            .validate_apple_token(apple.token, nonce_bytes)
            .await?;
        handle_sign_in_with_info(&state, address, tokens.disable_registering, info).await
    } else if let Some(google) = tokens.google {
        let nonce_bytes = base64::engine::general_purpose::URL_SAFE
            .decode(google.nonce)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let info = state
            .sign_in_with_manager()
            .validate_google_token(google.token, nonce_bytes)
            .await?;
        handle_sign_in_with_info(&state, address, tokens.disable_registering, info).await
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }?;

    if let Some(aid) = r.aid {
        // Login successful
        let id = state.get_internal_id(aid).await?;
        db_write!(state, move |cmds| {
            cmds.common()
                .client_config()
                .client_login_session_platform(id, tokens.client_info.client_type)
                .await
        })?;
    }

    Ok(r.into())
}

async fn handle_sign_in_with_info(
    state: &S,
    address: SocketAddr,
    disable_registering: bool,
    info: impl SignInWithInfoTrait,
) -> Result<LoginResult, StatusCode> {
    if !info.email_verified() {
        return Ok(LoginResult::error_sign_in_with_email_unverified());
    }

    let already_existing_account = info.already_existing_account(state).await?;

    if let Some(already_existing_account) = already_existing_account {
        login_impl(already_existing_account.as_id(), address, state).await
    } else if disable_registering {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    } else {
        let email: EmailAddress = info
            .email()
            .try_into()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // Check if email is already used by another account
        let email_already_used = state
            .read()
            .account()
            .email()
            .account_id_from_email(email.clone())
            .await?;

        if email_already_used.is_some() {
            return Ok(LoginResult::error_email_already_used());
        }

        let id = state
            .data_all_access()
            .register_impl(info.sign_in_with_info(), Some(email))
            .await?;
        login_impl(id.as_id(), address, state).await
    }
}

pub const PATH_SIGN_IN_WITH_APPLE_REDIRECT_TO_APP: &str =
    "/account_api/sign_in_with_apple_redirect_to_app";

/// Sign in with Apple related redirect back to Android app.
///
/// This is specific to <https://pub.dev/packages/sign_in_with_apple> library.
pub async fn post_sign_in_with_apple_redirect_to_app(
    State(state): State<S>,
    Form(form): Form<HashMap<String, String>>,
) -> Result<Redirect, StatusCode> {
    ACCOUNT.post_sign_in_with_apple_redirect_to_app.incr();

    let package_id = state
        .config()
        .simple_backend()
        .sign_in_with_apple_config()
        .map(|v| &v.android_package_id)
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let query_params: String =
        serde_urlencoded::to_string(form).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let redirect = format!(
        "intent://callback?{query_params}#Intent;package={package_id};scheme=signinwithapple;end",
    );

    // Temporary redirect reuses current HTTP method POST which
    // means that URL is not displayed in web browser address bar.
    Ok(Redirect::temporary(&redirect))
}

pub const PATH_POST_REQUEST_EMAIL_LOGIN_TOKEN: &str = "/account_api/request_email_login_token";

/// Request email login token to be sent via email.
///
/// The route always takes at least 5 seconds to complete to prevent timing attacks
/// that could be used to enumerate existing email addresses.
///
/// No error is returned to prevent attackers from discovering which email
/// addresses exist in the system.
#[utoipa::path(
    post,
    path = PATH_POST_REQUEST_EMAIL_LOGIN_TOKEN,
    request_body = RequestEmailLoginToken,
    responses(
        (status = 200, description = "Request processed.", body = RequestEmailLoginTokenResult),
    ),
    security(),
)]
pub async fn post_request_email_login_token(
    State(state): State<S>,
    Json(request): Json<RequestEmailLoginToken>,
) -> Result<Json<RequestEmailLoginTokenResult>, StatusCode> {
    ACCOUNT.post_request_email_login_token.incr();

    let wait_until = Instant::now() + Duration::from_secs(5);

    let _ = timeout(Duration::from_secs(10), async {
        let account_id = state
            .read()
            .account()
            .email()
            .account_id_from_email(request.email.clone())
            .await
            .ok()??;

        db_write!(state, move |cmds| {
            let internal = cmds
                .read()
                .account()
                .email_address_state_internal(account_id)
                .await?;

            if !internal.email_login_enabled {
                // Email login is disabled, but don't return error to prevent
                // email enumeration.
                return Ok(());
            }

            if let Some(token_time) = internal.email_login_token_unix_time {
                let min_wait_duration = GetConfig::config(&cmds)
                    .limits_account()
                    .email_login_resend_min_wait_duration;
                if !token_time.duration_value_elapsed(min_wait_duration) {
                    // Too soon to send another token, but don't return error
                    return Ok(());
                }
            }

            cmds.account()
                .email()
                .set_email_login_token(account_id)
                .await?;

            cmds.account()
                .email()
                .send_email_login_token_high_priority(account_id)
                .await?;

            Ok(())
        })
        .ok()
    })
    .await;

    // Wait until at least 5 seconds have elapsed
    tokio::time::sleep_until(wait_until.into()).await;

    // Always return success with config values to prevent email enumeration
    Ok(Json(RequestEmailLoginTokenResult {
        token_validity_seconds: state
            .config()
            .limits_account()
            .email_login_token_validity_duration
            .seconds as i64,
        resend_wait_seconds: state
            .config()
            .limits_account()
            .email_login_resend_min_wait_duration
            .seconds as i64,
    }))
}

pub const PATH_POST_EMAIL_LOGIN_WITH_TOKEN: &str = "/account_api/email_login_with_token";

/// Login using email login token (single use).
///
/// The route always takes at least 5 seconds to complete to make
/// token guessing slower.
#[utoipa::path(
    post,
    path = PATH_POST_EMAIL_LOGIN_WITH_TOKEN,
    security(),
    request_body = EmailLoginToken,
    responses(
        (status = 200, description = "Login successful.", body = LoginResult),
        (status = 401, description = "Invalid token."),
        (status = 500, description = "Internal server error."),
    ),
)]
pub async fn post_email_login_with_token(
    State(state): State<S>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    Json(request): Json<EmailLoginToken>,
) -> Result<Json<LoginResult>, StatusCode> {
    ACCOUNT.post_email_login_with_token.incr();

    let wait_until = Instant::now() + Duration::from_secs(5);

    let r = post_email_login_with_token_impl(state, address, request).await;

    // Wait until at least 5 seconds have elapsed
    tokio::time::sleep_until(wait_until.into()).await;

    r
}

async fn post_email_login_with_token_impl(
    state: S,
    address: SocketAddr,
    request: EmailLoginToken,
) -> Result<Json<LoginResult>, StatusCode> {
    if let Some(min_version) = state.config().min_client_version() {
        if !min_version.received_version_is_accepted(request.client_info.client_version) {
            return Ok(LoginResult::error_unsupported_client().into());
        }
    }

    let token = request
        .token
        .bytes()
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let account_id = db_write!(state, move |cmds| {
        cmds.account()
            .email()
            .verify_email_login_token_and_invalidate(token)
            .await
    })
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let account_id = match account_id {
        Some(id) => id,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    let r = login_impl(account_id.as_id(), address, &state).await?;

    if let Some(aid) = r.aid {
        // Login successful
        let id = state.get_internal_id(aid).await?;
        db_write!(state, move |cmds| {
            cmds.common()
                .client_config()
                .client_login_session_platform(id, request.client_info.client_type)
                .await
        })?;
    }

    Ok(r.into())
}

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_LOGIN_COUNTERS_LIST,
    post_sign_in_with_login,
    post_sign_in_with_apple_redirect_to_app,
    post_request_email_login_token,
    post_email_login_with_token,
);
