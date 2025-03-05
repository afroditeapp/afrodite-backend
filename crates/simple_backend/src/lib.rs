#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]
#![allow(
    async_fn_in_trait,
    clippy::single_match,
    clippy::while_let_loop,
    clippy::large_enum_variant
)]

pub mod app;
pub mod email;
pub mod event;
pub mod file_package;
pub mod image;
pub mod manager_client;
pub mod map;
pub mod perf;
pub mod sign_in_with;
pub mod utils;
pub mod web_socket;
pub mod tls;

use std::{convert::Infallible, future::IntoFuture, net::{Ipv4Addr, SocketAddr, SocketAddrV4}, pin::Pin, sync::Arc};

use app::{
    GetManagerApi, GetSimpleBackendConfig, GetTileMap, PerfCounterDataProvider, SignInWith,
    SimpleBackendAppState,
};
use axum::Router;
use futures::future::poll_fn;
use hyper::body::Incoming;
use hyper_util::rt::{TokioExecutor, TokioIo};
use manager_client::{ManagerApiClient, ManagerConnectionManager, ManagerEventHandler};
use perf::AllCounters;
use tls::{LetsEncryptAcmeSocketUtils, SimpleBackendTlsConfig, TlsManager};
use tokio_rustls_acme::AcmeAcceptor;
use simple_backend_config::SimpleBackendConfig;
use tokio::{
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
    LazyConfigAcceptor,
};
use tower::Service;
use tower_http::trace::TraceLayer;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};
use utoipa_swagger_ui::SwaggerUi;

