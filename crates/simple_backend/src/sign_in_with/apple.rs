use std::{sync::Arc, time::{Duration, Instant}};

use error_stack::{Result, ResultExt};
use futures::lock::Mutex;
use jsonwebtoken::{jwk::{Jwk, JwkSet}, DecodingKey, Validation};
use serde::Deserialize;
use simple_backend_config::SimpleBackendConfig;
use simple_backend_utils::ContextExt;
use tracing::error;

const APPLE_JWT_ISS_VALUE: &str = "https://appleid.apple.com";

#[derive(thiserror::Error, Debug)]
pub enum SignInWithAppleError {
    #[error("Token (from client) header parsing failed")]
    InvalidTokenHeader,

    #[error("Token was invalid")]
    InvalidToken,

    #[error("Not configured")]
    NotConfigured,

    #[error("Requested Jwk was not found")]
    JwkNotFound,

    #[error("Couldn't download Apple public key")]
    PublicKeyDownloadFailed,

    #[error("HTTP GET for Apple public keys didn't contain valid JwkSet")]
    JwkSetParsingFailed,

    #[error("Token kid property not found from token received from client")]
    MissingJwtKid,

    #[error("Decoding key generation failed")]
    DecodingKeyGenerationFailed,
}

#[derive(Debug, Deserialize)]
struct AppleTokenClaims {
    /// Unique user identifier
    sub: String,
    /// Email address
    email: String,
    /// Email verification status
    email_verified: serde_json::Value,
}

impl AppleTokenClaims {
    fn email_verified(&self) -> bool {
        self.email_verified.as_bool() == Some(true) ||
            self.email_verified.as_str() == Some("true")
    }
}

pub struct AppleAccountInfo {
    pub id: String,
    pub email: String,
}

pub struct SignInWithAppleManager {
    config: Arc<SimpleBackendConfig>,
    public_keys: ApplePublicKeysState,
}

impl SignInWithAppleManager {
    pub fn new(config: Arc<SimpleBackendConfig>, client: reqwest::Client) -> Self {
        Self {
            config: config.clone(),
            public_keys: ApplePublicKeysState::new(config, client),
        }
    }
    pub async fn validate_apple_token(
        &self,
        token: String,
    ) -> Result<AppleAccountInfo, SignInWithAppleError> {
        let config = self
            .config
            .sign_in_with_apple_config()
            .ok_or(SignInWithAppleError::NotConfigured)?;

        let not_validated_header = jsonwebtoken::decode_header(&token)
            .change_context(SignInWithAppleError::InvalidTokenHeader)?;
        let wanted_kid = not_validated_header
            .kid
            .ok_or(SignInWithAppleError::MissingJwtKid)?;

        let apple_public_key = self.public_keys.get_public_key(&wanted_kid).await?;
        let key = DecodingKey::from_jwk(&apple_public_key)
            .change_context(SignInWithAppleError::DecodingKeyGenerationFailed)?;

        let mut v = Validation::new(not_validated_header.alg);
        v.set_required_spec_claims(&["exp", "iss", "aud"]);
        v.set_issuer(&[APPLE_JWT_ISS_VALUE]);
        v.set_audience(&[&config.app_bundle_id]);

        let data = jsonwebtoken::decode::<AppleTokenClaims>(&token, &key, &v)
            .change_context(SignInWithAppleError::InvalidToken)?;

        if data.claims.email_verified() {
            Ok(AppleAccountInfo {
                id: data.claims.sub,
                email: data.claims.email,
            })
        } else {
            Err(SignInWithAppleError::InvalidToken.report())
        }
    }
}

enum KeyStatus<'a> {
    Found(Jwk),
    KeyRefreshNeeded,
    UnknownKeyRefreshNeeded {
        current_state: &'a mut ApplePublicKeys
    },
}

struct ApplePublicKeys {
    keys: JwkSet,
    downloading_time: Instant,
    /// There is no cahce info about keys, so this prevents
    /// attackers from triggering unlimited amount of public key downloads.
    unknown_key_refresh_done: bool,
}

