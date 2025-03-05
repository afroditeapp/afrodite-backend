use std::sync::{atomic::{AtomicI64, Ordering}, Arc};

use error_stack::Result;
use manager_api::{backup::BackupSourceClient, ClientConfig, ClientError, ManagerClient, ManagerClientWithRequestReceiver, ServerEventListerner};
use manager_model::{
    ManagerInstanceName, ServerEventType
};
use simple_backend_config::SimpleBackendConfig;
use simple_backend_model::UnixTime;
use simple_backend_utils::ContextExt;
use tokio::task::JoinHandle;
use tracing::{info, warn, error};

use crate::ServerQuitWatcher;

#[derive(Debug, Clone)]
struct BackupLinkPassword(String);

#[derive(Debug)]
pub struct ManagerApiClient {
    manager: Option<(ClientConfig, ManagerInstanceName, Option<BackupLinkPassword>)>,
    latest_scheduled_reboot: AtomicI64,
}

impl ManagerApiClient {
    pub fn empty() -> Self {
        Self {
            manager: None,
            latest_scheduled_reboot: AtomicI64::new(0),
        }
    }

    pub async fn new(config: &SimpleBackendConfig) -> Result<Self, ClientError> {
        let manager = if let Some(c) = config.manager_config() {
            let certificate = if let Some(certificate) = &c.root_certificate {
                Some(ManagerClient::load_root_certificate(certificate)?)
            } else {
                None
            };

            let config = ClientConfig {
                api_key: c.api_key.to_string(),
                url: c.address.clone(),
                root_certificate: certificate,
            };

            info!("Manager API URL: {}", c.address);

            Some((config, c.manager_name.clone(), c.backup_link_password.clone().map(BackupLinkPassword)))
        } else {
            None
        };

        Ok(Self {
            manager,
            latest_scheduled_reboot: AtomicI64::new(0),
        })
    }

    pub async fn new_request(&self) -> Result<ManagerClientWithRequestReceiver, ClientError> {
        if let Some((c, name, _)) = self.manager.clone() {
            let c = ManagerClient::connect(c)
                .await?
                .request_to(name);
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
            let c = ManagerClient::connect(c)
                .await?
                .request_to(name);
            Ok(c)
        } else {
            Err(ClientError::MissingConfiguration.report())
        }
    }

    /// None is returned when the backup link password is not configured
    pub async fn new_backup_connection(&self, backup_session: u32) -> Result<Option<BackupSourceClient>, ClientError> {
        if let Some((c, _, password)) = self.manager.clone() {
            if let Some(password) = password {
                let (reader, writer) = ManagerClient::connect(c)
                    .await?
                    .backup_link(password.0)
                    .await?;
                Ok(Some(BackupSourceClient::new(reader, writer, backup_session)))
            } else {
                Ok(None)
            }
        } else {
            Err(ClientError::MissingConfiguration.report())
        }
    }

    pub async fn listen_events(&self) -> Result<ServerEventListerner, ClientError> {
        if let Some((c, _, _)) = self.manager.clone() {
            let c = ManagerClient::connect(c)
                .await?
                .listen_events()
                .await?;
            Ok(c)
        } else {
            Err(ClientError::MissingConfiguration.report())
        }
    }

    pub fn latest_scheduled_reboot(&self) -> Option<UnixTime> {
        let v = self.latest_scheduled_reboot.load(Ordering::Relaxed);
        if v == 0 {
            None
        } else {
            Some(UnixTime::new(v))
        }
    }

    pub fn set_latest_scheduled_reboot(&self, ut: Option<UnixTime>) {
        let v = ut.map(|v| v.ut).unwrap_or_default();
        self.latest_scheduled_reboot.store(v, Ordering::Relaxed);
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

impl <T: ManagerEventHandler> ManagerConnectionManager<T> {
    pub async fn new_manager(
        client: Arc<ManagerApiClient>,
        event_handler: T,
        quit_notification: ServerQuitWatcher,
    ) -> Result<ManagerConnectionManagerQuitHandle, ClientError> {
        let manager = Self { client: client.clone(), event_handler };

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
                    let ut = if let Some(time) = time {
                        time.0.ut
                    } else {
                        0
                    };
                    self.client.latest_scheduled_reboot.store(ut, Ordering::Relaxed);
                }
            }
            self.event_handler.handle(event.event()).await;
        }
    }
}

pub trait ManagerEventHandler: Send + Sync + 'static {
    fn handle(&self, event: &ServerEventType) -> impl std::future::Future<Output = ()> + std::marker::Send;
}
