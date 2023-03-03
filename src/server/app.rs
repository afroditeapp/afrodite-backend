use std::{collections::HashMap, sync::Arc};

use axum::{
    middleware,
    routing::{get, post},
    Json, Router,
};
use tokio::sync::{Mutex, RwLock};

use utoipa::OpenApi;

use crate::api::{
    self,
    model::{ApiKey, AccountId},
    ApiDoc, GetApiKeys, GetCoreServerInternalApi, GetMediaServerInternalApi,
    GetRouterDatabaseHandle, GetSessionManager, GetUsers, ReadDatabase, WriteDatabase,
};

use super::{
    database::{read::ReadCommands, write::WriteCommands, RouterDatabaseHandle},
    internal::{CoreServerInternalApi, MediaServerInternalApi},
    session::{SessionManager, UserState},
};

#[derive(Clone)]
pub struct AppState {
    session_manager: Arc<SessionManager>,
    client: reqwest::Client,
}

impl GetSessionManager for AppState {
    fn session_manager(&self) -> &SessionManager {
        &self.session_manager
    }
}

impl GetRouterDatabaseHandle for AppState {
    fn database(&self) -> &RouterDatabaseHandle {
        &self.session_manager.database
    }
}

impl GetApiKeys for AppState {
    fn api_keys(&self) -> &RwLock<HashMap<ApiKey, UserState>> {
        &self.session_manager.api_keys
    }
}

impl GetUsers for AppState {
    fn users(&self) -> &RwLock<HashMap<AccountId, Mutex<WriteCommands>>> {
        &self.session_manager.users
    }
}

impl ReadDatabase for AppState {
    fn read_database(&self) -> ReadCommands {
        self.session_manager.database.read()
    }
}

impl WriteDatabase for AppState {
    fn write_database_with_db_macro_do_not_call_this_outside_macros(
        &self,
    ) -> &RwLock<HashMap<AccountId, Mutex<WriteCommands>>> {
        &self.session_manager.users
    }
}

impl GetCoreServerInternalApi for AppState {
    fn core_server_internal_api(&self) -> CoreServerInternalApi {
        CoreServerInternalApi::new(self.client.clone())
    }
}

impl GetMediaServerInternalApi for AppState {
    fn media_server_internal_api(&self) -> MediaServerInternalApi {
        MediaServerInternalApi::new(self.client.clone())
    }
}

pub struct App {
    state: AppState,
}

impl App {
    pub async fn new(database_handle: RouterDatabaseHandle) -> Self {
        let state = AppState {
            session_manager: Arc::new(SessionManager::new(database_handle).await),
            client: reqwest::Client::new(),
        };

        Self { state }
    }

    pub fn state(&self) -> AppState {
        self.state.clone()
    }

    pub fn create_core_server_router(&self) -> Router {
        let public = Router::new()
            .route(
                "/",
                get({
                    let state = self.state.clone();
                    move || root(state)
                }),
            )
            .route(
                api::account::PATH_REGISTER,
                post({
                    let state = self.state.clone();
                    move || api::account::register(state)
                }),
            )
            .route(
                api::account::PATH_LOGIN,
                post({
                    let state = self.state.clone();
                    move |body| api::account::login(body, state)
                }),
            );

        let private = Router::new()
            .route(
                api::profile::PATH_GET_PROFILE,
                get({
                    let state = self.state.clone();
                    move |body| api::profile::get_profile(body, state)
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
                    move |req, next| api::utils::authenticate_core_api(state.clone(), req, next)
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
            .route_layer({
                middleware::from_fn({
                    let state = self.state.clone();
                    move |req, next| api::media::authenticate_media_api(state.clone(), req, next)
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
