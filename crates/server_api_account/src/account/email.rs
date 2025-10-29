use axum::{
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
};
use axum_extra::{TypedHeader, headers::ContentType};
use model::AccessToken;
use server_api::{S, app::WriteData, common::AcceptLanguage, db_write};
use server_data::app::GetConfig;
use server_data_account::write::{GetWriteCommandsAccount, account::email::TokenCheckResult};
use simple_backend::create_counters;

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

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_EMAIL_COUNTERS_LIST,
    get_confirm_email,
);
