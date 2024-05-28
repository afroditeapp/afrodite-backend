//! Functions which makes requests to internal API. If the required
//! feature (server component) is located on the current server, then
//! request is not made.

use api_internal::Configuration;
use config::InternalApiUrls;
use tracing::{error, info};

use crate::result::{Result, WrappedContextExt};

pub mod common;
pub mod media;
pub mod profile;

// TODO: Use TLS for checking that all internal communication comes from trusted
//       sources.

#[derive(thiserror::Error, Debug)]
pub enum InternalApiError {
    #[error("API request failed")]
    ApiRequest,

    #[error("Database call failed")]
    DataError,

    #[error("Account API URL not configured")]
    AccountApiUrlNotConfigured,

    #[error("Media API URL not configured")]
    MediaApiUrlNotConfigured,
    // #[error("Wrong status code")]
    // StatusCode,

    // #[error("Joining text to URL failed")]
    // ApiUrlJoinError,
    #[error("Missing value")]
    MissingValue,

    #[error("Invalid value")]
    InvalidValue,

    #[error("Content secure capture flag is false")]
    SecureCaptureFlagFalse,

    #[error("Security content is not in moderation request")]
    SecurityContentNotInModerationRequest,

    #[error("Required security content for initial setup is not set")]
    SecurityContentNotSet,

    #[error("Content is not in moderation request")]
    ContentNotInModerationRequest,

    #[error("Required content for initial setup is not set")]
    ContentNotSet,

    #[error("Required server component is not enabled")]
    MissingComponent,
}

// TOOD: What is PrintWarningsTriggersAtomics?
pub struct PrintWarningsTriggersAtomics {}

pub struct InternalApiClient {
    account: Option<Configuration>,
    media: Option<Configuration>,
}

impl InternalApiClient {
    pub fn new(base_urls: InternalApiUrls) -> Self {
        let client = reqwest::Client::new();

        let account = base_urls.account_base_url.map(|url| {
            let url = url.as_str().trim_end_matches('/').to_string();

            info!("Account internal API base url: {}", url);

            Configuration {
                base_path: url,
                client: client.clone(),
                ..Configuration::default()
            }
        });

        let media = base_urls.media_base_url.map(|url| {
            let url = url.as_str().trim_end_matches('/').to_string();

            info!("Media internal API base url: {}", url);

            Configuration {
                base_path: url,
                client: client.clone(),
                ..Configuration::default()
            }
        });

        Self { account, media }
    }

    pub fn account(&self) -> Result<&Configuration, InternalApiError> {
        self.account
            .as_ref()
            .ok_or(InternalApiError::AccountApiUrlNotConfigured.report())
    }

    pub fn media(&self) -> Result<&Configuration, InternalApiError> {
        self.media
            .as_ref()
            .ok_or(InternalApiError::MediaApiUrlNotConfigured.report())
    }
}
