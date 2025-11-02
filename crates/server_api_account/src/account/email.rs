use std::time::Duration;

use axum::{
    Extension,
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
};
use axum_extra::{TypedHeader, headers::ContentType};
use model::{AccessToken, AccountIdInternal, AccountState};
use model_account::{
    InitEmailChange, InitEmailChangeResult, SendVerifyEmailMessageResult, SetInitialEmail,
};
use server_api::{
    S, app::WriteData, common::AcceptLanguage, create_open_api_router, db_write, utils::Json,
};
use server_data::{app::GetConfig, read::GetReadCommandsCommon};
use server_data_account::{
    read::GetReadCommandsAccount,
    write::{GetWriteCommandsAccount, account::email::TokenCheckResult},
};
use simple_backend::create_counters;
use tokio::time::timeout;

pub const PATH_GET_VERIFY_EMAIL: &str = "/account_api/verify_email/{token}";

/// Verify email address using the token sent via email.
/// This endpoint is meant to be accessed via a link in the verification email.
/// To workaround email security scanning related link accessing, the link
/// can be opened multiple times.
///
/// This modifies server state even if the HTTP method is GET.
///
/// Returns plain text response indicating success or failure.
#[utoipa::path(
    get,
    path = PATH_GET_VERIFY_EMAIL,
    params(AccessToken),
    responses(
        (status = 200, description = "Email verified successfully.", content_type = "text/plain"),
        (status = 400, description = "Invalid or expired token.", content_type = "text/plain"),
        (status = 500, description = "Internal server error.", content_type = "text/plain"),
    ),
    security(),
)]
pub async fn get_verify_email(
    State(state): State<S>,
    Path(token): Path<AccessToken>,
    accept_language: Option<TypedHeader<AcceptLanguage>>,
) -> Result<(TypedHeader<ContentType>, Bytes), (StatusCode, TypedHeader<ContentType>, Bytes)> {
    ACCOUNT.get_verify_email.incr();

    let token = match token.bytes() {
        Ok(bytes) => bytes,
        Err(_) => {
            return create_invalid_token_response(&state, accept_language);
        }
    };

    let result = db_write!(state, move |cmds| {
        cmds.account().email().verify_email_with_token(token).await
    });

    match result {
        Ok(TokenCheckResult::Valid) => create_success_response(&state, accept_language),
        Ok(TokenCheckResult::Invalid) => create_invalid_token_response(&state, accept_language),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            TypedHeader(ContentType::text_utf8()),
            Bytes::from("Internal Server Error"),
        )),
    }
}

#[allow(clippy::result_large_err)]
#[allow(clippy::type_complexity)]
fn create_success_response(
    state: &S,
    accept_language: Option<TypedHeader<AcceptLanguage>>,
) -> Result<(TypedHeader<ContentType>, Bytes), (StatusCode, TypedHeader<ContentType>, Bytes)> {
    let web_config = state.config().web_content();
    let language = accept_language.as_ref().map(|h| h.language());
    match web_config.get(language.as_ref()).email_verified() {
        Ok(page) => {
            let content_type = if page.is_html {
                ContentType::html()
            } else {
                ContentType::text_utf8()
            };
            Ok((TypedHeader(content_type), Bytes::from(page.content)))
        }
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            TypedHeader(ContentType::text_utf8()),
            Bytes::from("Internal Server Error"),
        )),
    }
}

#[allow(clippy::result_large_err)]
#[allow(clippy::type_complexity)]
fn create_invalid_token_response(
    state: &S,
    accept_language: Option<TypedHeader<AcceptLanguage>>,
) -> Result<(TypedHeader<ContentType>, Bytes), (StatusCode, TypedHeader<ContentType>, Bytes)> {
    let web_config = state.config().web_content();
    let language = accept_language.as_ref().map(|h| h.language());
    match web_config.get(language.as_ref()).invalid_link() {
        Ok(page) => {
            let content_type = if page.is_html {
                ContentType::html()
            } else {
                ContentType::text_utf8()
            };
            Err((
                StatusCode::BAD_REQUEST,
                TypedHeader(content_type),
                Bytes::from(page.content),
            ))
        }
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            TypedHeader(ContentType::text_utf8()),
            Bytes::from("Internal Server Error"),
        )),
    }
}

