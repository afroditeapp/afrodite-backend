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
