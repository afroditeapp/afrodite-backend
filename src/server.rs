
pub mod user;
pub mod database;
pub mod app;
pub mod session;


use std::sync::Arc;
use std::net::SocketAddr;

use axum::{Router, routing::get};

use tokio::{sync::{mpsc, watch::error}, signal};
use tracing::{debug, error, info};


use crate::{config::{Config, self}, server::{database::{DatabaseOperationHandle, util::DatabasePath}, app::App}};

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

        let (database_handle, mut database_quit_receiver) = DatabaseOperationHandle::new();

        let app = App::new(DatabasePath::new(self.config.database_dir.clone()), database_handle.clone());
        let router = app.create_router();

        let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
        debug!("listening on {}", addr);
        let server = axum::Server::bind(&addr)
            .serve(router.into_make_service());

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

        drop(database_handle);
        drop(app);
        loop {
            match database_quit_receiver.recv().await {
                None => break,
                Some(()) => ()
            }
        }

        info!("Server quit done");
    }
}
