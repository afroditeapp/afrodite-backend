use std::{
    convert::Infallible,
    future::IntoFuture,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    pin::Pin,
    sync::Arc,
    time::Duration,
};

use axum::Router;
use futures::future::poll_fn;
use hyper::body::Incoming;
use hyper_util::rt::{TokioExecutor, TokioIo};
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
use tower::Service;
use tower_http::trace::TraceLayer;
use tracing::{error, info, log::warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    api::ApiDoc,
    config::Config,
    server::{
        app::App, backend_controller::BackendController, client::ApiClient,
        mount::MountManager, state::StateStorage,
    },
};

pub mod app;
pub mod backend_controller;
pub mod client;
pub mod info;
pub mod mount;
pub mod reboot;
pub mod state;
pub mod update;

/// Drop this when quit starts
pub type ServerQuitHandle = broadcast::Sender<()>;

/// Use resubscribe() for cloning.
pub type ServerQuitWatcher = broadcast::Receiver<()>;

pub struct AppServer {
    config: Arc<Config>,
}

impl AppServer {
    pub fn new(config: Config) -> Self {
        Self {
            config: config.into(),
        }
    }

    pub async fn run(self) {
        let log_with_timestamp_layer = if self.config.log_timestamp() {
            Some(tracing_subscriber::fmt::layer().with_filter(EnvFilter::from_default_env()))
        } else {
            None
        };

        let log_without_timestamp_layer = if self.config.log_timestamp() {
            None
        } else {
            Some(tracing_subscriber::fmt::layer().without_time().with_filter(EnvFilter::from_default_env()))
        };

        tracing_subscriber::registry()
            .with(log_with_timestamp_layer)
            .with(log_without_timestamp_layer)
            .init();

        info!(
            "Manager version: {}-{}",
            self.config.backend_semver_version(),
            self.config.backend_code_version()
        );

        if self.config.debug_mode() {
            warn!("Debug mode is enabled");
        }

        let (server_quit_handle, server_quit_watcher) = broadcast::channel(1);
        let mut terminate_signal = signal::unix::signal(SignalKind::terminate()).unwrap();

        // Create API client

        let api_client: Arc<ApiClient> = ApiClient::new(&self.config).unwrap().into();
        let state: Arc<StateStorage> = StateStorage::new().into();

        // Start reboot manager

        let (reboot_manager_quit_handle, reboot_manager_handle) = reboot::RebootManager::new_manager(
            self.config.clone(),
            api_client.clone(),
            state.clone(),
            server_quit_watcher.resubscribe(),
        );

        // Start update manager

        let (update_manager_quit_handle, update_manager_handle) = update::UpdateManager::new_manager(
            self.config.clone(),
            server_quit_watcher.resubscribe(),
            api_client.clone(),
            reboot_manager_handle,
        );

        // Create app

        let mut app = App::new(
            self.config.clone(),
            api_client,
            update_manager_handle.into(),
        )
        .await;

        // Start API server

        let (server_task1, server_task2) = self
            .create_public_api_server_task(&mut app, server_quit_watcher.resubscribe())
            .await;

        // Mount encrypted storage if needed

        let mount_manager = MountManager::new(self.config.clone(), app.state(), state.clone());

        if let Some(encryption_key_provider) = self.config.secure_storage_config() {
            loop {
                match mount_manager.mount_if_needed(encryption_key_provider).await {
                    Ok(()) => {
                        break;
                    }
                    Err(e) => {
                        warn!("Failed to mount encrypted storage. Error: {:?}", e);
                    }
                }

                info!("Retrying after one hour");

                tokio::select! {
                    _ = Self::wait_quit_signal(&mut terminate_signal) => {
                        return;
                    }
                    _ = tokio::time::sleep(Duration::from_secs(60*60)) => {} // check again in an hour
                }
            }
        } else {
            info!("Encrypted storage is disabled");
        }

        // Try to create storage directory if it doesn't exist
        if !self.config.storage_dir().exists() {
            match tokio::fs::create_dir(self.config.storage_dir()).await {
                Ok(()) => {
                    info!("Storage directory created");
                }
                Err(e) => {
                    error!("Failed to create storage directory. Error: {:?}", e);
                }
            }
        }

        // Start backend if it is installed

        if let Some(update_config) = self.config.software_update_provider() {
            if update_config.backend_install_location.exists() {
                info!("Starting backend");
                match BackendController::new(&self.config).start_backend().await {
                    Ok(()) => {
                        info!("Backend started");
                    }
                    Err(e) => {
                        warn!("Backend start failed. Error: {:?}", e);
                    }
                }
            } else {
                warn!("Backend starting failed. Backend is not installed");
            }
        }

        // Wait until quit signal
        Self::wait_quit_signal(&mut terminate_signal).await;

        // Quit started

        info!("Manager quit started");

        drop(server_quit_handle);

        // Wait until all tasks quit
        server_task1
            .await
            .expect("Manager API server task panic detected");

        if let Some(server_task2) = server_task2 {
            server_task2
                .await
                .expect("Second Manager API server task panic detected");
        }

        reboot_manager_quit_handle.wait_quit().await;
        update_manager_quit_handle.wait_quit().await;

        if self.config.software_update_provider().is_some() {
            info!("Stopping backend");
            match BackendController::new(&self.config).stop_backend().await {
                Ok(()) => {
                    info!("Backend stopped");
                }
                Err(e) => {
                    warn!("Backend stopping failed. Error: {:?}", e);
                }
            }
        }

        drop(app);

        if let Some(config) = self.config.secure_storage_config() {
            match mount_manager.unmount_if_needed(config).await {
                Ok(()) => {
                    info!("Secure storage is now unmounted");
                }
                Err(e) => {
                    warn!("Failed to unmount secure storage. Error: {:?}", e);
                }
            }
        }

        info!("Manager quit done");
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
    ) -> (JoinHandle<()>, Option<JoinHandle<()>>) {
        let router = {
            let router = self.create_public_router(app);
            let router = if self.config.debug_mode() {
                router.merge(Self::create_swagger_ui())
            } else {
                router
            };
            if self.config.debug_mode() {
                router.route_layer(TraceLayer::new_for_http())
            } else {
                router
            }
        };

        let addr = self.config.socket().public_api;
        info!("Public API is available on {}", addr);

        let join_handle = if let Some(tls_config) = self.config.public_api_tls_config() {
            self.create_server_task_with_tls(
                addr,
                router.clone(),
                tls_config.clone(),
                quit_notification.resubscribe(),
            )
            .await
        } else {
            self.create_server_task_no_tls(
                router.clone(),
                addr,
                "Public API",
                quit_notification.resubscribe(),
            )
            .await
        };

        let second_join_handle =
            if let Some(port) = self.config.socket().second_public_api_localhost_only_port {
                let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
                info!("Public API is available also on {}", addr);
                let handle = self
                    .create_server_task_no_tls(router, addr, "Second public API", quit_notification)
                    .await;
                Some(handle)
            } else {
                None
            };

        (join_handle, second_join_handle)
    }

