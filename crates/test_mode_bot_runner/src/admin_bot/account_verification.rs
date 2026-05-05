use config::bot_config_file::internal::AccountVerificationConfig;
use error_stack::Result;
use test_mode_bot::actions::admin::account_verification::{
    AccountVerificationState, AdminBotAccountVerificationLogic,
};
use test_mode_utils::client::{ApiClient, TestError};

use super::notification::ModerationHandler;

pub struct AccountVerificationHandler {
    api_client: ApiClient,
    config: Option<AccountVerificationConfig>,
    reqwest_client: reqwest::Client,
    state: Option<AccountVerificationState>,
}

impl AccountVerificationHandler {
    pub fn new(
        api_client: ApiClient,
        config: Option<AccountVerificationConfig>,
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

impl ModerationHandler for AccountVerificationHandler {
    async fn handle(&mut self) -> Result<(), TestError> {
        let config = match &self.config {
            Some(c) => c,
            None => return Ok(()),
        };

        let state = self.state.get_or_insert_with(|| {
            AccountVerificationState::new(config, self.reqwest_client.clone())
        });

        AdminBotAccountVerificationLogic::run_account_verification(&self.api_client, config, state)
            .await?;

        Ok(())
    }
}
