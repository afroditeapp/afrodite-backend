use std::sync::Arc;

use api_client::models::ProfileStringModerationContentType;
use config::bot_config_file::BotConfigFile;
use error_stack::Result;
use test_mode_bot::actions::admin::profile_text::{
    AdminBotProfileStringModerationLogic, ProfileStringModerationState,
};
use test_mode_utils::client::{ApiClient, TestError};

use super::notification::ModerationHandler;

/// Profile text moderation handler
pub struct ProfileTextModerationHandler {
    api_client: ApiClient,
    bot_config_file: Arc<BotConfigFile>,
    reqwest_client: reqwest::Client,
    state: Option<ProfileStringModerationState>,
}

impl ProfileTextModerationHandler {
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

impl ModerationHandler for ProfileTextModerationHandler {
    async fn handle(&mut self) -> Result<(), TestError> {
        if let Some(config) = &self.bot_config_file.profile_text_moderation {
            let moderation_state = self
                .state
                .get_or_insert_with(|| ProfileStringModerationState {
                    moderation_started: None,
                    client: None,
                    reqwest_client: self.reqwest_client.clone(),
                });

            AdminBotProfileStringModerationLogic::run_profile_string_moderation(
                ProfileStringModerationContentType::ProfileText,
                &self.api_client,
                config,
                moderation_state,
            )
            .await?;
        }
        Ok(())
    }
}
