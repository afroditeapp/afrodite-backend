use std::sync::{atomic::{AtomicI64, Ordering}, Arc};

use error_stack::Result;
use manager_api::{ClientConfig, ClientError, ManagerClient, ManagerClientWithRequestReceiver, ServerEventListerner};
use manager_model::{
    ManagerInstanceName, ServerEventType
};
use simple_backend_config::SimpleBackendConfig;
use simple_backend_model::UnixTime;
use simple_backend_utils::ContextExt;
use tokio::task::JoinHandle;
use tracing::{info, warn, error};

use crate::ServerQuitWatcher;

#[derive(Debug)]
pub struct ManagerApiClient {
    manager: Option<(ClientConfig, ManagerInstanceName)>,
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

            Some((config, c.manager_name.clone()))
        } else {
            None
        };

        Ok(Self {
            manager,
            latest_scheduled_reboot: AtomicI64::new(0),
        })
    }

    pub async fn new_request(&self) -> Result<ManagerClientWithRequestReceiver, ClientError> {
        if let Some((c, name)) = self.manager.clone() {
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
        if let Some((c, _)) = self.manager.clone() {
            let c = ManagerClient::connect(c)
                .await?
                .request_to(name);
            Ok(c)
        } else {
            Err(ClientError::MissingConfiguration.report())
        }
    }

    pub async fn listen_events(&self) -> Result<ServerEventListerner, ClientError> {
        if let Some((c, _)) = self.manager.clone() {
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

pub struct ManagerConnectionManager {
    client: Arc<ManagerApiClient>,
}

impl ManagerConnectionManager {
    pub async fn new_manager(
        config: &SimpleBackendConfig,
        quit_notification: ServerQuitWatcher,
    ) -> Result<(Arc<ManagerApiClient>, ManagerConnectionManagerQuitHandle), ClientError> {
        let client = Arc::new(ManagerApiClient::new(config).await?);
        let manager = Self { client: client.clone() };

        let task = tokio::spawn(manager.run(quit_notification));

        Ok((client, ManagerConnectionManagerQuitHandle { task }))
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
            match listener.next_event().await?.event() {
                ServerEventType::MaintenanceSchedulingStatus(time) => {
                    let ut = if let Some(time) = time {
                        time.0.ut
                    } else {
                        0
                    };
                    self.client.latest_scheduled_reboot.store(ut, Ordering::Relaxed);
                }
            }
        }
    }
}
