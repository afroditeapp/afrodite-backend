use api_client::{
    apis::account_admin_api::post_account_verification_queue_remove_next_item,
    models::AccountVerificationErrorFlagsValue,
};
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let admin = context.new_admin().await?;
    let mut no_token_api = (*admin.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = post_account_verification_queue_remove_next_item(
        &no_token_api,
        api_client::models::PostAccountVerificationQueueRemoveNextItem::new(
            account_id(),
            AccountVerificationErrorFlagsValue::new(0),
        ),
    )
    .await;
    assert_failure(result)
}

fn account_id() -> api_client::models::AccountId {
    api_client::models::AccountId::default()
}

/// Request without admin permission fails.
#[server_test]
async fn request_without_admin_permission_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    let result = post_account_verification_queue_remove_next_item(
        &account.api(),
        api_client::models::PostAccountVerificationQueueRemoveNextItem::new(
            account.account_id(),
            AccountVerificationErrorFlagsValue::new(0),
        ),
    )
    .await;
    assert_failure(result)
}

/// Removing an item from an empty queue is successful. Removal succeeds
/// only when the provided account id matches queue head item owner.
#[server_test]
async fn removing_from_empty_queue_is_ok(mut context: TestContext) -> TestResult {
    let admin = context.new_admin().await?;
    let account = context.new_account().await?;

    post_account_verification_queue_remove_next_item(
        &admin.api(),
        api_client::models::PostAccountVerificationQueueRemoveNextItem::new(
            account.account_id(),
            AccountVerificationErrorFlagsValue::new(0),
        ),
    )
    .await?;

    Ok(())
}
