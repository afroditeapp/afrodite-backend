use api_client::apis::account_api::get_account_state;
use test_mode_tests::prelude::*;

#[server_test]
async fn admin_rights_granting_only_grants_rights_once_by_default(
    mut context: TestContext,
) -> TestResult {
    let account1 = context.new_admin().await?;
    assert(
        get_account_state(account1.account().account_api())
            .await?
            .permissions
            .admin_moderate_media_content
            .unwrap_or_default(),
    )?;

    let account2 = context.new_admin().await?;
    assert(
        !get_account_state(account2.account().account_api())
            .await?
            .permissions
            .admin_moderate_media_content
            .unwrap_or_default(),
    )?;

    let account3 = context.new_admin().await?;
    assert(
        !get_account_state(account3.account().account_api())
            .await?
            .permissions
            .admin_moderate_media_content
            .unwrap_or_default(),
    )
}

#[server_test]
async fn normal_account_does_not_have_admin_rights(mut context: TestContext) -> TestResult {
    let account1 = context.new_account().await?;
    assert_eq(
        get_account_state(account1.account_api()).await?.permissions,
        Default::default(),
    )
}
