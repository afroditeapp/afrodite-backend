pub mod app;
pub mod database;
pub mod session;
pub mod user;
pub mod internal;

use std::net::SocketAddr;
use std::sync::Arc;

use tokio::signal;
use tracing::{debug, error, info};

use crate::{
    config::{Config, ServerMode},
    server::{app::App, database::DatabaseManager, internal::InternalApp},
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
            DatabaseManager::new(self.config.database_dir.clone())
                .await
                .expect("Database init failed");

        // Public API. This can have WAN access.
        let app = App::new(router_database_handle).await;
        let (router, port) = match self.config.mode {
            ServerMode::Core => (app.create_core_server_router(), 3000),
            ServerMode::Media => (app.create_media_server_router(), 4000),
        };
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        info!("Public API is available on {}", addr);
        let server = axum::Server::bind(&addr).serve(router.into_make_service());

        // Internal server to server API. This must be only LAN accessible.
        let (internal_api_router, port) = match self.config.mode {
            ServerMode::Core => (InternalApp::create_core_server_router(app.state()), 3001),
            ServerMode::Media => (InternalApp::create_media_server_router(app.state()), 4001),
        };
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        info!("Internal API is available on {}", addr);
        let internal_server = axum::Server::bind(&addr).serve(internal_api_router.into_make_service());

        let server_task = tokio::spawn(async move {
            let shutdown_handle = server.with_graceful_shutdown(async {
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
            let shutdown_handle = internal_server.with_graceful_shutdown(async {
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
