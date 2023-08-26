use std::sync::Arc;

use config::Config;
use error_stack::{ResultExt, Result};
use tracing::{error, info};
use utils::{ ContextExt};

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
        Self { client, config }
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
