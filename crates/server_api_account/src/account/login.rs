use std::{collections::HashMap, net::SocketAddr, time::Instant};

use axum::{
    Form,
    extract::{ConnectInfo, State},
    response::Redirect,
};
use base64::Engine;
use model::{AccountIdInternal, ClientType, EmailLoginToken, UnixTime};
use model_account::{
    AccessToken, AccountId, AppleAccountId, AuthPair, EmailAddress, EmailLogin, GoogleAccountId,
    LoginResult, RefreshToken, RequestEmailLoginToken, RequestEmailLoginTokenResult,
    SignInWithInfo, SignInWithLoginInfo,
};
use server_api::{
    S, TokenData,
    app::{GetConfig, GetDynamicServerConfig},
    db_write,
};
use server_data::{
    IntoDataError, app::RegisterImplResult, db_manager::InternalReading, email::EmailSendingHandle,
    write::GetWriteCommandsCommon,
};
use server_data_account::{read::GetReadCommandsAccount, write::GetWriteCommandsAccount};
use simple_backend::{
    app::SignInWith,
    create_counters,
    sign_in_with::{apple::AppleAccountInfo, google::GoogleAccountInfo},
};
use tokio::time::{Duration, timeout};

use crate::{
    account::login::register::request_email_registration_token,
    app::{GetAccounts, ReadData, WriteData},
    utils::{Json, StatusCode},
};

pub mod register;

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

    let email = state.read().account().email_address(id).await?;

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

    Ok(LoginResult::ok(tokens, id.as_id(), email))
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

async fn validate_registration_platform(
    state: &S,
    client_type: ClientType,
) -> Result<(), LoginResult> {
    let config = state
        .dynamic_server_config_manager()
        .dynamic_server_config()
        .await
        .unwrap_or_default()
        .account_registration_platforms;

    let enabled = match client_type {
        ClientType::Android => config.android,
        ClientType::Ios => config.ios,
        ClientType::Web => config.web,
        ClientType::Bot => false,
    };

    if enabled {
        return Ok(());
    }

    let any_enabled = config.android || config.ios || config.web;
    if any_enabled {
        Err(LoginResult::error_registration_platform_disabled())
    } else {
        Err(LoginResult::error_registration_all_platforms_disabled())
    }
}

async fn validate_login_platform(state: &S, client_type: ClientType) -> Result<(), LoginResult> {
    let config = state
        .dynamic_server_config_manager()
        .dynamic_server_config()
        .await
        .unwrap_or_default()
        .account_login_platforms;

    let enabled = match client_type {
        ClientType::Android => config.android,
        ClientType::Ios => config.ios,
        ClientType::Web => config.web,
        ClientType::Bot => false,
    };

    if enabled {
        return Ok(());
    }

    let any_enabled = config.android || config.ios || config.web;
    if any_enabled {
        Err(LoginResult::error_login_platform_disabled())
    } else {
        Err(LoginResult::error_login_all_platforms_disabled())
    }
}

