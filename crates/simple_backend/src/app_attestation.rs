use std::sync::Arc;

use base64::Engine;
use sha2::{Digest, Sha256};
use simple_backend_config::SimpleBackendConfig;
use simple_backend_model::{AppAttestation, DebugAppAttestationToken};

use crate::app_attestation::play_integrity::{PlayIntegrityError, PlayIntegrityManager};

mod play_integrity;

#[derive(PartialEq)]
pub enum AppAttestationClientType {
    Android,
}

pub enum AppAttestationError {
    Failed,
    DeviceIntegrity,
    AppIntegrity,
}

pub struct AppAttestationManager {
    config: Arc<SimpleBackendConfig>,
    play_integrity: PlayIntegrityManager,
}

impl AppAttestationManager {
    pub async fn new(config: Arc<SimpleBackendConfig>, reqwest_client: reqwest::Client) -> Self {
        let service_account_key_path = config
            .app_attestation()
            .and_then(|c| c.play_integrity.as_ref())
            .map(|c| c.service_account_key_path.clone());
        let play_integrity =
            PlayIntegrityManager::new(service_account_key_path, reqwest_client).await;
        Self {
            config,
            play_integrity,
        }
    }

    /// Validate app attestation provided by the client against server config.
    ///
    /// If app attestation is not configured, attestation is not required and
    /// this always succeeds.
    pub async fn validate(
        &self,
        client_type: Option<AppAttestationClientType>,
        attestation: Option<&AppAttestation>,
    ) -> std::result::Result<(), AppAttestationError> {
        let Some(config) = self.config.app_attestation() else {
            return Ok(());
        };

        let Some(attestation) = attestation else {
            return Err(AppAttestationError::Failed);
        };

        if let Some(debug) = &attestation.debug {
            let Some(debug_config) = &config.debug else {
                return Err(AppAttestationError::Failed);
            };
            let Ok(token) = serde_json::from_str::<DebugAppAttestationToken>(&debug.token) else {
                return Err(AppAttestationError::Failed);
            };
            if debug_config.require_device_integrity && !token.device_integrity {
                return Err(AppAttestationError::DeviceIntegrity);
            }
            if debug_config.require_app_integrity && !token.app_integrity {
                return Err(AppAttestationError::AppIntegrity);
            }
            let Ok(nonce_bytes) = base64::engine::general_purpose::URL_SAFE.decode(&debug.nonce)
            else {
                return Err(AppAttestationError::Failed);
            };
            let token_nonce =
                base64::engine::general_purpose::URL_SAFE.encode(Sha256::digest(nonce_bytes));
            if token.nonce != token_nonce {
                return Err(AppAttestationError::Failed);
            }
            return Ok(());
        }

        if let Some(play_integrity) = &attestation.play_integrity {
            // Google Play Integrity API is only supported on Android clients.
            if client_type != Some(AppAttestationClientType::Android) {
                return Err(AppAttestationError::Failed);
            }
            let Some(play_integrity_config) = &config.play_integrity else {
                return Err(AppAttestationError::Failed);
            };
            return self
                .play_integrity
                .validate(play_integrity, play_integrity_config)
                .await
                .map_err(|error| match error {
                    PlayIntegrityError::Failed => AppAttestationError::Failed,
                    PlayIntegrityError::DeviceIntegrity => AppAttestationError::DeviceIntegrity,
                    PlayIntegrityError::AppIntegrity => AppAttestationError::AppIntegrity,
                });
        }

        Err(AppAttestationError::Failed)
    }
}
