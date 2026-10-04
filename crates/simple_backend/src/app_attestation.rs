use std::sync::Arc;

use simple_backend_config::{
    SimpleBackendConfig,
    file::{AppleAppAttestConfig, PlayIntegrityAppAttestationConfig},
};
use simple_backend_model::{AppAttestation, AppAttestationResult, AppAttestationTypeNumber};

use crate::{
    app::AppleAppAttestKeyStore,
    app_attestation::{
        apple_app_attest::AppleAppAttestManager,
        play_integrity::{PlayIntegrityError, PlayIntegrityManager},
    },
};

mod apple_app_attest;
mod play_integrity;

pub use apple_app_attest::decode_apple_app_attest_key_id;

#[derive(PartialEq)]
pub enum AppAttestationClientType {
    Android,
    Ios,
}

pub enum AppAttestationError {
    Failed,
    DeviceIntegrity,
    AppIntegrity,
}

pub struct AppAttestationManager {
    config: Arc<SimpleBackendConfig>,
    play_integrity: PlayIntegrityManager,
    apple_app_attest: AppleAppAttestManager,
}

impl AppAttestationManager {
    pub async fn new(config: Arc<SimpleBackendConfig>, reqwest_client: reqwest::Client) -> Self {
        let service_account_key_path = config
            .app_attestation()
            .and_then(|c| c.play_integrity.as_ref())
            .map(|c| c.service_account_key_path.clone());
        let play_integrity =
            PlayIntegrityManager::new(service_account_key_path, reqwest_client.clone()).await;
        let apple_app_attest = AppleAppAttestManager::new(reqwest_client.clone()).await;
        Self {
            config,
            play_integrity,
            apple_app_attest,
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
        challenge: Option<&str>,
        key_store: &impl AppleAppAttestKeyStore,
    ) -> std::result::Result<Option<AppAttestationResult>, AppAttestationError> {
        // Skip app attestation validation if server config for the required
        // app attestation method does not exist.
        match client_type {
            Some(AppAttestationClientType::Android) => {
                if let Some(config) = self.config.app_attestation()
                    && let Some(play_integrity_config) = &config.play_integrity
                {
                    self.handle_play_integrity(play_integrity_config, attestation, challenge)
                        .await
                } else {
                    Ok(None)
                }
            }
            Some(AppAttestationClientType::Ios) => {
                if let Some(config) = self.config.app_attestation()
                    && let Some(apple_config) = &config.apple_app_attest
                {
                    self.handle_apple_app_attest(apple_config, attestation, challenge, key_store)
                        .await
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    async fn handle_apple_app_attest(
        &self,
        apple_config: &AppleAppAttestConfig,
        attestation: Option<&AppAttestation>,
        challenge: Option<&str>,
        key_store: &impl AppleAppAttestKeyStore,
    ) -> std::result::Result<Option<AppAttestationResult>, AppAttestationError> {
        let result = match (
            attestation.and_then(|v| v.apple_app_attest.as_ref()),
            challenge,
        ) {
            (Some(attestation), Some(challenge)) => self
                .apple_app_attest
                .validate(apple_config, attestation, challenge, key_store)
                .await
                .map(|_| AppAttestationResult::Success {
                    attestation_type: AppAttestationTypeNumber::AppleAppAttest,
                    integrity: simple_backend_model::AppIntegrityResult {
                        app_integrity: false,
                        device_integrity: false,
                    },
                }),
            _ => Err(AppAttestationError::Failed),
        };

        match result {
            Ok(result) => Ok(Some(result)),
            Err(_) if !apple_config.require_valid_token => {
                Ok(Some(AppAttestationResult::Failure {
                    attestation_type: AppAttestationTypeNumber::AppleAppAttest,
                }))
            }
            Err(error) => Err(error),
        }
    }

    async fn handle_play_integrity(
        &self,
        play_integrity_config: &PlayIntegrityAppAttestationConfig,
        attestation: Option<&AppAttestation>,
        expected_request_hash: Option<&str>,
    ) -> std::result::Result<Option<AppAttestationResult>, AppAttestationError> {
        let nothing_required = !play_integrity_config.require_device_integrity
            && !play_integrity_config.require_app_integrity;

        if let Some(attestation) = attestation
            && let Some(play_integrity) = &attestation.play_integrity
        {
            match self
                .play_integrity
                .validate(play_integrity, play_integrity_config, expected_request_hash)
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
