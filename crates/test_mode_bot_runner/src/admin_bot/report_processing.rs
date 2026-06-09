use config::bot_config_file::internal::ReportProcessingConfigInternal;
use error_stack::Result;
use test_mode_bot::actions::admin::report_processing::{
    AdminBotReportProcessingLogic, ReportProcessingState,
};
use test_mode_utils::client::{ApiClient, TestError};

use super::notification::ModerationHandler;

pub struct ReportProcessingHandler {
    api_client: ApiClient,
    config: Option<ReportProcessingConfigInternal>,
    reqwest_client: reqwest::Client,
    state: Option<ReportProcessingState>,
}

impl ReportProcessingHandler {
    pub fn new(
        api_client: ApiClient,
        config: Option<ReportProcessingConfigInternal>,
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

impl ModerationHandler for ReportProcessingHandler {
    async fn handle(&mut self) -> Result<(), TestError> {
        let config = match &self.config {
            Some(c) => c,
            None => return Ok(()),
        };

        let state = if let Some(state) = &mut self.state {
            state
        } else {
            let state = ReportProcessingState::new(config, self.reqwest_client.clone());
            self.state.get_or_insert(state)
        };

        AdminBotReportProcessingLogic::run_report_processing(&self.api_client, config, state)
            .await?;

        Ok(())
    }
}
