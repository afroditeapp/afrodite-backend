#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]
#![allow(
    clippy::single_match,
    clippy::while_let_loop,
    clippy::large_enum_variant
)]

pub mod app;
pub mod event;
pub mod image;
pub mod litestream;
pub mod manager_client;
pub mod map;
pub mod media_backup;
pub mod perf;
pub mod sign_in_with;
pub mod utils;
pub mod web_socket;

use std::{convert::Infallible, future::IntoFuture, net::SocketAddr, pin::Pin, sync::Arc};

use app::{
    GetManagerApi, GetSimpleBackendConfig, GetTileMap, PerfCounterDataProvider, SignInWith,
    SimpleBackendAppState,
};
use axum::Router;
use futures::{future::poll_fn, StreamExt};
use hyper::body::Incoming;
use hyper_util::rt::{TokioExecutor, TokioIo};
use media_backup::MediaBackupHandle;
use perf::AllCounters;
use rustls_acme::{caches::DirCache, is_tls_alpn_challenge, AcmeConfig};
use simple_backend_config::{file::LetsEncryptConfig, SimpleBackendConfig};
use tokio::{
    io::AsyncWriteExt,
    net::TcpListener,
    signal::{
        self,
        unix::{Signal, SignalKind},
    },
    sync::{broadcast, mpsc},
    task::JoinHandle,
};
use tokio_rustls::{
    rustls::{server::Acceptor, ServerConfig},
    LazyConfigAcceptor, TlsAcceptor,
};
use tower::Service;
use tower_http::trace::TraceLayer;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};
use utoipa_swagger_ui::SwaggerUi;

use self::web_socket::WebSocketManager;
use crate::{
    media_backup::MediaBackupManager,
    perf::{PerfCounterManager, PerfCounterManagerData},
};

pub const HTTPS_DEFAULT_PORT: u16 = 443;
pub const SERVER_START_MESSAGE: &str = "Server start complete";

/// Drop this when quit starts
pub type ServerQuitHandle = broadcast::Sender<()>;

/// Use resubscribe() for cloning.
pub type ServerQuitWatcher = broadcast::Receiver<()>;

pub trait BusinessLogic: Sized + Send + Sync + 'static {
    type AppState: SignInWith
        + GetManagerApi
        + GetSimpleBackendConfig
        + GetTileMap
        + PerfCounterDataProvider
        + Send
        + Sync
        + Clone
        + 'static;

    /// Access prerformance counter list
    fn all_counters(&self) -> AllCounters {
        &[]
    }

    /// Create router for public API
    fn public_api_router(
        &self,
        _web_socket_manager: WebSocketManager,
        _state: &Self::AppState,
    ) -> Router {
        Router::new()
    }
    /// Create router for internal API
    fn internal_api_router(&self, _state: &Self::AppState) -> Router {
        Router::new()
    }

    /// Swagger UI which added to enabled internal API router
    /// only if debug mode is enabled.
    fn create_swagger_ui(&self) -> Option<SwaggerUi> {
        None
    }

    /// Callback for doing something before server start
    ///
    /// For example databases can be opened here.
    fn on_before_server_start(
        &mut self,
        simple_state: SimpleBackendAppState,
        media_backup_handle: MediaBackupHandle,
        quit_notification: ServerQuitWatcher,
    ) -> impl std::future::Future<Output = Self::AppState> + Send;

    /// Callback for doing something after server has been started
    fn on_after_server_start(&mut self) -> impl std::future::Future<Output = ()> + Send { async {} }

    /// Callback for doing something before server quit starts
    fn on_before_server_quit(&mut self) -> impl std::future::Future<Output = ()> + Send { async {} }

    /// Callback for doing something after server has quit
    ///
    /// For example databases can be closed here.
    fn on_after_server_quit(self) -> impl std::future::Future<Output = ()> + Send { async {} }
}

pub struct SimpleBackend<T: BusinessLogic> {
    logic: T,
    config: Arc<SimpleBackendConfig>,
}

impl<T: BusinessLogic> SimpleBackend<T> {
    pub fn new(logic: T, config: Arc<SimpleBackendConfig>) -> Self {
        Self { logic, config }
    }

