use std::path::PathBuf;

use fcm::yup_oauth2::{
    HyperClientBuilder, ServiceAccountAuthenticator, authenticator::Authenticator,
    client::DefaultHyperClientBuilder,
};
use simple_backend_config::file::PlayIntegrityAppAttestationConfig;
use simple_backend_model::{AppIntegrityResult, PlayIntegrityAppAttestation};
use tracing::error;

const PLAY_INTEGRITY_OAUTH_SCOPE: &str = "https://www.googleapis.com/auth/playintegrity";

type GoogleOAuthAuthenticator =
    Authenticator<<DefaultHyperClientBuilder as HyperClientBuilder>::Connector>;

pub enum PlayIntegrityError {
    Failed,
    DeviceIntegrity,
    AppIntegrity,
}

/// Validator for Google Play Integrity API tokens.
///
/// Decrypts and verifies verdict tokens on Google's servers using the
/// `decodeIntegrityToken` endpoint and then checks the decoded payload
/// against the server config.
pub struct PlayIntegrityManager {
    oauth: Option<GoogleOAuthAuthenticator>,
    reqwest_client: reqwest::Client,
}

impl PlayIntegrityManager {
    pub async fn new(
        service_account_key_path: Option<PathBuf>,
        reqwest_client: reqwest::Client,
    ) -> Self {
        let oauth = match service_account_key_path {
            Some(key_path) => match create_service_account_authenticator(key_path).await {
                Ok(oauth) => Some(oauth),
                Err(e) => {
                    error!("Creating Play Integrity service account authenticator failed: {e}");
                    None
                }
            },
            None => None,
        };
        Self {
            oauth,
            reqwest_client,
        }
    }

    pub async fn validate(
        &self,
        attestation: &PlayIntegrityAppAttestation,
        config: &PlayIntegrityAppAttestationConfig,
    ) -> Result<AppIntegrityResult, PlayIntegrityError> {
        let Some(oauth) = &self.oauth else {
            return Err(PlayIntegrityError::Failed);
        };

        // Fetch an access token for the Play Integrity API.
        let access_token = oauth
            .token(&[PLAY_INTEGRITY_OAUTH_SCOPE])
            .await
            .ok()
            .and_then(|t| t.token().map(str::to_owned))
            .ok_or(PlayIntegrityError::Failed)?;

        let response = self
            .reqwest_client
            .post(format!(
                "https://playintegrity.googleapis.com/v1/{}:decodeIntegrityToken",
                config.package_name
            ))
            .bearer_auth(access_token)
            .json(&serde_json::json!({
                "integrityToken": attestation.token,
            }))
            .send()
            .await
            .map_err(|_| PlayIntegrityError::Failed)?;

        if !response.status().is_success() {
            return Err(PlayIntegrityError::Failed);
        }

        let decode_response: serde_json::Value = response
            .json()
            .await
            .map_err(|_| PlayIntegrityError::Failed)?;
        let payload = decode_response
            .get("tokenPayloadExternal")
            .ok_or(PlayIntegrityError::Failed)?;

        // Verify the token was issued for our app.
        let request_package_name = payload
            .get("requestDetails")
            .and_then(|d| d.get("requestPackageName"))
            .and_then(|v| v.as_str())
            .ok_or(PlayIntegrityError::Failed)?;
        if request_package_name != config.package_name {
            return Err(PlayIntegrityError::Failed);
        }

        // Verify the token is fresh enough to prevent replay attacks. The
        // standard API mitigates replay attacks automatically, but checking
        // freshness is still recommended.
        let timestamp_millis = payload
            .get("requestDetails")
            .and_then(|d| d.get("timestampMillis"))
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
            .ok_or(PlayIntegrityError::Failed)?;
        let now_millis = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let max_age_millis = config.max_age_seconds.saturating_mul(1000);
        if now_millis.saturating_sub(timestamp_millis) > max_age_millis {
            return Err(PlayIntegrityError::Failed);
        }

        // Verify device integrity.
        let device_integrity = payload
            .get("deviceIntegrity")
            .and_then(|d| d.get("deviceRecognitionVerdict"))
            .and_then(|v| v.as_array())
            .map(|verdicts| {
                verdicts
                    .iter()
                    .any(|v| v.as_str() == Some("MEETS_DEVICE_INTEGRITY"))
            })
            .unwrap_or(false);
        if config.require_device_integrity && !device_integrity {
            return Err(PlayIntegrityError::DeviceIntegrity);
        }

        // Verify app integrity.
        let app_recognition_verdict = payload
            .get("appIntegrity")
            .and_then(|d| d.get("appRecognitionVerdict"))
            .and_then(|v| v.as_str())
            .ok_or(PlayIntegrityError::Failed)?;
        let app_integrity = app_recognition_verdict == "PLAY_RECOGNIZED";
        if config.require_app_integrity && !app_integrity {
            return Err(PlayIntegrityError::AppIntegrity);
        }

        Ok(AppIntegrityResult {
            app_integrity,
            device_integrity,
        })
    }
}

async fn create_service_account_authenticator(
    service_account_key_path: PathBuf,
) -> Result<GoogleOAuthAuthenticator, fcm::yup_oauth2::Error> {
    let key = fcm::yup_oauth2::read_service_account_key(service_account_key_path).await?;
    Ok(ServiceAccountAuthenticator::builder(key).build().await?)
}
