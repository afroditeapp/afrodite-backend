use std::sync::Arc;

use base64::Engine;
use sha2::{Digest, Sha256};
use simple_backend_config::SimpleBackendConfig;
use simple_backend_model::{AppAttestation, DebugAppAttestationToken};

pub enum AppAttestationError {
    Failed,
    DeviceIntegrity,
    AppIntegrity,
}

pub struct AppAttestationManager {
    config: Arc<SimpleBackendConfig>,
}

impl AppAttestationManager {
    pub fn new(config: Arc<SimpleBackendConfig>) -> Self {
        Self { config }
    }

    /// Validate app attestation provided by the client against server config.
    ///
    /// If app attestation is not configured, attestation is not required and
    /// this always succeeds.
    pub fn validate(
        &self,
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
            Ok(())
        } else {
            Err(AppAttestationError::Failed)
        }
    }
}
