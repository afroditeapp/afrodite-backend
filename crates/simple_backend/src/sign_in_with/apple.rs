use std::sync::Arc;

use error_stack::{Result, ResultExt};
use simple_backend_config::SimpleBackendConfig;
use simple_backend_utils::ContextExt;
use tracing::{error, info};

#[derive(thiserror::Error, Debug)]
pub enum SignInWithAppleError {
    #[error("Token (from client) header parsing failed")]
    InvalidTokenHeader,

    #[error("Token was invalid")]
    InvalidToken,
}

pub struct AppleAccountId;

pub struct SignInWithAppleManager;

impl SignInWithAppleManager {
    pub fn new(_config: Arc<SimpleBackendConfig>, _client: reqwest::Client) -> Self {
        // TODO(prod): Implement Sign in with Apple support
        Self
    }
    pub async fn validate_apple_token(
        &self,
        token: String,
    ) -> Result<AppleAccountId, SignInWithAppleError> {
        let not_validated_header = jsonwebtoken::decode_header(&token)
            .change_context(SignInWithAppleError::InvalidTokenHeader)?;
        info!("{:?}", &not_validated_header);

        Err(SignInWithAppleError::InvalidToken.report())
    }
}