use self::web_socket::WebSocketManager;
use crate::perf::{PerfMetricsManager, PerfMetricsManagerData};

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
        + ManagerEventHandler
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
        _disable_api_obfuscation: bool,
    ) -> Router {
        Router::new()
    }
    /// Create router for public bot API
    fn public_bot_api_router(
        &self,
        _web_socket_manager: WebSocketManager,
        _state: &Self::AppState,
    ) -> Router {
        Router::new()
    }
    /// Create router for bot API
    fn bot_api_router(
        &self,
        _web_socket_manager: WebSocketManager,
        _state: &Self::AppState,
    ) -> Router {
        Router::new()
    }
    /// Create router for internal API
    fn internal_api_router(
        &self,
        _state: &Self::AppState,
    ) -> Router {
        Router::new()
    }

    /// Swagger UI which added to enabled internal API router
    /// only if debug mode is enabled.
    fn create_swagger_ui(&self, _state: &Self::AppState) -> Option<SwaggerUi> {
        None
    }

    /// Callback for doing something before server start
    ///
    /// For example databases can be opened here.
    fn on_before_server_start(
        &mut self,
        simple_state: SimpleBackendAppState,
        quit_notification: ServerQuitWatcher,
    ) -> impl std::future::Future<Output = Self::AppState> + Send;

    /// Callback for doing something after server has been started
    fn on_after_server_start(&mut self) -> impl std::future::Future<Output = ()> + Send {
        async {}
    }

    /// Callback for doing something before server quit starts
    fn on_before_server_quit(&mut self) -> impl std::future::Future<Output = ()> + Send {
        async {}
    }

    /// Callback for doing something after server has quit
    ///
    /// For example databases can be closed here.
    fn on_after_server_quit(self) -> impl std::future::Future<Output = ()> + Send {
        async {}
    }
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
        let log_with_timestamp_layer = if self.config.log_timestamp() {
            Some(tracing_subscriber::fmt::layer().with_filter(EnvFilter::from_default_env()))
        } else {
            None
        };

        let log_without_timestamp_layer = if self.config.log_timestamp() {
            None
        } else {
            Some(
                tracing_subscriber::fmt::layer()
                    .without_time()
                    .with_filter(EnvFilter::from_default_env()),
            )
        };

        // tokio-console is disabled currently
        // let tokio_console_layer = if cfg!(debug_assertions) {
        //     Some(console_subscriber::spawn())
        // } else {
        //     None
        // }

        tracing_subscriber::registry()
            // .with(tokio_console_layer)
            .with(log_with_timestamp_layer)
            .with(log_without_timestamp_layer)
            .init();

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

        let (server_quit_handle, server_quit_watcher) = broadcast::channel(1);

        let perf_data = Arc::new(PerfMetricsManagerData::new(self.logic.all_counters()));
        let perf_manager_quit_handle =
            PerfMetricsManager::new_manager(perf_data.clone(), server_quit_watcher.resubscribe());

        let manager: Arc<ManagerApiClient> = ManagerApiClient::new(&self.config)
            .await
            .expect("Creating manager API client failed")
            .into();

        let simple_state = SimpleBackendAppState::new(self.config.clone(), perf_data, manager.clone())
            .await
            .expect("State builder init failed");

        let state = self
            .logic
            .on_before_server_start(
                simple_state,
                server_quit_watcher.resubscribe(),
            )
            .await;

        let manager_quit_handle = ManagerConnectionManager::new_manager(manager, state.clone(), server_quit_watcher.resubscribe())
            .await
            .expect("Manager connection manager init failed");

        let (ws_manager, mut ws_watcher) =
            WebSocketManager::new(server_quit_watcher.resubscribe()).await;

        let (mut tls_manager, tls_manager_quit_handle) = TlsManager::new(&self.config, server_quit_watcher.resubscribe()).await;

        let public_api_server_task =
            if let Some(addr) = self.config.socket().public_api {
                Some(
                    self.create_api_server_task(
                        server_quit_watcher.resubscribe(),
                        &mut tls_manager,
                        addr,
                        self.logic.public_api_router(ws_manager.clone(), &state, false),
                        "Public API",
                    )
                    .await
                )
            } else {
                None
            };
        let public_bot_api_server_task =
            if let Some(addr) = self.config.socket().public_bot_api {
                Some(
                    self.create_api_server_task(
                        server_quit_watcher.resubscribe(),
                        &mut tls_manager,
                        addr,
                        self.logic.public_bot_api_router(ws_manager.clone(), &state),
                        "Public bot API",
                    )
                    .await,
                )
            } else {
                None
            };
        let bot_api_server_task =
            if let Some(port) = self.config.socket().local_bot_api_port {
                Some(
                    self.create_bot_api_server_task(
                        server_quit_watcher.resubscribe(),
                        ws_manager.clone(),
                        &state,
                        port,
                    )
                    .await,
                )
            } else {
                None
            };
        let internal_api_server_task =
            if let Some(internal_api_addr) = self.config.socket().experimental_internal_api {
                Some(
                    self.create_api_server_task(
                        server_quit_watcher.resubscribe(),
                        &mut tls_manager,
                        internal_api_addr,
                        self.logic.public_bot_api_router(ws_manager.clone(), &state),
                        "Internal API",
                    )
                    .await,
                )

            } else {
                None
            };

        if public_api_server_task.is_none() &&
            public_bot_api_server_task.is_none() &&
            bot_api_server_task.is_none() &&
            internal_api_server_task.is_none() {
                warn!("No enabled APIs in config file");
            }

        self.logic.on_after_server_start().await;
        // Use println to make sure that this message is visible in logs.
        // Test mode backend starting requires this.
        println!("{SERVER_START_MESSAGE}");

        Self::wait_quit_signal(&mut terminate_signal).await;
        info!("Server quit signal received");

        self.logic.on_before_server_quit().await;

        info!("Server quit started");

        drop(server_quit_handle);

        // Wait until all tasks quit
        if let Some(task) = internal_api_server_task {
            task
                .await
                .expect("Internal API server task panic detected");
        }
        if let Some(task) = bot_api_server_task {
            task
                .await
                .expect("Bot API server task panic detected");
        }
        if let Some(task) = public_bot_api_server_task {
            task
                .await
                .expect("Public bot API server task panic detected");
        }
        if let Some(task) = public_api_server_task {
            task
                .await
                .expect("Public API server task panic detected");
        }

        tls_manager_quit_handle.wait_quit().await;
        ws_watcher.wait_for_quit().await;

        drop(state);
        manager_quit_handle.wait_quit().await;
        perf_manager_quit_handle.wait_quit().await;
        self.logic.on_after_server_quit().await;

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

    async fn create_api_server_task(
        &self,
        quit_notification: ServerQuitWatcher,
        tls_manager: &mut TlsManager,
        addr: SocketAddr,
        api_router: Router,
        server_name: &'static str,
    ) -> JoinHandle<()> {
        let router = {
            if self.config.debug_mode() {
                api_router.route_layer(TraceLayer::new_for_http())
            } else {
                api_router
            }
        };

        info!("{} is available on {}", server_name, addr);

        if let Some(tls_config) = tls_manager.config_mut() {
            self.create_server_task_with_tls(
                addr,
                router,
                tls_config,
                server_name,
                quit_notification,
            )
            .await
        } else {
            self.create_server_task_no_tls(router, addr, server_name, quit_notification)
                .await
        }
    }

    // Bot API for local bot clients. Only available from localhost.
    async fn create_bot_api_server_task(
        &self,
        quit_notification: ServerQuitWatcher,
        web_socket_manager: WebSocketManager,
        state: &T::AppState,
        localhost_port: u16,
    ) -> JoinHandle<()> {
        let router = self.logic.bot_api_router(web_socket_manager, state);
        let router = if self.config.debug_mode() {
            if let Some(swagger) = self.logic.create_swagger_ui(state) {
                router.merge(swagger)
            } else {
                router
            }
        } else {
            router
        };

        let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, localhost_port));
        info!("Bot API is available on {}", addr);
        self.create_server_task_no_tls(router, addr, "Bot API", quit_notification)
            .await
    }

    async fn create_server_task_with_tls(
        &self,
        addr: SocketAddr,
        router: Router,
        tls_config: &mut SimpleBackendTlsConfig,
        name_for_log_message: &'static str,
        mut quit_notification: ServerQuitWatcher,
    ) -> JoinHandle<()> {
        let (listener_for_router, acme_only_listener) =
            if let Some(utils) = tls_config.take_acme_utils() {
                if addr.port() == HTTPS_DEFAULT_PORT {
                    (ListenerAndConfig::with_acme_acceptor(utils, tls_config), None)
                } else {
                    let listener = TcpListener::bind(addr)
                        .await
                        .expect("Address not available");
                    (
                        ListenerAndConfig::new(listener, tls_config),
                        Some(ListenerAndConfig::with_acme_acceptor(utils, tls_config)),
                    )
                }
            } else {
                let listener = TcpListener::bind(addr)
                    .await
                    .expect("Address not available");
                (
                    ListenerAndConfig::new(listener, tls_config),
                    None,
                )
            };

        let (drop_after_connection, mut wait_all_connections) = mpsc::channel::<()>(1);

        let router_task = create_tls_listening_task(
            listener_for_router,
            Some(router),
            drop_after_connection.clone(),
            quit_notification.resubscribe(),
        );

        let acme_only_task = acme_only_listener.map(|acme_only_listener| {
            create_tls_listening_task(
                acme_only_listener,
                None,
                drop_after_connection.clone(),
                quit_notification.resubscribe(),
            )
        });

        tokio::spawn(async move {
            let _ = quit_notification.recv().await;

            match router_task.await {
                Ok(()) => (),
                Err(e) => {
                    error!("{name_for_log_message} router task server quit error: {e}");
                }
            }

            if let Some(task) = acme_only_task {
                match task.await {
                    Ok(()) => (),
                    Err(e) => {
                        error!("{name_for_log_message} ACME only task quit error: {e}");
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
            // There is no graceful shutdown for connections because
            // data writing is done in separate tasks.
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
}

fn create_tls_listening_task(
    config: ListenerAndConfig,
    router: Option<Router>,
    drop_after_connection: mpsc::Sender<()>,
    mut quit_notification: ServerQuitWatcher,
) -> JoinHandle<()> {
    let app_service = router.map(|v| v.into_make_service_with_connect_info::<SocketAddr>());
    let mut listener = config.listener;

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

            let config_clone = config.tls_config.clone();
            let acme_acceptor_clone = config.acme_acceptor.clone();
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
                    // There is no graceful shutdown for connections because
                    // data writing is done in separate tasks.
                    _ = quit_notification.recv() => {}
                    _ = handle_tls_related_tcp_stream(
                        tcp_stream,
                        config_clone,
                        acme_acceptor_clone,
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
    tls_config: Arc<ServerConfig>,
    acme_acceptor: Option<AcmeAcceptor>,
    app_service_with_connect_info: Option<
        axum::middleware::AddExtension<Router, axum::extract::ConnectInfo<SocketAddr>>,
    >,
) {
    let handshake = if let Some(acme_acceptor) = acme_acceptor {
        match acme_acceptor.accept(tcp_stream).await {
            Ok(None) => {
                info!("TLS-ALPN-01 challenge received");
                return;
            }
            Ok(Some(handshake)) => handshake,
            Err(_) => {
                // This error seems to be quite frequent when this port is on
                // public internet so do not log anything.
                SIMPLE_CONNECTION
                    .lets_encrypt_port_443_error
                    .incr();
                return;
            }
        }
    } else {
        let empty_acceptor: Acceptor = Default::default();
        match LazyConfigAcceptor::new(empty_acceptor, tcp_stream).await {
            Ok(v) => v,
            Err(_) => {
                // This error seems to be quite frequent when this port is on
                // public internet so do not log anything.
                SIMPLE_CONNECTION
                    .lets_encrypt_non_default_port_error
                    .incr();
                return;
            }
        }
    };

    if let Some(app_service_with_connect_info) = app_service_with_connect_info {
        match handshake.into_stream(tls_config).await {
            Ok(v) => handle_ready_tls_connection(v, app_service_with_connect_info).await,
            Err(e) => error!("Into TlsStream failed: {}", e),
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

struct ListenerAndConfig {
    listener: TcpListener,
    tls_config: Arc<ServerConfig>,
    acme_acceptor: Option<AcmeAcceptor>,
}

impl ListenerAndConfig {
    fn with_acme_acceptor(
        utils: LetsEncryptAcmeSocketUtils,
        config: &SimpleBackendTlsConfig,
    ) -> Self {
        Self {
            listener: utils.https_listener,
            tls_config: config.tls_config(),
            acme_acceptor: Some(utils.acceptor),
        }
    }

    fn new(
        listener: TcpListener,
        config: &SimpleBackendTlsConfig
    ) -> Self {
        Self {
            listener,
            tls_config: config.tls_config(),
            acme_acceptor: None,
        }
    }
}

create_counters!(
    SimpleConnectionCounters,
    SIMPLE_CONNECTION,
    SIMPLE_CONNECTION_COUNTERS_LIST,
    lets_encrypt_port_443_error,
    lets_encrypt_non_default_port_error,
);
