//! Access REST API from Rust



use api_client::{apis::configuration::Configuration, models::ApiKey};
use error_stack::{IntoReport, Result};

use hyper::StatusCode;
use reqwest::{Url};
use tracing::info;



#[derive(thiserror::Error, Debug)]
#[error("Wrong status code: {0}")]
pub struct StatusCodeError(StatusCode);

#[derive(thiserror::Error, Debug)]
pub enum HttpRequestError {
    #[error("Reqwest error")]
    Reqwest,

    // Other errors
    #[error("Serde deserialization error")]
    SerdeDeserialize,

    #[error("API URL not configured")]
    ApiUrlNotConfigured,

    #[error("Wrong status code")]
    StatusCode,

    #[error("Joining text to URL failed")]
    ApiUrlJoinError,

    #[error("Missing value")]
    MissingValue,

    #[error("API request failed")]
    ApiRequest,
}

#[derive(Debug, Clone)]
pub struct PublicApiUrls {
    pub account_base_url: Url,
    pub profile_base_url: Url,
}

impl PublicApiUrls {
    pub fn new(account_base_url: Url, profile_base_url: Url) -> Self {
        Self {
            account_base_url,
            profile_base_url,
        }
    }
}

pub struct ApiClient {
    account: Configuration,
    profile: Configuration,
}

impl ApiClient {
    pub fn new(base_urls: PublicApiUrls) -> Self {
        let account_path = base_urls
            .account_base_url
            .as_str()
            .trim_end_matches('/')
            .to_string();
        let account = Configuration {
            base_path: account_path,
            ..Configuration::default()
        };
        info!("Account API base url: {}", account.base_path);

        let profile_path = base_urls
            .profile_base_url
            .as_str()
            .trim_end_matches('/')
            .to_string();

        // Clone will also clone reqwest Client
        let mut profile = account.clone();
        profile.base_path = profile_path;
        info!("Profile API base url: {}", profile.base_path);

        Self { account, profile }
    }

    pub fn account(&self) -> &Configuration {
        &self.account
    }

    pub fn profile(&self) -> &Configuration {
        &self.profile
    }

    pub fn set_api_key(&mut self, key: ApiKey) {
        let config_key = api_client::apis::configuration::ApiKey {
            prefix: None,
            key: key.api_key,
        };
        self.account.api_key = Some(config_key.clone());
        self.profile.api_key = Some(config_key);
    }
}

pub fn get_api_url(url: &Option<Url>) -> Result<Url, HttpRequestError> {
    url.as_ref()
        .ok_or(HttpRequestError::ApiUrlNotConfigured)
        .map(Clone::clone)
        .into_report()
}
