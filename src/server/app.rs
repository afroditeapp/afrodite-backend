use std::{sync::Arc};

use axum::{
    middleware,
    routing::{get, post, put, patch},
    Json, Router,
};


use utoipa::OpenApi;

use crate::{
    api::{
        self,
        ApiDoc, GetApiKeys, GetConfig, GetInternalApi, GetUsers, ReadDatabase, WriteDatabase,
    },
    config::Config,
};

use super::{
    database::{
        commands::{WriteCommandRunnerHandle},
        read::ReadCommands,
        utils::{AccountIdManager, ApiKeyManager},
        RouterDatabaseReadHandle,
    },
    internal::{InternalApiClient, InternalApiManager},
};

#[derive(Clone)]
pub struct AppState {
    database: Arc<RouterDatabaseReadHandle>,
    internal_api: Arc<InternalApiClient>,
    config: Arc<Config>,
}

impl GetApiKeys for AppState {
    fn api_keys(&self) -> ApiKeyManager<'_> {
        self.database.api_key_manager()
    }
}

impl GetUsers for AppState {
    fn users(&self) -> AccountIdManager<'_> {
        self.database.account_id_manager()
    }
}

impl ReadDatabase for AppState {
    fn read_database(&self) -> ReadCommands<'_> {
        self.database.read()
    }
}

impl WriteDatabase for AppState {
    fn write_database(&self) -> &WriteCommandRunnerHandle {
        self.database.write()
    }
}

impl GetInternalApi for AppState {
    fn internal_api(&self) -> InternalApiManager {
        InternalApiManager::new(&self.config, &self.internal_api, self.api_keys(), self.read_database())
    }
}

impl GetConfig for AppState {
    fn config(&self) -> &Config {
        &self.config
    }
}

pub struct App {
    state: AppState,
}

impl App {
    pub async fn new(database_handle: RouterDatabaseReadHandle, config: Arc<Config>) -> Self {
        let state = AppState {
            database: Arc::new(database_handle),
            internal_api: InternalApiClient::new(config.external_service_urls().clone()).into(),
            config,
        };

        Self { state }
    }

    pub fn state(&self) -> AppState {
        self.state.clone()
    }

    pub fn create_account_server_router(&self) -> Router {
        let public = Router::new()
            .route(
                api::account::PATH_REGISTER,
                post({
                    let state = self.state.clone();
                    move || api::account::post_register(state)
                }),
            )
            .route(
                api::account::PATH_LOGIN,
                post({
                    let state = self.state.clone();
                    move |body| api::account::post_login(body, state)
                }),
            );

        let private = Router::new()
            .route(
                api::account::PATH_ACCOUNT_STATE,
                get({
                    let state = self.state.clone();
                    move |body| api::account::get_account_state(body, state)
                }),
            )
            .route(
                api::account::PATH_ACCOUNT_SETUP,
                post({
                    let state = self.state.clone();
                    move |arg1, arg2| api::account::post_account_setup(arg1, arg2, state)
                }),
            )
            .route(
                api::account::PATH_ACCOUNT_COMPLETE_SETUP,
                post({
                    let state = self.state.clone();
                    move |arg1| api::account::post_complete_setup(arg1, state)
                }),
            )
            .route_layer({
                middleware::from_fn({
                    let state = self.state.clone();
                    move |req, next| api::utils::authenticate_with_api_key(state.clone(), req, next)
                })
            });

        Router::new().merge(public).merge(private)
    }

    pub fn create_profile_server_router(&self) -> Router {
        let public = Router::new();

        let private = Router::new()
            .route(
                api::profile::PATH_GET_PROFILE,
                get({
                    let state = self.state.clone();
                    move |body| api::profile::get_profile(body, state)
                }),
            )
            .route(
                api::profile::PATH_GET_DEFAULT_PROFILE,
                get({
                    let state = self.state.clone();
                    move |body| api::profile::get_default_profile(body, state)
                }),
            )
            .route(
                api::profile::PATH_POST_PROFILE,
                post({
                    let state = self.state.clone();
                    move |header, body| api::profile::post_profile(header, body, state)
                }),
            )
            .route_layer({
                middleware::from_fn({
                    let state = self.state.clone();
                    move |req, next| api::utils::authenticate_with_api_key(state.clone(), req, next)
                })
            });

        Router::new().merge(public).merge(private)
    }

