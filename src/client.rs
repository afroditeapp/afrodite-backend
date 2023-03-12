//! Access REST API from Rust

use std::sync::Arc;

use error_stack::{Result, ResultExt, IntoReport};

use hyper::StatusCode;
use reqwest::{Client, Url};

use self::{account::{AccountApiUrls, AccountApi}, profile::{ProfileApiUrls, ProfileApi}};

pub mod account;
pub mod media;
pub mod profile;



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
}

// TODO: Move url parsing to happen at startup so that url typos are
// discovered earlier.


#[derive(Debug, Clone)]
pub struct PublicApiUrls {
    pub account: AccountApiUrls,
    pub proifle: ProfileApiUrls,
}

impl PublicApiUrls {
    pub fn new(account_base_url: Url, profile_base_url: Url) -> Result<Self, url::ParseError> {
        Ok(Self {
            account: AccountApiUrls::new(account_base_url)?,
            proifle: ProfileApiUrls::new(profile_base_url)?,
        })
    }
}

pub struct ApiClient {
    client: Client,
    urls: Arc<PublicApiUrls>,
}

impl ApiClient {
    pub fn new(client: Client, urls: Arc<PublicApiUrls>) -> Self {
        Self { client, urls }
    }

    pub fn account(&self) -> AccountApi {
        AccountApi::new(&self.client, &self.urls.as_ref().account)
    }

    pub fn profile(&self) -> ProfileApi {
        ProfileApi::new(&self.client, &self.urls.as_ref().proifle)
    }
}


pub fn get_api_url(url: &Option<Url>) -> Result<Url, HttpRequestError> {
    url
        .as_ref()
        .ok_or(HttpRequestError::ApiUrlNotConfigured)
        .map(Clone::clone)
        .into_report()
}
