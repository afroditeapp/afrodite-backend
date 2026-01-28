use api_client::models::ProfileStringModerationContentType;
use config::bot_config_file::internal::ProfileStringModerationConfig;
use error_stack::Result;
use test_mode_bot::actions::admin::profile_string::{
    AdminBotProfileStringModerationLogic, ProfileStringModerationState,
};
use test_mode_utils::client::{ApiClient, TestError};

use super::notification::ModerationHandler;

/// Profile name moderation handler
pub struct ProfileNameModerationHandler {
    api_client: ApiClient,
    config: ProfileStringModerationConfig,
    reqwest_client: reqwest::Client,
    state: Option<ProfileStringModerationState>,
}

impl ProfileNameModerationHandler {
    pub fn new(
        api_client: ApiClient,
        config: ProfileStringModerationConfig,
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

impl ModerationHandler for ProfileNameModerationHandler {
    async fn handle(&mut self) -> Result<(), TestError> {
        let config = &self.config;
        let moderation_state = self.state.get_or_insert_with(|| {
            ProfileStringModerationState::new(config, self.reqwest_client.clone())
        });

        AdminBotProfileStringModerationLogic::run_profile_string_moderation(
            ProfileStringModerationContentType::ProfileName,
            &self.api_client,
            config,
            moderation_state,
        )
        .await?;

        Ok(())
    }
}
