pub mod app;
pub mod database;
pub mod session;
pub mod user;
pub mod internal;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use tokio::signal;
use tracing::{debug, error, info};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    config::{Config},
    server::{app::App, database::DatabaseManager, internal::InternalApp}, api::ApiDoc,
};

pub const CORE_SERVER_INTERNAL_API_URL: &str = "http://127.0.0.1:3001";
pub const MEDIA_SERVER_INTERNAL_API_URL: &str = "http://127.0.0.1:4001";

pub struct PihkaServer {
    config: Arc<Config>,
}

impl PihkaServer {
    pub fn new(config: Config) -> Self {
        Self {
            config: config.into(),
        }
    }

    pub async fn run(self) {
        tracing_subscriber::fmt::init();

        let (database_manager, router_database_handle) =
            DatabaseManager::new(self.config.database_dir().to_path_buf())
                .await
                .expect("Database init failed");


        let app = App::new(router_database_handle).await;

        // Public API. This can have WAN access.
        let normal_api_server = {
            let mut router = Router::new();

            if self.config.components().core {
                router = router.merge(app.create_core_server_router())
            }

            if self.config.components().media {
                router = router.merge(app.create_media_server_router())
            }

            // TODO: Enable swagger-ui only if in debug mode.
            let router = router.merge(
                SwaggerUi::new("/swagger-ui")
                    .url("/api-doc/openapi.json", ApiDoc::openapi()),
            );

            let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
            info!("Public API is available on {}", addr);
            axum::Server::bind(&addr).serve(router.into_make_service())
        };

        // Internal server to server API. This must be only LAN accessible.
        let internal_api_server = {
            let mut router = Router::new();
            if self.config.components().core {
                router = router.merge(InternalApp::create_core_server_router(app.state()))
            }

            if self.config.components().media {
                router = router.merge(InternalApp::create_media_server_router(app.state()))
            }

            // TODO: Enable swagger-ui only if in debug mode.
            let router = router.merge(
                SwaggerUi::new("/swagger-ui")
                    .url("/api-doc/openapi.json", ApiDoc::openapi()),
            );

            let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
            info!("Internal API is available on {}", addr);
            axum::Server::bind(&addr).serve(router.into_make_service())
        };

        let server_task = tokio::spawn(async move {
            let shutdown_handle = normal_api_server.with_graceful_shutdown(async {
                match signal::ctrl_c().await {
                    Ok(()) => (),
                    Err(e) =>
                        error!("Failed to listen CTRL+C. Error: {}", e),
                }
            });

            match shutdown_handle.await {
                Ok(()) => {
                    info!("Public API server future returned Ok()");
                }
                Err(e) => {
                    error!("Public API server future returned error: {}", e);
                }
            }
        });

        let internal_server_task = tokio::spawn(async move {
            let shutdown_handle = internal_api_server.with_graceful_shutdown(async {
                match signal::ctrl_c().await {
                    Ok(()) => (),
                    Err(e) =>
                        error!("Failed to listen CTRL+C. Error: {}", e),
                }
            });

            match shutdown_handle.await {
                Ok(()) => {
                    info!("Internal API server future returned Ok()");
                }
                Err(e) => {
                    error!("Internal API server future returned error: {}", e);
                }
            }
        });

        server_task.await.expect("Public API server task panic detected");
        internal_server_task.await.expect("Internal API server task panic detected");

        info!("Server quit started");

        drop(app);
        database_manager.close().await;

        info!("Server quit done");
    }
}
