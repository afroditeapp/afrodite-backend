use axum::extract::State;
use model::{AccountIdInternal, ClientType};
use model_account::{
    LoginPhaseTwoResult, RequestAppAttestChallenge, RequestAppAttestChallengeResult,
    VerifyAppAttestationLogin,
};
use server_api::{
    S,
    app::{ApiLimitsProvider, GetAccounts},
    db_write,
    utils::ClientIp,
};
use server_data::{read::GetReadCommandsCommon, write::GetWriteCommandsCommon};
use simple_backend::{
    app::{AppAttestationProvider, AppleAppAttestKeyStore, AppleAppAttestKeyStoreError},
    app_attestation::{AppAttestationClientType, AppAttestationError},
    create_counters,
};
use simple_backend_model::{AppAttestation, AppAttestationResult};
use simple_backend_utils::ContextExt;

use super::login::login_impl;
use crate::{
    app::{ReadData, WriteData},
    utils::{ClientConnectionId, Json, StatusCode},
};

/// Apple App Attest key store for a specific account.
struct AppleAppAttestKeyStoreImpl<'a> {
    state: &'a S,
    account_id: AccountIdInternal,
}

impl AppleAppAttestKeyStore for AppleAppAttestKeyStoreImpl<'_> {
    async fn apple_app_attest_public_key(
        &self,
        key_id: &[u8],
    ) -> simple_backend_utils::Result<Option<Vec<u8>>, AppleAppAttestKeyStoreError> {
        let key = self
            .state
            .read()
            .common()
            .client_config()
            .apple_app_attest_key(self.account_id, key_id)
            .await
            .map_err(|_| AppleAppAttestKeyStoreError::Failed.report())?;
        Ok(key.map(|k| k.public_key))
    }

    async fn insert_apple_app_attest_key(
        &self,
        key_id: &[u8],
        public_key: &[u8],
        created_at: i64,
    ) -> simple_backend_utils::Result<(), AppleAppAttestKeyStoreError> {
        let key_id = key_id.to_vec();
        let public_key = public_key.to_vec();
        let account_id = self.account_id;
        self.state
            .write(move |cmds| async move {
                cmds.common()
                    .client_config()
                    .insert_apple_app_attest_key(account_id, &key_id, &public_key, created_at, 5)
                    .await
            })
            .await
            .map_err(|_| AppleAppAttestKeyStoreError::Failed.report())?;
        Ok(())
    }
}

pub const PATH_POST_REQUEST_APP_ATTEST_CHALLENGE: &str =
    "/account_api/request_app_attest_challenge";

/// Request a random string for app attestation token.
///
/// The client must call this after login phase one to obtain a challenge and
/// then bind it to the app attestation before presenting it to
/// [post_verify_app_attestation]. The challenge is used as the Play Integrity
/// API `requestHash` and as the Apple App Attest `clientDataHash` input.
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
    ClientIp(ip): ClientIp,
    Json(request): Json<RequestAppAttestChallenge>,
) -> Result<Json<RequestAppAttestChallengeResult>, StatusCode> {
    ACCOUNT.post_request_app_attest_challenge.incr();

    let Ok(token_bytes) = request.token.bytes() else {
        return Ok(
            RequestAppAttestChallengeResult::error_invalid_verify_app_attestation_token().into(),
        );
    };

    let Some((challenge, account_id)) = state
        .verify_app_attestation_tokens()
        .generate_challenge(&token_bytes, ip, request.request_challenge)
        .await
    else {
        return Ok(
            RequestAppAttestChallengeResult::error_invalid_verify_app_attestation_token().into(),
        );
    };

    let key_exists = if let Some(key_id) = &request.apple_app_attest_key_id {
        let key_id_bytes =
            match simple_backend::app_attestation::decode_apple_app_attest_key_id(key_id) {
                Ok(bytes) => bytes,
                Err(_) => {
                    return Ok(
                    RequestAppAttestChallengeResult::error_invalid_verify_app_attestation_token()
                        .into(),
                );
                }
            };
        let key_exists = state
            .read()
            .common()
            .client_config()
            .apple_app_attest_key_exists(account_id, &key_id_bytes)
            .await?;
        Some(key_exists)
    } else {
        None
    };

    Ok(RequestAppAttestChallengeResult::ok(challenge, key_exists).into())
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

    if client_type != ClientType::Ios
        || request
            .app_attestation
            .as_ref()
            .map(|a| a.apple_app_attest.is_some())
            .unwrap_or_default()
    {
        state
            .verify_app_attestation_tokens()
            .remove_challenge(account_id)
            .await;
    } else {
        // Allow reusing old challenge on iOS when App Attest key attestation
        // fails because of App Attest server is unavailable.
        // AppAttestation::apple_app_attest is None in that case
        // (and also in other error cases but let's ignore that).
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
        account_id,
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
    account_id: AccountIdInternal,
    client_type: ClientType,
    attestation: Option<&AppAttestation>,
    challenge: Option<&str>,
) -> Result<Option<AppAttestationResult>, LoginPhaseTwoResult> {
    let client_type = match client_type {
        ClientType::Android => Some(AppAttestationClientType::Android),
        ClientType::Ios => Some(AppAttestationClientType::Ios),
        _ => None,
    };
    let key_store = AppleAppAttestKeyStoreImpl { state, account_id };
    state
        .app_attestation_manager()
        .validate(client_type, attestation, challenge, &key_store)
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
