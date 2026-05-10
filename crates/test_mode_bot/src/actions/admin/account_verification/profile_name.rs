use api_client::{
    apis::profile_admin_api,
    models::{AccountId, AccountVerificationScope, PostProfileNameVerifiedValue},
};
use config::bot_config_file::internal::AccountVerificationConfig;
use error_stack::{Result, ResultExt};
use test_mode_utils::client::{ApiClient, TestError};

use super::{LazyProfileAgeAndName, VerificationMethodAction};

pub async fn handle_profile_name_verification(
    api: &ApiClient,
    config: &AccountVerificationConfig,
    account_id: &AccountId,
    verification_scope: &AccountVerificationScope,
    method_action: &VerificationMethodAction,
    age_and_name: &mut LazyProfileAgeAndName<'_>,
) -> Result<(), TestError> {
    let value = if config.profile_name && verification_scope.profile_name.unwrap_or_default() {
        let accept = match method_action {
            VerificationMethodAction::Accept => true,
            VerificationMethodAction::Reject => false,
            VerificationMethodAction::_PersonIdentificationData { names, .. } => {
                let profile_name = age_and_name
                    .name()
                    .await?
                    .unwrap_or_default()
                    .trim()
                    .to_lowercase();
                if profile_name.is_empty() {
                    false
                } else {
                    let mut accepted = false;
                    for name in names {
                        if name.trim().to_lowercase() == profile_name {
                            accepted = true;
                            break;
                        }
                    }
                    accepted
                }
            }
        };
        Some(Some(accept))
    } else {
        None
    };

    let request = PostProfileNameVerifiedValue {
        account_id: Box::new(account_id.clone()),
        value,
    };

    profile_admin_api::post_profile_name_verified_value(&api.api(), request)
        .await
        .change_context(TestError::ApiRequest)
}
