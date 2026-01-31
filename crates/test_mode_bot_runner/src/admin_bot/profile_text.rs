use api_client::models::ProfileStringModerationContentType;
use config::bot_config_file::internal::ProfileStringModerationConfig;
use error_stack::Result;
use test_mode_bot::actions::admin::profile_string::{
    AdminBotProfileStringModerationLogic, ProfileStringModerationState,
};
use test_mode_utils::client::{ApiClient, TestError};

use super::notification::ModerationHandler;

/// Profile text moderation handler
pub struct ProfileTextModerationHandler {
    api_client: ApiClient,
    config: Option<ProfileStringModerationConfig>,
    reqwest_client: reqwest::Client,
    state: Option<ProfileStringModerationState>,
}

impl ProfileTextModerationHandler {
    pub fn new(
        api_client: ApiClient,
        config: Option<ProfileStringModerationConfig>,
        reqwest_client: reqwest::Client,
    ) -> Self {
        Self {
            api_client,
            config,
            reqwest_client,
            state: None,
        }
    }
}

impl ModerationHandler for ProfileTextModerationHandler {
    async fn handle(&mut self) -> Result<(), TestError> {
        let config = match &self.config {
            Some(c) => c,
            None => return Ok(()),
        };

        let moderation_state = self.state.get_or_insert_with(|| {
            ProfileStringModerationState::new(config, self.reqwest_client.clone())
        });

        AdminBotProfileStringModerationLogic::run_profile_string_moderation(
            ProfileStringModerationContentType::ProfileText,
            &self.api_client,
            config,
            moderation_state,
        )
        .await?;

        Ok(())
    }
}
