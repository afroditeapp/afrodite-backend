use api_client::{
    apis::profile_admin_api,
    models::{AccountId, AccountVerificationScope, PostProfileAgeRangeVerifiedValue},
};
use config::bot_config_file::internal::AccountVerificationConfig;
use error_stack::{Result, ResultExt};
use test_mode_utils::client::{ApiClient, TestError};

use super::VerificationMethodAction;

pub async fn handle_profile_age_range_verification(
    api: &ApiClient,
    config: &AccountVerificationConfig,
    account_id: &AccountId,
    verification_scope: &AccountVerificationScope,
    method_action: &VerificationMethodAction,
    current_age: i64,
) -> Result<(), TestError> {
    let value =
        if config.profile_age_range && verification_scope.profile_age_range.unwrap_or_default() {
            let accept = match method_action {
                VerificationMethodAction::Accept => true,
                VerificationMethodAction::Reject
                | VerificationMethodAction::_PersonIdentificationData { age: None, .. } => false,
                VerificationMethodAction::_PersonIdentificationData { age: Some(age), .. } => {
                    current_age == Into::<i64>::into(*age)
                }
            };
            Some(Some(accept))
        } else {
            None
        };

    let request = PostProfileAgeRangeVerifiedValue {
        account_id: Box::new(account_id.clone()),
        value,
    };

    profile_admin_api::post_profile_age_range_verified_value(&api.api(), request)
        .await
        .change_context(TestError::ApiRequest)
}
