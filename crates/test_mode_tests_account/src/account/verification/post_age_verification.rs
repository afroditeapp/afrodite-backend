use api_client::{apis::account_api::post_age_verification, models::PostAgeVerification};
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let mut no_token_api = (*account.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = post_age_verification(
        &no_token_api,
        PostAgeVerification::new(
            "data".to_string(),
            api_client::models::AgeVerificationMethod::Debug,
        ),
    )
    .await;
    assert_failure(result)
}

/// Age verification fails because verification methods are not enabled
/// for the bot client type used by test accounts.
#[server_test]
async fn age_verification_fails_for_bot_client_type(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    let result = post_age_verification(
        &account.api(),
        PostAgeVerification::new(
            "data".to_string(),
            api_client::models::AgeVerificationMethod::Debug,
        ),
    )
    .await;
    assert_failure(result)?;

    Ok(())
}
