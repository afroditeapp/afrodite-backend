
use api_client::{apis::account_api::get_account_state, models::AccountState};
use error_stack::ResultExt;
use test_mode_macro::server_test;

use crate::{TestContext, TestError};


#[server_test]
async fn account_state_is_initial_setup_after_login(context: TestContext) -> error_stack::Result<(), TestError> {
    let account = context.new_account().await?;
    let state = get_account_state(account.account_api())
            .await
            .change_context(TestError::ApiRequest)?;

    assert_eq!(state.state, AccountState::Banned);
    Ok(())
}
