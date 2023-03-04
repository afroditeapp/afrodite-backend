//! Routes for server to server connections

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
    utils::IntoReportExt,
};

use super::{
    app::AppState,
    database::{read::ReadCommands, write::WriteCommands, RouterDatabaseHandle},
    session::{SessionManager, AccountState},
    CORE_SERVER_INTERNAL_API_URL,
};

// TODO: Use TLS for checking that all internal communication comes from trusted
//       sources.

/// Internal route handlers for server to server communication.
pub struct InternalApp;

impl InternalApp {
    pub fn create_core_server_router(state: AppState) -> Router {
        Router::new().route(
            api::account::internal::PATH_CHECK_API_KEY,
            get({
                let state = state.clone();
                move |body| api::account::internal::check_api_key(body, state)
            }),
        )
    }

    pub fn create_media_server_router(state: AppState) -> Router {
        Router::new().route(
            api::media::internal::PATH_POST_IMAGE,
            post({
                let state = state.clone();
                move |parameter1, parameter2, header1, header2, body| {
                    api::media::internal::post_image(
                        parameter1, parameter2, header1, header2, body, state,
                    )
                }
            }),
        )
    }
}

#[derive(thiserror::Error, Debug)]
pub enum HttpRequestError {
    #[error("Reqwest error")]
    Reqwest,

    // Other errors
    #[error("Serde deserialization error")]
    SerdeDeserialize,
}

#[derive(Debug, Clone)]
pub enum InternalApiRequest {
    CheckApiKey,
}

impl std::fmt::Display for InternalApiRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Internal API request: {:?}", self))
    }
}

// TODO: Move url parsing to happen at startup so that url typos are
// discovered earlier.

pub struct CoreServerInternalApi {
    client: Client,
    base_url: Url,
}

impl CoreServerInternalApi {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            base_url: Url::parse(CORE_SERVER_INTERNAL_API_URL).unwrap(),
        }
    }

    pub async fn check_api_key(&self, api_key: ApiKey) -> Result<Option<AccountIdLight>, HttpRequestError> {
        let request = self
            .client
            .get(self.base_url.join(PATH_CHECK_API_KEY).unwrap())
            .header(ApiKeyHeader::name(), api_key.as_str())
            .build()
            .unwrap();

        let response = self
            .client
            .execute(request)
            .await
            .into_error_with_info(HttpRequestError::Reqwest, InternalApiRequest::CheckApiKey)?;

        if response.status() == StatusCode::OK {
            let id: AccountIdLight = response.json().await.into_error_with_info(
                HttpRequestError::SerdeDeserialize,
                InternalApiRequest::CheckApiKey,
            )?;
            Ok(Some(id))
        } else {
            Ok(None)
        }
    }
}

pub struct MediaServerInternalApi {
    client: Client,
}

impl MediaServerInternalApi {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn post_image() {}
}
