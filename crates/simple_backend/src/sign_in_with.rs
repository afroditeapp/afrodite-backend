use std::sync::Arc;

use apple::AppleAccountInfo;
use error_stack::Result;
use simple_backend_config::SimpleBackendConfig;

use self::{
    apple::{SignInWithAppleError, SignInWithAppleManager},
    google::{GoogleAccountInfo, SignInWithGoogleError, SignInWithGoogleManager},
};

pub mod apple;
pub mod google;

pub struct SignInWithManager {
    google: SignInWithGoogleManager,
    apple: SignInWithAppleManager,
}

impl SignInWithManager {
    pub fn new(config: Arc<SimpleBackendConfig>, client: reqwest::Client) -> Self {
        Self {
            google: SignInWithGoogleManager::new(config.clone(), client.clone()),
            apple: SignInWithAppleManager::new(config.clone(), client),
        }
    }

    pub async fn validate_google_token(
        &self,
        token: String,
        nonce: Vec<u8>,
    ) -> Result<GoogleAccountInfo, SignInWithGoogleError> {
        self.google.validate_google_token(token, nonce).await
    }

    pub async fn validate_apple_token(
        &self,
        token: String,
        nonce: Vec<u8>,
    ) -> Result<AppleAccountInfo, SignInWithAppleError> {
        self.apple.validate_apple_token(token, nonce).await
    }
}
