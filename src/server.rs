pub mod app;
pub mod database;
pub mod internal;

use std::{sync::Arc, task::Poll, net::SocketAddr};

use axum::{Router, BoxError};
use hyper::server::accept::Accept;
use tokio::{signal, task::JoinHandle, io::DuplexStream, sync::mpsc};
use tower::ServiceBuilder;
use tower_http::trace::{TraceLayer, DefaultOnResponse};
use tracing::{error, info};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    api::ApiDoc,
    config::Config,
    server::{app::{App, connection::WebSocketManager}, database::DatabaseManager, internal::InternalApp},
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

        let (database_manager, router_database_handle) = DatabaseManager::new(
            self.config.database_dir().to_path_buf(),
            self.config.clone(),
        )
        .await
        .expect("Database init failed");

        let (ws_manager, mut ws_quit_ready, server_quit_handle) = WebSocketManager::new();

        let mut app = App::new(router_database_handle, self.config.clone(), ws_manager).await;

        let server_task = self.create_public_api_server_task(&mut app);
        let internal_server_task = if self.config.debug_mode() {
            None
        } else {
            Some(self.create_internal_api_server_task(&app))
        };

        match signal::ctrl_c().await {
            Ok(()) => (),
            Err(e) => error!("Failed to listen CTRL+C. Error: {}", e),
        }

        info!("Server quit started");

        drop(server_quit_handle);

        // Wait until all tasks quit
        server_task
            .await
            .expect("Public API server task panic detected");
        if let Some(handle) = internal_server_task {
            handle
                .await
                .expect("Internal API server task panic detected");
        }

        loop {
            match ws_quit_ready.recv().await {
                Some(()) => (),
                None => break,
            }
        }

        drop(app);
        database_manager.close().await;

        info!("Server quit done");
    }

    pub fn create_public_api_server_task(&self, app: &mut App) -> JoinHandle<()> {
        // Public API. This can have WAN access.
        let normal_api_server = {
            let router = self.create_public_router(app);
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

            let router = if self.config.debug_mode() {
                router.route_layer(
                    TraceLayer::new_for_http()
                )
            } else {
                router
            };

            axum::Server::bind(&addr).serve(router.into_make_service_with_connect_info::<SocketAddr>())
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

            let addr = self.config.socket().internal_api;
            info!("Internal API is available on {}", addr);
            axum::Server::bind(&addr).serve(router.into_make_service_with_connect_info::<SocketAddr>())
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

    pub fn create_public_router(&self, app: &mut App) -> Router {
        let mut router = app.create_common_server_router();

        if self.config.components().account {
            router = router.merge(app.create_account_server_router())
        }

        if self.config.components().profile {
            router = router.merge(app.create_profile_server_router())
        }

        if self.config.components().media {
            router = router.merge(app.create_media_server_router())
        }

        router
    }

    pub fn create_internal_router(&self, app: &App) -> Router {
        let mut router = Router::new();
        if self.config.components().account {
            router = router.merge(InternalApp::create_account_server_router(app.state()))
        }

        if self.config.components().profile {
            router = router.merge(InternalApp::create_profile_server_router(app.state()))
        }

        if self.config.components().media {
            router = router.merge(InternalApp::create_media_server_router(app.state()))
        }

        router
    }

    pub fn create_swagger_ui() -> SwaggerUi {
        SwaggerUi::new("/swagger-ui").url("/api-doc/pihka_api.json", ApiDoc::openapi())
    }
}
