use std::sync::Arc;

use simple_backend_config::SimpleBackendConfig;
use simple_backend_model::{AppAttestation, AppAttestationResult, AppAttestationTypeNumber};

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
    /// this returns `None`.
    pub async fn validate(
        &self,
        client_type: Option<AppAttestationClientType>,
        attestation: Option<&AppAttestation>,
    ) -> std::result::Result<Option<AppAttestationResult>, AppAttestationError> {
        let Some(config) = self.config.app_attestation() else {
            return Ok(None);
        };

        let Some(attestation) = attestation else {
            return Err(AppAttestationError::Failed);
        };

        if let Some(play_integrity) = &attestation.play_integrity {
            // Google Play Integrity API is only supported on Android clients.
            if client_type != Some(AppAttestationClientType::Android) {
                return Err(AppAttestationError::Failed);
            }
            let Some(play_integrity_config) = &config.play_integrity else {
                return Err(AppAttestationError::Failed);
            };
            let nothing_required = !play_integrity_config.require_device_integrity
                && !play_integrity_config.require_app_integrity;
            match self
                .play_integrity
                .validate(play_integrity, play_integrity_config)
                .await
            {
                Ok(integrity) => {
                    return Ok(Some(AppAttestationResult::Success {
                        attestation_type: AppAttestationTypeNumber::PlayIntegrity,
                        integrity,
                    }));
                }
                Err(_) if nothing_required => {
                    return Ok(Some(AppAttestationResult::Failure {
                        attestation_type: AppAttestationTypeNumber::PlayIntegrity,
                    }));
                }
                Err(error) => {
                    return Err(match error {
                        PlayIntegrityError::Failed => AppAttestationError::Failed,
                        PlayIntegrityError::DeviceIntegrity => AppAttestationError::DeviceIntegrity,
                        PlayIntegrityError::AppIntegrity => AppAttestationError::AppIntegrity,
                    });
                }
            }
        }

        Err(AppAttestationError::Failed)
    }
}