/// Start new session with sign in with Apple or Google.
///
/// Registers new account if it does not exist, when registration is enabled
/// for the current client platform in dynamic server config.
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

    if let Some(min_version) = state.config().min_client_version()
        && !min_version.received_version_is_accepted(tokens.client_info.client_version)
    {
        return Ok(LoginResult::error_unsupported_client().into());
    }

    if let Err(error) = validate_login_platform(&state, tokens.client_info.client_type).await {
        return Ok(error.into());
    }

    let r = if let Some(apple) = tokens.apple {
        let nonce_bytes = base64::engine::general_purpose::URL_SAFE
            .decode(apple.nonce)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let info = state
            .sign_in_with_manager()
            .validate_apple_token(apple.token, nonce_bytes)
            .await?;
        handle_sign_in_with_info(&state, address, tokens.client_info.client_type, info).await
    } else if let Some(google) = tokens.google {
        let nonce_bytes = base64::engine::general_purpose::URL_SAFE
            .decode(google.nonce)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let info = state
            .sign_in_with_manager()
            .validate_google_token(google.token, nonce_bytes)
            .await?;
        handle_sign_in_with_info(&state, address, tokens.client_info.client_type, info).await
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }?;

    if let Some(aid) = r.aid() {
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
    client_type: ClientType,
    info: impl SignInWithInfoTrait,
) -> Result<LoginResult, StatusCode> {
    if !info.email_verified() {
        return Ok(LoginResult::error_sign_in_with_email_unverified());
    }

    let already_existing_account = info.already_existing_account(state).await?;

    if let Some(already_existing_account) = already_existing_account {
        login_impl(already_existing_account.as_id(), address, state).await
    } else {
        if let Err(error) = validate_registration_platform(state, client_type).await {
            return Ok(error);
        }

        let email: EmailAddress = info
            .email()
            .try_into()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let id = match state
            .data_all_access()
            .register_impl(info.sign_in_with_info(), Some(email))
            .await?
        {
            RegisterImplResult::Ok(id) => id,
            RegisterImplResult::EmailAlreadyExists => {
                return Ok(LoginResult::error_email_already_used());
            }
        };
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

struct EmailLoginResultInternal {
    token: EmailLoginToken,
    handle: Option<EmailSendingHandle>,
    error_registration_ip_address_limit_reached: bool,
}

impl EmailLoginResultInternal {
    fn successful(token: EmailLoginToken, handle: EmailSendingHandle) -> Self {
        Self {
            token,
            handle: Some(handle),
            error_registration_ip_address_limit_reached: false,
        }
    }

    fn error_hidden() -> Self {
        Self {
            token: EmailLoginToken::generate_new(),
            handle: None,
            error_registration_ip_address_limit_reached: false,
        }
    }

    fn error_registration_ip_address_limit_reached() -> Self {
        Self {
            token: EmailLoginToken::generate_new(),
            handle: None,
            error_registration_ip_address_limit_reached: true,
        }
    }
}

pub const PATH_POST_REQUEST_EMAIL_LOGIN_TOKEN: &str = "/account_api/request_email_login_token";

/// Request email login token to be sent via email.
///
/// The route always takes at least 5 seconds to complete to prevent timing attacks
/// that could be used to enumerate existing email addresses.
#[utoipa::path(
    post,
    path = PATH_POST_REQUEST_EMAIL_LOGIN_TOKEN,
    request_body = RequestEmailLoginToken,
    responses(
        (status = 200, description = "Request processed.", body = RequestEmailLoginTokenResult),
        (status = 500, description = "Internal server error."),
    ),
    security(),
)]
pub async fn post_request_email_login_token(
    State(state): State<S>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    Json(request): Json<RequestEmailLoginToken>,
) -> Result<Json<RequestEmailLoginTokenResult>, StatusCode> {
    ACCOUNT.post_request_email_login_token.incr();

    let wait_until = Instant::now() + Duration::from_secs(5);

    let r = handle_login_token_sending(&state, address, request).await?;

    if let Some(handle) = r.handle {
        let _ = timeout(Duration::from_secs(10), handle.wait()).await;
    }

    // Wait until at least 5 seconds have elapsed
    tokio::time::sleep_until(wait_until.into()).await;

    if r.error_registration_ip_address_limit_reached {
        Ok(Json(
            RequestEmailLoginTokenResult::error_email_registration_ip_address_limit_reached(),
        ))
    } else {
        Ok(Json(RequestEmailLoginTokenResult::successful(
            r.token,
            state
                .config()
                .limits_account()
                .email_login_token_validity_duration
                .seconds as i64,
            state
                .config()
                .limits_account()
                .email_login_resend_min_wait_duration
                .seconds as i64,
            state.config().limits_account().email_login_emails_per_month as i64,
        )))
    }
}

