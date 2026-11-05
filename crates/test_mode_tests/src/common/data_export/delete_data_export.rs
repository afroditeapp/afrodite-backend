use api_client::{
    apis::common_api::{delete_data_export, post_start_data_export},
    models::{DataExportType, PostStartDataExport},
};
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let mut no_token_api = (*account.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = delete_data_export(&no_token_api).await;
    assert_failure(result)
}

/// Deleting a non existing export is successful.
#[server_test]
async fn deleting_export_which_does_not_exist_is_ok(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    delete_data_export(&account.api()).await?;

    Ok(())
}

/// Deleting an ongoing export fails.
#[server_test]
async fn deleting_ongoing_export_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    post_start_data_export(
        &account.api(),
        PostStartDataExport::new(DataExportType::User, account.account_id()),
    )
    .await?;

    let result = delete_data_export(&account.api()).await;
    assert_failure(result)
}

/// Deleting own export is successful after the export has finished.
#[server_test]
async fn deleting_own_export_is_ok(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let api = account.api();

    post_start_data_export(
        &api,
        PostStartDataExport::new(DataExportType::User, account.account_id()),
    )
    .await?;

    // Wait until the export is not ongoing anymore.
    let mut state = api_client::apis::common_api::get_data_export_state(&api).await?;
    for _ in 0..100 {
        if state.state != api_client::models::DataExportStateType::InProgress {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
        state = api_client::apis::common_api::get_data_export_state(&api).await?;
    }

    delete_data_export(&api).await?;

    let state = api_client::apis::common_api::get_data_export_state(&api).await?;
    assert_eq(api_client::models::DataExportStateType::Empty, state.state)?;

    Ok(())
}
