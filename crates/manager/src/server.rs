use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    pin::Pin,
    sync::Arc,
    time::Duration,
};

use app::S;
use futures::future::poll_fn;
use link::{backup::{server::BackupLinkManagerServer, target::BackupLinkManagerTarget}, json_rpc::{client::JsonRcpLinkManagerClient, server::JsonRcpLinkManagerServer}};
use manager_config::Config;
use scheduled_task::ScheduledTaskManager;
use task::TaskManager;
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
use tracing::{error, info, log::warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};
use update::UpdateManager;

use crate::{
    api::server::handle_connection_to_server,
    server::{
        app::App, backend_controller::BackendController,
        mount::MountManager, state::MountStateStorage,
    },
};

pub mod app;
pub mod backend_controller;
pub mod backend_events;
pub mod client;
pub mod info;
pub mod mount;
pub mod task;
pub mod scheduled_task;
pub mod reboot;
pub mod state;
pub mod update;
pub mod link;

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

        let state: Arc<MountStateStorage> = MountStateStorage::new().into();
        let (task_manager_handle, task_manager_internal_state) =
            TaskManager::new_channel();
        let (scheduled_task_manager_handle, scheduled_task_manager_internal_state) =
            ScheduledTaskManager::new_channel();
        let (update_manager_handle, update_manager_internal_state) =
            UpdateManager::new_channel();
        let (json_rpc_link_manager_server_handle, json_rpc_link_manager_server_internal_state) =
            JsonRcpLinkManagerServer::new_channel();
        let (backup_link_manager_server_handle, backup_link_manager_server_internal_state) =
            BackupLinkManagerServer::new_channel();

        let mut app = App::new(
            self.config.clone(),
            update_manager_handle.into(),
            task_manager_handle.into(),
            scheduled_task_manager_handle.into(),
            json_rpc_link_manager_server_handle.into(),
            backup_link_manager_server_handle.into(),
        )
        .await;

        // Start task manager

        let task_manager_quit_handle = task::TaskManager::new_manager(
            task_manager_internal_state,
            app.state(),
            state.clone(),
            server_quit_watcher.resubscribe(),
        );

        // Start scheduled task manager

        let scheduled_task_manager_quit_handle = scheduled_task::ScheduledTaskManager::new_manager(
            scheduled_task_manager_internal_state,
            app.state(),
            server_quit_watcher.resubscribe(),
        );

        let reboot_manager_quit_handle = reboot::RebootManager::new_manager(
            app.state(),
            server_quit_watcher.resubscribe(),
        );

        // Start update manager

        let update_manager_quit_handle = update::UpdateManager::new_manager(
            update_manager_internal_state,
            app.state(),
            server_quit_watcher.resubscribe(),
        );

        // Start JSON RPC link manager server logic

        let json_rpc_link_manager_server_quit_handle = JsonRcpLinkManagerServer::new_manager(
            json_rpc_link_manager_server_internal_state,
            server_quit_watcher.resubscribe(),
        );

        // Start JSON RPC link manager client logic

        let json_rpc_link_manager_client_quit_handle = JsonRcpLinkManagerClient::new_manager(
            app.state(),
            server_quit_watcher.resubscribe(),
        );

        // Start backup link manager server logic

        let backup_link_manager_server_quit_handle = BackupLinkManagerServer::new_manager(
            backup_link_manager_server_internal_state,
            server_quit_watcher.resubscribe(),
        );

        // Start backup target client logic

        let backup_target_quit_handle = BackupLinkManagerTarget::new_manager(
            app.state(),
            server_quit_watcher.resubscribe(),
        );

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

        // TODO(prod): Add specific config for backend automatic starting

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
        if let Some(server_task1) = server_task1 {
            server_task1
                .await
                .expect("Manager API server task panic detected");
        }

        if let Some(server_task2) = server_task2 {
            server_task2
                .await
                .expect("Second Manager API server task panic detected");
        }

        backup_target_quit_handle.wait_quit().await;
        backup_link_manager_server_quit_handle.wait_quit().await;
        json_rpc_link_manager_client_quit_handle.wait_quit().await;
        json_rpc_link_manager_server_quit_handle.wait_quit().await;
        update_manager_quit_handle.wait_quit().await;
        reboot_manager_quit_handle.wait_quit().await;
        scheduled_task_manager_quit_handle.wait_quit().await;
        task_manager_quit_handle.wait_quit().await;

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
    ) -> (Option<JoinHandle<()>>, Option<JoinHandle<()>>) {
        let join_handle = if let Some(addr) = self.config.socket().public_api {
            info!("Public API is available on {}", addr);

            let handle = if let Some(tls_config) = self.config.public_api_tls_config() {
                self.create_server_task_with_tls(
                    app.state(),
                    addr,
                    tls_config.clone(),
                    quit_notification.resubscribe(),
                )
                .await
            } else {
                self.create_server_task_no_tls(
                    app.state(),
                    addr,
                    "Public API",
                    quit_notification.resubscribe(),
                )
                .await
            };

            Some(handle)
        } else {
            None
        };


        let second_join_handle =
            if let Some(port) = self.config.socket().second_public_api_localhost_only_port {
                let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
                info!("Public API is available also on {}", addr);
                let handle = self
                    .create_server_task_no_tls(app.state(), addr, "Second public API", quit_notification)
                    .await;
                Some(handle)
            } else {
                None
            };

        (join_handle, second_join_handle)
    }

    pub async fn create_server_task_with_tls(
        &self,
        state: S,
        addr: SocketAddr,
        tls_config: Arc<ServerConfig>,
        mut quit_notification: ServerQuitWatcher,
    ) -> JoinHandle<()> {
        let mut listener = TcpListener::bind(addr)
            .await
            .expect("Address not available");
        let acceptor = TlsAcceptor::from(tls_config);

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
                                error!("TLS task address stream error {e}");
                                return;
                            }
                        }
                    }
                };

                let acceptor = acceptor.clone();
                let state = state.clone();

                let mut quit_notification = quit_notification.resubscribe();
                let drop_on_quit = drop_after_connection.clone();
                tokio::spawn(async move {
                    tokio::select! {
                        _ = quit_notification.recv() => {}
                        connection = acceptor.accept(tcp_stream) => {
                            match connection {
                                Ok(tls_connection) => {
                                    tokio::select! {
                                        _ = quit_notification.recv() => (),
                                        _ = handle_connection_to_server(
                                            tls_connection,
                                            addr,
                                            state
                                        ) => (),
                                    };
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
        state: S,
        addr: SocketAddr,
        name_for_log_message: &'static str,
        mut quit_notification: ServerQuitWatcher,
    ) -> JoinHandle<()> {
        let mut listener = TcpListener::bind(addr)
            .await
            .expect("Address not available");

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
                                error!("{name_for_log_message}, address stream error {e}");
                                return;
                            }
                        }
                    }
                };

                let state = state.clone();

                let mut quit_notification = quit_notification.resubscribe();
                let drop_on_quit = drop_after_connection.clone();
                tokio::spawn(async move {
                    tokio::select! {
                        _ = quit_notification.recv() => (),
                        _ = handle_connection_to_server(
                            tcp_stream,
                            addr,
                            state
                        ) => (),
                    };
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
}
