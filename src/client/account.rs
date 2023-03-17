use headers::Header;
use hyper::StatusCode;
use reqwest::{Client, Url};

use error_stack::Result;

use crate::{
    api::{
        account::{internal::PATH_CHECK_API_KEY, PATH_LOGIN, PATH_REGISTER},
        model::{AccountId, AccountIdLight, ApiKey},
        utils::ApiKeyHeader,
    },
    utils::IntoReportExt,
};

use super::{get_api_url, HttpRequestError, StatusCodeError};

// Internal API

#[derive(Debug, Clone)]
pub enum AccountInternalApiRequest {
    CheckApiKey,
}

impl std::fmt::Display for AccountInternalApiRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Account internal API request: {:?}", self))
    }
}

#[derive(Debug, Default)]
pub struct AccountInternalApiUrls {
    check_api_key_url: Option<Url>,
}

impl AccountInternalApiUrls {
    pub fn new(base_url: Url) -> Result<Self, url::ParseError> {
        Ok(Self {
            check_api_key_url: Some(base_url.join(PATH_CHECK_API_KEY)?),
        })
    }
}

pub struct AccountInternalApi<'a> {
    client: Client,
    urls: &'a AccountInternalApiUrls,
}

impl<'a> AccountInternalApi<'a> {
    pub fn new(client: Client, urls: &'a AccountInternalApiUrls) -> Self {
        Self { client, urls }
    }

    pub async fn check_api_key(
        &self,
        api_key: ApiKey,
    ) -> Result<Option<AccountIdLight>, HttpRequestError> {
        let url = get_api_url(&self.urls.check_api_key_url)?;

        let request = self
            .client
            .get(url)
            .header(ApiKeyHeader::name(), api_key.as_str())
            .build()
            .unwrap();

        let response = self.client.execute(request).await.into_error_with_info(
            HttpRequestError::Reqwest,
            AccountInternalApiRequest::CheckApiKey,
        )?;

        if response.status() == StatusCode::OK {
            let id: AccountIdLight = response.json().await.into_error_with_info(
                HttpRequestError::SerdeDeserialize,
                AccountInternalApiRequest::CheckApiKey,
            )?;
            Ok(Some(id))
        } else {
            Ok(None)
        }
    }
}

// Public API

#[derive(Debug, Clone)]
pub enum AccountApiRequest {
    Register,
    Login,
}

impl std::fmt::Display for AccountApiRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Account API request: {:?}", self))
    }
}

#[derive(Debug, Default, Clone)]
pub struct AccountApiUrls {
    register_url: Option<Url>,
    login_url: Option<Url>,
}

impl AccountApiUrls {
    pub fn new(base_url: Url) -> Result<Self, url::ParseError> {
        Ok(Self {
            register_url: Some(base_url.join(PATH_REGISTER)?),
            login_url: Some(base_url.join(PATH_LOGIN)?),
        })
    }
}

pub struct AccountApi<'a> {
    client: &'a Client,
    urls: &'a AccountApiUrls,
}

impl<'a> AccountApi<'a> {
    pub fn new(client: &'a Client, urls: &'a AccountApiUrls) -> Self {
        Self { client, urls }
    }

    pub async fn register(&self) -> Result<AccountIdLight, HttpRequestError> {
        let url = get_api_url(&self.urls.register_url)?;

        let request = self.client.post(url).build().unwrap();

        let response = self
            .client
            .execute(request)
            .await
            .into_error_with_info(HttpRequestError::Reqwest, AccountApiRequest::Register)?;

        if response.status() == StatusCode::OK {
            let id: AccountIdLight = response.json().await.into_error_with_info(
                HttpRequestError::SerdeDeserialize,
                AccountApiRequest::Register,
            )?;
            Ok(id)
        } else {
            Err(StatusCodeError(response.status()))
                .into_error_with_info(HttpRequestError::StatusCode, AccountApiRequest::Register)
        }
    }

    pub async fn login(&self, id: &AccountId) -> Result<ApiKey, HttpRequestError> {
        let url = get_api_url(&self.urls.login_url)?;

        let request = self.client.post(url).json(&id.as_light()).build().unwrap();

        let response = self
            .client
            .execute(request)
            .await
            .into_error_with_info(HttpRequestError::Reqwest, AccountApiRequest::Register)?;

        if response.status() == StatusCode::OK {
            let key: ApiKey = response.json().await.into_error_with_info(
                HttpRequestError::SerdeDeserialize,
                AccountApiRequest::Register,
            )?;
            Ok(key)
        } else {
            Err(StatusCodeError(response.status()))
                .into_error_with_info(HttpRequestError::StatusCode, AccountApiRequest::Register)
        }
    }
}
