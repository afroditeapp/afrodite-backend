use std::time::Duration;

use axum::{
    Extension,
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
};
use axum_extra::{TypedHeader, headers::ContentType};
use model::{AccessToken, AccountIdInternal, AccountState, Permissions};
use model_account::{
    EmailAddressState, InitEmailChange, InitEmailChangeResult, SendVerifyEmailMessageResult,
    SetEmailLoginEnabled, SetInitialEmail,
};
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
use simple_backend_utils::time::seconds_until_current_time_is_at;
use tokio::time::timeout;

use crate::app::GetAccounts;

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

            let internal = cmds
                .read()
                .account()
                .email_address_state_internal(account_id)
                .await?;

            if let Some(token_time) = internal.email_verification_token_unix_time {
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

pub(crate) async fn init_email_change_impl(
    state: &S,
    account_id: AccountIdInternal,
    new_email: model_account::EmailAddress,
) -> Result<InitEmailChangeResult, crate::utils::StatusCode> {
    let send_result = timeout(Duration::from_secs(10), async {
        db_write!(state, move |cmds| {
            let internal = cmds
                .read()
                .account()
                .email_address_state_internal(account_id)
                .await?;

            if internal.email.is_none() {
                return Ok(InitEmailChangeResult::error_email_sending_failed());
            }

            if internal.email.as_ref() == Some(&new_email) {
                return Ok(InitEmailChangeResult::error_email_sending_failed());
            }

            if let Some(change_time) = internal.email_change_unix_time {
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
                .init_email_change(account_id, new_email)
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
        Ok(Ok(r)) => Ok(r),
        Ok(Err(_)) => Ok(InitEmailChangeResult::error_email_sending_failed()),
        Err(_) => Ok(InitEmailChangeResult::error_email_sending_timeout()),
    }
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

    let result = init_email_change_impl(&state, account_id, request.new_email).await?;
    Ok(result.into())
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

pub const PATH_POST_SET_EMAIL_LOGIN_ENABLED: &str = "/account_api/set_email_login_enabled";

/// Enable or disable email login for an account.
///
/// Users can set this for their own account.
/// Admins with `admin_edit_login` permission can set this for any account.
///
/// This is useful to prevent email login spam attacks.
#[utoipa::path(
    post,
    path = PATH_POST_SET_EMAIL_LOGIN_ENABLED,
    request_body = SetEmailLoginEnabled,
    responses(
        (status = 200, description = "Successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_set_email_login_enabled(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Extension(api_caller_permissions): Extension<Permissions>,
    Json(request): Json<SetEmailLoginEnabled>,
) -> Result<(), crate::utils::StatusCode> {
    ACCOUNT.post_set_email_login_enabled.incr();

    let target_account = state.get_internal_id(request.aid).await?;

    let is_own_account = api_caller_account_id == target_account;
    let has_admin_permission = api_caller_permissions.admin_edit_login;

    if !is_own_account && !has_admin_permission {
        return Err(crate::utils::StatusCode::INTERNAL_SERVER_ERROR);
    }

    db_write!(state, move |cmds| {
        cmds.account()
            .email()
            .set_email_login_enabled(target_account, request.enabled)
            .await
    })?;

    Ok(())
}

const PATH_GET_EMAIL_ADDRESS_STATE: &str = "/account_api/email_address_state";

#[utoipa::path(
    get,
    path = PATH_GET_EMAIL_ADDRESS_STATE,
    responses(
        (status = 200, description = "Request successfull.", body = EmailAddressState),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_email_address_state(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
) -> Result<Json<EmailAddressState>, crate::utils::StatusCode> {
    ACCOUNT.get_email_address_state.incr();
    let mut data = state
        .read()
        .account()
        .email_address_state(api_caller_account_id)
        .await?;

    let internal = state
        .read()
        .account()
        .email_address_state_internal(api_caller_account_id)
        .await?;

    if let Some(init_time) = internal.email_change_unix_time {
        let wait_duration_seconds = state
            .config()
            .limits_account()
            .email_change_min_wait_duration
            .seconds;

        let scheduled_tasks_config = state.config().simple_backend().scheduled_tasks();
        let next_scheduled_tasks_run =
            seconds_until_current_time_is_at(scheduled_tasks_config.daily_start_time)
                .map_err(|_| crate::utils::StatusCode::INTERNAL_SERVER_ERROR)?;
        let next_scheduled_tasks_run = TryInto::<u32>::try_into(next_scheduled_tasks_run)
            .map_err(|_| crate::utils::StatusCode::INTERNAL_SERVER_ERROR)?;

        data.email_change_completion_time = Some(
            init_time
                .add_seconds(wait_duration_seconds)
                .add_seconds(next_scheduled_tasks_run),
        );
    }

    Ok(data.into())
}

create_open_api_router!(
    fn router_email_private,
    post_send_verify_email_message,
    post_cancel_email_change,
    post_init_email_change,
    post_initial_email,
    post_set_email_login_enabled,
    get_email_address_state,
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
    post_set_email_login_enabled,
    get_email_address_state,
);
