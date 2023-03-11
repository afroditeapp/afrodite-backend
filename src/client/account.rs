
use std::{collections::HashMap, sync::Arc};

use axum::{
    middleware,
    routing::{get, post},
    Json, Router,
};
use headers::Header;
use hyper::StatusCode;
use reqwest::{Client, Request, Url};
use tokio::sync::{Mutex, RwLock};

use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use error_stack::{Result, ResultExt};

use crate::{
    api::{
        self,
        model::{ApiKey, AccountId, AccountIdLight},
        account::internal::PATH_CHECK_API_KEY,
        utils::{
            ApiKeyHeader,
        },
        ApiDoc, GetApiKeys, GetRouterDatabaseHandle, GetSessionManager, GetUsers, ReadDatabase,
        WriteDatabase,
    },
    utils::IntoReportExt, server::internal::{},
};

use crate::server::{
    app::AppState,
    database::{read::ReadCommands, write::WriteCommands, RouterDatabaseHandle},
    session::{SessionManager, AccountStateInRam},
};

use super::{HttpRequestError, get_api_url};


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

impl <'a> AccountInternalApi<'a> {
    pub fn new(client: Client, urls: &'a AccountInternalApiUrls) -> Self {
        Self {
            client,
            urls,
        }
    }

    pub async fn check_api_key(&self, api_key: ApiKey) -> Result<Option<AccountIdLight>, HttpRequestError> {
        let url = get_api_url(&self.urls.check_api_key_url)?;

        let request = self
            .client
            .get(url)
            .header(ApiKeyHeader::name(), api_key.as_str())
            .build()
            .unwrap();

        let response = self
            .client
            .execute(request)
            .await
            .into_error_with_info(
                HttpRequestError::Reqwest,
                 AccountInternalApiRequest::CheckApiKey
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
