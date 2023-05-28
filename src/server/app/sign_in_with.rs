
pub mod google;
pub mod apple;

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
    config::InternalApiUrls,
    utils::IntoReportExt,
};

use crate::{api::model::ApiKey, config::Config};

use self::{apple::{SignInWithAppleManager, AppleAccountId, SignInWithAppleError}, google::{SignInWithGoogleManager, GoogleAccountInfo, SignInWithGoogleError}};
pub struct SignInWithManager {
    google: SignInWithGoogleManager,
    apple: SignInWithAppleManager,
}

impl SignInWithManager {
    pub fn new(config: Arc<Config>) -> Self {
        let client = reqwest::Client::new();
        Self {
            google: SignInWithGoogleManager::new(config.clone(), client.clone()),
            apple: SignInWithAppleManager::new(config.clone(), client.clone())
        }
    }

    pub async fn validate_google_token(&self, token: String) -> Result<GoogleAccountInfo, SignInWithGoogleError> {
        self.google.validate_google_token(token).await
    }

    pub async fn validate_apple_token(&self, token: String) -> Result<AppleAccountId, SignInWithAppleError> {
        self.apple.validate_apple_token(token).await
    }
}
