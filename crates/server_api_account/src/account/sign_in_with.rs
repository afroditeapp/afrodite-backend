use axum::{Extension, extract::State};
use base64::Engine;
use model::AccountIdInternal;
use model_account::{
    AppleAccountId, GoogleAccountId, PutSignInWithApple, PutSignInWithGoogle, SignInWithState,
};
use server_api::{S, create_open_api_router, db_write};
use server_data_account::{read::GetReadCommandsAccount, write::GetWriteCommandsAccount};
use simple_backend::{app::SignInWith, create_counters};

use crate::{
    app::{ReadData, WriteData},
    utils::{Json, StatusCode},
};

const PATH_GET_SIGN_IN_WITH_INFO: &str = "/account_api/sign_in_with_info";

/// Get current sign in with Apple and Google state.
#[utoipa::path(
    get,
    path = PATH_GET_SIGN_IN_WITH_INFO,
    responses(
        (status = 200, description = "Successful.", body = SignInWithState),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_sign_in_with_info(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<SignInWithState>, StatusCode> {
    ACCOUNT.get_sign_in_with_info.incr();

    let info = state
        .read()
        .account()
        .account_sign_in_with_info(account_id)
        .await?;

    Ok(SignInWithState {
        apple: info.apple_account_id.is_some(),
        google: info.google_account_id.is_some(),
    }
    .into())
}

const PATH_PUT_SIGN_IN_WITH_APPLE: &str = "/account_api/sign_in_with_apple";

/// Associate or disassociate Apple sign in with account.
#[utoipa::path(
    put,
    path = PATH_PUT_SIGN_IN_WITH_APPLE,
    request_body = PutSignInWithApple,
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn put_sign_in_with_apple(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(body): Json<PutSignInWithApple>,
) -> Result<(), StatusCode> {
    ACCOUNT.put_sign_in_with_apple.incr();

    if let Some(apple) = body.apple {
        let nonce_bytes = base64::engine::general_purpose::URL_SAFE
            .decode(apple.nonce)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let info = state
            .sign_in_with_manager()
            .validate_apple_token(apple.token, nonce_bytes)
            .await?;

        if !info.email_verified {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }

        db_write!(state, move |cmds| {
            cmds.account()
                .sign_in_with()
                .update_apple_account_id(account_id, Some(AppleAccountId(info.id)))
                .await
        })?;
    } else {
        db_write!(state, move |cmds| {
            cmds.account()
                .sign_in_with()
                .update_apple_account_id(account_id, None)
                .await
        })?;
    }

    Ok(())
}

const PATH_PUT_SIGN_IN_WITH_GOOGLE: &str = "/account_api/sign_in_with_google";

/// Associate or disassociate Google sign in with account.
#[utoipa::path(
    put,
    path = PATH_PUT_SIGN_IN_WITH_GOOGLE,
    request_body = PutSignInWithGoogle,
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn put_sign_in_with_google(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(body): Json<PutSignInWithGoogle>,
) -> Result<(), StatusCode> {
    ACCOUNT.put_sign_in_with_google.incr();

    if let Some(google) = body.google {
        let nonce_bytes = base64::engine::general_purpose::URL_SAFE
            .decode(google.nonce)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let info = state
            .sign_in_with_manager()
            .validate_google_token(google.token, nonce_bytes)
            .await?;

        if !info.email_verified {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }

        db_write!(state, move |cmds| {
            cmds.account()
                .sign_in_with()
                .update_google_account_id(account_id, Some(GoogleAccountId(info.id)))
                .await
        })?;
    } else {
        db_write!(state, move |cmds| {
            cmds.account()
                .sign_in_with()
                .update_google_account_id(account_id, None)
                .await
        })?;
    }

    Ok(())
}

create_open_api_router!(
    fn router_sign_in_with,
    get_sign_in_with_info,
    put_sign_in_with_apple,
    put_sign_in_with_google,
);

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_SIGN_IN_WITH_COUNTERS_LIST,
    get_sign_in_with_info,
    put_sign_in_with_apple,
    put_sign_in_with_google,
);
