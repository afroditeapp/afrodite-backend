use config::bot_config_file::internal::SecurityContentVerificationConfig;
use error_stack::Result;
use test_mode_bot::actions::admin::security_content_verification::{
    AdminBotSecurityContentVerificationLogic, SecurityContentVerificationState,
};
use test_mode_utils::client::{ApiClient, TestError};

use super::notification::ModerationHandler;

pub struct SecurityContentVerificationHandler {
    api_client: ApiClient,
    config: Option<SecurityContentVerificationConfig>,
    reqwest_client: reqwest::Client,
    state: Option<SecurityContentVerificationState>,
}

impl SecurityContentVerificationHandler {
    pub fn new(
        api_client: ApiClient,
        config: Option<SecurityContentVerificationConfig>,
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

impl ModerationHandler for SecurityContentVerificationHandler {
    async fn handle(&mut self) -> Result<(), TestError> {
        let config = match &self.config {
            Some(c) => c,
            None => return Ok(()),
        };

        let state = self.state.get_or_insert_with(|| {
            SecurityContentVerificationState::new(config, self.reqwest_client.clone())
        });

        AdminBotSecurityContentVerificationLogic::run_security_content_verification(
            &self.api_client,
            config,
            state,
        )
        .await?;

        Ok(())
    }
}
