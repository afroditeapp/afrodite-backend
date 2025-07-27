use test_mode_bot::{
    action_array,
    actions::{AssertFailure, media::SendImageToSlot},
};
use test_mode_tests::prelude::*;

#[server_test]
async fn save_content_only_has_7_slots_available(mut context: TestContext) -> TestResult {
    let mut account = context.new_account_in_initial_setup_state().await?;
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
