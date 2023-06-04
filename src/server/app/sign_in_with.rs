pub mod apple;
pub mod google;

use std::{sync::Arc, time::Instant};

use api_client::{
    apis::{accountinternal_api, configuration::Configuration, mediainternal_api},
    models::boolean_setting,
};
use axum::{
    routing::{get, post},
    Router,
};

use error_stack::{IntoReport, Result, ResultExt};

use headers::{CacheControl, HeaderMapExt};
use hyper::{Method, StatusCode};

use jsonwebtoken::{
    jwk::{Jwk, JwkSet},
    DecodingKey, Validation,
};
use serde::Deserialize;
use serde_json::Value;
use tokio::sync::RwLock;
use tracing::{error, info};
use url::Url;

use crate::{
    api::{
        self,
        model::{
            Account, AccountIdInternal, AccountState, BooleanSetting, Capabilities, Profile,
            ProfileInternal,
        },
    },
    config::InternalApiUrls,
    utils::IntoReportExt,
};

use crate::{api::model::ApiKey, config::Config};

use self::{
    apple::{AppleAccountId, SignInWithAppleError, SignInWithAppleManager},
    google::{GoogleAccountInfo, SignInWithGoogleError, SignInWithGoogleManager},
};
pub struct SignInWithManager {
    google: SignInWithGoogleManager,
    apple: SignInWithAppleManager,
}

impl SignInWithManager {
    pub fn new(config: Arc<Config>) -> Self {
        let client = reqwest::Client::new();
        Self {
            google: SignInWithGoogleManager::new(config.clone(), client.clone()),
            apple: SignInWithAppleManager::new(config.clone(), client.clone()),
        }
    }

    pub async fn validate_google_token(
        &self,
        token: String,
    ) -> Result<GoogleAccountInfo, SignInWithGoogleError> {
        self.google.validate_google_token(token).await
    }

    pub async fn validate_apple_token(
        &self,
        token: String,
    ) -> Result<AppleAccountId, SignInWithAppleError> {
        self.apple.validate_apple_token(token).await
    }
}
