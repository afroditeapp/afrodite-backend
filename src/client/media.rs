





use reqwest::{Client, Url};





use error_stack::{Result};

use crate::{
    api::{
        account::internal::PATH_CHECK_API_KEY,
    },
};






#[derive(Debug, Clone)]
pub enum MediaInternalApiRequest {
    Todo,
}

impl std::fmt::Display for MediaInternalApiRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Media internal API request: {:?}", self))
    }
}


#[derive(Debug, Default)]
pub struct MediaInternalApiUrls {
    // TODO
    check_api_key_url: Option<Url>,
}

impl MediaInternalApiUrls {
    pub fn new(base_url: &Url) -> Result<Self, url::ParseError> {
        Ok(Self {
            check_api_key_url: Some(base_url.join(PATH_CHECK_API_KEY)?),
        })
    }
}

pub struct MediaInternalApi<'a> {
    client: Client,
    urls: &'a MediaInternalApiUrls,
}

impl <'a> MediaInternalApi<'a> {
    pub fn new(client: Client, urls: &'a MediaInternalApiUrls) -> Self {
        Self {
            client,
            urls,
        }
    }

}
