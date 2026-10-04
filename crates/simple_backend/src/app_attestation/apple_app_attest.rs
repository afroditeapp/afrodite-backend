use appattest::{assertion::Assertion, attestation::Attestation};
use simple_backend_config::file::AppleAppAttestConfig;
use simple_backend_model::{AppleAppAttest, UnixTime};
use tokio::sync::Mutex;
use tracing::{error, warn};

use crate::{app::AppleAppAttestKeyStore, app_attestation::AppAttestationError};

const APPLE_ROOT_CERT_URL: &str =
    "https://www.apple.com/certificateauthority/Apple_App_Attestation_Root_CA.pem";

pub struct AppleAppAttestManager {
    reqwest_client: reqwest::Client,
    root_cert: Mutex<AppleAppAttestRootCert>,
}

impl AppleAppAttestManager {
    pub async fn new(reqwest_client: reqwest::Client) -> Self {
        let manager = Self {
            reqwest_client,
            root_cert: Mutex::new(AppleAppAttestRootCert::default()),
        };
        let mut lock = manager.root_cert.lock().await;
        let result = lock.get(&manager.reqwest_client).await;
        if result.is_err() {
            warn!("Fetching Apple App Attest root certificate at startup failed");
        }
        drop(lock);
        manager
    }

    pub async fn validate(
        &self,
        config: &AppleAppAttestConfig,
        attestation: &AppleAppAttest,
        challenge: &str,
        key_store: &impl AppleAppAttestKeyStore,
    ) -> Result<(), AppAttestationError> {
        if attestation.attestation.is_some() {
            // New key: verify the attestation and store the device public key.
            let (public_key, key_id) = self
                .verify_attestation(config, attestation, challenge)
                .await?;
            let created_at = UnixTime::current_time().ut;
            key_store
                .insert_apple_app_attest_key(&key_id, &public_key, created_at)
                .await
                .map_err(|_| AppAttestationError::Failed)?;
            Ok(())
        } else if attestation.assertion.is_some() {
            // Existing key: verify the assertion against the stored public key.
            let key_id = decode_apple_app_attest_key_id(&attestation.key_id)?;
            let public_key = key_store
                .apple_app_attest_public_key(&key_id)
                .await
                .map_err(|_| AppAttestationError::Failed)?
                .ok_or(AppAttestationError::Failed)?;
            self.verify_assertion(config, attestation, challenge, &public_key)
                .await
        } else {
            Err(AppAttestationError::Failed)
        }
    }

    /// Verify an Apple App Attest attestation.
    ///
    /// Returns the device public key (65 bytes) and the key id (32 bytes)
    /// when the attestation is valid for the configured app ID and the
    /// given challenge. The public key must be stored on the server for
    /// later assertion verification.
    async fn verify_attestation(
        &self,
        config: &AppleAppAttestConfig,
        attestation: &AppleAppAttest,
        challenge: &str,
    ) -> Result<([u8; 65], [u8; 32]), AppAttestationError> {
        let mut lock = self.root_cert.lock().await;
        let root_cert_pem = lock.get(&self.reqwest_client).await?;

        let Some(attestation_b64) = &attestation.attestation else {
            return Err(AppAttestationError::Failed);
        };

        let cbor =
            Attestation::decode_base64(attestation_b64).map_err(|_| AppAttestationError::Failed)?;
        let parsed =
            Attestation::from_cbor_bytes(&cbor).map_err(|_| AppAttestationError::Failed)?;

        let key_id = decode_apple_app_attest_key_id(&attestation.key_id)?;
        let app_id = app_id(config);
        let (public_key, _receipt) = parsed
            .verify(challenge, &app_id, &attestation.key_id, root_cert_pem)
            .map_err(|_| AppAttestationError::Failed)?;

        Ok((public_key, key_id))
    }

    /// Verify an Apple App Attest assertion.
    ///
    /// The assertion must be signed with the stored device public key of the
    /// attested key and the challenge must match the one issued for this login.
    async fn verify_assertion(
        &self,
        config: &AppleAppAttestConfig,
        attestation: &AppleAppAttest,
        challenge: &str,
        public_key: &[u8],
    ) -> Result<(), AppAttestationError> {
        let Some(assertion_b64) = &attestation.assertion else {
            return Err(AppAttestationError::Failed);
        };

        let mut buf = [0u8; 192];
        let assertion = Assertion::from_base64(assertion_b64, &mut buf)
            .map_err(|_| AppAttestationError::Failed)?;

        let app_id = app_id(config);
        let client_data_hash = client_data_hash(challenge);
        assertion
            .verify(
                client_data_hash,
                challenge,
                &app_id,
                public_key,
                0, // Assertion must have 1 or greater value
                challenge,
            )
            .map_err(|_| AppAttestationError::Failed)?;

        Ok(())
    }
}

/// Decode a base64-encoded key id into 32 raw bytes.
pub fn decode_apple_app_attest_key_id(key_id: &str) -> Result<[u8; 32], AppAttestationError> {
    use base64::Engine;
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(key_id)
        .map_err(|_| AppAttestationError::Failed)?;
    let arr: [u8; 32] = decoded
        .try_into()
        .map_err(|_| AppAttestationError::Failed)?;
    Ok(arr)
}

/// SHA-256 hash of the challenge string. The same value the device hashes
/// with SHA-256 before signing the assertion.
fn client_data_hash(challenge: &str) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    Sha256::digest(challenge.as_bytes()).into()
}

fn app_id(config: &AppleAppAttestConfig) -> String {
    format!("{}.{}", config.team_id, config.bundle_id)
}

/// Cached Apple App Attestation Root CA certificate PEM.
#[derive(Default)]
struct AppleAppAttestRootCert {
    pem: Option<Vec<u8>>,
}

impl AppleAppAttestRootCert {
    async fn get(
        &mut self,
        reqwest_client: &reqwest::Client,
    ) -> Result<&[u8], AppAttestationError> {
        let empty = match &mut self.pem {
            Some(pem) => return Ok(pem),
            empty @ None => empty,
        };

        let response = reqwest_client
            .get(APPLE_ROOT_CERT_URL)
            .send()
            .await
            .map_err(|e| {
                error!("Fetching Apple App Attest root certificate failed: {e}");
                AppAttestationError::Failed
            })?;
        if !response.status().is_success() {
            error!(
                "Fetching Apple App Attest root certificate failed: HTTP {}",
                response.status()
            );
            return Err(AppAttestationError::Failed);
        }
        let pem = response.bytes().await.map_err(|e| {
            error!("Reading Apple App Attest root certificate response failed: {e}");
            AppAttestationError::Failed
        })?;

        Ok(empty.insert(pem.to_vec()))
    }
}