    pub async fn create_server_task_with_tls(
        &self,
        addr: SocketAddr,
        router: Router,
        tls_config: Arc<ServerConfig>,
        mut quit_notification: ServerQuitWatcher,
    ) -> JoinHandle<()> {
        let mut listener = TcpListener::bind(addr)
            .await
            .expect("Address not available");
        let acceptor = TlsAcceptor::from(tls_config);
        let app_service = router.into_make_service_with_connect_info::<SocketAddr>();

        tokio::spawn(async move {
            let (drop_after_connection, mut wait_all_connections) = mpsc::channel::<()>(1);

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

                let acceptor = acceptor.clone();
                let app_service_with_connect_info =
                    unwrap_infallible_result(app_service.clone().call(addr).await);

                let mut quit_notification = quit_notification.resubscribe();
                let drop_on_quit = drop_after_connection.clone();
                tokio::spawn(async move {
                    tokio::select! {
                        _ = quit_notification.recv() => {} // Graceful shutdown for connections?
                        connection = acceptor.accept(tcp_stream) => {
                            match connection {
                                Ok(tls_connection) => {
                                    let data_stream = TokioIo::new(tls_connection);

                                    let hyper_service = hyper::service::service_fn(move |request: hyper::Request<Incoming>| {
                                        app_service_with_connect_info.clone().call(request)
                                    });

                                    let connection_serving_result =
                                        hyper_util::server::conn::auto::Builder::new(TokioExecutor::new())
                                            .serve_connection_with_upgrades(data_stream, hyper_service)
                                            .await;

                                    match connection_serving_result {
                                        Ok(()) => {},
                                        Err(e) => {
                                            // TODO: Remove to avoid log spam?
                                            error!("Connection serving error: {}", e);
                                        }
                                    }
                                }
                                Err(_) => {}, // TLS handshake failed
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

    pub async fn create_server_task_no_tls(
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
            // TODO: Is graceful shutdown needed?
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

    pub fn create_public_router(&self, app: &mut App) -> Router {
        app.create_manager_server_router()
    }

    pub fn create_swagger_ui() -> SwaggerUi {
        SwaggerUi::new("/swagger-ui").url("/api-doc/app_api.json", ApiDoc::openapi())
    }
}

fn unwrap_infallible_result<T>(r: Result<T, Infallible>) -> T {
    match r {
        Ok(v) => v,
        Err(i) => match i {},
    }
}
