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
    runner::server_tests::assert::{assert, assert_eq, assert_failure},
    TestContext, TestResult,
};

#[server_test]
async fn admin_rights_granting_only_grants_rights_once_by_default(context: TestContext) -> TestResult {
    let account1 = context.new_admin().await?;
    assert(
        get_account_state(account1.account().account_api())
            .await?
            .capabilities
            .admin_moderate_images
            .unwrap_or_default(),
    )?;

    let account2 = context.new_admin().await?;
    assert(
        !get_account_state(account2.account().account_api())
            .await?
            .capabilities
            .admin_moderate_images
            .unwrap_or_default(),
    )?;

    let account3 = context.new_admin().await?;
    assert(
        !get_account_state(account3.account().account_api())
            .await?
            .capabilities
            .admin_moderate_images
            .unwrap_or_default(),
    )
}

#[server_test]
async fn normal_account_does_not_have_admin_rights(context: TestContext) -> TestResult {
    let account1 = context.new_account().await?;
    assert_eq(
        get_account_state(account1.account_api())
            .await?
            .capabilities,
        Default::default(),
    )
}
