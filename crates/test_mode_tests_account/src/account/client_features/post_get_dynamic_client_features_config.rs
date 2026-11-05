use api_client::{
    apis::account_api::post_get_dynamic_client_features_config,
    models::DynamicClientFeaturesConfigHash,
};
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let mut no_token_api = (*account.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = post_get_dynamic_client_features_config(
        &no_token_api,
        DynamicClientFeaturesConfigHash::default(),
    )
    .await;
    assert_failure(result)
}

/// Dynamic client features config can be requested with a hash.
#[server_test]
async fn dynamic_config_can_be_requested(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    // Dynamic client features might not be configured. The request still
    // succeeds.
    post_get_dynamic_client_features_config(
        &account.api(),
        DynamicClientFeaturesConfigHash::default(),
    )
    .await?;

    Ok(())
}
