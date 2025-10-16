use api_client::{
    apis::profile_api::{get_profile, post_profile},
    models::ProfileUpdate,
};
use test_mode_tests::prelude::*;
use test_mode_utils::client::TestError;

#[server_test]
async fn updating_profile_works(mut context: TestContext) -> TestResult {
    let name = Some("A".to_string());
    let account = context.new_account_in_initial_setup_state().await?;
    let profile = ProfileUpdate {
        attributes: vec![],
        age: 18,
        name: name.clone(),
        ptext: None,
    };
    post_profile(account.account_api(), profile).await?;
    assert_eq(
        name,
        get_profile(
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
