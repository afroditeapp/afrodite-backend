use api_client::apis::common_api::{get_data_export_archive, get_data_export_state};
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let mut no_token_api = (*account.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = get_data_export_archive(&no_token_api, "invalid_name").await;
    assert_failure(result)
}

/// Archive download fails when state is not Done.
#[server_test]
async fn archive_download_fails_when_export_state_is_not_done(
    mut context: TestContext,
) -> TestResult {
    let account = context.new_account().await?;

    let state = get_data_export_state(&account.api()).await?;
    assert_eq(api_client::models::DataExportStateType::Empty, state.state)?;

    let result = get_data_export_archive(&account.api(), "invalid_name").await;
    assert_failure(result)
}
