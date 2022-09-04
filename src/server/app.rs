use std::sync::Arc;

use axum::{Router, routing::{get, post}, Json};
use tracing::{debug, error, info};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::api::{core::{profile::{RegisterBody, RegisterResponse}, ApiDocCore}, self, GetDatabaseTaskSender};

use super::database::{DatabaseManager, DatabaseTaskSender};


#[derive(Debug, Clone)]
pub struct AppState {
    database: DatabaseTaskSender,
}

impl GetDatabaseTaskSender for AppState {
    fn database(&mut self) -> &mut DatabaseTaskSender {
        &mut self.database
    }
}


pub struct App {
    state: AppState,
}

impl App {
    pub fn new(database: DatabaseTaskSender) -> Self {
        let state = AppState {
            database,
        };

        Self { state }
    }

    pub fn create_router(&self) -> Router {
        Router::new()
            .merge(
                SwaggerUi::new("/swagger-ui/*tail")
                    .url("/api-doc/openapi.json", ApiDocCore::openapi())
                )
            .route(
                "/openapi.json",
                get({
                    let state = self.state.clone();
                    move || openapi(state)
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
                    move |body| api::core::register(body, state)
                }),
            )
            .route(
                api::core::PATH_LOGIN,
                post({
                    let state = self.state.clone();
                    move |body| api::core::login(body, state)
                }),
            )
    }
}


async fn root(state: AppState) -> &'static str {
    "Test123"
}

async fn openapi(state: AppState) -> Json<utoipa::openapi::OpenApi> {
    ApiDocCore::openapi().into()
}

#[cfg(test)]
mod tests {
    use std::path::{PathBuf, Path};

    use axum::{Router, http::{Request, StatusCode, Method, header}, body::{Body}};
    use hyper::header::HeaderName;
    use serde_json::json;
    use tokio::sync::mpsc;
    use tower::ServiceExt;

    use crate::{
        server::{database::DatabaseManager, app::{App}},
        config::Config,
        api::core::profile::{RegisterResponse},
    };

    fn router() -> Router {
        let config = Config {
            database_dir: Path::new("unit-test-data").to_owned(),
        };
        let (sender, receiver) = mpsc::channel(64);
        let (database_handle, quit_sender, database_task_sender) =
            DatabaseManager::start_task(config.into(), sender, receiver);
        let app = App::new(database_task_sender);
        app.create_router()
    }

    #[tokio::test]
    async fn root() {
        let response = router()
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn register() {
        let response = router()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/register")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        serde_json::to_vec(&json!({
                            "name": "test"
                        })).unwrap()
                    ))
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let _response: RegisterResponse = serde_json::from_slice(&body).unwrap();
    }
}
