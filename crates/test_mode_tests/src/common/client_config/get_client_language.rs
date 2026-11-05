use api_client::apis::common_api::get_client_language;
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let mut no_token_api = (*account.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = get_client_language(&no_token_api).await;
    assert_failure(result)
}

/// Default client language is empty.
#[server_test]
async fn default_client_language_is_empty(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let language = get_client_language(&account.api()).await?;
    assert(language.l.is_none())
}
