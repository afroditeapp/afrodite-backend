use api_client::{
    apis::profile_api::{get_profile, post_profile},
    models::ProfileUpdate,
};
use test_mode_macro::server_test;

use crate::{TestContext, TestResult, client::TestError, runner::server_tests::assert::assert_eq};

#[server_test]
async fn updating_profile_works(mut context: TestContext) -> TestResult {
    let account = context.new_account_in_initial_setup_state().await?;
    let profile = ProfileUpdate {
        attributes: vec![],
        age: 18,
        name: "A".to_string(),
        ptext: "".to_string(),
    };
    post_profile(account.account_api(), profile).await?;
    assert_eq(
        "A",
        &get_profile(
            account.account_api(),
            &account.account_id_string(),
            None,
            None,
        )
        .await?
        .p
        .flatten()
        .ok_or(TestError::MissingValue.report())?
        .name,
    )
}
