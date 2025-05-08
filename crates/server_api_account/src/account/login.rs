use std::collections::HashMap;

use axum::{extract::State, response::Redirect, Form};
use model::AccountIdInternal;
use model_account::{
    AccessToken, AccountId, AppleAccountId, AuthPair, EmailAddress, GoogleAccountId, LoginResult, RefreshToken, SignInWithInfo, SignInWithLoginInfo
};
use server_api::{app::GetConfig, db_write_multiple, S};
use server_data::write::GetWriteCommandsCommon;
use server_data_account::{read::GetReadCommandsAccount, write::GetWriteCommandsAccount};
use simple_backend::{app::SignInWith, create_counters, sign_in_with::{apple::AppleAccountInfo, google::GoogleAccountInfo}};

use crate::{
    app::{GetAccounts, ReadData, WriteData},
    utils::{Json, StatusCode},
};

pub async fn login_impl(id: AccountId, state: S) -> Result<LoginResult, StatusCode> {
    let id = state.get_internal_id(id).await?;
    let email = state.read().account().account_data(id).await?;

    let access = AccessToken::generate_new();
    let refresh = RefreshToken::generate_new();
    let account = AuthPair { access, refresh };
    let account_clone = account.clone();

    db_write_multiple!(state, move |cmds| {
        cmds.common()
            .push_notification()
            .remove_fcm_device_token_and_pending_notification_token(id)
            .await?;
        cmds.common()
            .set_new_auth_pair(id, account_clone, None)
            .await
    })?;

    // TODO(microservice): microservice support

    let result = LoginResult {
        account: Some(account),
        profile: None,
        media: None,
        aid: Some(id.as_id()),
        email: email.email,
        error_unsupported_client: false,
    };
    Ok(result)
}

pub const PATH_SIGN_IN_WITH_LOGIN: &str = "/account_api/sign_in_with_login";

// TODO(prod): Add error for unverified email address

trait SignInWithInfoTrait {
    fn email(&self) -> String;
    fn sign_in_with_info(&self) -> SignInWithInfo;
    async fn already_existing_account(&self, state: &S) -> Result<Option<AccountIdInternal>, StatusCode>;
}

impl SignInWithInfoTrait for GoogleAccountInfo {
    fn email(&self) -> String {
        self.email.clone()
    }

    fn sign_in_with_info(&self) -> SignInWithInfo {
        SignInWithInfo {
            google_account_id: Some(GoogleAccountId(self.id.clone())),
            ..Default::default()
        }
    }

    async fn already_existing_account(&self, state: &S) -> Result<Option<AccountIdInternal>, StatusCode> {
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

    fn sign_in_with_info(&self) -> SignInWithInfo {
        SignInWithInfo {
            apple_account_id: Some(AppleAccountId(self.id.clone())),
            ..Default::default()
        }
    }

    async fn already_existing_account(&self, state: &S) -> Result<Option<AccountIdInternal>, StatusCode> {
        let already_existing_account = state
            .read()
            .account()
            .apple_account_id_to_account_id(AppleAccountId(self.id.clone()))
            .await?;

        Ok(already_existing_account)
    }
}

/// Start new session with sign in with Apple or Google. Creates new account if
/// it does not exists.
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
    Json(tokens): Json<SignInWithLoginInfo>,
) -> Result<Json<LoginResult>, StatusCode> {
    ACCOUNT.post_sign_in_with_login.incr();

    if let Some(min_version) = state.config().min_client_version() {
        if !min_version.received_version_is_accepted(tokens.client_info.client_version) {
            return Ok(LoginResult::error_unsupported_client().into());
        }
    }

    if let Some(apple) = tokens.apple_token {
        let info = state
            .sign_in_with_manager()
            .validate_apple_token(apple)
            .await?;
        handle_sign_in_with_info(state, info).await
    } else if let Some(google) = tokens.google_token {
        let info = state
            .sign_in_with_manager()
            .validate_google_token(google)
            .await?;
        handle_sign_in_with_info(state, info).await
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

async fn handle_sign_in_with_info(
    state: S,
    info: impl SignInWithInfoTrait,
) -> Result<Json<LoginResult>, StatusCode> {
    let email: EmailAddress = info
        .email()
        .try_into()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let already_existing_account = info.already_existing_account(&state)
        .await?;

    if let Some(already_existing_account) = already_existing_account {
        db_write_multiple!(state, move |cmds| cmds
            .account()
            .email()
            .account_email(already_existing_account, email).await)?;

        login_impl(already_existing_account.as_id(), state)
            .await
            .map(|d| d.into())
    } else {
        let id = state
            .data_all_access()
            .register_impl(
                info.sign_in_with_info(),
                Some(email),
            )
            .await?;
        login_impl(id.as_id(), state).await.map(|d| d.into())
    }
}

pub const PATH_SIGN_IN_WITH_APPLE_REDIRECT_TO_APP: &str = "/account_api/sign_in_with_apple_redirect_to_app";

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

    let query_params: String = serde_urlencoded::to_string(form)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let redirect = format!(
        "intent://callback?{}#Intent;package={};scheme=signinwithapple;end",
        query_params,
        package_id,
    );

    // Temporary redirect reuses current HTTP method POST which
    // means that URL is not displayed in web browser address bar.
    Ok(Redirect::temporary(&redirect))
}

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_LOGIN_COUNTERS_LIST,
    post_sign_in_with_login,
    post_sign_in_with_apple_redirect_to_app,
);
