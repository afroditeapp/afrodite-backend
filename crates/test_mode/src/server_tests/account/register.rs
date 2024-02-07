
use api_client::{apis::account_api::get_account_state, models::AccountState};
use test_mode_macro::server_test;

use crate::{TestContext, TestResult};


#[server_test]
async fn account_state_is_initial_setup_after_login(context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let state = get_account_state(account.account_api()).await?;

    assert_eq!(state.state, AccountState::Banned);
    Ok(())
}
