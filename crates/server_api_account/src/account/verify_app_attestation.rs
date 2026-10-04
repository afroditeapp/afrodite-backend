use axum::extract::State;
use model::ClientType;
use model_account::{
    LoginPhaseTwoResult, RequestAppAttestChallenge, RequestAppAttestChallengeResult,
    VerifyAppAttestationLogin,
};
use server_api::{
    S,
    app::{ApiLimitsProvider, GetAccounts},
    db_write,
};
use server_data::write::GetWriteCommandsCommon;
use simple_backend::{
    app::AppAttestationProvider,
    app_attestation::{AppAttestationClientType, AppAttestationError},
    create_counters,
};
use simple_backend_model::{AppAttestation, AppAttestationResult};

use super::login::login_impl;
use crate::{
    app::WriteData,
    utils::{ClientConnectionId, Json, StatusCode},
};

pub const PATH_POST_REQUEST_APP_ATTEST_CHALLENGE: &str =
    "/account_api/request_app_attest_challenge";

/// Request a random string for app attestation token.
///
/// The client must call this after login phase one to obtain a challenge and
/// then put it into the `requestHash` field of the Play Integrity API verdict
/// token before presenting it to `post_verify_app_attestation`.
#[utoipa::path(
    post,
    path = PATH_POST_REQUEST_APP_ATTEST_CHALLENGE,
    security(),
    request_body = RequestAppAttestChallenge,
    responses(
        (status = 200, description = "Challenge generated.", body = RequestAppAttestChallengeResult),
        (status = 500, description = "Internal server error."),
    ),
)]
pub async fn post_request_app_attest_challenge(
    State(state): State<S>,
    ClientConnectionId(connection): ClientConnectionId,
    Json(request): Json<RequestAppAttestChallenge>,
) -> Result<Json<RequestAppAttestChallengeResult>, StatusCode> {
    ACCOUNT.post_request_app_attest_challenge.incr();

    let Ok(token_bytes) = request.token.bytes() else {
        return Ok(
            RequestAppAttestChallengeResult::error_invalid_verify_app_attestation_token().into(),
        );
    };

    let Some(challenge) = state
        .verify_app_attestation_tokens()
        .generate_challenge(&token_bytes, connection.ip())
        .await
    else {
        return Ok(
            RequestAppAttestChallengeResult::error_invalid_verify_app_attestation_token().into(),
        );
    };

    Ok(RequestAppAttestChallengeResult::ok(challenge).into())
}

pub const PATH_POST_VERIFY_APP_ATTESTATION: &str = "/account_api/verify_app_attestation";

/// Complete a pending login by verifying app attestation.
///
/// The login routes (`post_sign_in_with_login`, `post_email_login_with_token`,
/// `post_demo_account_login_to_account`) return a short-lived
/// `verify_app_attestation_token` instead of logging the account in directly.
/// This route consumes that token and completes the login only if the app
/// attestation passes.
#[utoipa::path(
    post,
    path = PATH_POST_VERIFY_APP_ATTESTATION,
    security(),
    request_body = VerifyAppAttestationLogin,
    responses(
        (status = 200, description = "Login successful.", body = LoginPhaseTwoResult),
        (status = 500, description = "Internal server error."),
    ),
)]
pub async fn post_verify_app_attestation(
    State(state): State<S>,
    ClientConnectionId(connection): ClientConnectionId,
    Json(request): Json<VerifyAppAttestationLogin>,
) -> Result<Json<LoginPhaseTwoResult>, StatusCode> {
    ACCOUNT.post_verify_app_attestation.incr();

    let Ok(token_bytes) = request.token.bytes() else {
        return Ok(LoginPhaseTwoResult::error_invalid_verify_app_attestation_token().into());
    };

    let Some((account_id, client_type, connection_ip, challenge)) = state
        .verify_app_attestation_tokens()
        .consume(&token_bytes)
        .await
    else {
        return Ok(LoginPhaseTwoResult::error_invalid_verify_app_attestation_token().into());
    };

    // Bind the token to the IP address that initiated the login so it cannot
    // be replayed from a different IP.
    if connection.ip() != connection_ip {
        return Ok(LoginPhaseTwoResult::error_invalid_verify_app_attestation_token().into());
    }

    if request
        .app_attestation
        .as_ref()
        .is_some_and(|attestation| attestation.play_integrity.is_some())
        && state
            .api_limits(account_id)
            .account()
            .play_integrity_daily_max_count()
            .await
            .is_err()
    {
        return Ok(LoginPhaseTwoResult::error_play_integrity_limit_reached().into());
    }

    let attestation_result = match validate_app_attestation(
        &state,
        client_type,
        request.app_attestation.as_ref(),
        challenge.as_deref(),
    )
    .await
    {
        Ok(result) => result,
        Err(error) => return Ok(error.into()),
    };

    let r = login_impl(account_id.as_id(), connection, &state).await?;

    if let Some(aid) = r.aid() {
        // Login successful
        let id = state.get_internal_id(aid).await?;
        db_write!(state, move |cmds| {
            cmds.common()
                .client_config()
                .client_login_session_platform(id, client_type)
                .await?;
            cmds.common()
                .client_config()
                .app_attestation(id, attestation_result)
                .await?;
            Ok(())
        })?;
    }

    Ok(LoginPhaseTwoResult::from(r).into())
}

/// Validate app attestation provided by the client against server config.
///
/// If app attestation is not configured, attestation is not required and
/// this returns `Ok(None)`.
async fn validate_app_attestation(
    state: &S,
    client_type: ClientType,
    attestation: Option<&AppAttestation>,
    challenge: Option<&str>,
) -> Result<Option<AppAttestationResult>, LoginPhaseTwoResult> {
    let client_type = match client_type {
        ClientType::Android => Some(AppAttestationClientType::Android),
        _ => None,
    };
    state
        .app_attestation_manager()
        .validate(client_type, attestation, challenge)
        .await
        .map_err(|error| match error {
            AppAttestationError::Failed => LoginPhaseTwoResult::error_app_attestation_failed(),
            AppAttestationError::DeviceIntegrity => {
                LoginPhaseTwoResult::error_app_attestation_device_integrity()
            }
            AppAttestationError::AppIntegrity => {
                LoginPhaseTwoResult::error_app_attestation_app_integrity()
            }
        })
}

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_VERIFY_APP_ATTESTATION_COUNTERS_LIST,
    post_verify_app_attestation,
    post_request_app_attest_challenge,
);
