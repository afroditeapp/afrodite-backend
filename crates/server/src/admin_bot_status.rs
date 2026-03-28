use std::time::Duration;

use model::EventToClientInternal;
use server_data::{app::EventManagerProvider, read::GetReadCommandsCommon};
use server_state::{S, app::AdminBotStatusProvider, state_impl::ReadData};
use simple_backend::{ServerQuitWatcher, app::GetManagerApi, perf::websocket::AdminBotConnections};
use tokio::task::JoinHandle;
use tracing::{error, warn};

const INITIAL_CHECK_DELAY: Duration = Duration::from_secs(60);

#[derive(Debug)]
pub struct AdminBotStatusManagerQuitHandle {
    task: JoinHandle<()>,
}

impl AdminBotStatusManagerQuitHandle {
    pub async fn wait_quit(self) {
        match self.task.await {
            Ok(()) => (),
            Err(e) => {
                warn!("AdminBotStatusManager quit failed. Error: {:?}", e);
            }
        }
    }
}

pub struct AdminBotStatusManager {
    state: S,
}

impl AdminBotStatusManager {
    pub fn new_manager(
        state: S,
        quit_notification: ServerQuitWatcher,
    ) -> AdminBotStatusManagerQuitHandle {
        let manager = Self { state };
        let task = tokio::spawn(manager.run(quit_notification));
        AdminBotStatusManagerQuitHandle { task }
    }

    async fn run(self, mut quit_notification: ServerQuitWatcher) {
        tokio::select! {
            _ = tokio::time::sleep(INITIAL_CHECK_DELAY) => {
                self.update_status_if_needed().await;
            }
            _ = quit_notification.recv() => {
                return;
            }
        }

        loop {
            tokio::select! {
                _ = self.state.admin_bot_status_data().wait_update_trigger() => {
                    self.update_status_if_needed().await;
                }
                _ = quit_notification.recv() => {
                    return;
                }
            }
        }
    }

    async fn update_status_if_needed(&self) {
        let bot_config = match self.state.read().common().bot_config().bot_config().await {
            Ok(config) => config.unwrap_or_default(),
            Err(e) => {
                error!("Failed to read bot config while updating admin bot status: {e:?}");
                return;
            }
        };

        let admin_bot_offline = if bot_config.admin_bot {
            AdminBotConnections::connection_count() == 0
        } else {
            // Admin bot disabled so prevent client showing offline info
            false
        };

        let changed = self
            .state
            .manager_api_client()
            .set_admin_bot_offline(admin_bot_offline)
            .await;

        if changed {
            let status = self.state.manager_api_client().maintenance_status().await;
            self.state
                .event_manager()
                .send_connected_event_to_logged_in_clients(
                    EventToClientInternal::ScheduledMaintenanceStatus(status),
                )
                .await;
        }
    }
}
