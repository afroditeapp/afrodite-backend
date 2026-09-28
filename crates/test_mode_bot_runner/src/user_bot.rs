use std::sync::Arc;

use api_client::models::AccountId;
use config::{args::TestMode, bot_config_file::BotConfigFile};
use simple_backend_utils::Result;
use test_mode_bot::{
    BotState, action_array,
    actions::{
        ActionArray, BotAction, RunActionsIf,
        account::{Register, SetProfileVisibility, login},
        profile::{ChangeProfileTextDaily, GetProfile, UpdateLocationRandomOrConfigured},
    },
};
use test_mode_utils::{
    client::{ApiClient, TestError},
    state::{BotPersistentState, StateData},
};
use tokio::{
    select,
    sync::{mpsc, oneshot, watch},
    time::{self, Duration},
};
use tracing::error;

use crate::actions::user::{
    AcceptReceivedLikesAndSendMessage, AnswerReceivedMessages, DoInitialSetupIfNeeded,
    SendLikeIfNeeded,
};

pub struct UserBot {
    state: BotState,
    bot_running_handle: mpsc::Sender<BotPersistentState>,
}

impl UserBot {
    pub fn new(
        task_id: u32,
        config: Arc<TestMode>,
        bot_config_file: Arc<BotConfigFile>,
        old_state: Option<Arc<StateData>>,
        bot_running_handle: mpsc::Sender<BotPersistentState>,
        account_id_from_api: AccountId,
        reqwest_client: &reqwest::Client,
    ) -> Self {
        // Use API account ID
        let account_id = Some(account_id_from_api.aid);

        let keys = old_state
            .as_ref()
            .and_then(|v| v.find_matching(task_id))
            .and_then(|v| v.keys.clone());

        let state = BotState::new(
            account_id.map(AccountId::new),
            keys,
            config.clone(),
            bot_config_file.clone(),
            task_id,
            ApiClient::new(config.api_urls.clone(), reqwest_client),
            config.api_urls.clone(),
            reqwest_client.clone(),
        );

        Self {
            state,
            bot_running_handle,
        }
    }

    async fn handle_quit(
        persistent_state: Option<BotPersistentState>,
        bot_running_handle: mpsc::Sender<BotPersistentState>,
    ) {
        if let Some(persistent_state) = persistent_state
            && let Err(e) = bot_running_handle.send(persistent_state).await
        {
            error!("Failed to send user bot state: {:?}", e);
        }
    }

    pub async fn run(mut self, mut bot_quit_receiver: watch::Receiver<()>) {
        let (internal_quit_handle, mut internal_quit_watcher) = oneshot::channel();

        select! {
            result = Self::run_bot_setup_logic(&mut self.state, internal_quit_handle) => {
                if let Err(e) = result {
                    error!("User bot setup logic error: {:?}", e);
                    Self::handle_quit(self.state.persistent_state(), self.bot_running_handle).await;
                    return;
                }
            },
            _ = bot_quit_receiver.changed() => {
                Self::handle_quit(self.state.persistent_state(), self.bot_running_handle).await;
                return;
            }
        };

        select! {
            result = Self::run_user_action_loop(&mut self.state, &mut internal_quit_watcher) => {
                if let Err(e) = result {
                    error!("User bot action loop error: {:?}", e);
                }
            },
            _ = bot_quit_receiver.changed() => (),
        };

        Self::handle_quit(self.state.persistent_state(), self.bot_running_handle).await;
    }

    async fn run_bot_setup_logic(
        state: &mut BotState,
        internal_quit_handle: oneshot::Sender<()>,
    ) -> Result<(), TestError> {
        Register.execute(state).await?;
        login(state, Some(internal_quit_handle)).await?;
        DoInitialSetupIfNeeded { admin: false }
            .execute(state)
            .await?;
        UpdateLocationRandomOrConfigured::new(None)
            .execute(state)
            .await?;
        SetProfileVisibility(true).execute(state).await?;
        SendLikeIfNeeded.execute(state).await?;

        Ok(())
    }

    async fn run_user_action_loop(
        state: &mut BotState,
        internal_quit_watcher: &mut oneshot::Receiver<()>,
    ) -> Result<(), TestError> {
        const ACTION_LOOP: ActionArray = action_array![
            GetProfile,
            RunActionsIf(
                action_array!(UpdateLocationRandomOrConfigured::new(None)),
                |s| { s.get_bot_config().change_location() && rand::random::<f32>() < 0.2 }
            ),
            RunActionsIf(action_array!(SetProfileVisibility(true)), |s| {
                s.get_bot_config().change_visibility() && rand::random::<f32>() < 0.5
            }),
            RunActionsIf(action_array!(SetProfileVisibility(false)), |s| {
                s.get_bot_config().change_visibility() && rand::random::<f32>() < 0.1
            }),
            RunActionsIf(action_array!(ChangeProfileTextDaily), |s| {
                s.get_bot_config().change_profile_text_time().is_some()
            }),
            AcceptReceivedLikesAndSendMessage,
            AnswerReceivedMessages,
        ];

        // Interval for action loop iterations
        let mut interval = time::interval(Duration::from_secs(5));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    for action in ACTION_LOOP.iter() {
                        action.execute(state).await?;
                    }
                }
                // Internal bot quit signal - websocket task exited
                _ = &mut *internal_quit_watcher => {
                    return Ok(());
                }
            }
        }
    }
}
