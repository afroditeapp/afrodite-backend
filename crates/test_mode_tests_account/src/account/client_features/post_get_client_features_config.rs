use api_client::{
    apis::account_api::post_get_client_features_config, models::ClientFeaturesConfigHash,
};
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let mut no_token_api = (*account.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = post_get_client_features_config(
        &no_token_api,
        ClientFeaturesConfigHash::new("invalid".to_string()),
    )
    .await;
    assert_failure(result)
}

/// Requesting with an invalid hash returns no config.
#[server_test]
async fn invalid_hash_returns_no_config(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    let result = post_get_client_features_config(
        &account.api(),
        ClientFeaturesConfigHash::new("invalid".to_string()),
    )
    .await?;
    assert(result.config.is_none())?;

    Ok(())
}
