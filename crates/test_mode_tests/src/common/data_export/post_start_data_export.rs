use api_client::{
    apis::common_api::post_start_data_export,
    models::{DataExportType, PostStartDataExport},
};
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let mut no_token_api = (*account.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = post_start_data_export(
        &no_token_api,
        PostStartDataExport::new(DataExportType::User, account.account_id()),
    )
    .await;
    assert_failure(result)
}

/// User can start a user type export of own account.
#[server_test]
async fn own_user_export_starts(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    post_start_data_export(
        &account.api(),
        PostStartDataExport::new(DataExportType::User, account.account_id()),
    )
    .await?;

    let state = api_client::apis::common_api::get_data_export_state(&account.api()).await?;
    assert_eq(
        api_client::models::DataExportStateType::InProgress,
        state.state,
    )
}

/// Admin type exports and exports of another account
/// are not allowed without admin permission.
#[server_test]
async fn exporting_without_admin_permission_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let other = context.new_account().await?;

    let result = post_start_data_export(
        &account.api(),
        PostStartDataExport::new(DataExportType::Admin, account.account_id()),
    )
    .await;
    assert_failure(result)?;

    let result = post_start_data_export(
        &account.api(),
        PostStartDataExport::new(DataExportType::User, other.account_id()),
    )
    .await;
    assert_failure(result)
}

/// Export can be done only once per 24 hours without admin permission.
#[server_test]
async fn second_export_within_24_hours_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let api = account.api().clone();

    post_start_data_export(
        &api,
        PostStartDataExport::new(DataExportType::User, account.account_id()),
    )
    .await?;

    // Wait until the first export is not ongoing anymore.
    let mut state = api_client::apis::common_api::get_data_export_state(&api).await?;
    for _ in 0..100 {
        if state.state != api_client::models::DataExportStateType::InProgress {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
        state = api_client::apis::common_api::get_data_export_state(&api).await?;
    }

    let result = post_start_data_export(
        &api,
        PostStartDataExport::new(DataExportType::User, account.account_id()),
    )
    .await;
    assert_failure(result)
}

/// Export state stays InProgress while the export is being generated.
#[server_test]
async fn export_state_is_in_progress_after_starting(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let api = account.api().clone();

    post_start_data_export(
        &api,
        PostStartDataExport::new(DataExportType::User, account.account_id()),
    )
    .await?;

    let state = api_client::apis::common_api::get_data_export_state(&api).await?;
    assert_eq(
        api_client::models::DataExportStateType::InProgress,
        state.state,
    )
}
