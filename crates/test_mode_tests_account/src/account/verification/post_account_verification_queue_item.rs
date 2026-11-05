use api_client::{
    apis::account_api::{
        get_account_verification_queue_status, post_account_verification_queue_item,
    },
    models::{AccountVerificationQueueItem, AccountVerificationScope, VerificationMethod},
};
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let mut no_token_api = (*account.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = post_account_verification_queue_item(
        &no_token_api,
        AccountVerificationQueueItem::new(
            "data".to_string(),
            VerificationMethod::Debug,
            AccountVerificationScope::new(),
        ),
    )
    .await;
    assert_failure(result)
}

/// Verification request without any scope enabled fails with an internal
/// server error because the request is not valid.
#[server_test]
async fn empty_scope_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    let result = post_account_verification_queue_item(
        &account.api(),
        AccountVerificationQueueItem::new(
            "data".to_string(),
            VerificationMethod::Debug,
            AccountVerificationScope::new(),
        ),
    )
    .await;
    assert_failure(result)
}

fn invalid_verification_item() -> api_client::models::AccountVerificationQueueItem {
    AccountVerificationQueueItem::new(
        "data".to_string(),
        VerificationMethod::Debug,
        AccountVerificationScope::new(),
    )
}

/// Verification request from an account which has not completed initial
/// setup returns an error result.
#[server_test]
async fn initial_setup_not_completed_returns_error_result(mut context: TestContext) -> TestResult {
    let account = context.new_account_in_initial_setup_state().await?;

    let result =
        post_account_verification_queue_item(&account.api(), invalid_verification_item()).await?;
    assert(result.error_initial_setup_not_completed.unwrap_or_default())?;

    // Another request returns the same error result.
    let result =
        post_account_verification_queue_item(&account.api(), invalid_verification_item()).await?;
    assert(result.error_initial_setup_not_completed.unwrap_or_default())?;

    Ok(())
}

/// Verification queue status stays without a queue position because
/// verification methods are not configured for bot client type in the test
/// environment.
#[server_test]
async fn queue_position_stays_empty(mut context: TestContext) -> TestResult {
    let account = context.new_account_in_initial_setup_state().await?;

    // Initial setup not completed does not add the item to the queue.
    let status = get_account_verification_queue_status(&account.api()).await?;
    assert(status.queue_position.is_none())?;

    Ok(())
}
