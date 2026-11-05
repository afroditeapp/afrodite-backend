use api_client::apis::common_api::{delete_data_export, get_data_export_state};
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let mut no_token_api = (*account.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = get_data_export_state(&no_token_api).await;
    assert_failure(result)
}

/// Own export state is Empty by default.
#[server_test]
async fn export_state_is_empty_by_default(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let state = get_data_export_state(&account.api()).await?;
    assert_eq(api_client::models::DataExportStateType::Empty, state.state)
}

/// Export state of another account is not accessible.
#[server_test]
async fn other_account_state_is_not_accessed(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let other = context.new_account().await?;

    delete_data_export(&other.api()).await?;
    let state = get_data_export_state(&account.api()).await?;
    assert_eq(api_client::models::DataExportStateType::Empty, state.state)
}