    pub async fn run(mut self) {
        if cfg!(debug_assertions) {
            // tokio-console is disabled currently
            //let layer = console_subscriber::spawn();
            tracing_subscriber::registry()
                // .with(layer)
                .with(tracing_subscriber::fmt::layer().with_filter(EnvFilter::from_default_env()))
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

        if let Some(lets_encrypt) = self.config.lets_encrypt_config() {
            if !lets_encrypt.production_servers {
                warn!("Let's Encrypt is configured to use staging environment");
            }
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
            MediaBackupManager::new_manager(self.config.clone(), server_quit_watcher.resubscribe());

        let perf_data = Arc::new(PerfCounterManagerData::new(self.logic.all_counters()));
        let perf_manager_quit_handle =
            PerfCounterManager::new_manager(perf_data.clone(), server_quit_watcher.resubscribe());

        let simple_state = SimpleBackendAppState::new(self.config.clone(), perf_data)
            .expect("State builder init failed");

        let state = self
            .logic
            .on_before_server_start(
                simple_state,
                media_backup_handle,
                server_quit_watcher.resubscribe(),
            )
            .await;

        let (ws_manager, mut ws_watcher) =
            WebSocketManager::new(server_quit_watcher.resubscribe()).await;

        let server_task = self
            .create_public_api_server_task(server_quit_watcher.resubscribe(), ws_manager, &state)
            .await;
        let internal_server_task =
            if let Some(internal_api_addr) = self.config.socket().internal_api {
                Some(
                    self.create_internal_api_server_task(
                        server_quit_watcher.resubscribe(),
                        &state,
                        internal_api_addr,
                    )
                    .await,
                )
            } else {
                None
            };

        self.logic.on_after_server_start().await;
        // Use println to make sure that this message is visible in logs.
        println!("{SERVER_START_MESSAGE}");

        Self::wait_quit_signal(&mut terminate_signal).await;
        info!("Server quit signal received");

        self.logic.on_before_server_quit().await;

        info!("Server quit started");

        drop(server_quit_handle);

        // Wait until all tasks quit
        server_task
            .await
            .expect("Public API server task panic detected");
        if let Some(internal_server_task) = internal_server_task {
            internal_server_task
                .await
                .expect("Internal API server task panic detected");
        }

        ws_watcher.wait_for_quit().await;

        drop(state);
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
    async fn create_public_api_server_task(
        &self,
        quit_notification: ServerQuitWatcher,
        web_socket_manager: WebSocketManager,
        app_state: &T::AppState,
    ) -> JoinHandle<()> {
        let router = {
            let router = self.logic.public_api_router(web_socket_manager, app_state);
            if self.config.debug_mode() {
                router.route_layer(TraceLayer::new_for_http())
            } else {
                router
            }
        };

        let addr = self.config.socket().public_api;
        info!("Public API is available on {}", addr);

        if let Some(tls_config) = self.config.public_api_tls_config() {
            self.create_server_task_with_tls(
                addr,
                router,
                SimpleBackendTlsConfig::ManualSertificates(tls_config.clone()),
                quit_notification,
            )
            .await
        } else if let Some(lets_encrypt) = self.config.lets_encrypt_config() {
            self.create_server_task_with_tls(
                addr,
                router,
                SimpleBackendTlsConfig::LetsEncrypt(lets_encrypt.clone()),
                quit_notification,
            )
            .await
        } else {
            self.create_server_task_no_tls(router, addr, "Public API", quit_notification)
                .await
        }
    }

    async fn create_server_task_with_tls(
        &self,
        addr: SocketAddr,
        router: Router,
        tls_config: SimpleBackendTlsConfig,
        mut quit_notification: ServerQuitWatcher,
    ) -> JoinHandle<()> {
        let public_api_listener = TcpListener::bind(addr)
            .await
            .expect("Address not available");

        let https_socket_listener =
            if addr.port() != HTTPS_DEFAULT_PORT && tls_config.is_lets_encrypt() {
                let mut https_addr = addr;
                https_addr.set_port(HTTPS_DEFAULT_PORT);
                info!(
                    "HTTPS socket for Let's Encrypt ACME challenge is available on {}",
                    https_addr
                );
                Some(
                    TcpListener::bind(https_addr)
                        .await
                        .expect("Address not available"),
                )
            } else {
                None
            };

        let (drop_after_connection, mut wait_all_connections) = mpsc::channel::<()>(1);

        let tls_config = tls_config.start_acme_task_if_needed(
            quit_notification.resubscribe(),
            drop_after_connection.clone(),
        );

        let public_api_tls_config =
            if addr.port() != HTTPS_DEFAULT_PORT && tls_config.is_lets_encrypt() {
                tls_config.clone().remove_challenge_config()
            } else {
                tls_config.clone()
            };

        let public_api_server = create_tls_listening_task(
            public_api_listener,
            Some(router),
            public_api_tls_config,
            drop_after_connection.clone(),
            quit_notification.resubscribe(),
        );

        let https_socket_server = https_socket_listener.map(|https_socket_listener| {
            create_tls_listening_task(
                https_socket_listener,
                None,
                tls_config,
                drop_after_connection.clone(),
                quit_notification.resubscribe(),
            )
        });

        tokio::spawn(async move {
            let _ = quit_notification.recv().await;

            match public_api_server.await {
                Ok(()) => (),
                Err(e) => {
                    error!("Public API server quit error: {}", e);
                }
            }

            if let Some(https_socket_server) = https_socket_server {
                match https_socket_server.await {
                    Ok(()) => (),
                    Err(e) => {
                        error!("HTTPS socket server quit error: {}", e);
                    }
                }
            }

            loop {
                match wait_all_connections.recv().await {
                    Some(()) => (),
                    None => break,
                }
            }
        })
    }

    async fn create_server_task_no_tls(
        &self,
        router: Router,
        addr: SocketAddr,
        name_for_log_message: &'static str,
        mut quit_notification: ServerQuitWatcher,
    ) -> JoinHandle<()> {
        let normal_api_server = {
            let listener = tokio::net::TcpListener::bind(addr)
                .await
                .expect("Address not available");
            axum::serve(
                listener,
                router.into_make_service_with_connect_info::<SocketAddr>(),
            )
        };

        tokio::spawn(async move {
            // There is no graceful shutdown because data writing is done in
            // separate tasks.
            tokio::select! {
                _ = quit_notification.recv() => {
                    info!("{name_for_log_message} server quit signal received");
                }
                result = normal_api_server.into_future() => {
                    match result {
                        Ok(()) => {
                            info!("{name_for_log_message} server quit by itself");
                        }
                        Err(e) => {
                            error!("{name_for_log_message} server quit by error: {}", e);
                        }
                    }
                }
            }
        })
    }

    // Internal server to server API. This must be only LAN accessible.
    async fn create_internal_api_server_task(
        &self,
        quit_notification: ServerQuitWatcher,
        state: &T::AppState,
        addr: SocketAddr,
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

        info!("Internal API is available on {}", addr);
        if let Some(tls_config) = self.config.internal_api_tls_config() {
            self.create_server_task_with_tls(
                addr,
                router,
                SimpleBackendTlsConfig::ManualSertificates(tls_config.clone()),
                quit_notification,
            )
            .await
        } else {
            self.create_server_task_no_tls(router, addr, "Internal API", quit_notification)
                .await
        }
    }
}

fn create_tls_listening_task(
    mut listener: TcpListener,
    router: Option<Router>,
    tls_config: SimpleBackendTlsConfigAcmeTaskRunning,
    drop_after_connection: mpsc::Sender<()>,
    mut quit_notification: ServerQuitWatcher,
) -> JoinHandle<()> {
    let app_service = router.map(|v| v.into_make_service_with_connect_info::<SocketAddr>());

    tokio::spawn(async move {
        loop {
            let next_addr_stream = poll_fn(|cx| Pin::new(&mut listener).poll_accept(cx));

            let (tcp_stream, addr) = tokio::select! {
                _ = quit_notification.recv() => {
                    break;
                }
                addr = next_addr_stream => {
                    match addr {
                        Ok(stream_and_addr) => {
                            stream_and_addr
                        }
                        Err(e) => {
                            // TODO: Can this happen if there is no more
                            //       file descriptors available?
                            error!("Address stream error {e}");
                            return;
                        }
                    }
                }
            };

            let config_clone = tls_config.clone();
            let app_service_with_connect_info: Option<
                axum::middleware::AddExtension<Router, axum::extract::ConnectInfo<SocketAddr>>,
            > = if let Some(mut app_service) = app_service.clone() {
                Some(unwrap_infallible_result(app_service.call(addr).await))
            } else {
                None
            };

            let mut quit_notification = quit_notification.resubscribe();
            let drop_on_quit = drop_after_connection.clone();
            tokio::spawn(async move {
                tokio::select! {
                    _ = quit_notification.recv() => {} // Graceful shutdown for connections?
                    _ = handle_tls_related_tcp_stream(
                        tcp_stream,
                        config_clone,
                        app_service_with_connect_info,
                    ) => (),
                }

                drop(drop_on_quit);
            });
        }
        drop(drop_after_connection);
    })
}

async fn handle_tls_related_tcp_stream(
    tcp_stream: tokio::net::TcpStream,
    config: SimpleBackendTlsConfigAcmeTaskRunning,
    app_service_with_connect_info: Option<
        axum::middleware::AddExtension<Router, axum::extract::ConnectInfo<SocketAddr>>,
    >,
) {
    match config {
        SimpleBackendTlsConfigAcmeTaskRunning::ManualSertificates(manual) => {
            if let Some(app_service_with_connect_info) = app_service_with_connect_info {
                let acceptor = TlsAcceptor::from(manual);
                let connection = acceptor.accept(tcp_stream).await;
                match connection {
                    Ok(tls_connection) => {
                        handle_ready_tls_connection(tls_connection, app_service_with_connect_info)
                            .await
                    }
                    Err(_) => {} // TLS handshake failed
                }
            }
        }
        SimpleBackendTlsConfigAcmeTaskRunning::LetsEncrypt {
            challenge_config,
            default_config,
        } => {
            handle_lets_encrypt_related_tcp_stream(
                tcp_stream,
                challenge_config,
                default_config,
                app_service_with_connect_info,
            )
            .await;
        }
    }
}

async fn handle_lets_encrypt_related_tcp_stream(
    tcp_stream: tokio::net::TcpStream,
    challenge_config: Option<Arc<ServerConfig>>,
    default_config: Arc<ServerConfig>,
    app_service_with_connect_info: Option<
        axum::middleware::AddExtension<Router, axum::extract::ConnectInfo<SocketAddr>>,
    >,
) {
    let empty_acceptor: Acceptor = Default::default();
    let start_handshake = match LazyConfigAcceptor::new(empty_acceptor, tcp_stream).await {
        Ok(v) => v,
        Err(_) => {
            // This error seems to be quite frequent when this port is on
            // public internet so do not log anything.
            CONNECTION.lets_encrypt_port_start_handshake_failed.incr();
            return;
        }
    };

    if let Some(challenge_config) = challenge_config {
        if is_tls_alpn_challenge(&start_handshake.client_hello()) {
            info!("TLS-ALPN-01 challenge received");
            match start_handshake.into_stream(challenge_config).await {
                Ok(mut v) => match v.shutdown().await {
                    Ok(()) => (),
                    Err(e) => error!("Challenge connection shutdown failed: {}", e),
                },
                Err(e) => error!("Challenge connection failed: {}", e),
            }
            return;
        }
    }

    if let Some(app_service_with_connect_info) = app_service_with_connect_info {
        match start_handshake.into_stream(default_config).await {
            Ok(v) => handle_ready_tls_connection(v, app_service_with_connect_info).await,
            Err(e) => error!("Normal connection failed: {}", e),
        }
    }
}

async fn handle_ready_tls_connection<
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Send + std::marker::Unpin + 'static,
>(
    tls_connection: T,
    app_service_with_connect_info: axum::middleware::AddExtension<
        Router,
        axum::extract::ConnectInfo<SocketAddr>,
    >,
) {
    let data_stream = TokioIo::new(tls_connection);

    let hyper_service = hyper::service::service_fn(move |request: hyper::Request<Incoming>| {
        app_service_with_connect_info.clone().call(request)
    });

    let connection_serving_result =
        hyper_util::server::conn::auto::Builder::new(TokioExecutor::new())
            .serve_connection_with_upgrades(data_stream, hyper_service)
            .await;

    match connection_serving_result {
        Ok(()) => {}
        Err(e) => {
            // TODO: Remove to avoid log spam?
            error!("Ready TLS connection serving error: {}", e);
        }
    }
}

fn unwrap_infallible_result<T>(r: Result<T, Infallible>) -> T {
    match r {
        Ok(v) => v,
        Err(i) => match i {},
    }
}

#[derive(Debug, Clone)]
enum SimpleBackendTlsConfig {
    ManualSertificates(Arc<ServerConfig>),
    LetsEncrypt(LetsEncryptConfig),
}

impl SimpleBackendTlsConfig {
    fn is_lets_encrypt(&self) -> bool {
        matches!(self, SimpleBackendTlsConfig::LetsEncrypt(_))
    }

    fn start_acme_task_if_needed(
        self,
        mut quit_notification: ServerQuitWatcher,
        drop_on_quit: mpsc::Sender<()>,
    ) -> SimpleBackendTlsConfigAcmeTaskRunning {
        match self {
            SimpleBackendTlsConfig::ManualSertificates(manual) => {
                SimpleBackendTlsConfigAcmeTaskRunning::ManualSertificates(manual)
            }
            SimpleBackendTlsConfig::LetsEncrypt(lets_encrypt) => {
                let mut state = AcmeConfig::new(lets_encrypt.domains)
                    .contact([format!("mailto:{}", lets_encrypt.email)])
                    .cache(DirCache::new(lets_encrypt.cache_dir))
                    .directory_lets_encrypt(lets_encrypt.production_servers)
                    .state();
                let challenge_config = state.challenge_rustls_config();
                let default_config = state.default_rustls_config();

                tokio::spawn(async move {
                    loop {
                        tokio::select! {
                            _ = quit_notification.recv() => break,
                            next_state = state.next() => {
                                match next_state {
                                    None => break,
                                    Some(Ok(value)) => info!("ACME state updated: {:?}", value),
                                    Some(Err(e)) => error!("ACME state error: {}", e),
                                }
                            }
                        }
                    }
                    drop(drop_on_quit);
                });

                SimpleBackendTlsConfigAcmeTaskRunning::LetsEncrypt {
                    challenge_config: Some(challenge_config),
                    default_config,
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
enum SimpleBackendTlsConfigAcmeTaskRunning {
    ManualSertificates(Arc<ServerConfig>),
    LetsEncrypt {
        challenge_config: Option<Arc<ServerConfig>>,
        default_config: Arc<ServerConfig>,
    },
}

impl SimpleBackendTlsConfigAcmeTaskRunning {
    fn is_lets_encrypt(&self) -> bool {
        matches!(
            self,
            SimpleBackendTlsConfigAcmeTaskRunning::LetsEncrypt { .. }
        )
    }

    fn remove_challenge_config(self) -> SimpleBackendTlsConfigAcmeTaskRunning {
        match self {
            SimpleBackendTlsConfigAcmeTaskRunning::ManualSertificates(manual) => {
                SimpleBackendTlsConfigAcmeTaskRunning::ManualSertificates(manual)
            }
            SimpleBackendTlsConfigAcmeTaskRunning::LetsEncrypt {
                challenge_config: _,
                default_config,
            } => SimpleBackendTlsConfigAcmeTaskRunning::LetsEncrypt {
                challenge_config: None,
                default_config,
            },
        }
    }
}

create_counters!(
    ConnectionCounters,
    CONNECTION,
    CONNECTION_COUNTERS_LIST,
    lets_encrypt_port_start_handshake_failed,
);
