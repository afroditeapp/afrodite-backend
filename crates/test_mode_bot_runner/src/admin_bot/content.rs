use std::sync::Arc;

use config::bot_config_file::BotConfigFile;
use error_stack::Result;
use test_mode_bot::actions::admin::content::{
    AdminBotContentModerationLogic, ContentModerationState,
};
use test_mode_utils::client::{ApiClient, TestError};

use super::notification::ModerationHandler;

/// Content moderation handler for images/media
pub struct ContentModerationHandler {
    api_client: ApiClient,
    bot_config_file: Arc<BotConfigFile>,
    reqwest_client: reqwest::Client,
    state: Option<ContentModerationState>,
}

impl ContentModerationHandler {
    pub fn new(
        api_client: ApiClient,
        bot_config_file: Arc<BotConfigFile>,
        reqwest_client: reqwest::Client,
    ) -> Self {
        Self {
            api_client,
            bot_config_file,
            reqwest_client,
            state: None,
        }
    }
}

impl ModerationHandler for ContentModerationHandler {
    async fn handle(&mut self) -> Result<(), TestError> {
        if let Some(config) = &self.bot_config_file.content_moderation {
            let moderation_state = if let Some(state) = &mut self.state {
                state
            } else {
                let moderation_state =
                    ContentModerationState::new(config, self.reqwest_client.clone()).await?;
                self.state.get_or_insert(moderation_state)
            };

            AdminBotContentModerationLogic::run_content_moderation(
                &self.api_client,
                config,
                moderation_state,
            )
            .await?;
        }
        Ok(())
    }
}
