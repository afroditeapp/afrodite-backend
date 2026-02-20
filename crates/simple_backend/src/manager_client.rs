use std::sync::Arc;

use error_stack::Result;
use manager_api::{
    ClientConfig, ClientError, ManagerClient, ManagerClientWithRequestReceiver,
    ServerEventListerner, TlsConfig, backup::BackupSourceClient,
};
use manager_model::{ManagerInstanceName, ServerEventType};
use simple_backend_config::SimpleBackendConfig;
use simple_backend_model::ScheduledMaintenanceStatus;
use simple_backend_utils::ContextExt;
use tokio::{sync::RwLock, task::JoinHandle};
use tracing::{error, info, warn};

use crate::ServerQuitWatcher;

#[derive(Debug, Clone)]
struct BackupLinkPassword(String);

#[derive(Debug)]
pub struct ManagerApiClient {
    manager: Option<(
        ClientConfig,
        ManagerInstanceName,
        Option<BackupLinkPassword>,
    )>,
    maintenance_status: RwLock<ScheduledMaintenanceStatus>,
}

impl ManagerApiClient {
    pub fn empty() -> Self {
        Self {
            manager: None,
            maintenance_status: RwLock::default(),
        }
    }

    pub async fn new(config: &SimpleBackendConfig) -> Result<Self, ClientError> {
        let manager = if let Some(c) = config.manager_config() {
            let certificate = if let Some(config) = c.tls.clone() {
                Some(TlsConfig::new(
                    config.root_cert,
                    config.client_auth_cert,
                    config.client_auth_cert_private_key,
                )?)
            } else {
                None
            };

            let config = ClientConfig {
                api_key: c.api_key.to_string(),
                url: c.address.clone(),
                tls_config: certificate,
            };

            info!("Manager API URL: {}", c.address);

            Some((
                config,
                c.name.clone(),
                c.backup_link_password.clone().map(BackupLinkPassword),
            ))
        } else {
            None
        };

        Ok(Self {
            manager,
            maintenance_status: RwLock::default(),
        })
    }

    pub async fn new_request(&self) -> Result<ManagerClientWithRequestReceiver, ClientError> {
        if let Some((c, name, _)) = self.manager.clone() {
            let c = ManagerClient::connect(c).await?.request_to(name);
            Ok(c)
        } else {
            Err(ClientError::MissingConfiguration.report())
        }
    }

    pub async fn new_request_to_instance(
        &self,
        name: ManagerInstanceName,
    ) -> Result<ManagerClientWithRequestReceiver, ClientError> {
        if let Some((c, _, _)) = self.manager.clone() {
            let c = ManagerClient::connect(c).await?.request_to(name);
            Ok(c)
        } else {
            Err(ClientError::MissingConfiguration.report())
        }
    }

    /// None is returned when the backup link password is not configured
    pub async fn new_backup_connection(
        &self,
        backup_session: u32,
    ) -> Result<Option<BackupSourceClient>, ClientError> {
        if let Some((c, _, password)) = self.manager.clone() {
            if let Some(password) = password {
                let (reader, writer) = ManagerClient::connect(c)
                    .await?
                    .backup_link(password.0)
                    .await?;
                Ok(Some(BackupSourceClient::new(
                    reader,
                    writer,
                    backup_session,
                )))
            } else {
                Ok(None)
            }
        } else {
            Err(ClientError::MissingConfiguration.report())
        }
    }

    pub async fn listen_events(&self) -> Result<ServerEventListerner, ClientError> {
        if let Some((c, _, _)) = self.manager.clone() {
            let c = ManagerClient::connect(c).await?.listen_events().await?;
            Ok(c)
        } else {
            Err(ClientError::MissingConfiguration.report())
        }
    }

    pub async fn maintenance_status(&self) -> ScheduledMaintenanceStatus {
        let status = self.maintenance_status.read().await.clone();
        if status.expired() {
            let empty = ScheduledMaintenanceStatus::default();
            self.set_maintenance_status(empty.clone()).await;
            empty
        } else {
            status
        }
    }

    pub async fn set_maintenance_status(&self, status: ScheduledMaintenanceStatus) {
        *self.maintenance_status.write().await = status;
    }
}

#[derive(Debug)]
pub struct ManagerConnectionManagerQuitHandle {
    task: JoinHandle<()>,
}

impl ManagerConnectionManagerQuitHandle {
    pub async fn wait_quit(self) {
        match self.task.await {
            Ok(()) => (),
            Err(e) => {
                warn!("ManagerConnectionManager quit failed. Error: {:?}", e);
            }
        }
    }
}

pub struct ManagerConnectionManager<T: ManagerEventHandler> {
    client: Arc<ManagerApiClient>,
    event_handler: T,
}

impl<T: ManagerEventHandler> ManagerConnectionManager<T> {
    pub async fn new_manager(
        client: Arc<ManagerApiClient>,
        event_handler: T,
        quit_notification: ServerQuitWatcher,
    ) -> Result<ManagerConnectionManagerQuitHandle, ClientError> {
        let manager = Self {
            client: client.clone(),
            event_handler,
        };

        let task = tokio::spawn(manager.run(quit_notification));

        Ok(ManagerConnectionManagerQuitHandle { task })
    }

    async fn run(self, mut quit_notification: ServerQuitWatcher) {
        tokio::select! {
            r = self.handle_connection() => {
                match r {
                    Ok(()) => (),
                    Err(e) => error!("{:?}", e),
                }
            },
            _ = quit_notification.recv() => (),
        }
    }

    async fn handle_connection(&self) -> Result<(), ClientError> {
        let mut listener = self.client.listen_events().await?;
        loop {
            let event = listener.next_event().await?;
            match event.event() {
                ServerEventType::MaintenanceSchedulingStatus(time) => {
                    let status = ScheduledMaintenanceStatus::server_maintenance(
                        time.map(|v| v.0),
                        time.map(|v| v.0.add_seconds(5 * 60)),
                    );
                    self.client.set_maintenance_status(status.clone()).await;
                    self.event_handler.send_maintenance_status(status).await;
                }
            }
        }
    }
}

pub trait ManagerEventHandler: Send + Sync + 'static {
    fn send_maintenance_status(
        &self,
        status: ScheduledMaintenanceStatus,
    ) -> impl std::future::Future<Output = ()> + std::marker::Send;
}
