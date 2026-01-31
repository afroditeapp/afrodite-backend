use std::sync::Arc;

use api_client::models::{AccountId, EventType};
use config::{BotConfig, args::TestMode, bot_config_file::BotConfigFile};
use error_stack::{Result, ResultExt};
use test_mode_bot::{
    BotState, action_array,
    actions::account::{Login, Register},
    connection::BotConnections,
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

use crate::{
    actions::user::DoInitialSetupIfNeeded,
    admin_bot::{
        content::ContentModerationHandler,
        notification::{ModerationHandler, NotificationSender},
        profile_name::ProfileNameModerationHandler,
        profile_text::ProfileTextModerationHandler,
    },
};

mod content;
mod notification;
mod profile_name;
mod profile_text;

pub struct AdminBot {
    state: BotState,
    bot_running_handle: mpsc::Sender<BotPersistentState>,
}

impl AdminBot {
    pub fn new(
        task_id: u32,
        config: Arc<TestMode>,
        bot_config_file: Arc<BotConfigFile>,
        old_state: Option<Arc<StateData>>,
        bot_running_handle: mpsc::Sender<BotPersistentState>,
        account_id_from_api: AccountId,
        reqwest_client: &reqwest::Client,
    ) -> Self {
        // Use API account ID directly
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
            error!("Failed to send admin bot state: {:?}", e);
        }
        info!("Admin bot stopped");
    }

    pub async fn run(mut self, mut bot_quit_receiver: watch::Receiver<()>) {
        info!("Admin bot started - Task {}", self.state.task_id,);

        select! {
            result = Self::run_admin_initial_logic(&mut self.state) => {
                if let Err(e) = result {
                    error!("Admin bot logic error: {:?}", e);
                    Self::handle_quit(self.state.persistent_state(), self.bot_running_handle).await;
                    return;
                }
            },
            _ = bot_quit_receiver.changed() => {
                Self::handle_quit(self.state.persistent_state(), self.bot_running_handle).await;
                return;
            }
        };

        // Admin bot persistent state does not change after initial logic
        let persistent_state = self.state.persistent_state();
        select! {
            result = Self::run_admin_logic(self.state) => {
                if let Err(e) = result {
                    error!("Admin bot logic error: {:?}", e);
                }
            },
            _ = bot_quit_receiver.changed() => (),
        };

        Self::handle_quit(persistent_state, self.bot_running_handle).await;
    }

    async fn run_admin_initial_logic(state: &mut BotState) -> Result<(), TestError> {
        state.connections.enable_events();

        for action in action_array![Register, Login, DoInitialSetupIfNeeded { admin: true }].iter()
        {
            action.excecute(state).await?;
        }

        Ok(())
    }

    async fn run_admin_logic(state: BotState) -> Result<(), TestError> {
        let bot_config_api = api_client::apis::common_admin_api::get_bot_config(state.api.api())
            .await
            .change_context(TestError::Reqwest)?;

        let bot_config_json =
            serde_json::to_string(&bot_config_api).change_context(TestError::Reqwest)?;
        let bot_config: BotConfig =
            serde_json::from_str(&bot_config_json).change_context(TestError::Reqwest)?;

        let (profile_name_config, profile_text_config, content_config) =
            config::bot_config_file::internal::merge(
                bot_config.admin_bot_config.unwrap_or_default(),
                (*state.bot_config_file).clone(),
            );

        // Create separate notification pipelines for each content type
        let (content_sender, mut content_receiver) = ContentModerationHandler::new(
            state.api.clone(),
            content_config,
            state.reqwest_client.clone(),
        )
        .create_notification_channel();

        let (profile_name_sender, mut profile_name_receiver) = ProfileNameModerationHandler::new(
            state.api.clone(),
            profile_name_config,
            state.reqwest_client.clone(),
        )
        .create_notification_channel();

        let (profile_text_sender, mut profile_text_receiver) = ProfileTextModerationHandler::new(
            state.api.clone(),
            profile_text_config,
            state.reqwest_client.clone(),
        )
        .create_notification_channel();

        select! {
            result = Self::run_admin_main_logic(
                state.connections,
                content_sender,
                profile_name_sender,
                profile_text_sender,
            ) => {
                if let Err(e) = result {
                    error!("Admin bot logic error: {:?}", e);
                }
            },
            result = content_receiver.process_notifications_loop() => {
                if let Err(e) = result {
                    error!("Content moderation pipeline error: {:?}", e);
                }
            },
            result = profile_name_receiver.process_notifications_loop() => {
                if let Err(e) = result {
                    error!("Profile name moderation pipeline error: {:?}", e);
                }
            },
            result = profile_text_receiver.process_notifications_loop() => {
                if let Err(e) = result {
                    error!("Profile text moderation pipeline error: {:?}", e);
                }
            },
        };

        Ok(())
    }

    async fn run_admin_main_logic(
        mut connections: BotConnections,
        content_sender: NotificationSender,
        profile_name_sender: NotificationSender,
        profile_text_sender: NotificationSender,
    ) -> Result<(), TestError> {
        // Hourly fallback timer in case there is some event related bug or
        // error for example. The timer ticks right away after creation as
        // server only sends events when there is moderation related change.
        let mut hourly_timer = tokio::time::interval(tokio::time::Duration::from_secs(60 * 60));

        loop {
            tokio::select! {
                // Receive events from websocket and notify appropriate pipelines
                event = connections.recv_event() => {
                    let event = event?;
                    if event.event == EventType::AdminBotNotification
                        && let Some(Some(notification)) = event.admin_bot_notification {
                            if notification.moderate_initial_media_content_bot.unwrap_or(false)
                                || notification.moderate_media_content_bot.unwrap_or(false)
                            {
                                content_sender.notify().await;
                            }

                            if notification.moderate_profile_names_bot.unwrap_or(false) {
                                profile_name_sender.notify().await;
                            }

                            if notification.moderate_profile_texts_bot.unwrap_or(false) {
                                profile_text_sender.notify().await;
                            }
                        }
                }
                // Forced moderation every hour as fallback - notify all pipelines
                _ = hourly_timer.tick() => {
                    content_sender.notify().await;
                    profile_name_sender.notify().await;
                    profile_text_sender.notify().await;
                }
            }
        }
    }
}
