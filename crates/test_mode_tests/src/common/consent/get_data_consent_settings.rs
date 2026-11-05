use api_client::apis::common_api::get_data_consent_settings;
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let mut no_token_api = (*account.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = get_data_consent_settings(&no_token_api).await;
    assert_failure(result)
}

/// Default data consent settings are empty.
#[server_test]
async fn default_data_consent_settings_are_empty(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let settings = get_data_consent_settings(&account.api()).await?;
    assert(settings.allow_face_verification.is_none())
}
