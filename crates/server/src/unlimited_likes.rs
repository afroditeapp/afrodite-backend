use std::time::Duration;

use model::EventToClientInternal;
use model_profile::AccountIdInternal;
use server_api::app::{EventManagerProvider, GetConfig, ReadData};
use server_common::result::{Result, WrappedResultExt};
use server_data::read::GetReadCommandsCommon;
use server_data_profile::read::GetReadProfileCommands;
use server_state::S;
use simple_backend::ServerQuitWatcher;
use simple_backend_utils::time::{UtcTimeValue, sleep_until_current_time_is_at};
use tokio::{task::JoinHandle, time::sleep};
use tracing::{error, warn};

#[derive(thiserror::Error, Debug)]
enum UnlimitedLikesError {
    #[error("Sleep until next task running failed")]
    Time,

    #[error("Database error")]
    Database,

    #[error("Event sending error")]
    EventSending,
}

#[derive(Debug)]
pub struct UnlimitedLikesManagerQuitHandle {
    task: JoinHandle<()>,
}

impl UnlimitedLikesManagerQuitHandle {
    pub async fn wait_quit(self) {
        match self.task.await {
            Ok(()) => (),
            Err(e) => {
                warn!("UnlimitedLikesManager quit failed. Error: {:?}", e);
            }
        }
    }
}

pub struct UnlimitedLikesManager {
    state: S,
}

impl UnlimitedLikesManager {
    pub fn new_manager(
        state: S,
        quit_notification: ServerQuitWatcher,
    ) -> UnlimitedLikesManagerQuitHandle {
        let manager = Self { state };

        let task = tokio::spawn(manager.run(quit_notification));

        UnlimitedLikesManagerQuitHandle { task }
    }

    async fn run(self, mut quit_notification: ServerQuitWatcher) {
        let mut check_cooldown = false;
        let Some(unlimited_likes_disabling_time) = self
            .state
            .config()
            .client_features_internal()
            .likes
            .unlimited_likes_disabling_time
        else {
            return;
        };

        loop {
            tokio::select! {
                _ = sleep(Duration::from_secs(120)), if check_cooldown => {
                    check_cooldown = false;
                }
                result = Self::sleep_until(unlimited_likes_disabling_time), if !check_cooldown => {
                    match result {
                        Ok(()) => {
                            self.run_tasks(&mut quit_notification).await;
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

    async fn sleep_until(time: UtcTimeValue) -> Result<(), UnlimitedLikesError> {
        sleep_until_current_time_is_at(time)
            .await
            .change_context(UnlimitedLikesError::Time)?;
        Ok(())
    }

    async fn run_tasks(&self, quit_notification: &mut ServerQuitWatcher) {
        tokio::select! {
            r = self.run_tasks_and_return_result() => {
                match r {
                    Ok(()) => (),
                    Err(e) => {
                        error!("Unlimited likes manager error: {:?}", e);
                    }
                }
            }
            _ = quit_notification.recv() => (),
        }
    }

    async fn run_tasks_and_return_result(&self) -> Result<(), UnlimitedLikesError> {
        let accounts = self
            .state
            .read()
            .common()
            .account_ids_internal_vec()
            .await
            .change_context(UnlimitedLikesError::Database)?;

        for a in accounts {
            self.handle_account(a).await?;
        }

        Ok(())
    }

    async fn handle_account(&self, account: AccountIdInternal) -> Result<(), UnlimitedLikesError> {
        let profile = self
            .state
            .read()
            .profile()
            .profile(account)
            .await
            .change_context(UnlimitedLikesError::Database)?;
        if !profile.profile.unlimited_likes() {
            return Ok(());
        }

        self.state
            .data_all_access()
            .update_unlimited_likes(account, false)
            .await
            .change_context(UnlimitedLikesError::Database)?;

        self.state
            .event_manager()
            .send_connected_event(account, EventToClientInternal::ProfileChanged)
            .await
            .change_context(UnlimitedLikesError::EventSending)?;

        Ok(())
    }
}
