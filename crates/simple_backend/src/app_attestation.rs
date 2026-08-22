use std::sync::Arc;

use simple_backend_config::{SimpleBackendConfig, file::PlayIntegrityAppAttestationConfig};
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
        // Skip app attestation validation if server config for the required
        // app attestation method does not exist.
        match client_type {
            Some(AppAttestationClientType::Android) => {
                if let Some(config) = self.config.app_attestation()
                    && let Some(play_integrity_config) = &config.play_integrity
                {
                    self.handle_play_integrity(play_integrity_config, attestation)
                        .await
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    async fn handle_play_integrity(
        &self,
        play_integrity_config: &PlayIntegrityAppAttestationConfig,
        attestation: Option<&AppAttestation>,
    ) -> std::result::Result<Option<AppAttestationResult>, AppAttestationError> {
        let nothing_required = !play_integrity_config.require_device_integrity
            && !play_integrity_config.require_app_integrity;

        if let Some(attestation) = attestation
            && let Some(play_integrity) = &attestation.play_integrity
        {
            match self
                .play_integrity
                .validate(play_integrity, play_integrity_config)
                .await
            {
                Ok(integrity) => Ok(Some(AppAttestationResult::Success {
                    attestation_type: AppAttestationTypeNumber::PlayIntegrity,
                    integrity,
                })),
                Err(_) if nothing_required => Ok(Some(AppAttestationResult::Failure {
                    attestation_type: AppAttestationTypeNumber::PlayIntegrity,
                })),
                Err(error) => Err(match error {
                    PlayIntegrityError::Failed => AppAttestationError::Failed,
                    PlayIntegrityError::DeviceIntegrity => AppAttestationError::DeviceIntegrity,
                    PlayIntegrityError::AppIntegrity => AppAttestationError::AppIntegrity,
                }),
            }
        } else {
            if nothing_required {
                Ok(None)
            } else {
                Err(AppAttestationError::Failed)
            }
        }
    }
}
