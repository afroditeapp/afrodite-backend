use std::net::SocketAddr;

use model::EmailLoginToken;
use model_account::{LoginResult, RequestEmailLoginToken, SignInWithInfo};
use server_api::{S, app::GetConfig, db_write};
use server_data::app::RegisterImplResult;
use server_data_account::write::GetWriteCommandsAccount;

use super::login_impl;
use crate::{account::login::EmailLoginResultInternal, app::WriteData, utils::StatusCode};

pub(super) async fn request_email_registration_token(
    state: &S,
    request: &RequestEmailLoginToken,
) -> Result<EmailLoginResultInternal, StatusCode> {
    let (client_token, client_token_bytes) = EmailLoginToken::generate_new_with_bytes();
    let (email_token, email_token_bytes) = EmailLoginToken::generate_new_with_bytes();

    let unix_time = model::UnixTime::current_time();
    let email = request.email.clone();

    state
        .email_registration_tokens()
        .insert(
            client_token_bytes,
            email_token_bytes.clone(),
            email.clone(),
            unix_time,
            state
                .config()
                .limits_account()
                .email_registration_token_validity_duration,
        )
        .await;

    let handle = state
        .email_channel_sender()
        .send_registration_login_token(request.email.0.clone(), email_token.into_string())?;

    Ok(EmailLoginResultInternal::successful(client_token, handle))
}

pub(super) async fn email_registration_with_token_impl(
    state: S,
    address: SocketAddr,
    client_token: Vec<u8>,
    email_token: Vec<u8>,
) -> Result<LoginResult, StatusCode> {
    let email = state
        .email_registration_tokens()
        .consume(
            &client_token,
            &email_token,
            state
                .config()
                .limits_account()
                .email_registration_token_validity_duration,
        )
        .await;

    let Some(email) = email else {
        return Ok(LoginResult::error_invalid_email_login_token());
    };

    let id = match state
        .data_all_access()
        .register_impl(SignInWithInfo::default(), Some(email))
        .await?
    {
        RegisterImplResult::Ok(id) => id,
        RegisterImplResult::EmailAlreadyExists => {
            return Ok(LoginResult::error_email_already_used());
        }
    };

    db_write!(state, move |cmds| {
        cmds.account()
            .update_syncable_account_data(id, |account| {
                account.email_verified = true;
                Ok(())
            })
            .await
    })?;

    // email_verified: no need to send events as user hasn't yet logged in

    login_impl(id.as_id(), address, &state).await
}
