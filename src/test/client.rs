//! Access REST API from Rust

use std::fmt::Debug;

use api_client::{apis::configuration::Configuration, models::ApiKey};
use error_stack::{IntoReport, Result};

use hyper::StatusCode;
use reqwest::Url;
use tracing::info;

#[derive(thiserror::Error, Debug)]
#[error("Wrong status code: {0}")]
pub struct StatusCodeError(StatusCode);

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum TestError {
    #[error("Reqwest error")]
    Reqwest,

    #[error("WebSocket error")]
    WebSocket,
    #[error("WebSocket wrong value received")]
    WebSocketWrongValue,

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

    #[error("Account ID missing from bot state")]
    AccountIdMissing,

    #[error("Assert error. message: {0}")]
    AssertError(String),

    #[error("Not an error. Just an indication that bot is waiting.")]
    BotIsWaiting,
}

#[derive(Debug, Clone)]
pub struct PublicApiUrls {
    pub account_base_url: Url,
    pub profile_base_url: Url,
    pub media_base_url: Url,
}

impl PublicApiUrls {
    pub fn new(account_base_url: Url, profile_base_url: Url, media_base_url: Url) -> Self {
        Self {
            account_base_url,
            profile_base_url,
            media_base_url,
        }
    }
}

#[derive(Debug)]
pub struct ApiClient {
    account: Configuration,
    profile: Configuration,
    media: Configuration,
}

impl ApiClient {
    pub fn new(base_urls: PublicApiUrls) -> Self {
        let client = reqwest::Client::new();

        let account_path = base_urls
            .account_base_url
            .as_str()
            .trim_end_matches('/')
            .to_string();
        let account = Configuration {
            base_path: account_path,
            client: client.clone(),
            ..Configuration::default()
        };

        let profile_path = base_urls
            .profile_base_url
            .as_str()
            .trim_end_matches('/')
            .to_string();
        let profile = Configuration {
            base_path: profile_path,
            client: client.clone(),
            ..Configuration::default()
        };

        let media_path = base_urls
            .media_base_url
            .as_str()
            .trim_end_matches('/')
            .to_string();
        let media = Configuration {
            base_path: media_path,
            client: client.clone(),
            ..Configuration::default()
        };

        Self {
            account,
            profile,
            media,
        }
    }

    pub fn print_to_log(&self) {
        info!("Account API base url: {}", self.account.base_path);
        info!("Profile API base url: {}", self.profile.base_path);
        info!("Media API base url: {}", self.media.base_path);
    }

    pub fn account(&self) -> &Configuration {
        &self.account
    }

    pub fn profile(&self) -> &Configuration {
        &self.profile
    }

    pub fn media(&self) -> &Configuration {
        &self.media
    }

    pub fn set_access_token(&mut self, token: String) {
        let token = api_client::apis::configuration::ApiKey {
            prefix: None,
            key: token,
        };
        self.account.api_key = Some(token.clone());
        self.profile.api_key = Some(token.clone());
        self.media.api_key = Some(token.clone());
    }

    pub fn is_access_token_available(&self) -> bool {
        self.account.api_key.is_some()
            && self.profile.api_key.is_some()
            && self.profile.api_key.is_some()
    }

    pub fn api_key(&self) -> Option<String> {
        self.account.api_key.clone().map(|k| k.key)
    }
}

pub fn get_api_url(url: &Option<Url>) -> Result<Url, TestError> {
    url.as_ref()
        .ok_or(TestError::ApiUrlNotConfigured)
        .map(Clone::clone)
        .into_report()
}
