//! Access REST API from Rust

use error_stack::{Result, ResultExt, IntoReport};

use reqwest::{Client, Url};

pub mod account;
pub mod media;
pub mod profile;



#[derive(thiserror::Error, Debug)]
pub enum HttpRequestError {
    #[error("Reqwest error")]
    Reqwest,

    // Other errors
    #[error("Serde deserialization error")]
    SerdeDeserialize,

    #[error("API URL not configured")]
    ApiUrlNotConfigured,
}

// TODO: Move url parsing to happen at startup so that url typos are
// discovered earlier.


pub struct ApiClient {
    client: Client,
}

impl ApiClient {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub fn account_internal() -> () {

    }
}


pub fn get_api_url(url: &Option<Url>) -> Result<Url, HttpRequestError> {
    url
        .as_ref()
        .ok_or(HttpRequestError::ApiUrlNotConfigured)
        .map(Clone::clone)
        .into_report()
}