impl ApplePublicKeys {
    fn new(keys: JwkSet) -> Self {
        Self {
            keys,
            downloading_time: Instant::now(),
            unknown_key_refresh_done: false,
        }
    }

    fn redownload_needed(&self) -> bool {
        const SECONDS_IN_DAY: u64 = 60 * 60 * 24;
        Instant::now() >= self.downloading_time + Duration::from_secs(SECONDS_IN_DAY)
    }
}

struct ApplePublicKeysState {
    config: Arc<SimpleBackendConfig>,
    client: reqwest::Client,
    keys: Mutex<Option<ApplePublicKeys>>,
}

impl ApplePublicKeysState {
    fn new(
        config: Arc<SimpleBackendConfig>,
        client: reqwest::Client,
    ) -> Self {
        Self {
            client,
            config,
            keys: Mutex::new(None),
        }
    }

    async fn get_public_key(&self, wanted_kid: &str) -> Result<Jwk, SignInWithAppleError> {
        let mut state = self.keys.lock().await;
        match self
            .get_public_key_from_local_keys(state.as_mut(), wanted_kid)
            .await?
        {
            KeyStatus::Found(key) => Ok(key),
            KeyStatus::KeyRefreshNeeded =>
                self.download_public_keys_and_get_key(&mut state, wanted_kid)
                    .await,
            KeyStatus::UnknownKeyRefreshNeeded { current_state } =>
                self.handle_unknown_key_refresh(current_state, wanted_kid)
                    .await,
        }
    }

    async fn get_public_key_from_local_keys<'a>(
        &self,
        keys: Option<&'a mut ApplePublicKeys>,
        wanted_kid: &str,
    ) -> Result<KeyStatus<'a>, SignInWithAppleError> {
        match keys {
            None => Ok(KeyStatus::KeyRefreshNeeded),
            Some(keys) => {
                if keys.redownload_needed() {
                    Ok(KeyStatus::KeyRefreshNeeded)
                } else if let Some(jwk) = keys.keys.find(wanted_kid) {
                    Ok(KeyStatus::Found(jwk.clone()))
                } else if keys.unknown_key_refresh_done {
                    Err(SignInWithAppleError::JwkNotFound.report())
                } else {
                    Ok(KeyStatus::UnknownKeyRefreshNeeded { current_state: keys })
                }
            }
        }
    }

    async fn download_public_keys_and_get_key(
        &self,
        key_store: &mut Option<ApplePublicKeys>,
        wanted_kid: &str,
    ) -> Result<Jwk, SignInWithAppleError> {
        let jwk_set = self.download_public_key().await?;
        *key_store = Some(ApplePublicKeys::new(jwk_set.clone()));
        let jwk = jwk_set
            .find(wanted_kid)
            .ok_or(SignInWithAppleError::JwkNotFound)?
            .clone();
        Ok(jwk)
    }

    async fn handle_unknown_key_refresh(
        &self,
        key_store: &mut ApplePublicKeys,
        wanted_kid: &str,
    ) -> Result<Jwk, SignInWithAppleError> {
        let jwk_set = self.download_public_key().await?;
        if let Some(jwk) = jwk_set.find(wanted_kid).cloned() {
            *key_store = ApplePublicKeys::new(jwk_set);
            Ok(jwk)
        } else {
            key_store.unknown_key_refresh_done = true;
            Err(SignInWithAppleError::JwkNotFound.report())
        }
    }

    async fn download_public_key(
        &self,
    ) -> Result<JwkSet, SignInWithAppleError> {
        let download_request = reqwest::Request::new(
            reqwest::Method::GET,
            self.config.sign_in_with_urls().apple_public_keys.clone(),
        );

        let r = self
            .client
            .execute(download_request)
            .await
            .change_context(SignInWithAppleError::PublicKeyDownloadFailed)?;

        let jwk_set: JwkSet = r
            .json()
            .await
            .change_context(SignInWithAppleError::JwkSetParsingFailed)?;

        Ok(jwk_set)
    }
}
