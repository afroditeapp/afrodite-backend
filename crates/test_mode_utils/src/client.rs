//! Access REST API from Rust

use std::fmt::Debug;

use api_client::apis::configuration::Configuration;
use config::args::PublicApiUrl;
use error_stack::Result;
use hyper::StatusCode;
use reqwest::{Client, Url};
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
    #[error("Event channel closed")]
    EventChannelClosed,
    #[error("Event receiving handle disabled")]
    EventReceivingHandleDisabled,
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

    #[error("Message encryption error")]
    MessageEncryptionError,

    #[error("Content moderation failed")]
    ContentModerationFailed,

    #[error("Admin bot internal error")]
    AdminBotInternalError,
}

impl TestError {
    #[track_caller]
    pub fn report(self) -> error_stack::Report<Self> {
        error_stack::Report::from(self)
    }
}

#[derive(Debug, Clone)]
pub struct ApiClient {
    api: Configuration,
}

impl ApiClient {
    pub fn new(base_urls: PublicApiUrl, client: &reqwest::Client) -> Self {
        Self {
            api: Self::create_configuration(client, base_urls.api_url.as_str()),
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
        info!("API base url: {}", self.api.base_path);
    }

    pub fn api(&self) -> &Configuration {
        &self.api
    }

    pub fn set_access_token(&mut self, token: String) {
        self.api.bearer_access_token = Some(token);
    }

    pub fn is_access_token_available(&self) -> bool {
        self.api.bearer_access_token.is_some()
    }

    pub fn api_key(&self) -> Option<String> {
        self.api.bearer_access_token.clone()
    }
}

pub fn get_api_url(url: &Option<Url>) -> Result<Url, TestError> {
    url.as_ref()
        .ok_or(TestError::ApiUrlNotConfigured.report())
        .cloned()
}
