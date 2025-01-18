use std::time::Duration;

use error_stack::{report, Result, ResultExt};
use manager_model::SecureStorageEncryptionKey;

use crate::{api::client::{LocalOrRemoteApiClient, RequestSenderCmds}, config::Config};

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("Client build failed")]
    ClientBuildFailed,

    #[error("API request failed")]
    ApiRequest,

    #[error("Database call failed")]
    DatabaseError,

    #[error("Manager API URL not configured for {0}")]
    ManagerApiUrlNotConfigured(&'static str),

    #[error("Missing value")]
    MissingValue,

    #[error("Invalid value")]
    InvalidValue,

    #[error("Missing configuration")]
    MissingConfiguration,

    #[error("API request timeout")]
    RequestTimeout,

    #[error("Invalid API response")]
    InvalidResponse,
}

pub struct ApiManager<'a> {
    config: &'a Config,
}

impl<'a> ApiManager<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    pub async fn get_encryption_key(self) -> Result<SecureStorageEncryptionKey, ApiError> {
        let provider = self
            .config
            .secure_storage_config()
            .ok_or(ApiError::MissingConfiguration)?;
        let client = LocalOrRemoteApiClient::new(self.config, provider.key_storage_manager_name.clone());
        let request = client.get_secure_storage_encryption_key(self.config.manager_name().clone());
        if let Some(timeout) = provider.key_download_timeout_seconds {
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(timeout.into())) => {
                    Err(report!(ApiError::RequestTimeout))
                }
                r = request => {
                    r.change_context(ApiError::ApiRequest)
                },
            }
        } else {
            request
                .await
                .change_context(ApiError::ApiRequest)
        }
    }
}
