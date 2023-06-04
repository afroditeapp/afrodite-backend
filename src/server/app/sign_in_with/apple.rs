use std::{sync::Arc};




use error_stack::{IntoReport, Result};








use tracing::{error, info};


use crate::{
    config::{Config},
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
        Self { client, config }
    }
    pub async fn validate_apple_token(
        &self,
        token: String,
    ) -> Result<AppleAccountId, SignInWithAppleError> {
        let not_validated_header = jsonwebtoken::decode_header(&token)
            .into_error(SignInWithAppleError::InvalidTokenHeader)?;
        info!("{:?}", &not_validated_header);

        Err(SignInWithAppleError::InvalidToken).into_report()
    }
}