async fn handle_login_token_sending(
    state: &S,
    address: SocketAddr,
    request: RequestEmailLoginToken,
) -> Result<EmailLoginResultInternal, StatusCode> {
    if !request.login_only {
        let max_per_day = state
            .config()
            .limits_account()
            .email_registration_max_per_day_per_ip;
        if state
            .email_registration_rate_limiter()
            .check_and_increment(address.ip(), max_per_day)
            .await
        {
            return Ok(EmailLoginResultInternal::error_registration_ip_address_limit_reached());
        }
    }

    let account_id = state
        .read()
        .account()
        .email()
        .account_id_from_email(request.email.clone())
        .await?;
    if let Some(account_id) = account_id {
        let internal = state
            .read()
            .account()
            .email_address_state_internal(account_id)
            .await?;

        if !internal.email_login_enabled {
            // Email login is disabled, but don't return error to prevent
            // email enumeration.
            return Ok(EmailLoginResultInternal::error_hidden());
        }

        let min_wait_duration = state
            .config()
            .limits_account()
            .email_login_resend_min_wait_duration;
        let emails_per_month =
            TryInto::<i16>::try_into(state.config().limits_account().email_login_emails_per_month)
                .unwrap_or(i16::MAX);

        let error = db_write!(state, move |cmds| {
            let mut limits = cmds
                .read()
                .account()
                .email()
                .email_login_limits(account_id)
                .await?
                .unwrap_or_default();

            let now = UnixTime::current_time();
            const MONTH_SECONDS: i64 = 60 * 60 * 24 * 30;
            let monthly_reset = limits
                .monthly_limit_reset_unix_time
                .map(|t| t.ut + MONTH_SECONDS)
                .unwrap_or(0);
            if monthly_reset <= now.ut {
                limits.monthly_email_count = 0;
                limits.monthly_limit_reset_unix_time = Some(now);
            }

            if let Some(sent_time) = limits.token_sent_unix_time
                && !sent_time.duration_value_elapsed(min_wait_duration)
            {
                // Too soon to send another token, but don't return error
                return Ok(Some(EmailLoginResultInternal::error_hidden()));
            }

            if limits.monthly_email_count >= emails_per_month {
                // Monthly email limit reached, but don't return error
                return Ok(Some(EmailLoginResultInternal::error_hidden()));
            }

            limits.token_sent_unix_time = Some(now);
            limits.monthly_email_count += 1;

            cmds.account()
                .email()
                .upsert_email_login_limits(account_id, limits)
                .await?;

            Ok(None)
        })?;

        if let Some(error) = error {
            return Ok(error);
        }

        let (client_token, email_token) = state
            .email_registration_tokens()
            .insert(
                TokenData::Account(account_id),
                state
                    .config()
                    .limits_account()
                    .email_login_token_validity_duration,
            )
            .await;

        let Some(email) = internal.email else {
            return Ok(EmailLoginResultInternal::error_hidden());
        };

        let handle = state
            .email_channel_sender()
            .send_registration_login_token(email.0, email_token.into_string())
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(EmailLoginResultInternal::successful(client_token, handle))
    } else if request.login_only {
        Ok(EmailLoginResultInternal::error_hidden())
    } else {
        request_email_registration_token(state, &request).await
    }
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
    request_body = EmailLogin,
    responses(
        (status = 200, description = "Successful.", body = LoginResult),
        (status = 500, description = "Internal server error."),
    ),
)]
pub async fn post_email_login_with_token(
    State(state): State<S>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    Json(request): Json<EmailLogin>,
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
    request: EmailLogin,
) -> Result<Json<LoginResult>, StatusCode> {
    if let Some(min_version) = state.config().min_client_version()
        && !min_version.received_version_is_accepted(request.client_info.client_version)
    {
        return Ok(LoginResult::error_unsupported_client().into());
    }

    if let Err(error) = validate_login_platform(&state, request.client_info.client_type).await {
        return Ok(error.into());
    }

    let Ok(client_token) = request.client_token.bytes() else {
        return Ok(LoginResult::error_invalid_email_login_token().into());
    };

    let Ok(email_token) = request.email_token.bytes() else {
        return Ok(LoginResult::error_invalid_email_login_token().into());
    };

    let client_token_clone = client_token.clone();
    let email_token_clone = email_token.clone();

    // First try login token from RAM store (existing account)
    let account_id = match state
        .email_registration_tokens()
        .consume(
            &client_token_clone,
            &email_token_clone,
            state
                .config()
                .limits_account()
                .email_login_token_validity_duration,
        )
        .await
    {
        Some(TokenData::Account(id)) => Some(id),
        _ => None,
    };

    if let Some(account_id) = account_id {
        // Login token was valid
        let r = login_impl(account_id.as_id(), address, &state).await?;

        if let Some(aid) = r.aid() {
            let id = state.get_internal_id(aid).await?;
            db_write!(state, move |cmds| {
                cmds.common()
                    .client_config()
                    .client_login_session_platform(id, request.client_info.client_type)
                    .await
            })?;
        }

        return Ok(r.into());
    }

    let r = register::email_registration_with_token_impl(state, address, client_token, email_token)
        .await?;

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
