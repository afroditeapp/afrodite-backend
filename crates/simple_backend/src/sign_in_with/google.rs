use std::{sync::Arc, time::Instant};

use error_stack::{Result, ResultExt};
use jsonwebtoken::{
    jwk::{Jwk, JwkSet},
    DecodingKey, Validation,
};
use reqwest_headers::{CacheControl, HeaderMapExt};
use serde::Deserialize;
use simple_backend_config::SimpleBackendConfig;
use simple_backend_utils::ContextExt;
use tokio::sync::RwLock;
use tracing::error;
// use utils::ContextExt;

// TODO: Send serverAuthCode to server. Get refresh and access tokens from
// google with that and save the tokens to database. After server receives app's
// own refresh token from WebSocket then get new access token from google if
// required.

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
}

pub struct GoogleAccountInfo {
    pub id: String,
    pub email: String,
}
struct GooglePublicKeys {
    keys: JwkSet,
    valid_until_this: std::time::Instant,
}

enum KeyStatus {
    Found(Jwk),
    KeyRefreshNeeded,
}

pub struct SignInWithGoogleManager {
    client: reqwest::Client,
    config: Arc<SimpleBackendConfig>,
    google_public_keys: RwLock<Option<GooglePublicKeys>>,
}

impl SignInWithGoogleManager {
    pub fn new(config: Arc<SimpleBackendConfig>, client: reqwest::Client) -> Self {
        Self {
            client,
            config,
            google_public_keys: RwLock::new(None),
        }
    }

    pub async fn validate_google_token(
        &self,
        token: String,
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

        let google_public_key = self.get_google_public_key(&wanted_kid).await?;

        let key = DecodingKey::from_jwk(&google_public_key)
            .change_context(SignInWithGoogleError::DecodingKeyGenerationFailed)?;

        let mut v = Validation::new(not_validated_header.alg);
        v.set_required_spec_claims(&["exp", "aud", "iss"]);
        v.set_issuer(POSSIBLE_ISS_VALUES_GOOGLE);
        v.set_audience(&[&google_config.client_id_server]);

        let data = jsonwebtoken::decode::<GoogleTokenClaims>(&token, &key, &v)
            .change_context(SignInWithGoogleError::InvalidToken)?;

        let azp_invalid = [
            google_config.client_id_android.as_str(),
            google_config.client_id_ios.as_str(),
        ]
        .into_iter()
        .find(|id| *id == data.claims.azp)
        .is_none();

        if azp_invalid || !data.claims.email_verified {
            return Err(SignInWithGoogleError::InvalidToken.report());
        }

        Ok(GoogleAccountInfo {
            id: data.claims.sub,
            email: data.claims.email,
        })
    }

    async fn get_google_public_key(&self, wanted_kid: &str) -> Result<Jwk, SignInWithGoogleError> {
        match self
            .get_google_public_key_from_local_keys(wanted_kid)
            .await?
        {
            KeyStatus::Found(key) => Ok(key),
            KeyStatus::KeyRefreshNeeded => {
                self.download_google_public_keys_and_get_key(wanted_kid)
                    .await
            }
        }
    }

    async fn get_google_public_key_from_local_keys(
        &self,
        wanted_kid: &str,
    ) -> Result<KeyStatus, SignInWithGoogleError> {
        let keys = self.google_public_keys.read().await;
        match keys.as_ref() {
            None => Ok(KeyStatus::KeyRefreshNeeded),
            Some(keys) => {
                if Instant::now() >= keys.valid_until_this {
                    Ok(KeyStatus::KeyRefreshNeeded)
                } else {
                    let jwk = keys
                        .keys
                        .find(&wanted_kid)
                        .ok_or(SignInWithGoogleError::JwkNotFound)?
                        .clone();
                    Ok(KeyStatus::Found(jwk))
                }
            }
        }
    }

    async fn download_google_public_keys_and_get_key(
        &self,
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
        let mut key_store = self.google_public_keys.write().await;
        *key_store = Some(GooglePublicKeys {
            keys: jwk_set.clone(),
            valid_until_this,
        });

        let jwk = jwk_set
            .find(&wanted_kid)
            .ok_or(SignInWithGoogleError::JwkNotFound)?
            .clone();
        Ok(jwk)
    }
}
