use config::bot_config_file::internal::FaceVerificationConfig;
use error_stack::Result;
use test_mode_bot::actions::admin::face_verification::{
    AdminBotFaceVerificationLogic, FaceVerificationState,
};
use test_mode_utils::client::{ApiClient, TestError};

use super::notification::ModerationHandler;

/// Face verification handler
pub struct FaceVerificationHandler {
    api_client: ApiClient,
    config: Option<FaceVerificationConfig>,
    reqwest_client: reqwest::Client,
    state: Option<FaceVerificationState>,
}

impl FaceVerificationHandler {
    pub fn new(
        api_client: ApiClient,
        config: Option<FaceVerificationConfig>,
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

impl ModerationHandler for FaceVerificationHandler {
    async fn handle(&mut self) -> Result<(), TestError> {
        let config = match &self.config {
            Some(c) => c,
            None => return Ok(()),
        };

        let moderation_state = self
            .state
            .get_or_insert_with(|| FaceVerificationState::new(config, self.reqwest_client.clone()));

        AdminBotFaceVerificationLogic::run_face_verification(
            &self.api_client,
            config,
            moderation_state,
        )
        .await?;

        Ok(())
    }
}
