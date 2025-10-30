use std::time::Duration;

use axum::{
    Extension,
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
};
use axum_extra::{TypedHeader, headers::ContentType};
use model::{AccessToken, AccountIdInternal};
use model_account::SendConfirmEmailMessageResult;
use server_api::{
    S,
    app::{ReadData, WriteData},
    common::AcceptLanguage,
    create_open_api_router, db_write,
    utils::Json,
};
use server_data::{app::GetConfig, read::GetReadCommandsCommon};
use server_data_account::{
    read::GetReadCommandsAccount,
    write::{GetWriteCommandsAccount, account::email::TokenCheckResult},
};
use simple_backend::create_counters;
use tokio::time::timeout;

pub const PATH_GET_CONFIRM_EMAIL: &str = "/account_api/confirm_email/{token}";

/// Confirm email address using the token sent via email.
/// This endpoint is meant to be accessed via a link in the confirmation email.
///
/// This modifies server state even if the HTTP method is GET.
///
/// Returns plain text response indicating success or failure.
#[utoipa::path(
    get,
    path = PATH_GET_CONFIRM_EMAIL,
    params(AccessToken),
    responses(
        (status = 200, description = "Email confirmed successfully.", content_type = "text/plain"),
        (status = 400, description = "Invalid or expired token.", content_type = "text/plain"),
        (status = 500, description = "Internal server error.", content_type = "text/plain"),
    ),
    security(),
)]
pub async fn get_confirm_email(
    State(state): State<S>,
    Path(token): Path<AccessToken>,
    accept_language: Option<TypedHeader<AcceptLanguage>>,
) -> Result<(TypedHeader<ContentType>, Bytes), (StatusCode, TypedHeader<ContentType>, Bytes)> {
    ACCOUNT.get_confirm_email.incr();

    let token = match token.bytes() {
        Ok(bytes) => bytes,
        Err(_) => {
            return create_invalid_token_response(&state, accept_language);
        }
    };

    let result = db_write!(state, move |cmds| {
        cmds.account().email().confirm_email_with_token(token).await
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
    match web_config.get(language.as_ref()).email_confirmed() {
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
    match web_config
        .get(language.as_ref())
        .email_confirmation_invalid()
    {
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

pub const PATH_POST_SEND_CONFIRM_EMAIL_MESSAGE: &str = "/account_api/send_confirm_email_message";

#[utoipa::path(
    post,
    path = PATH_POST_SEND_CONFIRM_EMAIL_MESSAGE,
    responses(
        (status = 200, description = "Successfull.", body = SendConfirmEmailMessageResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(),
)]
pub async fn post_send_confirm_email_message(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<SendConfirmEmailMessageResult>, StatusCode> {
    ACCOUNT.post_send_confirm_email_message.incr();

    // Check if email is already verified
    let account = state
        .read()
        .common()
        .account(account_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if account.email_verified() {
        return Ok(SendConfirmEmailMessageResult::error_email_already_verified().into());
    }

    // Check email confirmation token age
    let account_internal = state
        .read()
        .account()
        .account_internal(account_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(token_time) = account_internal.email_confirmation_token_unix_time {
        let min_wait_duration = state
            .config()
            .limits_account()
            .email_confirmation_resend_min_wait_duration;
        if !token_time.duration_value_elapsed(min_wait_duration) {
            return Ok(
                SendConfirmEmailMessageResult::error_try_again_later_after_seconds(
                    min_wait_duration.seconds,
                )
                .into(),
            );
        }
    }

    // Try to send email with 10 second timeout
    let send_result = timeout(Duration::from_secs(10), async {
        db_write!(state, move |cmds| {
            cmds.account()
                .email()
                .send_email_confirmation_high_priority(account_id)
                .await
        })
    })
    .await;

    match send_result {
        Ok(Ok(())) => {
            // Email sent successfully
            Ok(SendConfirmEmailMessageResult::ok().into())
        }
        Ok(Err(_)) => {
            // Email sending failed
            Ok(SendConfirmEmailMessageResult::error_email_sending_failed().into())
        }
        Err(_) => {
            // Timeout
            Ok(SendConfirmEmailMessageResult::error_email_sending_timeout().into())
        }
    }
}

create_open_api_router!(
    fn router_email_private,
    post_send_confirm_email_message,
);

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_EMAIL_COUNTERS_LIST,
    get_confirm_email,
    post_send_confirm_email_message,
);
