use std::{sync::Arc, time::Instant};

use base64::Engine;
use error_stack::{Result, ResultExt};
use headers::{CacheControl, HeaderMapExt};
use jsonwebtoken::{
    DecodingKey, TokenData, Validation,
    jwk::{Jwk, JwkSet},
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use simple_backend_config::SimpleBackendConfig;
use simple_backend_utils::ContextExt;
use tokio::sync::Mutex;
use tracing::error;

/// Possible Google ID token (from client) iss field (issuer) values.
const POSSIBLE_ISS_VALUES_GOOGLE: &[&str] = &["accounts.google.com", "https://accounts.google.com"];

#[derive(thiserror::Error, Debug)]
pub enum SignInWithGoogleError {
    #[error("Token (from client) header parsing failed")]
    InvalidTokenHeader,

    #[error("Token from client was invalid")]
    InvalidToken,

    #[error("Couldn't download Google public key")]
    PublicKeyDownloadFailed,

    #[error("Token kid property not found from token received from client")]
    MissingJwtKid,

    #[error("HTTP GET for Google public key didn't include cache control header.")]
    MissingCacheControlHeader,

    #[error("Parsing HTTP GET for Google public key response cache control header failed.")]
    ParsingCacheControlHeader,

    #[error(
        "HTTP GET for Google public key response cache control header didn't have max age field"
    )]
    InvalidCacheControlHeader,

    #[error("Max age related time calculation failed")]
    CacheCalculation,

    #[error("HTTP GET for Google public keys didn't contain valid JwkSet")]
    JwkSetParsingFailed,

    #[error("Requested Jwk was not found")]
    JwkNotFound,

    #[error("Decoding key generation failed")]
    DecodingKeyGenerationFailed,

    #[error("Sign in with Google is not enabled from server settings file")]
    NotEnabled,
}

#[derive(Debug, Deserialize)]
struct GoogleTokenClaims {
    /// Server OAuth client ID
    azp: String,
    /// Google Account ID
    sub: String,
    /// Email linked to Google Account
    email: String,
    /// Email verification status.
    email_verified: bool,
    /// Base64 URL (with possible padding) encoded SHA-256 of client generated
    /// nonce.
    nonce: String,
}

pub struct GoogleAccountInfo {
    pub id: String,
    pub email: String,
}

enum KeyStatus {
    Found(Jwk),
    KeyRefreshNeeded,
}

pub struct SignInWithGoogleManager {
    config: Arc<SimpleBackendConfig>,
    key_state: GooglePublicKeysState,
}

impl SignInWithGoogleManager {
    pub fn new(config: Arc<SimpleBackendConfig>, client: reqwest::Client) -> Self {
        Self {
            config: config.clone(),
            key_state: GooglePublicKeysState::new(config, client),
        }
    }

    pub async fn validate_google_token(
        &self,
        token: String,
        nonce: Vec<u8>,
    ) -> Result<GoogleAccountInfo, SignInWithGoogleError> {
        let google_config = self
            .config
            .sign_in_with_google_config()
            .ok_or(SignInWithGoogleError::NotEnabled)?;

        let not_validated_header = jsonwebtoken::decode_header(&token)
            .change_context(SignInWithGoogleError::InvalidTokenHeader)?;
        let wanted_kid = not_validated_header
            .kid
            .ok_or(SignInWithGoogleError::MissingJwtKid)?;

        let google_public_key = self.key_state.get_google_public_key(&wanted_kid).await?;

        let key = DecodingKey::from_jwk(&google_public_key)
            .change_context(SignInWithGoogleError::DecodingKeyGenerationFailed)?;

        let mut v = Validation::new(not_validated_header.alg);
        v.set_required_spec_claims(&["exp", "iss"]);
        v.set_issuer(POSSIBLE_ISS_VALUES_GOOGLE);
        v.validate_aud = false;

        let data = jsonwebtoken::decode::<GoogleTokenClaims>(&token, &key, &v)
            .change_context(SignInWithGoogleError::InvalidToken)?;

        let azp_valid = if data.claims.azp == google_config.client_id_web {
            // Sign in with Google happened on the web client
            true
        } else {
            // Mobile clients support audience
            let mut validate_aud = Validation::new(not_validated_header.alg);
            validate_aud.set_required_spec_claims(&["aud"]);
            validate_aud.set_audience(&[&google_config.client_id_server]);
            let _: TokenData<GoogleTokenClaims> =
                jsonwebtoken::decode::<GoogleTokenClaims>(&token, &key, &validate_aud)
                    .change_context(SignInWithGoogleError::InvalidToken)?;

            let valid_client_ids = [
                google_config.client_id_android.as_str(),
                google_config.client_id_ios.as_str(),
            ];

            valid_client_ids.into_iter().any(|id| id == data.claims.azp)
        };

        if !azp_valid || !data.claims.email_verified {
            return Err(SignInWithGoogleError::InvalidToken.report());
        }

        let token_nonce = base64::engine::general_purpose::URL_SAFE.encode(Sha256::digest(nonce));
        if data.claims.nonce != token_nonce {
            return Err(SignInWithGoogleError::InvalidToken.report());
        }

        Ok(GoogleAccountInfo {
            id: data.claims.sub,
            email: data.claims.email,
        })
    }
}

