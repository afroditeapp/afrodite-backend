use api_client::{
    apis::common_api::{get_data_consent_settings, post_data_consent_settings},
    models::UpdateDataConsentSettings,
};
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let mut no_token_api = (*account.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = post_data_consent_settings(&no_token_api, UpdateDataConsentSettings::new()).await;
    assert_failure(result)
}

/// Data consent settings can be updated.
#[server_test]
async fn data_consent_settings_can_be_updated(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    let mut update = UpdateDataConsentSettings::new();
    update.allow_face_verification = Some(true);
    post_data_consent_settings(&account.api(), update).await?;

    let settings = get_data_consent_settings(&account.api()).await?;
    assert_eq(Some(true), settings.allow_face_verification)
}
