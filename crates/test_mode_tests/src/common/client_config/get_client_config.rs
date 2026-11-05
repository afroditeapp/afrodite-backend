use api_client::apis::common_api::get_client_config;
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let mut no_token_api = (*account.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = get_client_config(&no_token_api).await;
    assert_failure(result)
}

/// Client config is available for logged in account.
#[server_test]
async fn client_config_is_available(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let config = get_client_config(&account.api()).await?;
    assert(config.profile_attributes.is_some())
}
