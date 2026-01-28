use config::bot_config_file::internal::ContentModerationConfig;
use error_stack::Result;
use test_mode_bot::actions::admin::content::{
    AdminBotContentModerationLogic, ContentModerationState,
};
use test_mode_utils::client::{ApiClient, TestError};

use super::notification::ModerationHandler;

/// Content moderation handler for images/media
pub struct ContentModerationHandler {
    api_client: ApiClient,
    config: ContentModerationConfig,
    reqwest_client: reqwest::Client,
    state: Option<ContentModerationState>,
}

impl ContentModerationHandler {
    pub fn new(
        api_client: ApiClient,
        config: ContentModerationConfig,
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

impl ModerationHandler for ContentModerationHandler {
    async fn handle(&mut self) -> Result<(), TestError> {
        let moderation_state = if let Some(state) = &mut self.state {
            state
        } else {
            let moderation_state =
                ContentModerationState::new(&self.config, self.reqwest_client.clone()).await?;
            self.state.get_or_insert(moderation_state)
        };

        AdminBotContentModerationLogic::run_content_moderation(
            &self.api_client,
            &self.config,
            moderation_state,
        )
        .await?;

        Ok(())
    }
}
