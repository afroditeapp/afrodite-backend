use api_client::{
    apis::profile_api::{get_profile, post_profile},
    models::ProfileUpdate,
};
use test_mode_macro::server_test;

use crate::{runner::server_tests::assert::assert_eq, TestContext, TestResult};

#[server_test]
async fn updating_profile_works(context: TestContext) -> TestResult {
    let account = context.new_account_in_initial_setup_state().await?;
    let profile = ProfileUpdate {
        attributes: vec![],
        age: 18,
        name: format!(""),
        profile_text: format!("test"),
    };
    post_profile(account.account_api(), profile).await?;
    assert_eq(
        "test",
        &get_profile(account.account_api(), &account.account_id_string())
            .await?
            .profile_text,
    )
}
