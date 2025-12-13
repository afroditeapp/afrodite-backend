use std::sync::Arc;

use api_client::models::AccountId;
use config::{args::TestMode, bot_config_file::BotConfigFile};
use error_stack::Result;
use test_mode_bot::{
    BotState, action_array,
    actions::{
        BotAction,
        account::{Login, Register},
        admin::{
            content::AdminBotContentModerationLogic,
            profile_text::AdminBotProfileStringModerationLogic,
        },
    },
};
use test_mode_utils::{
    client::{ApiClient, TestError},
    state::{BotPersistentState, StateData},
};
use tokio::{
    select,
    sync::{mpsc, watch},
};
use tracing::{error, info};

pub struct AdminBot {
    state: BotState,
    bot_running_handle: mpsc::Sender<Vec<BotPersistentState>>,
}

impl AdminBot {
    pub fn new(
        task_id: u32,
        bot_id: u32,
        config: Arc<TestMode>,
        bot_config_file: Arc<BotConfigFile>,
        old_state: Option<Arc<StateData>>,
        bot_running_handle: mpsc::Sender<Vec<BotPersistentState>>,
        reqwest_client: &reqwest::Client,
    ) -> Self {
        let account_id = if config.bot_mode().is_some() {
            bot_config_file.admin_bot_config.account_id.clone()
        } else {
            None
        };
        let account_id = account_id.or_else(|| {
            old_state
                .as_ref()
                .and_then(|v| v.find_matching(task_id, bot_id))
                .map(|v| v.account_id.clone())
        });
        let keys = old_state
            .as_ref()
            .and_then(|v| v.find_matching(task_id, bot_id))
            .and_then(|v| v.keys.clone());

        let state = BotState::new(
            account_id.map(AccountId::new),
            keys,
            config.clone(),
            bot_config_file.clone(),
            task_id,
            bot_id,
            ApiClient::new(config.api_urls.clone(), reqwest_client),
            config.api_urls.clone(),
            reqwest_client.clone(),
        );

        Self {
            state,
            bot_running_handle,
        }
    }

    pub async fn run(mut self, mut bot_quit_receiver: watch::Receiver<()>) {
        info!(
            "Admin bot started - Task {}, Bot {}",
            self.state.task_id, self.state.bot_id
        );

        let result = select! {
            result = self.run_admin_logic() => result,
            result = bot_quit_receiver.changed() => {
                match result {
                    Ok(()) => {
                        info!("Admin bot received quit signal");
                        Ok(())
                    }
                    Err(e) => {
                        error!("Admin bot quit receiver error: {:?}", e);
                        Ok(())
                    }
                }
            }
        };

        if let Err(e) = result {
            error!(
                "Admin bot error - Task {}, Bot {}: {:?}",
                self.state.task_id, self.state.bot_id, e
            );
        }

        // Save state before quitting
        if let Some(persistent_state) = self.state.persistent_state() {
            if let Err(e) = self.bot_running_handle.send(vec![persistent_state]).await {
                error!(
                    "Failed to send bot state - Task {}, Bot {}: {:?}",
                    self.state.task_id, self.state.bot_id, e
                );
            }
        }

        info!(
            "Admin bot stopped - Task {}, Bot {}",
            self.state.task_id, self.state.bot_id
        );
    }

    async fn run_admin_logic(&mut self) -> Result<(), TestError> {
        // Initial setup - inline run_actions
        for action in action_array![Register, Login].iter() {
            action.excecute_impl(&mut self.state).await?;
        }

        // Complete initial setup if needed
        self.complete_initial_setup_if_needed().await?;

        // Main moderation loop
        loop {
            AdminBotContentModerationLogic
                .excecute_impl(&mut self.state)
                .await?;

            AdminBotProfileStringModerationLogic::profile_name()
                .excecute_impl(&mut self.state)
                .await?;

            AdminBotProfileStringModerationLogic::profile_text()
                .excecute_impl(&mut self.state)
                .await?;

            // Small delay between iterations to prevent tight loop
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }

    async fn complete_initial_setup_if_needed(&mut self) -> Result<(), TestError> {
        use api_client::apis::account_api::get_account_state;
        use test_mode_bot::actions::account::AccountState;

        let account_state = get_account_state(self.state.api()).await.map_err(|e| {
            TestError::ApiRequest
                .report()
                .attach_printable(e.to_string())
        })?;

        if AccountState::from(account_state) == AccountState::InitialSetup {
            use test_mode_bot::actions::{
                account::{CompleteAccountSetup, SetAccountSetup},
                media::{SendImageToSlot, SetContent},
            };

            // Inline run_actions
            for action in action_array![
                SetAccountSetup::admin(),
                SendImageToSlot::slot(0),
                SetContent {
                    security_content_slot_i: Some(0),
                    content_0_slot_i: Some(0),
                },
                CompleteAccountSetup,
            ]
            .iter()
            {
                action.excecute_impl(&mut self.state).await?;
            }
        }

        Ok(())
    }
}
