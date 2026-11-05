use api_client::apis::account_api::post_complete_setup;
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account_in_initial_setup_state().await?;
    let mut no_token_api = (*account.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = post_complete_setup(&no_token_api).await;
    assert_failure(result)
}

/// Completing setup fails when initial setup is not ongoing.
#[server_test]
async fn complete_setup_fails_when_not_in_initial_setup_state(
    mut context: TestContext,
) -> TestResult {
    let account = context.new_account().await?;

    let result = post_complete_setup(&account.api()).await;
    assert_failure(result)?;

    Ok(())
}

/// Completing setup is successful from initial setup state.
#[server_test]
async fn complete_setup_is_successful_from_initial_setup_state(
    mut context: TestContext,
) -> TestResult {
    let account = context.new_account_in_initial_setup_state().await?;

    post_complete_setup(&account.api()).await?;

    assert_eq(
        test_mode_bot::actions::account::AccountState::Normal,
        api_client::apis::account_api::get_account_state(&account.api())
            .await?
            .into(),
    )?;

    Ok(())
}