struct GooglePublicKeys {
    keys: JwkSet,
    valid_until_this: std::time::Instant,
}

struct GooglePublicKeysState {
    config: Arc<SimpleBackendConfig>,
    client: reqwest::Client,
    keys: Mutex<Option<GooglePublicKeys>>,
}

impl GooglePublicKeysState {
    fn new(config: Arc<SimpleBackendConfig>, client: reqwest::Client) -> Self {
        Self {
            client,
            config,
            keys: Mutex::new(None),
        }
    }

    async fn get_google_public_key(&self, wanted_kid: &str) -> Result<Jwk, SignInWithGoogleError> {
        let mut state = self.keys.lock().await;
        match self
            .get_google_public_key_from_local_keys(state.as_ref(), wanted_kid)
            .await?
        {
            KeyStatus::Found(key) => Ok(key),
            KeyStatus::KeyRefreshNeeded => {
                self.download_google_public_keys_and_get_key(&mut state, wanted_kid)
                    .await
            }
        }
    }

    async fn get_google_public_key_from_local_keys(
        &self,
        keys: Option<&GooglePublicKeys>,
        wanted_kid: &str,
    ) -> Result<KeyStatus, SignInWithGoogleError> {
        match keys.as_ref() {
            None => Ok(KeyStatus::KeyRefreshNeeded),
            Some(keys) => {
                if Instant::now() >= keys.valid_until_this {
                    Ok(KeyStatus::KeyRefreshNeeded)
                } else {
                    let jwk = keys
                        .keys
                        .find(wanted_kid)
                        .ok_or(SignInWithGoogleError::JwkNotFound)?
                        .clone();
                    Ok(KeyStatus::Found(jwk))
                }
            }
        }
    }

    async fn download_google_public_keys_and_get_key(
        &self,
        key_store: &mut Option<GooglePublicKeys>,
        wanted_kid: &str,
    ) -> Result<Jwk, SignInWithGoogleError> {
        let download_request = reqwest::Request::new(
            reqwest::Method::GET,
            self.config.sign_in_with_urls().google_public_keys.clone(),
        );

        let r = self
            .client
            .execute(download_request)
            .await
            .change_context(SignInWithGoogleError::PublicKeyDownloadFailed)?;

        let possible_header = r
            .headers()
            .typed_try_get::<CacheControl>()
            .change_context(SignInWithGoogleError::ParsingCacheControlHeader)?;
        let cache_header =
            possible_header.ok_or(SignInWithGoogleError::MissingCacheControlHeader)?;
        let max_age = cache_header
            .max_age()
            .ok_or(SignInWithGoogleError::InvalidCacheControlHeader)?;
        let valid_until_this = Instant::now()
            .checked_add(max_age)
            .ok_or(SignInWithGoogleError::CacheCalculation)?;

        let jwk_set: JwkSet = r
            .json()
            .await
            .change_context(SignInWithGoogleError::JwkSetParsingFailed)?;

        *key_store = Some(GooglePublicKeys {
            keys: jwk_set.clone(),
            valid_until_this,
        });

        let jwk = jwk_set
            .find(wanted_kid)
            .ok_or(SignInWithGoogleError::JwkNotFound)?
            .clone();
        Ok(jwk)
    }
}
