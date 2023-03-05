pub mod app;
pub mod database;
pub mod internal;
pub mod session;
pub mod user;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use tokio::{signal, task::JoinHandle};
use tracing::{debug, error, info};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    api::ApiDoc,
    config::Config,
    server::{app::App, database::DatabaseManager, internal::InternalApp}, client::account::AccountInternalApiUrls,
};

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

        let app = App::new(router_database_handle, self.config.external_service_urls()).await;

        let server_task = self.create_public_api_server_task(&app);
        let internal_server_task = if self.config.debug_mode() {
            None
        } else {
            Some(self.create_internal_api_server_task(&app))
        };

        // Wait until both tasks quit
        server_task
            .await
            .expect("Public API server task panic detected");
        if let Some(handle) = internal_server_task {
            handle
                .await
                .expect("Internal API server task panic detected");
        }

        info!("Server quit started");

        drop(app);
        database_manager.close().await;

        info!("Server quit done");
    }

    pub fn create_public_api_server_task(&self, app: &App) -> JoinHandle<()> {
        // Public API. This can have WAN access.
        let normal_api_server = {
            let router = self.create_public_router(&app);
            let router = if self.config.debug_mode() {
                router
                    .merge(Self::create_swagger_ui())
                    .merge(self.create_internal_router(&app))
            } else {
                router
            };

            let addr = self.config.socket().public_api;
            info!("Public API is available on {}", addr);
            if self.config.debug_mode() {
                info!("Internal API is available on {}", addr);
            }
            axum::Server::bind(&addr).serve(router.into_make_service())
        };

        tokio::spawn(async move {
            let shutdown_handle = normal_api_server.with_graceful_shutdown(async {
                match signal::ctrl_c().await {
                    Ok(()) => (),
                    Err(e) => error!("Failed to listen CTRL+C. Error: {}", e),
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
        })
    }

    pub fn create_internal_api_server_task(&self, app: &App) -> JoinHandle<()> {
        // Internal server to server API. This must be only LAN accessible.
        let internal_api_server = {
            let router = self.create_internal_router(&app);
            let router = if self.config.debug_mode() {
                router.merge(Self::create_swagger_ui())
            } else {
                router
            };

            let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
            info!("Internal API is available on {}", addr);
            axum::Server::bind(&addr).serve(router.into_make_service())
        };

        tokio::spawn(async move {
            let shutdown_handle = internal_api_server.with_graceful_shutdown(async {
                match signal::ctrl_c().await {
                    Ok(()) => (),
                    Err(e) => error!("Failed to listen CTRL+C. Error: {}", e),
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
        })
    }

    pub fn create_public_router(&self, app: &App) -> Router {
        let mut router = Router::new();

        if self.config.components().profile {
            router = router.merge(app.create_core_server_router())
        }

        if self.config.components().media {
            router = router.merge(app.create_media_server_router())
        }

        router
    }

    pub fn create_internal_router(&self, app: &App) -> Router {
        let mut router = Router::new();
        if self.config.components().profile {
            router = router.merge(InternalApp::create_core_server_router(app.state()))
        }

        if self.config.components().media {
            router = router.merge(InternalApp::create_media_server_router(app.state()))
        }

        router
    }

    pub fn create_swagger_ui() -> SwaggerUi {
        SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", ApiDoc::openapi())
    }
}
