pub mod connected_routes;
pub mod connection;
pub mod sign_in_with;

use std::sync::Arc;

use axum::{
    routing::{get, post},
    Json, Router,
};


use utoipa::OpenApi;

use crate::{
    api::{
        self, ApiDoc, GetApiKeys, GetConfig, GetInternalApi, GetUsers, ReadDatabase, SignInWith,
        WriteDatabase,
    },
    config::Config,
};

use self::{
    connected_routes::ConnectedApp, connection::WebSocketManager, sign_in_with::SignInWithManager,
};

use super::{
    database::{
        commands::WriteCommandRunnerHandle,
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
    sign_in_with: Arc<SignInWithManager>,
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

impl SignInWith for AppState {
    fn sign_in_with_manager(&self) -> &SignInWithManager {
        &self.sign_in_with
    }
}

impl GetInternalApi for AppState {
    fn internal_api(&self) -> InternalApiManager {
        InternalApiManager::new(
            &self.config,
            &self.internal_api,
            self.api_keys(),
            self.read_database(),
            self.write_database(),
            self.database.account_id_manager(),
        )
    }
}

impl GetConfig for AppState {
    fn config(&self) -> &Config {
        &self.config
    }
}

pub struct App {
    state: AppState,
    ws_manager: Option<WebSocketManager>,
}

impl App {
    pub async fn new(
        database_handle: RouterDatabaseReadHandle,
        config: Arc<Config>,
        ws_manager: WebSocketManager,
    ) -> Self {
        let state = AppState {
            config: config.clone(),
            database: Arc::new(database_handle),
            internal_api: InternalApiClient::new(config.external_service_urls().clone()).into(),
            sign_in_with: SignInWithManager::new(config).into(),
        };

        Self {
            state,
            ws_manager: Some(ws_manager),
        }
    }

    pub fn state(&self) -> AppState {
        self.state.clone()
    }

    pub fn create_common_server_router(&mut self) -> Router {
        Router::new().route(
            api::common::PATH_CONNECT,
            get({
                let state = self.state.clone();
                let ws_manager = self.ws_manager.take().unwrap(); // Only one instance required.
                move |param1, param2, param3| {
                    api::common::get_connect_websocket(param1, param2, param3, state, ws_manager)
                }
            }),
        )
        // This route checks the access token by itself.
    }

    pub fn create_account_server_router(&self) -> Router {
        let public = Router::new()
            .route(
                api::account::PATH_SIGN_IN_WITH_LOGIN,
                post({
                    let state = self.state.clone();
                    move |body| api::account::post_sign_in_with_login(body, state)
                }),
            );

        public.merge(ConnectedApp::new(self.state.clone()).private_account_server_router())
    }

    pub fn create_profile_server_router(&self) -> Router {
        let public = Router::new();

        public.merge(ConnectedApp::new(self.state.clone()).private_profile_server_router())
    }

    pub fn create_media_server_router(&self) -> Router {
        let public = Router::new();

        public.merge(ConnectedApp::new(self.state.clone()).private_media_server_router())
    }

    pub fn create_chat_server_router(&self) -> Router {
        let public = Router::new();

        public.merge(ConnectedApp::new(self.state.clone()).private_chat_server_router())
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
