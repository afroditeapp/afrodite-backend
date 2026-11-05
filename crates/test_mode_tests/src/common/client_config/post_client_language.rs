use api_client::{
    apis::common_api::{get_client_language, post_client_language},
    models::ClientLanguage,
};
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let mut no_token_api = (*account.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = post_client_language(&no_token_api, ClientLanguage::new("fi".to_string())).await;
    assert_failure(result)
}

/// Client language can be set.
#[server_test]
async fn client_language_can_be_set(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    post_client_language(&account.api(), ClientLanguage::new("fi".to_string())).await?;

    let language = get_client_language(&account.api()).await?;
    assert_eq(Some(ClientLanguage::new("fi".to_string())), language.l)
}
