use std::sync::Arc;

use api_client::models::{AccountId, EventType};
use config::{args::TestMode, bot_config_file::BotConfigFile};
use error_stack::Result;
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
    admin_bot::{
        content::ContentModerationHandler,
        notification::{ModerationHandler, NotificationSender},
        profile_name::ProfileNameModerationHandler,
        profile_text::ProfileTextModerationHandler,
    },
    client_bot::DoInitialSetupIfNeeded,
};

mod content;
mod notification;
mod profile_name;
mod profile_text;

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

    async fn handle_quit(
        persistent_state: Option<BotPersistentState>,
        bot_running_handle: mpsc::Sender<Vec<BotPersistentState>>,
    ) {
        if let Some(persistent_state) = persistent_state
            && let Err(e) = bot_running_handle.send(vec![persistent_state]).await
        {
            error!("Failed to send admin bot state: {:?}", e);
        }
        info!("Admin bot stopped",);
    }

    pub async fn run(mut self, mut bot_quit_receiver: watch::Receiver<()>) {
        info!(
            "Admin bot started - Task {}, Bot {}",
            self.state.task_id, self.state.bot_id
        );

        select! {
            result = Self::run_admin_initial_logic(&mut self.state) => {
                if let Err(e) = result {
                    error!("Admin bot logic error: {:?}", e);
                    Self::handle_quit(self.state.persistent_state(), self.bot_running_handle).await;
                    return;
                }
            },
            result = bot_quit_receiver.changed() => {
                match result {
                    Ok(()) => {
                        info!("Admin bot received quit signal");
                    }
                    Err(e) => {
                        error!("Admin bot quit receiver error: {:?}", e);
                    }
                }
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
            result = bot_quit_receiver.changed() => {
                match result {
                    Ok(()) => {
                        info!("Admin bot received quit signal");
                    }
                    Err(e) => {
                        error!("Admin bot quit receiver error: {:?}", e);
                    }
                }
            }
        };

        Self::handle_quit(persistent_state, self.bot_running_handle).await;
    }

    async fn run_admin_initial_logic(state: &mut BotState) -> Result<(), TestError> {
        state.connections.enable_events();

        for action in action_array![Register, Login, DoInitialSetupIfNeeded { admin: true }].iter()
        {
            action.excecute_impl(state).await?;
        }

        Ok(())
    }

    async fn run_admin_logic(state: BotState) -> Result<(), TestError> {
        // Create separate notification pipelines for each content type
        let (content_sender, mut content_receiver) = ContentModerationHandler::new(
            state.api.clone(),
            state.bot_config_file.clone(),
            state.reqwest_client.clone(),
        )
        .create_notification_channel();

        let (profile_name_sender, mut profile_name_receiver) = ProfileNameModerationHandler::new(
            state.api.clone(),
            state.bot_config_file.clone(),
            state.reqwest_client.clone(),
        )
        .create_notification_channel();

        let (profile_text_sender, mut profile_text_receiver) = ProfileTextModerationHandler::new(
            state.api.clone(),
            state.bot_config_file.clone(),
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
