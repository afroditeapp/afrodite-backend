use api_client::apis::{account_admin_api::post_delete_account, account_api::get_account_state};
use test_mode_test_utils::prelude::*;

fn disable_grant_admin_access(config: ServerConfigEditor) {
    config.server.grant_admin_access = None;
}

#[server_test(modify_server_config_with = "disable_grant_admin_access")]
async fn admin_rights_granting_does_not_work_when_disabled(mut context: TestContext) -> TestResult {
    let admin = context.new_admin().await?;
    assert_eq(
        get_account_state(&admin.api()).await?.permissions,
        Default::default(),
    )
}

#[server_test]
async fn admin_rights_granting_only_grants_rights_once_by_default(
    mut context: TestContext,
) -> TestResult {
    let admin1 = context.new_admin().await?;
    assert(
        get_account_state(&admin1.api())
            .await?
            .permissions
            .admin_edit_permissions
            .unwrap_or_default(),
    )?;

    post_delete_account(&admin1.api(), &admin1.account_id().aid).await?;

    let admin2 = context.new_admin().await?;
    assert(
        !get_account_state(&admin2.api())
            .await?
            .permissions
            .admin_edit_permissions
            .unwrap_or_default(),
    )?;

    Ok(())
}

#[server_test]
async fn normal_account_does_not_have_admin_rights(mut context: TestContext) -> TestResult {
    let account1 = context.new_account().await?;
    assert_eq(
        get_account_state(&account1.api()).await?.permissions,
        Default::default(),
    )
}
