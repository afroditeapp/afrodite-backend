use error_stack::ResultExt;
use model::BackendConfig;
use server_api::{
    app::{GetAccounts, GetConfig, ReadDynamicConfig, WriteData, WriteDynamicConfig},
    db_write_raw,
};
use server_common::result::Result;
use server_data::write::GetWriteCommandsCommon;
use server_state::{
    S,
    dynamic_config::{DynamicConfigEvent, DynamicConfigEventReceiver},
};
use tokio::{sync::broadcast, task::JoinHandle};
use tracing::{error, warn};

use crate::dynamic_config::bot::BotClient;

mod bot;

#[derive(thiserror::Error, Debug)]
enum DynamicConfigManagerError {
    #[error("Database error")]
    Database,
    #[error("Bot client error")]
    BotClient,
    #[error("Data error")]
    Data,
}

/// Drop this when quit starts
type ManagerQuitHandle = broadcast::Sender<()>;

/// Use resubscribe() for cloning.
type ManagerQuitWatcher = broadcast::Receiver<()>;

#[derive(Debug)]
pub struct DynamicConfigManagerQuitHandle {
    handle: Option<ManagerQuitHandle>,
    task: JoinHandle<()>,
}

impl DynamicConfigManagerQuitHandle {
    pub async fn quit(mut self) {
        drop(self.handle.take());
        match self.task.await {
            Ok(()) => (),
            Err(e) => {
                warn!("DynamicConfigManager quit failed. Error: {:?}", e);
            }
        }
    }
}

pub struct DynamicConfigManager {
    state: S,
    bots: Option<BotClient>,
    current_config: BackendConfig,
}

impl DynamicConfigManager {
    pub fn new_manager(
        receiver: DynamicConfigEventReceiver,
        state: S,
    ) -> DynamicConfigManagerQuitHandle {
        let manager = Self {
            state,
            bots: None,
            current_config: BackendConfig::default(),
        };

        let (manager_quit_handle, manager_quit_watcher) = broadcast::channel(1);
        let task = tokio::spawn(manager.run(receiver, manager_quit_watcher));

        DynamicConfigManagerQuitHandle {
            task,
            handle: Some(manager_quit_handle),
        }
    }

    async fn run(
        mut self,
        mut receiver: DynamicConfigEventReceiver,
        mut quit_notification: ManagerQuitWatcher,
    ) {
        if !self.state.is_remote_bot_login_enabled() {
            match self.logout_remote_bots().await {
                Ok(()) => (),
                Err(e) => error!("{e:?}"),
            }
        }

        loop {
            tokio::select! {
                item = receiver.0.recv() => {
                    match item {
                        Some(DynamicConfigEvent::Reload) => {
                            match self.update_config().await {
                                Ok(()) => (),
                                Err(e) => error!("{e:?}"),
                            }
                        }
                        None => {
                            error!("Dynamic config manager event channel is broken");
                            return;
                        },
                    }
                }
                _ = quit_notification.recv() => {
                    if let Some(bots) = self.bots {
                        match bots.stop_bots().await {
                            Ok(()) => (),
                            Err(e) => error!("{e:?}"),
                        }
                    }
                    return;
                }
            }
        }
    }

    async fn update_config(&mut self) -> Result<(), DynamicConfigManagerError> {
        let new_config = self
            .state
            .read_config()
            .await
            .change_context(DynamicConfigManagerError::Database)?;

        let load_remote_bot_login_enabled_value =
            self.current_config.remote_bot_login != new_config.remote_bot_login;
        let restart_bots = self.current_config.admin_bot != new_config.admin_bot
            || self.current_config.user_bots != new_config.user_bots;

        self.current_config = new_config;

        if load_remote_bot_login_enabled_value {
            self.load_remote_bot_login_enabled_value_and_logout_remote_bots_if_needed()
                .await?;
        }
        if restart_bots {
            self.restart_bots().await?;
        }

        Ok(())
    }

    async fn restart_bots(&mut self) -> Result<(), DynamicConfigManagerError> {
        if let Some(bots) = self.bots.take() {
            match bots.stop_bots().await {
                Ok(()) => (),
                Err(e) => error!("{e:?}"),
            };
        }

        if (self.current_config.admin_bot || self.current_config.user_bots > 0)
            && !self.current_config.remote_bot_login
        {
            let bots = BotClient::start_bots(
                self.state.config(),
                self.current_config.admin_bot,
                self.current_config.user_bots,
            )
            .await
            .change_context(DynamicConfigManagerError::BotClient)?;
            self.bots = Some(bots);
        }

        Ok(())
    }

    async fn load_remote_bot_login_enabled_value_and_logout_remote_bots_if_needed(
        &self,
    ) -> Result<(), DynamicConfigManagerError> {
        if !self.current_config.remote_bot_login {
            self.logout_remote_bots().await?
        }

        self.state
            .set_remote_bot_login_enabled(self.current_config.remote_bot_login);

        Ok(())
    }

    async fn logout_remote_bots(&self) -> Result<(), DynamicConfigManagerError> {
        for b in self.state.config().remote_bots() {
            if let Some(id) = self.state.get_internal_id_optional(b.account_id()).await {
                db_write_raw!(self.state, move |cmds| { cmds.common().logout(id).await })
                    .await
                    .map_err(|e| e.into_report())
                    .change_context(DynamicConfigManagerError::Data)?;
            }
        }
        Ok(())
    }
}
