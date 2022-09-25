pub mod app;
pub mod database;
pub mod session;
pub mod user;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{routing::get, Router};

use tokio::{
    signal,
    sync::{mpsc, watch::error},
};
use tracing::{debug, error, info};

use crate::{
    config::{self, Config},
    server::{
        app::App,
        database::{DatabaseManager},
    },
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

        let (database_manager, router_database_handle) = DatabaseManager::new(self.config.database_dir.clone()).await.unwrap();

        let app = App::new(
            router_database_handle
        ).await;
        let router = app.create_router();

        let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
        debug!("listening on {}", addr);
        let server = axum::Server::bind(&addr).serve(router.into_make_service());

        let shutdown_handle = server.with_graceful_shutdown(async {
            loop {
                tokio::select! {
                    quit_request = signal::ctrl_c() => {
                        match quit_request {
                            Ok(()) => (),
                            Err(e) =>
                                error!("Failed to listen CTRL+C. Error: {}", e),
                        }
                        break
                    }
                }
            }
        });

        loop {
            tokio::select! {
                result = shutdown_handle => {
                    match result {
                        Ok(()) => {
                            info!("Server future returned Ok()");
                        }
                        Err(e) => {
                            error!("Server future returned error: {}", e);
                        }
                    }

                    break;
                }
            }
        }

        info!("Server quit started");

        drop(app);
        database_manager.close().await;

        info!("Server quit done");
    }
}