pub const PATH_POST_SEND_VERIFY_EMAIL_MESSAGE: &str = "/account_api/send_verify_email_message";

#[utoipa::path(
    post,
    path = PATH_POST_SEND_VERIFY_EMAIL_MESSAGE,
    responses(
        (status = 200, description = "Successfull.", body = SendVerifyEmailMessageResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_send_verify_email_message(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<SendVerifyEmailMessageResult>, StatusCode> {
    ACCOUNT.post_send_verify_email_message.incr();

    let send_result = timeout(Duration::from_secs(10), async {
        db_write!(state, move |cmds| {
            let account = cmds.read().common().account(account_id).await?;

            if account.email_verified() {
                return Ok(SendVerifyEmailMessageResult::error_email_already_verified());
            }

            let account_internal = cmds.read().account().account_internal(account_id).await?;

            if let Some(token_time) = account_internal.email_verification_token_unix_time {
                let min_wait_duration = cmds
                    .config()
                    .limits_account()
                    .email_verification_resend_min_wait_duration;
                if !token_time.duration_value_elapsed(min_wait_duration) {
                    return Ok(
                        SendVerifyEmailMessageResult::error_try_again_later_after_seconds(
                            min_wait_duration.seconds,
                        ),
                    );
                }
            }

            cmds.account()
                .email()
                .send_email_verification_message_high_priority(account_id)
                .await?;

            Ok(SendVerifyEmailMessageResult::ok())
        })
    })
    .await;

    match send_result {
        Ok(Ok(r)) => Ok(r.into()),
        Ok(Err(_)) => {
            // Email sending failed
            Ok(SendVerifyEmailMessageResult::error_email_sending_failed().into())
        }
        Err(_) => {
            // Timeout
            Ok(SendVerifyEmailMessageResult::error_email_sending_timeout().into())
        }
    }
}

pub const PATH_GET_VERIFY_NEW_EMAIL: &str = "/account_api/verify_new_email/{token}";

/// Verify new email address using the token sent via email.
/// This endpoint is meant to be accessed via a link in the verification email.
/// To workaround email security scanning related link accessing, the link
/// can be opened multiple times.
///
/// This modifies server state even if the HTTP method is GET.
///
/// Returns plain text response indicating success or failure.
#[utoipa::path(
    get,
    path = PATH_GET_VERIFY_NEW_EMAIL,
    params(AccessToken),
    responses(
        (status = 200, description = "New email verified successfully.", content_type = "text/plain"),
        (status = 400, description = "Invalid or expired token.", content_type = "text/plain"),
        (status = 500, description = "Internal server error.", content_type = "text/plain"),
    ),
    security(),
)]
pub async fn get_verify_new_email(
    State(state): State<S>,
    Path(token): Path<AccessToken>,
    accept_language: Option<TypedHeader<AcceptLanguage>>,
) -> Result<(TypedHeader<ContentType>, Bytes), (StatusCode, TypedHeader<ContentType>, Bytes)> {
    ACCOUNT.get_verify_new_email.incr();

    let token = match token.bytes() {
        Ok(bytes) => bytes,
        Err(_) => {
            return create_invalid_token_response(&state, accept_language);
        }
    };

    let result = db_write!(state, move |cmds| {
        cmds.account()
            .email()
            .email_change_try_to_verify_new_email(token)
            .await
    });

    match result {
        Ok(TokenCheckResult::Valid) => create_success_response(&state, accept_language),
        Ok(TokenCheckResult::Invalid) => create_invalid_token_response(&state, accept_language),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            TypedHeader(ContentType::text_utf8()),
            Bytes::from("Internal Server Error"),
        )),
    }
}

pub const PATH_POST_CANCEL_EMAIL_CHANGE: &str = "/account_api/cancel_email_change";

