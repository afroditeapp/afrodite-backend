#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

pub mod app;
pub mod litestream;
pub mod manager_client;
pub mod media_backup;
pub mod utils;
pub mod image;
pub mod map;
pub mod event;
pub mod perf;
pub mod web_socket;
pub mod sign_in_with;

use std::{net::SocketAddr, pin::Pin, sync::Arc};

use app::SimpleBackendAppState;
use async_trait::async_trait;
use axum::Router;
use media_backup::MediaBackupHandle;
use perf::AllCounters;
use simple_backend_config::SimpleBackendConfig;
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
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer, EnvFilter};

use utoipa_swagger_ui::SwaggerUi;

use self::{
    web_socket::{WebSocketManager},
    app::{
        // routes_internal::InternalApp,
        App,
    },
    // data::{write_commands::WriteCommandRunnerHandle, DatabaseManager},
};
use crate::{media_backup::MediaBackupManager, perf::{PerfCounterManager, PerfCounterManagerData}};

/// Drop this when quit starts
pub type ServerQuitHandle = broadcast::Sender<()>;

/// Use resubscribe() for cloning.
pub type ServerQuitWatcher = broadcast::Receiver<()>;


#[async_trait]
pub trait BusinessLogic: Sized + Send + Sync + 'static {
    type AppState: Clone;

    /// Access prerformance counter list
    fn all_counters(&self) -> AllCounters { &[] }

    /// Create router for public API
    fn public_api_router(
        &self,
        _web_socket_manager: WebSocketManager,
        _state: &SimpleBackendAppState<Self::AppState>,
    ) -> Router { Router::new() }
    /// Create router for internal API
    fn internal_api_router(
        &self,
        _state: &SimpleBackendAppState<Self::AppState>,
    ) -> Router { Router::new() }

    /// Swagger UI which added to enabled in public and internal API router
    /// only if debug mode is enabled.
    fn create_swagger_ui(&self) -> Option<SwaggerUi> { None }

    /// Callback for doing something before server start
    ///
    /// For example databases can be opened here.
    async fn on_before_server_start(
        &mut self,
        media_backup_handle: MediaBackupHandle
    ) -> Self::AppState;

    /// Callback for doing something after server has been started
    async fn on_after_server_start(&mut self) {}

    /// Callback for doing something before server quit starts
    async fn on_before_server_quit(&mut self) {}

    /// Callback for doing something after server has quit
    ///
    /// For example databases can be closed here.
    async fn on_after_server_quit(self) {}
}

pub struct SimpleBackend<T: BusinessLogic> {
    logic: T,
    config: Arc<SimpleBackendConfig>,
}

impl <T: BusinessLogic> SimpleBackend<T> {
    pub fn new(logic: T, config: Arc<SimpleBackendConfig>) -> Self {
        Self {
            logic,
            config,
        }
    }

    pub async fn run(mut self) {
        if cfg!(debug_assertions) {
            let layer = console_subscriber::spawn();
            tracing_subscriber::registry()
                .with(layer)
                .with(
                    tracing_subscriber::fmt::layer()
                        .with_filter(EnvFilter::from_default_env())
                )
                .init();
        } else {
            tracing_subscriber::fmt::init();
        }

        info!(
            "Backend version: {}-{}",
            self.config.backend_semver_version(),
            self.config.backend_code_version()
        );

        if self.config.debug_mode() {
            warn!("Debug mode is enabled");
        }

        let mut terminate_signal = signal::unix::signal(SignalKind::terminate()).unwrap();

        // let mut litestream = None;
        // if let Some(litestream_config) = self.config.litestream() {
        //     let mut litestream_manager =
        //         LitestreamManager::new(self.config.clone(), litestream_config.clone());
        //     litestream_manager
        //         .start_litestream()
        //         .await
        //         .expect("Litestream start failed");
        //     litestream = Some(litestream_manager);
        // }

        let (server_quit_handle, server_quit_watcher) = broadcast::channel(1);

        let (media_backup_quit, media_backup_handle) =
            MediaBackupManager::new(self.config.clone(), server_quit_watcher.resubscribe());

        let perf_data = Arc::new(PerfCounterManagerData::new(self.logic.all_counters()));
        let perf_manager_quit_handle =
            PerfCounterManager::new(
                perf_data.clone(),
                self.config.clone(),
                server_quit_watcher.resubscribe()
            );

        let logic_app_state = self.logic.on_before_server_start(media_backup_handle).await;
        // let (database_manager, router_database_handle, router_database_write_handle) =
        //     DatabaseManager::new(
        //         self.config.data_dir().to_path_buf(),
        //         self.config.clone(),
        //         media_backup_handle,
        //     )
        //     .await
        //     .expect("Database init failed");

        let (ws_manager, mut ws_quit_ready) =
            WebSocketManager::new(server_quit_watcher.resubscribe());

        // let (write_cmd_runner_handle, write_cmd_waiter) =
        //     WriteCommandRunnerHandle::new(router_database_write_handle.clone(), &self.config);

        let app = App::new(
            self.config.clone(),
            perf_data,
            logic_app_state,
        )
        .await
        .expect("App init failed");

        let server_task = self
            .create_public_api_server_task(server_quit_watcher.resubscribe(), ws_manager, &app.state())
            .await;
        let internal_server_task = self
            .create_internal_api_server_task(server_quit_watcher.resubscribe(), &app.state())
            .await;

        self.logic.on_after_server_start().await;

        Self::wait_quit_signal(&mut terminate_signal).await;
        info!("Server quit signal received");

        self.logic.on_before_server_quit().await;

        info!("Server quit started");

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
        perf_manager_quit_handle.wait_quit().await;
        media_backup_quit.wait_quit().await;
        self.logic.on_after_server_quit().await;

        // if let Some(litestream) = litestream {
        //     match litestream.stop_litestream().await {
        //         Ok(()) => (),
        //         Err(e) => error!("Litestream stop failed: {:?}", e),
        //     }
        // }

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
        quit_notification: ServerQuitWatcher,
        web_socket_manager: WebSocketManager,
        app_state: &SimpleBackendAppState<T::AppState>,
    ) -> JoinHandle<()> {
        let router = {
            let router = self.logic.public_api_router(web_socket_manager, app_state);
            let router = if self.config.debug_mode() {
                let router = if let Some(swagger) = self.logic.create_swagger_ui() {
                    router.merge(swagger)
                } else {
                    router
                };

                router
                    .merge(self.logic.internal_api_router(app_state))
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
        quit_notification: ServerQuitWatcher,
        state: &SimpleBackendAppState<T::AppState>,
    ) -> JoinHandle<()> {
        let router = self.logic.internal_api_router(state);
        let router = if self.config.debug_mode() {
            if let Some(swagger) = self.logic.create_swagger_ui() {
                router.merge(swagger)
            } else {
                router
            }
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
}
