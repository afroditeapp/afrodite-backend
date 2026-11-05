use api_client::apis::account_api::get_account_verification_queue_status;
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let mut no_token_api = (*account.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = get_account_verification_queue_status(&no_token_api).await;
    assert_failure(result)
}

/// Verification queue status can be read. Account is not in the queue and
/// has no verification data by default.
#[server_test]
async fn queue_status_can_be_read(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    let status = get_account_verification_queue_status(&account.api()).await?;
    assert(status.queue_position.is_none())?;
    assert(status.verification_method.is_none())?;
    assert(status.verification_unix_time.is_none())?;

    Ok(())
}
