use std::time::Duration;

use model::{DailyLikesConfig, EventToClientInternal};
use model_profile::AccountIdInternal;
use server_api::{
    app::{EventManagerProvider, GetConfig, ReadData, WriteData},
    db_write_raw,
};
use server_common::result::{Result, WrappedResultExt};
use server_data::read::GetReadCommandsCommon;
use server_data_chat::write::GetWriteCommandsChat;
use server_state::S;
use simple_backend::ServerQuitWatcher;
use simple_backend_utils::time::sleep_until_current_time_is_at;
use tokio::{task::JoinHandle, time::sleep};
use tracing::{error, warn};

#[derive(thiserror::Error, Debug)]
enum DailyLikesError {
    #[error("Sleep until next task running failed")]
    Time,

    #[error("Database error")]
    Database,

    #[error("Event sending error")]
    EventSending,
}

#[derive(Debug)]
pub struct DailyLikesManagerQuitHandle {
    task: JoinHandle<()>,
}

impl DailyLikesManagerQuitHandle {
    pub async fn wait_quit(self) {
        match self.task.await {
            Ok(()) => (),
            Err(e) => {
                warn!("DailyLikesManager quit failed. Error: {:?}", e);
            }
        }
    }
}

pub struct DailyLikesManager {
    state: S,
}

impl DailyLikesManager {
    pub fn new_manager(
        state: S,
        quit_notification: ServerQuitWatcher,
    ) -> DailyLikesManagerQuitHandle {
        let manager = Self { state };

        let task = tokio::spawn(manager.run(quit_notification));

        DailyLikesManagerQuitHandle { task }
    }

    async fn run(self, mut quit_notification: ServerQuitWatcher) {
        let mut check_cooldown = false;
        let Some(like_sending_limits) = self
            .state
            .config()
            .client_features()
            .and_then(|v| v.limits.likes.daily.as_ref())
        else {
            return;
        };

        loop {
            tokio::select! {
                _ = sleep(Duration::from_secs(120)), if check_cooldown => {
                    check_cooldown = false;
                }
                result = Self::sleep_until(like_sending_limits), if !check_cooldown => {
                    match result {
                        Ok(()) => {
                            self.run_tasks(&mut quit_notification, like_sending_limits).await;
                        },
                        Err(e) => {
                            warn!("Sleep until failed. Error: {:?}", e);
                        }
                    }
                    check_cooldown = true;
                }
                _ = quit_notification.recv() => {
                    return;
                }
            }
        }
    }

    async fn sleep_until(config: &DailyLikesConfig) -> Result<(), DailyLikesError> {
        sleep_until_current_time_is_at(config.reset_time)
            .await
            .change_context(DailyLikesError::Time)?;
        Ok(())
    }

    async fn run_tasks(
        &self,
        quit_notification: &mut ServerQuitWatcher,
        config: &DailyLikesConfig,
    ) {
        tokio::select! {
            r = self.run_tasks_and_return_result(config) => {
                match r {
                    Ok(()) => (),
                    Err(e) => {
                        error!("Daily likes manager error: {:?}", e);
                    }
                }
            }
            _ = quit_notification.recv() => (),
        }
    }

    async fn run_tasks_and_return_result(
        &self,
        config: &DailyLikesConfig,
    ) -> Result<(), DailyLikesError> {
        let accounts = self
            .state
            .read()
            .common()
            .account_ids_internal_vec()
            .await
            .change_context(DailyLikesError::Database)?;

        for a in accounts {
            self.handle_account(a, config).await?;
        }

        Ok(())
    }

    async fn handle_account(
        &self,
        account: AccountIdInternal,
        config: &DailyLikesConfig,
    ) -> Result<(), DailyLikesError> {
        let limit = config.daily_likes.into();
        db_write_raw!(self.state, move |cmds| {
            cmds.chat()
                .limits()
                .reset_daily_likes_left(account, limit)
                .await
        })
        .await
        .change_context(DailyLikesError::Database)?;

        self.state
            .event_manager()
            .send_connected_event(account, EventToClientInternal::DailyLikesLeftChanged)
            .await
            .change_context(DailyLikesError::EventSending)?;

        Ok(())
    }
}
