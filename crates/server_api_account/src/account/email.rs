use axum::{
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
};
use axum_extra::{TypedHeader, headers::ContentType};
use model::AccessToken;
use server_api::{S, app::WriteData, db_write};
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
) -> Result<(TypedHeader<ContentType>, Bytes), (StatusCode, TypedHeader<ContentType>, Bytes)> {
    ACCOUNT.get_confirm_email.incr();

    let token = match token.bytes() {
        Ok(bytes) => bytes,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                TypedHeader(ContentType::text_utf8()),
                "Invalid token format".into(),
            ));
        }
    };

    let result = db_write!(state, move |cmds| {
        cmds.account().email().confirm_email_with_token(token).await
    })
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            TypedHeader(ContentType::text_utf8()),
            "Internal server error".into(),
        )
    })?;

    match result {
        TokenCheckResult::Valid => Ok((
            TypedHeader(ContentType::text_utf8()),
            "Email confirmed successfully!".into(),
        )),
        TokenCheckResult::Invalid => Err((
            StatusCode::BAD_REQUEST,
            TypedHeader(ContentType::text_utf8()),
            "Invalid or expired token".into(),
        )),
    }
}

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_EMAIL_COUNTERS_LIST,
    get_confirm_email,
);
