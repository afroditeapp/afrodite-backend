
pub mod profile;
pub mod database;
pub mod app;


use std::sync::Arc;
use std::net::SocketAddr;

use axum::{Router, routing::get};

use tokio::{sync::{mpsc, watch::error}, signal};
use tracing::{debug, error, info};


use crate::{config::{Config, self}, server::{database::DatabaseManager, app::App}};


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

        let (sender, receiver) = mpsc::channel(64);
        let (database_handle, database_quit_sender, database_task_sender) =
            DatabaseManager::start_task(self.config, sender, receiver);

        let app = App::new(database_task_sender);
        let router = app.create_router();

        let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
        debug!("listening on {}", addr);
        let mut server = axum::Server::bind(&addr)
            .serve(router.into_make_service());

        let mut ctrl_c_listener_enabled = true;

        loop {
            tokio::select! {
                quit_request = signal::ctrl_c(), if ctrl_c_listener_enabled => {
                    match quit_request {
                        Ok(()) => {
                            break;
                        }
                        Err(e) => {
                            ctrl_c_listener_enabled = false;
                            error!("Failed to listen CTRL+C. Error: {}", e);
                        }
                    }
                }
                result = &mut server => {
                    match result {
                        Ok(()) => {
                            error!("Server future returned Ok()");
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

        database_quit_sender.send(()).unwrap();

        database_handle.await.unwrap();

        info!("Server quit done");
    }
}
