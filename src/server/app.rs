use std::{sync::Arc, collections::HashMap};

use axum::{
    routing::{get, post},
    Json, Router, middleware,
};
use tokio::sync::{RwLock, Mutex};
use tracing::{debug, error, info};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::api::{
    self,
    core::{
        ApiDocCore, user::{ApiKey, UserId},
    },
    GetSessionManager, GetRouterDatabaseHandle, GetApiKeys, GetUsers,
};

use super::{
    database::{RouterDatabaseHandle, write::WriteCommands},
    session::{SessionManager, UserState},
};

#[derive(Clone)]
pub struct AppState {
    session_manager: Arc<SessionManager>,
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
    fn users(&self) -> &RwLock<HashMap<UserId, Mutex<WriteCommands>>> {
        &self.session_manager.users
    }
}

pub struct App {
    state: AppState,
}

impl App {
    pub async fn new(database_handle: RouterDatabaseHandle) -> Self {
        let state = AppState {
            session_manager: Arc::new(SessionManager::new(database_handle).await),
        };

        Self { state }
    }

    pub fn create_router(&self) -> Router {
        let public = Router::new()
            .merge(
                SwaggerUi::new("/swagger-ui/*tail")
                    .url("/api-doc/openapi.json", ApiDocCore::openapi()),
            )
            .route(
                "/openapi.json",
                get({
                    let state = self.state.clone();
                    move || openapi(state.clone())
                }),
            )
            .route(
                "/",
                get({
                    let state = self.state.clone();
                    move || root(state)
                }),
            )
            .route(
                api::core::PATH_REGISTER,
                post({
                    let state = self.state.clone();
                    move || api::core::register(state)
                }),
            )
            .route(
                api::core::PATH_LOGIN,
                post({
                    let state = self.state.clone();
                    move |body| api::core::login(body, state)
                }),
            );

        let private = Router::new()
            .route(
                api::core::PATH_PROFILE,
                get({
                    let state = self.state.clone();
                    move |body| api::core::profile(body, state)
                }),
            ).route_layer({
                middleware::from_fn({
                    let state = self.state.clone();
                    move |req, next| api::core::authenticate(state.clone(), req, next)
                })
            });

        Router::new()
            .merge(public)
            .merge(private)
    }
}

async fn root(state: AppState) -> &'static str {
    "Test123"
}

async fn openapi(state: AppState) -> Json<utoipa::openapi::OpenApi> {
    ApiDocCore::openapi().into()
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
