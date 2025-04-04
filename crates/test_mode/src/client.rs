//! Access REST API from Rust

use std::fmt::Debug;

use api_client::apis::configuration::Configuration;
use config::args::PublicApiUrls;
use error_stack::Result;
use hyper::StatusCode;
use reqwest::{Client, Url};
use tracing::info;

use crate::bot::utils::encrypt::MessageEncryptionError;

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
    #[error("Event channel closed")]
    EventChannelClosed,
    #[error("Event receiving handle missing")]
    EventReceivingHandleMissing,
    #[error("Event receiving timeout")]
    EventReceivingTimeout,

    // Other errors
    #[error("Serde deserialization error")]
    SerdeDeserialize,

    #[error("API URL not configured")]
    ApiUrlNotConfigured,

    #[error("API URL port configuration failed")]
    ApiUrlPortConfigFailed,

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

    #[error("Server integration test failed.")]
    ServerTestFailed,

    #[error("Message encryption error: {0:?}")]
    MessageEncryptionError(MessageEncryptionError),

    #[error("Content moderation failed")]
    ContentModerationFailed,

    #[error("OpenPGP related error")]
    OpenPgp,
}

impl TestError {
    #[track_caller]
    pub fn report(self) -> error_stack::Report<Self> {
        error_stack::Report::from(self)
    }
}

#[derive(Debug, Clone)]
pub struct ApiClient {
    account: Configuration,
    profile: Configuration,
    media: Configuration,
    chat: Configuration,
}

impl ApiClient {
    pub fn new(base_urls: PublicApiUrls) -> Self {
        let client = reqwest::Client::new();

        Self {
            account: Self::create_configuration(&client, base_urls.url_account.as_str()),
            profile: Self::create_configuration(&client, base_urls.url_profile.as_str()),
            media: Self::create_configuration(&client, base_urls.url_media.as_str()),
            chat: Self::create_configuration(&client, base_urls.url_chat.as_str()),
        }
    }

    fn create_configuration(client: &Client, base_url: &str) -> Configuration {
        let path = base_url.trim_end_matches('/').to_string();
        Configuration {
            base_path: path,
            client: client.clone(),
            ..Configuration::default()
        }
    }

    pub fn print_to_log(&self) {
        info!("Account API base url: {}", self.account.base_path);
        info!("Profile API base url: {}", self.profile.base_path);
        info!("Media API base url: {}", self.media.base_path);
        info!("Chat API base url: {}", self.chat.base_path);
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

    pub fn chat(&self) -> &Configuration {
        &self.chat
    }

    pub fn set_access_token(&mut self, token: String) {
        let token = api_client::apis::configuration::ApiKey {
            prefix: None,
            key: token,
        };
        self.account.api_key = Some(token.clone());
        self.profile.api_key = Some(token.clone());
        self.media.api_key = Some(token.clone());
        self.chat.api_key = Some(token.clone());
    }

    pub fn is_access_token_available(&self) -> bool {
        self.account.api_key.is_some()
            && self.profile.api_key.is_some()
            && self.media.api_key.is_some()
            && self.chat.api_key.is_some()
    }

    pub fn api_key(&self) -> Option<String> {
        self.account.api_key.clone().map(|k| k.key)
    }
}

pub fn get_api_url(url: &Option<Url>) -> Result<Url, TestError> {
    url.as_ref()
        .ok_or(TestError::ApiUrlNotConfigured.report())
        .cloned()
}
