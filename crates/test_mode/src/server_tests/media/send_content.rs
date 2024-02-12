use api_client::{
    apis::account_api::{get_account_state, post_complete_setup},
    models::AccountState,
};
use test_mode_macro::server_test;

use crate::{
    action_array,
    bot::actions::{
        account::SetAccountSetup,
        media::{MakeModerationRequest, SendImageToSlot, SetPendingContent}, AssertFailure,
    },
    runner::server_tests::assert::{assert_eq, assert_failure},
    TestContext, TestResult,
};

#[server_test]
async fn save_content_only_has_7_slots_available(context: TestContext) -> TestResult {
    let mut account = context.new_account().await?;
    account
        .run_actions(action_array![
            SendImageToSlot::slot(0),
            SendImageToSlot::slot(1),
            SendImageToSlot::slot(2),
            SendImageToSlot::slot(3),
            SendImageToSlot::slot(4),
            SendImageToSlot::slot(5),
            SendImageToSlot::slot(6),
            AssertFailure(SendImageToSlot::slot(7)),
        ])
        .await?;
    Ok(())
}
