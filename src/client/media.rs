
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
    database::{write::WriteCommands, RouterDatabaseHandle},
    session::{SessionManager, AccountStateInRam},
};

use super::HttpRequestError;


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
