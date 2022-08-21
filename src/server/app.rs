
pub mod api;

use std::sync::Arc;

use axum::{Router, routing::{get, post}, Json};
use tracing::{debug, error, info};

use self::api::{CreateProfile, CreateProfileResponse, PATH_REGISTER};

use super::database::{DatabaseManager, DatabaseTaskSender, DatabaseCommand};


#[derive(Debug, Clone)]
pub struct AppState {
    database: DatabaseTaskSender,
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
            .route(
                "/",
                get({
                    let state = self.state.clone();
                    move || root(state)
                }),
            )
            .route(
                PATH_REGISTER,
                post({
                    let state = self.state.clone();
                    move |body| register(body, state)
                }),
            )
    }
}


async fn root(state: AppState) -> &'static str {
    "Test123"
}

// TODO: Add timeout for database commands

async fn register(
    Json(profile_info): Json<CreateProfile>,
    mut state: AppState,
) -> Json<CreateProfileResponse> {
    let cmd = DatabaseCommand::RegisterProfile(profile_info);
    match state.database.send_command(cmd).await.await.unwrap() {
        Ok(response) => response.into(),
        Err(e) => {
            error!("Database task error: {:?}", e);
            CreateProfileResponse::error().into()
        }
    }
}


#[cfg(test)]
mod tests {
    use std::path::{PathBuf, Path};

    use axum::{Router, http::{Request, StatusCode, Method, header}, body::{Body}};
    use hyper::header::HeaderName;
    use serde_json::json;
    use tokio::sync::mpsc;
    use tower::ServiceExt;

    use crate::{server::{database::DatabaseManager, app::{App, api::{CreateProfileResponse, PATH_REGISTER}}, }, config::Config};

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
                    .uri(PATH_REGISTER)
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
        let _response: CreateProfileResponse = serde_json::from_slice(&body).unwrap();
    }
}
