//! Routes for server to server connections

use std::{collections::HashMap, sync::Arc};

use axum::{
    middleware,
    routing::{get, post},
    Json, Router,
};
use tokio::sync::{Mutex, RwLock};

use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::api::{
    self,
    core::{
        user::{ApiKey, UserId},
        ApiDocCore, internal::ApiDocCoreInternal,
    },
    GetApiKeys, GetRouterDatabaseHandle, GetSessionManager, GetUsers, ReadDatabase, WriteDatabase, media::{ApiDocMedia, internal::ApiDocMediaInternal},
};

use super::{
    database::{read::ReadCommands, write::WriteCommands, RouterDatabaseHandle},
    session::{SessionManager, UserState}, app::AppState,
};

// TODO: Use TLS for checking that all internal communication comes from trusted
//       sources.

/// Internal route handlers for server to server communication.
pub struct InternalApp;

impl InternalApp {
    pub fn create_core_server_router(state: AppState) -> Router {
        Router::new()
            .merge(
                SwaggerUi::new("/swagger-ui/*tail")
                    .url("/api-doc/openapi.json", ApiDocCoreInternal::openapi()),
            )
            .route(
                api::core::internal::PATH_CHECK_API_KEY,
                get({
                    let state = state.clone();
                    move |body| api::core::internal::check_api_key(body, state)
                }),
            )
    }

    pub fn create_media_server_router(state: AppState) -> Router {
        Router::new()
            .merge(
                SwaggerUi::new("/swagger-ui/*tail")
                    .url("/api-doc/openapi.json", ApiDocMediaInternal::openapi()),
            )
            .route(
                api::media::internal::PATH_POST_IMAGE,
                post({
                    let state = state.clone();
                    move |header1, header2, body| api::media::internal::post_image(header1, header2, body, state)
                })
            )
    }
}
