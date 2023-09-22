#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

pub mod api;
pub mod app;
pub mod bot;
pub mod data;
pub mod internal;
pub mod litestream;
pub mod manager_client;
pub mod media_backup;
pub mod utils;
pub mod image;
pub mod map;

use std::{net::SocketAddr, pin::Pin, sync::Arc};

use axum::Router;
use config::Config;
use futures::future::poll_fn;
use hyper::server::{
    accept::Accept,
    conn::{AddrIncoming, Http},
};
use tokio::{
    net::TcpListener,
    signal::{
        self,
        unix::{Signal, SignalKind},
    },
    sync::{broadcast, mpsc},
    task::JoinHandle,
};
use tokio_rustls::{rustls::ServerConfig, TlsAcceptor};
use tower::MakeService;
use tower_http::trace::TraceLayer;
use tracing::{error, info, warn};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use self::{
    app::{
        connection::{ServerQuitWatcher, WebSocketManager},
        routes_internal::InternalApp,
        App,
    },
    data::{write_commands::WriteCommandRunnerHandle, DatabaseManager},
};
use crate::{api::ApiDoc, litestream::LitestreamManager, media_backup::MediaBackupManager, bot::BotClient};

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

        info!(
            "Backend version: {}-{}",
            self.config.backend_semver_version(),
            self.config.backend_code_version()
        );

        if self.config.debug_mode() {
            warn!("Debug mode is enabled");
        }

        let mut terminate_signal = signal::unix::signal(SignalKind::terminate()).unwrap();

        let mut litestream = None;
        if let Some(litestream_config) = self.config.litestream() {
            let mut litestream_manager =
                LitestreamManager::new(self.config.clone(), litestream_config.clone());
            litestream_manager
                .start_litestream()
                .await
                .expect("Litestream start failed");
            litestream = Some(litestream_manager);
        }

        let (server_quit_handle, server_quit_watcher) = broadcast::channel(1);
        let (media_backup_quit, media_backup_handle) =
            MediaBackupManager::new(self.config.clone(), server_quit_watcher.resubscribe());

        let (database_manager, router_database_handle, router_database_write_handle) =
            DatabaseManager::new(
                self.config.database_dir().to_path_buf(),
                self.config.clone(),
                media_backup_handle,
            )
            .await
            .expect("Database init failed");

        let (ws_manager, mut ws_quit_ready) =
            WebSocketManager::new(server_quit_watcher.resubscribe());

        let (write_cmd_runner_handle, write_cmd_waiter) =
            WriteCommandRunnerHandle::new(router_database_write_handle.clone(), &self.config);

        let mut app = App::new(
            router_database_handle,
            router_database_write_handle,
            write_cmd_runner_handle,
            self.config.clone(),
            ws_manager,
        )
        .await
        .expect("App init failed");

        let server_task = self
            .create_public_api_server_task(&mut app, server_quit_watcher.resubscribe())
            .await;
        let internal_server_task = self
            .create_internal_api_server_task(&app, server_quit_watcher.resubscribe())
            .await;

        let bot_client = if let Some(bot_config) = self.config.bot_config() {
            let result = BotClient::start_bots(&self.config, bot_config)
                .await;

            match result {
                Ok(bot_manager) => Some(bot_manager),
                Err(e) => {
                    error!("Bot client start failed: {:?}", e);
                    None
                }
            }
        } else {
            None
        };

        Self::wait_quit_signal(&mut terminate_signal).await;

        info!("Server quit started");

        if let Some(bot_client) = bot_client {
            match bot_client.stop_bots().await {
                Ok(()) => (),
                Err(e) => error!("Bot client stop failed: {:?}", e),
            }
        }

        drop(server_quit_handle);

        // Wait until all tasks quit
        server_task
            .await
            .expect("Public API server task panic detected");
        internal_server_task
            .await
            .expect("Internal API server task panic detected");

        loop {
            match ws_quit_ready.recv().await {
                Some(()) => (),
                None => break,
            }
        }

        drop(app);
        write_cmd_waiter.wait_untill_all_writing_ends().await;
        database_manager.close().await;
        media_backup_quit.wait_quit().await;

        if let Some(litestream) = litestream {
            match litestream.stop_litestream().await {
                Ok(()) => (),
                Err(e) => error!("Litestream stop failed: {:?}", e),
            }
        }

        info!("Server quit done");
    }

    pub async fn wait_quit_signal(terminate_signal: &mut Signal) {
        tokio::select! {
            _ = terminate_signal.recv() => {}
            result = signal::ctrl_c() => {
                match result {
                    Ok(()) => (),
                    Err(e) => error!("Failed to listen CTRL+C. Error: {}", e),
                }
            }
        }
    }

    /// Public API. This can have WAN access.
    pub async fn create_public_api_server_task(
        &self,
        app: &mut App,
        quit_notification: ServerQuitWatcher,
    ) -> JoinHandle<()> {
        let router = {
            let router = self.create_public_router(app);
            let router = if self.config.debug_mode() {
                router
                    .merge(Self::create_swagger_ui())
                    .merge(self.create_internal_router(&app))
            } else {
                router
            };
            let router = if self.config.debug_mode() {
                router.route_layer(TraceLayer::new_for_http())
            } else {
                router
            };
            router
        };

        let addr = self.config.socket().public_api;
        info!("Public API is available on {}", addr);
        if self.config.debug_mode() {
            info!("Internal API is available on {}", addr);
        }

        if let Some(tls_config) = self.config.public_api_tls_config() {
            self.create_server_task_with_tls(addr, router, tls_config.clone(), quit_notification)
                .await
        } else {
            self.create_server_task_no_tls(router, addr, "Public API", quit_notification)
        }
    }

    pub async fn create_server_task_with_tls(
        &self,
        addr: SocketAddr,
        router: Router,
        tls_config: Arc<ServerConfig>,
        mut quit_notification: ServerQuitWatcher,
    ) -> JoinHandle<()> {
        let listener = TcpListener::bind(addr)
            .await
            .expect("Address not available");
        let mut listener =
            AddrIncoming::from_listener(listener).expect("AddrIncoming creation failed");
        listener.set_sleep_on_errors(true);

        let protocol = Arc::new(Http::new());
        let acceptor = TlsAcceptor::from(tls_config);

        let mut app_service = router.into_make_service_with_connect_info::<SocketAddr>();

        tokio::spawn(async move {
            let (drop_after_connection, mut wait_all_connections) = mpsc::channel::<()>(1);

            loop {
                let next_addr_stream = poll_fn(|cx| Pin::new(&mut listener).poll_accept(cx));

                let stream = tokio::select! {
                    _ = quit_notification.recv() => {
                        break;
                    }
                    addr = next_addr_stream => {
                        match addr {
                            None => {
                                error!("Socket closed");
                                break;
                            }
                            Some(Err(e)) => {
                                error!("Address stream error {e}");
                                continue;
                            }
                            Some(Ok(stream)) => {
                                stream
                            }
                        }
                    }
                };

                let acceptor = acceptor.clone();
                let protocol = protocol.clone();
                let service = app_service.make_service(&stream);

                let mut quit_notification = quit_notification.resubscribe();
                let drop_on_quit = drop_after_connection.clone();
                tokio::spawn(async move {
                    tokio::select! {
                        _ = quit_notification.recv() => {} // Graceful shutdown for connections?
                        connection = acceptor.accept(stream) => {
                            match connection {
                                Ok(connection) => {
                                    if let Ok(service) = service.await {
                                        let _ = protocol.serve_connection(connection, service).with_upgrades().await;
                                    }
                                }
                                Err(_) => {},
                            }
                        }
                    }

                    drop(drop_on_quit);
                });
            }
            drop(drop_after_connection);
            drop(quit_notification);

            loop {
                match wait_all_connections.recv().await {
                    Some(()) => (),
                    None => break,
                }
            }
        })
    }

    pub fn create_server_task_no_tls(
        &self,
        router: Router,
        addr: SocketAddr,
        name_for_log_message: &'static str,
        mut quit_notification: ServerQuitWatcher,
    ) -> JoinHandle<()> {
        let normal_api_server = {
            axum::Server::bind(&addr)
                .serve(router.into_make_service_with_connect_info::<SocketAddr>())
        };

        tokio::spawn(async move {
            let shutdown_handle = normal_api_server.with_graceful_shutdown(async {
                let _ = quit_notification.recv().await;
            });

            match shutdown_handle.await {
                Ok(()) => {
                    info!("{name_for_log_message} server future returned Ok()");
                }
                Err(e) => {
                    error!("{name_for_log_message} server future returned error: {}", e);
                }
            }
        })
    }

    // Internal server to server API. This must be only LAN accessible.
    pub async fn create_internal_api_server_task(
        &self,
        app: &App,
        quit_notification: ServerQuitWatcher,
    ) -> JoinHandle<()> {
        let router = self.create_internal_router(&app);
        let router = if self.config.debug_mode() {
            router.merge(Self::create_swagger_ui())
        } else {
            router
        };

        let addr = self.config.socket().internal_api;
        info!("Internal API is available on {}", addr);
        if let Some(tls_config) = self.config.internal_api_tls_config() {
            self.create_server_task_with_tls(addr, router, tls_config.clone(), quit_notification)
                .await
        } else {
            self.create_server_task_no_tls(router, addr, "Internal API", quit_notification)
        }
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

        if self.config.components().chat {
            router = router.merge(app.create_chat_server_router())
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

        if self.config.components().chat {
            router = router.merge(InternalApp::create_chat_server_router(app.state()))
        }

        router
    }

    pub fn create_swagger_ui() -> SwaggerUi {
        SwaggerUi::new("/swagger-ui").url("/api-doc/pihka_api.json", ApiDoc::openapi())
    }
}