    pub fn create_media_server_router(&self) -> Router {
        let public = Router::new();

        let private = Router::new()
            .route(
                api::media::PATH_GET_IMAGE,
                get({
                    let state = self.state.clone();
                    move |user_id, image_name| api::media::get_image(user_id, image_name, state)
                }),
            )
            .route(
                api::media::PATH_MODERATION_REQUEST,
                get({
                    let state = self.state.clone();
                    move |param1| api::media::get_moderation_request(param1, state)
                }),
            )
            .route(
                api::media::PATH_MODERATION_REQUEST,
                put({
                    let state = self.state.clone();
                    move |param1, param2| api::media::put_moderation_request(param1, param2, state)
                }),
            )
            .route(
                api::media::PATH_MODERATION_REQUEST_SLOT,
                put({
                    let state = self.state.clone();
                    move |param1, param2, param3| api::media::put_image_to_moderation_slot(param1, param2, param3, state)
                }),
            )
            .route(
                api::media::PATH_ADMIN_MODERATION_PAGE_NEXT,
                patch({
                    let state = self.state.clone();
                    move |param1| api::media::patch_moderation_request_list(param1, state)
                }),
            )
            .route(
                api::media::PATH_ADMIN_MODERATION_HANDLE_REQUEST,
                post({
                    let state = self.state.clone();
                    move |param1, param2| api::media::post_handle_moderation_request(param1, param2, state)
                }),
            )
            .route_layer({
                middleware::from_fn({
                    let state = self.state.clone();
                    move |req, next| api::utils::authenticate_with_api_key(state.clone(), req, next)
                })
            });

        Router::new().merge(public).merge(private)
    }
}

async fn root(_state: AppState) -> &'static str {
    "Test123"
}

async fn openapi(_state: AppState) -> Json<utoipa::openapi::OpenApi> {
    ApiDoc::openapi().into()
}

// #[cfg(test)]
// mod tests {
//     use std::path::{PathBuf, Path};

//     use axum::{Router, http::{Request, StatusCode, Method, header}, body::{Body}};
//     use hyper::header::HeaderName;
//     use serde_json::json;
//     use tokio::sync::mpsc;
//     use tower::ServiceExt;

//     use crate::{
//         server::{database::DatabaseManager, app::{App}},
//         config::Config,
//         api::core::user::{RegisterResponse},
//     };

//     fn router() -> Router {
//         let config = Config {
//             database_dir: Path::new("unit-test-data").to_owned(),
//         };
//         let (sender, receiver) = mpsc::channel(64);
//         let (database_handle, quit_sender, database_task_sender) =
//             DatabaseManager::start_task(config.into(), sender, receiver);
//         let app = App::new(database_task_sender);
//         app.create_router()
//     }

//     #[tokio::test]
//     async fn root() {
//         let response = router()
//             .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
//             .await
//             .unwrap();

//         assert_eq!(response.status(), StatusCode::OK);
//     }

//     #[tokio::test]
//     async fn register() {
//         let response = router()
//             .oneshot(
//                 Request::builder()
//                     .method(Method::POST)
//                     .uri("/register")
//                     .header(header::CONTENT_TYPE, "application/json")
//                     .body(Body::from(
//                         serde_json::to_vec(&json!({
//                             "name": "test"
//                         })).unwrap()
//                     ))
//                     .unwrap()
//             )
//             .await
//             .unwrap();

//         assert_eq!(response.status(), StatusCode::OK);

//         let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
//         let _response: RegisterResponse = serde_json::from_slice(&body).unwrap();
//     }
// }