/// Cancel email changing process
#[utoipa::path(
    post,
    path = PATH_POST_CANCEL_EMAIL_CHANGE,
    responses(
        (status = 200, description = "Successful."),
        (status = 400, description = "Invalid or expired token."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_cancel_email_change(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<(), StatusCode> {
    ACCOUNT.post_cancel_email_change.incr();

    db_write!(state, move |cmds| {
        cmds.account().email().cancel_email_change(account_id).await
    })?;

    Ok(())
}

pub const PATH_POST_INIT_EMAIL_CHANGE: &str = "/account_api/init_email_change";

/// Initiate email change process by providing a new email address.
///
/// The process:
/// 1. User provides new email address
/// 2. Verification email sent to new address
/// 3. Notification email sent to current address
/// 4. After configured time elapses and new email is verified, email changes
///
/// Error is returned when
///  - account does not already have email address set,
///  - the new email is the current email or
///  - email address change is already in progress.
#[utoipa::path(
    post,
    path = PATH_POST_INIT_EMAIL_CHANGE,
    request_body = InitEmailChange,
    responses(
        (status = 200, description = "Successfull.", body = InitEmailChangeResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_init_email_change(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(request): Json<InitEmailChange>,
) -> Result<Json<InitEmailChangeResult>, StatusCode> {
    ACCOUNT.post_init_email_change.incr();

    let send_result = timeout(Duration::from_secs(10), async {
        db_write!(state, move |cmds| {
            let account_data = cmds.read().account().account_data(account_id).await?;

            if account_data.email.is_none() {
                return Ok(InitEmailChangeResult::error_email_sending_failed());
            }

            if account_data.email.as_ref() == Some(&request.new_email) {
                return Ok(InitEmailChangeResult::error_email_sending_failed());
            }

            if let Some(change_time) = account_data.email_change_time {
                let min_wait_duration = cmds
                    .config()
                    .limits_account()
                    .email_change_resend_min_wait_duration;
                if !change_time.duration_value_elapsed(min_wait_duration) {
                    return Ok(InitEmailChangeResult::error_try_again_later_after_seconds(
                        min_wait_duration.seconds,
                    ));
                }
            }

            cmds.account()
                .email()
                .init_email_change(account_id, request.new_email)
                .await?;

            cmds.account()
                .email()
                .send_email_change_verification_high_priority(account_id)
                .await?;
            cmds.account()
                .email()
                .send_email_change_notification_high_priority(account_id)
                .await?;

            Ok(InitEmailChangeResult::ok())
        })
    })
    .await;

    match send_result {
        Ok(Ok(r)) => Ok(r.into()),
        Ok(Err(_)) => Ok(InitEmailChangeResult::error_email_sending_failed().into()),
        Err(_) => Ok(InitEmailChangeResult::error_email_sending_timeout().into()),
    }
}

const PATH_POST_INITIAL_EMAIL: &str = "/account_api/initial_email";

/// Set initial email when initial setup is ongoing
#[utoipa::path(
    post,
    path = PATH_POST_INITIAL_EMAIL,
    request_body(content = SetInitialEmail),
    responses(
        (status = 200, description = "Request successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_initial_email(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Extension(api_caller_account_state): Extension<AccountState>,
    Json(email): Json<SetInitialEmail>,
) -> Result<(), StatusCode> {
    ACCOUNT.post_initial_email.incr();

    if api_caller_account_state != AccountState::InitialSetup {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    db_write!(state, move |cmds| cmds
        .account()
        .email()
        .inital_setup_account_email_change(api_caller_account_id, email.email)
        .await)?;

    Ok(())
}

create_open_api_router!(
    fn router_email_private,
    post_send_verify_email_message,
    post_cancel_email_change,
    post_init_email_change,
    post_initial_email,
);

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_EMAIL_COUNTERS_LIST,
    get_verify_email,
    get_verify_new_email,
    post_cancel_email_change,
    post_send_verify_email_message,
    post_init_email_change,
    post_initial_email,
);
