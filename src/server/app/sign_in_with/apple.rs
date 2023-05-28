


use std::{sync::Arc, time::Instant};

use api_client::{apis::{accountinternal_api, configuration::Configuration, mediainternal_api}, models::boolean_setting};
use axum::{
    routing::{get, post},
    Router,
};

use error_stack::{Result, ResultExt, IntoReport};

use headers::{HeaderMapExt, CacheControl};
use hyper::{StatusCode, Method};

use jsonwebtoken::{jwk::{JwkSet, Jwk}, DecodingKey, Validation};
use serde::Deserialize;
use serde_json::Value;
use tokio::sync::RwLock;
use tracing::{error, info};
use url::Url;

use crate::{
    api::{
        self,
        model::{Account, AccountIdInternal, AccountState, Capabilities, BooleanSetting, Profile, ProfileInternal},
    },
    config::{InternalApiUrls, Config},
    utils::IntoReportExt,
};

#[derive(thiserror::Error, Debug)]
pub enum SignInWithAppleError {
    #[error("Token (from client) header parsing failed")]
    InvalidTokenHeader,

    #[error("Token was invalid")]
    InvalidToken,
}

pub struct AppleAccountId(String);

pub struct SignInWithAppleManager {
    client: reqwest::Client,
    config: Arc<Config>,
}

impl SignInWithAppleManager {
    pub fn new(config: Arc<Config>, client: reqwest::Client) -> Self {
        Self {
            client,
            config,
        }
    }
    pub async fn validate_apple_token(&self, token: String) -> Result<AppleAccountId, SignInWithAppleError> {
        let not_validated_header = jsonwebtoken::decode_header(&token).into_error(SignInWithAppleError::InvalidTokenHeader)?;
        info!("{:?}", &not_validated_header);

        Err(SignInWithAppleError::InvalidToken).into_report()
    }
}
